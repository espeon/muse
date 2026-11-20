use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::info;

use super::middleware::jwt::AdminUser;

#[derive(Serialize, utoipa::ToSchema)]
pub struct RescanResponse {
    pub status: &'static str,
}

#[derive(Debug, Deserialize)]
pub struct AnalyzeParams {
    pub kind: Option<crate::analysis::AnalysisKind>,
    pub retry_failures: Option<bool>,
    pub track_id: Option<Vec<i32>>,
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/rescan",
    tag = "admin",
    responses(
        (status = 202, description = "Rescan started", body = RescanResponse),
        (status = 403, description = "Admin access required"),
    ),
    security(("bearer_token" = []))
)]
/// POST /admin/rescan — trigger a full library rescan in the background.
/// Requires admin privileges. Returns 202 Accepted immediately.
pub async fn post_rescan(
    Extension(pool): Extension<PgPool>,
    AdminUser { .. }: AdminUser,
) -> Result<(StatusCode, Json<RescanResponse>), (StatusCode, String)> {
    let mount = std::env::var("MOUNT")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "MOUNT not set".to_string()))?;

    info!(target: "admin", "manual rescan triggered");

    tokio::spawn(async move {
        let cfg = crate::config::load_or_create_config("config/config.maki.json");
        crate::index::scan(&mount, pool, false, &cfg).await;
    });

    Ok((StatusCode::ACCEPTED, Json(RescanResponse { status: "scanning" })))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/analyze",
    tag = "admin",
    responses(
        (status = 202, description = "Audio analysis started", body = RescanResponse),
        (status = 403, description = "Admin access required"),
    ),
    security(("bearer_token" = []))
)]
/// POST /admin/analyze — retry failed files and analyze tracks missing features.
pub async fn post_analyze(
    Extension(pool): Extension<PgPool>,
    Extension(config): Extension<crate::config::Config>,
    Query(params): Query<AnalyzeParams>,
    AdminUser { .. }: AdminUser,
) -> Result<(StatusCode, Json<RescanResponse>), (StatusCode, String)> {
    let kind = params.kind.unwrap_or(crate::analysis::AnalysisKind::All);
    let enabled = match kind {
        crate::analysis::AnalysisKind::Similarity => config.audio_analysis_enabled,
        crate::analysis::AnalysisKind::Mix => config.mix_analysis_enabled,
        crate::analysis::AnalysisKind::All => {
            config.audio_analysis_enabled || config.mix_analysis_enabled
        }
    };
    if !enabled {
        return Err((
            StatusCode::CONFLICT,
            "requested audio analysis is disabled in config.maki.json".to_string(),
        ));
    }

    let track_ids = params.track_id.unwrap_or_default();
    info!(target: "analysis", "manual {kind:?} analysis triggered for {track_ids:?}");
    crate::analysis::enqueue_kind(
        pool,
        config,
        kind,
        params.retry_failures.unwrap_or(true),
        track_ids,
    );
    Ok((
        StatusCode::ACCEPTED,
        Json(RescanResponse {
            status: "analyzing",
        }),
    ))
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct EnrichmentError {
    pub id: i32,
    pub kind: String,
    pub target: i32,
    pub error: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct EnrichmentStatus {
    pub pending: i64,
    pub running: i64,
    pub done: i64,
    pub failed: i64,
    /// Age of the oldest due job, or None when nothing is pending.
    pub oldest_pending_secs: Option<f64>,
    pub recent_errors: Vec<EnrichmentError>,
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/enrichment",
    tag = "admin",
    responses(
        (status = 200, description = "Enrichment queue state", body = EnrichmentStatus),
        (status = 403, description = "Admin access required"),
    ),
    security(("bearer_token" = []))
)]
/// GET /admin/enrichment — inspect the post-scan metadata queue.
pub async fn get_enrichment(
    Extension(pool): Extension<PgPool>,
    AdminUser { .. }: AdminUser,
) -> Result<Json<EnrichmentStatus>, (StatusCode, String)> {
    let counts = sqlx::query!(
        "SELECT state, count(*) AS count FROM enrichment_job GROUP BY state"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut status = EnrichmentStatus {
        pending: 0,
        running: 0,
        done: 0,
        failed: 0,
        oldest_pending_secs: None,
        recent_errors: Vec::new(),
    };
    for row in counts {
        match row.state.as_str() {
            "pending" => status.pending = row.count.unwrap_or(0),
            "running" => status.running = row.count.unwrap_or(0),
            "done" => status.done = row.count.unwrap_or(0),
            "failed" => status.failed = row.count.unwrap_or(0),
            _ => {}
        }
    }

    status.oldest_pending_secs = sqlx::query_scalar!(
        "SELECT EXTRACT(EPOCH FROM (now() - min(not_before)))::double precision \
         FROM enrichment_job WHERE state = 'pending'"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let errors = sqlx::query!(
        "SELECT id, kind, target, last_error FROM enrichment_job \
         WHERE state = 'failed' AND last_error IS NOT NULL \
         ORDER BY updated_at DESC NULLS LAST LIMIT 10"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    status.recent_errors = errors
        .into_iter()
        .filter_map(|row| {
            row.last_error.clone().map(|error| EnrichmentError {
                id: row.id,
                kind: row.kind,
                target: row.target,
                error,
            })
        })
        .collect();

    Ok(Json(status))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/enrichment/{id}/retry",
    tag = "admin",
    params(("id" = i32, Path, description = "Enrichment job id")),
    responses(
        (status = 202, description = "Job requeued", body = RescanResponse),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "No such retryable job"),
    ),
    security(("bearer_token" = []))
)]
/// POST /admin/enrichment/{id}/retry — requeue a failed or completed job.
pub async fn post_retry_enrichment(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    AdminUser { .. }: AdminUser,
) -> Result<(StatusCode, Json<RescanResponse>), (StatusCode, String)> {
    let result = sqlx::query!(
        "UPDATE enrichment_job SET state = 'pending', attempts = 0, not_before = now(), \
         last_error = NULL, claim_id = NULL, lease_expires_at = NULL, updated_at = now() \
         WHERE id = $1",
        id
    )
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "no retryable job with that id".to_string()));
    }
    info!(target: "enrichment", "job {id} requeued by admin");
    Ok((
        StatusCode::ACCEPTED,
        Json(RescanResponse { status: "queued" }),
    ))
}


/// Requeue every terminally failed job. Failures are usually transient provider outages;
/// staggering `not_before` avoids a burst against a provider that just recovered.
#[utoipa::path(
    post,
    path = "/api/v1/admin/enrichment/retry-failed",
    tag = "admin",
    responses(
        (status = 202, description = "Number of jobs requeued", body = RescanResponse),
        (status = 403, description = "Admin access required"),
    ),
    security(("bearer_token" = []))
)]
pub async fn post_retry_failed_enrichment(
    Extension(pool): Extension<PgPool>,
    AdminUser { .. }: AdminUser,
) -> Result<(StatusCode, Json<RescanResponse>), (StatusCode, String)> {
    let count: i64 = sqlx::query_scalar!(
        "WITH failed AS (
            SELECT id, row_number() OVER (ORDER BY id) AS rn
            FROM enrichment_job WHERE state = 'failed'
         ), requeued AS (
            UPDATE enrichment_job j
            SET state = 'pending', attempts = 0,
                not_before = now() + (failed.rn * interval '1 second'),
                last_error = NULL, claim_id = NULL, lease_expires_at = NULL,
                updated_at = now()
            FROM failed WHERE j.id = failed.id
            RETURNING j.id
         )
         SELECT count(*) FROM requeued"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .unwrap_or(0);
    info!(target: "enrichment", "{count} failed job(s) requeued by admin");
    Ok((
        StatusCode::ACCEPTED,
        Json(RescanResponse { status: "requeued" }),
    ))
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct AlbumMatchEntry {
    pub release_mbid: String,
    pub release_group_mbid: Option<String>,
    pub score: f32,
    pub payload: serde_json::Value,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ApplyMatchRequest {
    pub release_mbid: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/albums/{id}/matches",
    tag = "admin",
    params(("id" = i32, Path, description = "Album id")),
    responses(
        (status = 200, description = "Scored release candidates", body = [AlbumMatchEntry]),
        (status = 403, description = "Admin access required"),
    ),
    security(("bearer_token" = []))
)]
pub async fn get_album_matches(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    AdminUser { .. }: AdminUser,
) -> Result<Json<Vec<AlbumMatchEntry>>, (StatusCode, String)> {
    let rows = sqlx::query!(
        "SELECT release_mbid, release_group_mbid, score, payload FROM album_match_candidate \
         WHERE album = $1 ORDER BY score DESC",
        id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(
        rows.into_iter()
            .map(|r| AlbumMatchEntry {
                release_mbid: r.release_mbid,
                release_group_mbid: r.release_group_mbid,
                score: r.score,
                payload: r.payload,
            })
            .collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/albums/{id}/match",
    tag = "admin",
    params(("id" = i32, Path, description = "Album id")),
    request_body = ApplyMatchRequest,
    responses(
        (status = 200, description = "Match applied", body = RescanResponse),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "Candidate not found"),
    ),
    security(("bearer_token" = []))
)]
pub async fn post_apply_match(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    AdminUser { .. }: AdminUser,
    Json(body): Json<ApplyMatchRequest>,
) -> Result<(StatusCode, Json<RescanResponse>), (StatusCode, String)> {
    let candidate = sqlx::query!(
        "SELECT release_mbid, release_group_mbid FROM album_match_candidate WHERE album = $1 AND release_mbid = $2",
        id,
        body.release_mbid
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "no such candidate".to_string()))?;

    let detail = crate::metadata::musicbrainz::get_release(&candidate.release_mbid)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    crate::index::db::apply_release_match(
        id,
        &candidate.release_mbid,
        candidate.release_group_mbid.as_deref(),
        &detail,
        &pool,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::OK, Json(RescanResponse { status: "matched" })))
}


#[derive(Serialize, utoipa::ToSchema)]
pub struct IdentityReviewEntry {
    pub id: i32,
    pub kind: String,
    pub target: i32,
    pub detail: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/identity-reviews",
    tag = "admin",
    responses(
        (status = 200, description = "Unresolved identity conflicts", body = [IdentityReviewEntry]),
        (status = 403, description = "Admin access required"),
    ),
    security(("bearer_token" = []))
)]
pub async fn get_identity_reviews(
    Extension(pool): Extension<PgPool>,
    AdminUser { .. }: AdminUser,
) -> Result<Json<Vec<IdentityReviewEntry>>, (StatusCode, String)> {
    let rows = sqlx::query!(
        "SELECT id, kind, target, detail FROM identity_review ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(
        rows.into_iter()
            .map(|r| IdentityReviewEntry {
                id: r.id,
                kind: r.kind,
                target: r.target,
                detail: r.detail,
            })
            .collect(),
    ))
}


#[utoipa::path(
    post,
    path = "/api/v1/admin/identity-reviews/{id}/resolve",
    tag = "admin",
    params(("id" = i32, Path, description = "Review id")),
    responses(
        (status = 202, description = "Review dismissed", body = RescanResponse),
        (status = 403, description = "Admin access required"),
    ),
    security(("bearer_token" = []))
)]
pub async fn post_resolve_identity_review(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    AdminUser { .. }: AdminUser,
) -> Result<(StatusCode, Json<RescanResponse>), (StatusCode, String)> {
    sqlx::query!("DELETE FROM identity_review WHERE id = $1", id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((
        StatusCode::ACCEPTED,
        Json(RescanResponse { status: "resolved" }),
    ))
}
