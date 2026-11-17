//! Network-dependent metadata work, deliberately kept out of the file scan.
//!
//! The scan writes rows and enqueues a job per piece of missing metadata; this worker drains
//! those jobs afterwards with a bounded number of concurrent HTTP calls. A job is claimed in
//! one SQL statement so a second worker (or a second process) cannot pick up the same row.

use sqlx::PgPool;
use std::time::Duration;
use tracing::{error, info, warn};

pub const ARTIST_MBID: &str = "artist_mbid";
pub const ARTIST_ALIAS: &str = "artist_alias";
pub const ARTIST_PROFILE: &str = "artist_profile";
pub const ALBUM_ART: &str = "album_art";
pub const TRACK_MBID: &str = "track_mbid";
pub const ALBUM_MATCH: &str = "album_match";
pub const ALBUM_ALIAS: &str = "album_alias";

/// Called from the scan's write path. Works with a pool or a transaction: a failed insert must
/// fail the surrounding transaction rather than silently losing the job.
pub async fn enqueue<'e, E>(executor: E, kind: &str, target: i32) -> anyhow::Result<()>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query!(
        "INSERT INTO enrichment_job (kind, target) VALUES ($1, $2) \
         ON CONFLICT (kind, target) DO NOTHING",
        kind,
        target
    )
    .execute(executor)
    .await?;
    Ok(())
}

/// Drop a completed job so fresh input can be enriched again.
pub async fn invalidate<'e, E>(executor: E, kind: &str, target: i32) -> anyhow::Result<()>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query!("DELETE FROM enrichment_job WHERE kind = $1 AND target = $2", kind, target)
        .execute(executor)
        .await?;
    Ok(())
}

pub fn spawn(pool: PgPool, cfg: &crate::config::Config) {
    tokio::spawn(run(
        pool,
        cfg.enrichment_concurrency,
        cfg.enrichment_max_attempts,
        cfg.enrichment_lease_secs,
        cfg.enrichment_job_timeout_secs,
    ));
}

pub async fn run(
    pool: PgPool,
    concurrency: u32,
    max_attempts: u32,
    lease_secs: u64,
    job_timeout_secs: u64,
) {
    // One reclaim task for the whole worker, not one per slot.
    let sweeper = pool.clone();
    let sweep_interval = Duration::from_secs(lease_secs / 3);
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(sweep_interval);
        loop {
            ticker.tick().await;
            reclaim(&sweeper, lease_secs).await;
        }
    });

    info!(target: "enrichment", "worker started ({concurrency} slot(s))");
    let mut slots: Vec<tokio::task::JoinHandle<()>> = (0..concurrency)
        .map(|_| {
            let pool = pool.clone();
            tokio::spawn(loop_jobs(pool, max_attempts, lease_secs, job_timeout_secs))
        })
        .collect();

    // A slot never returns on its own. If one panics, log it and replace it instead of silently
    // running with fewer workers (or none) for the rest of the process lifetime.
    loop {
        let (result, _index, remaining) = futures::future::select_all(slots).await;
        match result {
            Ok(()) => warn!(target: "enrichment", "worker slot exited unexpectedly; restarting"),
            Err(e) => error!(target: "enrichment", "worker slot panicked: {e}; restarting"),
        }
        let pool = pool.clone();
        let mut remaining = remaining;
        remaining.push(tokio::spawn(loop_jobs(pool, max_attempts, lease_secs, job_timeout_secs)));
        slots = remaining;
    }
}

async fn loop_jobs(pool: PgPool, max_attempts: u32, lease_secs: u64, job_timeout_secs: u64) {
    loop {
        match claim(&pool, lease_secs).await {
            Ok(Some(job)) => {
                process(&pool, job, max_attempts, lease_secs, job_timeout_secs).await
            }
            Ok(None) => tokio::time::sleep(Duration::from_secs(1)).await,
            Err(e) => {
                warn!(target: "enrichment", "claim failed: {e}");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn reclaim(pool: &PgPool, lease_secs: u64) {
    let lease = lease_secs as f64;
    match sqlx::query!(
        "UPDATE enrichment_job SET state = 'pending', not_before = now(), \
         claim_id = NULL, lease_expires_at = NULL, updated_at = now() \
         WHERE state = 'running' AND (lease_expires_at < now() \
           OR (lease_expires_at IS NULL AND updated_at < now() - ($1 * interval '1 second')))",
        lease
    )
    .execute(pool)
    .await
    {
        Ok(result) if result.rows_affected() > 0 => {
            info!(target: "enrichment", "reclaimed {} abandoned job(s)", result.rows_affected())
        }
        Ok(_) => {}
        Err(e) => warn!(target: "enrichment", "failed to reclaim abandoned jobs: {e}"),
    }
}

#[derive(Debug)]
struct ClaimedJob {
    id: i32,
    kind: String,
    target: i32,
    attempts: i32,
    claim_id: String,
}

/// Claim and mark running atomically. `FOR UPDATE SKIP LOCKED` means a second worker simply
/// takes the next row instead of waiting on a lock held by the first.
async fn claim(pool: &PgPool, lease_secs: u64) -> anyhow::Result<Option<ClaimedJob>> {
    let claim_id = crate::helpers::random_id(12);
    let lease = lease_secs as f64;
    let job = sqlx::query!(
        r#"
        WITH picked AS (
            SELECT id FROM enrichment_job
            WHERE state = 'pending' AND not_before <= now()
            ORDER BY not_before, id
            FOR UPDATE SKIP LOCKED
            LIMIT 1
        )
        UPDATE enrichment_job j
        SET state = 'running', attempts = j.attempts + 1, updated_at = now(),
            claim_id = $1, lease_expires_at = now() + ($2 * interval '1 second')
        FROM picked
        WHERE j.id = picked.id
        RETURNING j.id, j.kind, j.target, j.attempts
        "#,
        claim_id,
        lease
    )
    .fetch_optional(pool)
    .await?;
    Ok(job.map(|row| ClaimedJob {
        id: row.id,
        kind: row.kind,
        target: row.target,
        attempts: row.attempts,
        claim_id,
    }))
}

async fn process(
    pool: &PgPool,
    job: ClaimedJob,
    max_attempts: u32,
    lease_secs: u64,
    job_timeout_secs: u64,
) {
    let (lost_tx, mut lost_rx) = tokio::sync::oneshot::channel::<()>();
    let heartbeat = tokio::spawn(heartbeat(
        pool.clone(),
        job.id,
        job.claim_id.clone(),
        lease_secs,
        lost_tx,
    ));

    // A lost lease cancels the handler here, but there is an accepted window (up to one
    // heartbeat interval) where a reclaimed job may briefly run in two workers. The
    // claim_id-guarded final UPDATE below is the correctness backstop: only the current owner
    // can commit a result. Do not remove that guard to "simplify" this select!.
    let result = tokio::select! {
        result = tokio::time::timeout(
            Duration::from_secs(job_timeout_secs),
            dispatch(&job.kind, job.target, pool),
        ) => match result {
            Ok(result) => result,
            Err(_) => Err(anyhow::anyhow!("enrichment handler timed out after {job_timeout_secs}s")),
        },
        _ = &mut lost_rx => Err(anyhow::anyhow!("lease lost during enrichment handler")),
    };
    heartbeat.abort();

    let (state, error, delay) = match &result {
        Ok(()) => ("done".to_string(), None, 0.0),
        Err(e) => {
            warn!(target: "enrichment", "{} #{} failed: {e}", job.kind, job.target);
            let state = if job.attempts >= max_attempts as i32 {
                "failed"
            } else {
                "pending"
            };
            (state.to_string(), Some(e.to_string()), backoff(job.attempts))
        }
    };

    // Only the worker that still owns the lease may change the job's state.
    match sqlx::query!(
        "UPDATE enrichment_job SET state = $1, not_before = now() + ($2 * interval '1 second'), \
         last_error = $3, claim_id = NULL, lease_expires_at = NULL, updated_at = now() \
         WHERE id = $4 AND claim_id = $5",
        state,
        delay,
        error,
        job.id,
        job.claim_id
    )
    .execute(pool)
    .await
    {
        Ok(result) if result.rows_affected() == 0 => {
            warn!(target: "enrichment", "job {} lost its lease; outcome discarded", job.id)
        }
        Ok(_) => {}
        Err(e) => warn!(target: "enrichment", "failed to finish job {}: {e}", job.id),
    }
}

/// Renew the lease while a handler is running so a slow provider does not make another worker
/// reclaim a job that is still in flight.
async fn dispatch(kind: &str, target: i32, pool: &PgPool) -> anyhow::Result<()> {
    match kind {
        ARTIST_MBID => crate::index::db::enrich_artist_mbid(target, pool).await,
        ARTIST_ALIAS => crate::index::db::enrich_artist_aliases(target, pool).await,
        ALBUM_MATCH => crate::index::db::enrich_album_match(target, pool).await,
        ALBUM_ALIAS => crate::index::db::enrich_album_aliases(target, pool).await,
        ARTIST_PROFILE => crate::index::db::enrich_artist_profile(target, pool).await,
        ALBUM_ART => crate::index::db::enrich_album_art(target, pool).await,
        TRACK_MBID => crate::index::db::enrich_track_mbid(target, pool).await,
        other => Err(anyhow::anyhow!("unknown enrichment kind {other}")),
    }
}

async fn heartbeat(
    pool: PgPool,
    id: i32,
    claim_id: String,
    lease_secs: u64,
    lost_tx: tokio::sync::oneshot::Sender<()>,
) {
    let interval = Duration::from_secs(lease_secs / 3);
    let lease = lease_secs as f64;
    loop {
        tokio::time::sleep(interval).await;
        match sqlx::query!(
            "UPDATE enrichment_job SET lease_expires_at = now() + ($1 * interval '1 second') \
             WHERE id = $2 AND claim_id = $3",
            lease,
            id,
            claim_id
        )
        .execute(&pool)
        .await
        {
            Ok(result) if result.rows_affected() == 0 => {
                warn!(target: "enrichment", "lost lease for job {id}; cancelling handler");
                let _ = lost_tx.send(());
                break;
            }
            Ok(_) => {}
            Err(e) => warn!(target: "enrichment", "lease renewal failed for job {id}: {e}"),
        }
    }
}

/// Exponential backoff, capped at ~17 minutes. Kept pure so the schedule can be tested without
/// a database.
fn backoff(attempts: i32) -> f64 {
    2f64.powi(attempts.clamp(1, 10))
}

#[cfg(test)]
mod tests {
    use super::backoff;

    #[test]
    fn backoff_is_exponential_and_bounded() {
        assert_eq!(backoff(1), 2.0);
        assert_eq!(backoff(2), 4.0);
        assert_eq!(backoff(3), 8.0);
        assert_eq!(backoff(0), 2.0);
        assert_eq!(backoff(100), 1024.0);
    }
}
