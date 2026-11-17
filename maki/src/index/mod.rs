pub mod db;
pub mod folder;

use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};

use jwalk::WalkDir;
use notify::event::EventKind;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use sqlx::{postgres::Postgres, PgPool};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use time::OffsetDateTime;
use tracing::{error, info, warn};

use crate::{
    analysis,
    config::Config,
    metadata::{self, AudioMetadata},
};

/// Refuse to buffer an absurd folder instead of holding the whole library in memory.
const MAX_CANDIDATE_TRACKS: usize = 500;

pub async fn start<P: AsRef<Path>>(
    path: P,
    pathstr: &str,
    pool: sqlx::Pool<Postgres>,
    dry_run: bool,
    cfg: &Config,
) {
    if Path::new(&pathstr).is_file() {
        return error!(
                target: "index",
            "critical error !!!!!\nthe path {} is not a folder.",
            &pathstr
        );
    }
    info!(target: "index", "scanning folder {:?}", &pathstr);
    scan(&path, pool.clone(), dry_run, cfg).await;
    info!(target: "index", "watching folder {:?}", pathstr);
    if let Err(e) = watch(path, pool, dry_run, cfg).await {
        error!(target: "index", "error: {:?}", e)
    }
}

pub async fn scan<P: AsRef<Path>>(
    path: P,
    pool: sqlx::Pool<Postgres>,
    dry_run: bool,
    cfg: &Config,
) {
    tokio::time::sleep(Duration::from_millis(250)).await;
    let scan_start = OffsetDateTime::now_utc();
    let root = path.as_ref();
    let stats = scan_tree(root, root, &pool, cfg, dry_run).await;
    if dry_run {
        return;
    }

    let root_key = std::fs::canonicalize(root)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    let known_root = sqlx::query_scalar!("SELECT value FROM index_state WHERE key = 'library_root'")
        .fetch_optional(&pool)
        .await;
    let total_songs = db::song_count(&pool).await.unwrap_or(0);

    let mut skip_detail: Option<String> = None;
    match &known_root {
        Err(e) => {
            warn!(target: "index", "cannot read library root guard: {e}; skipping prune");
            skip_detail = Some(format!("guard unreadable: {e}"));
        }
        Ok(None) if total_songs > 0 => {
            // Never infer the root from a scan of a populated DB: a subtree scan would poison it.
            warn!(target: "index", "library root is not established on a populated DB; skipping prune");
            skip_detail = Some(format!(
                "library root not set; run `maki set-library-root --path {root_key}` to accept this root"
            ));
        }
        Ok(None) => {
            if stats.audio_files_seen > 0 {
                if let Err(e) = db::set_library_root(&pool, &root_key).await {
                    warn!(target: "index", "failed to record library root: {e}");
                }
            }
        }
        Ok(Some(known)) if known != &root_key => {
            warn!(target: "index", "scan root changed since the last accepted scan; skipping prune");
            skip_detail = Some(format!(
                "root changed from {known} to {root_key}; run `maki set-library-root --path {root_key}` to accept it"
            ));
        }
        Ok(Some(_)) => {}
    }

    if skip_detail.is_none() && !should_prune(stats.audio_files_seen) {
        warn!(target: "index", "skipping stale prune: scan found no audio files");
        skip_detail = Some("scan found no audio files".to_string());
    }

    let excluded: Vec<String> = stats.failed_folders.into_iter().collect();
    if let Some(detail) = skip_detail {
        db::record_prune(&pool, "skipped", &detail).await;
    } else {
        match db::delete_stale_songs(scan_start, &pool, cfg.max_prune_fraction, &excluded).await {
            Ok(n) => {
                info!(target: "index", "pruned {n} stale song(s)");
                db::record_prune(
                    &pool,
                    "pruned",
                    &format!("pruned {n} of {total_songs} songs; {} folder(s) held back", excluded.len()),
                )
                .await;
            }
            Err(e) => {
                warn!(target: "index", "stale prune rejected: {e}");
                db::record_prune(&pool, "refused", &e.to_string()).await;
            }
        }
    }

    analysis::enqueue(pool.clone(), cfg.clone(), false);
}


#[derive(Debug, Default)]
struct ScanStats {
    failures: usize,
    files_seen: usize,
    audio_files_seen: usize,
    failed_folders: std::collections::HashSet<String>,
}

/// Walk `walk_root` and group files into album candidates by their resolved album folder.
/// The whole folder is read before any of its rows are written.
async fn scan_tree(
    root: &Path,
    walk_root: &Path,
    pool: &PgPool,
    cfg: &Config,
    dry_run: bool,
) -> ScanStats {
    let mut stats = ScanStats::default();
    let mut current: Option<(folder::AlbumKey, Vec<AudioMetadata>, bool)> = None;

    for entry in WalkDir::new(walk_root).sort(true) {
        let ent = match entry {
            Ok(ent) => ent,
            Err(e) => {
                error!(target: "index", "directory walk failed: {}", e);
                stats.failures += 1;
                continue;
            }
        };
        if !ent.path().is_file() {
            continue;
        }
        stats.files_seen += 1;

        let key = folder::album_key(&ent.path(), root);
        if current.as_ref().map_or(true, |(k, _, _)| *k != key) {
            if let Some((old_key, tracks, complete)) = current.take() {
                stats.failures +=
                    flush_candidate(old_key, tracks, Some(complete), pool, dry_run).await;
            }
            current = Some((key.clone(), Vec::new(), true));
        }
        let (_, tracks, complete) = current.as_mut().expect("candidate just set");
        if tracks.len() >= MAX_CANDIDATE_TRACKS {
            if *complete {
                warn!(target: "index", "folder {} exceeds {} tracks; remainder skipped", key.folder.display(), MAX_CANDIDATE_TRACKS);
            }
            *complete = false;
            stats.failures += 1;
            stats.failed_folders.insert(key.folder.to_string_lossy().into_owned());
            continue;
        }
        match metadata::scan_file(&ent.path(), cfg, Some(&key.folder)).await {
            Ok(Some(meta)) => {
                stats.audio_files_seen += 1;
                tracks.push(meta);
            }
            Ok(None) => {}
            Err(e) => {
                error!(target: "index", "skipping {}: {}", ent.path().display(), e);
                *complete = false;
                stats.failures += 1;
                stats.failed_folders.insert(key.folder.to_string_lossy().into_owned());
            }
        }
    }
    if let Some((key, tracks, complete)) = current.take() {
        stats.failures += flush_candidate(key, tracks, Some(complete), pool, dry_run).await;
    }
    stats
}

async fn flush_candidate(
    key: folder::AlbumKey,
    tracks: Vec<AudioMetadata>,
    complete: Option<bool>,
    pool: &PgPool,
    dry_run: bool,
) -> usize {
    if dry_run {
        info!(target: "index", "dry run: album candidate {} ({} track(s))", key.folder.display(), tracks.len());
        return 0;
    }
    let candidate = folder::Candidate { key, tracks, complete };
    match db::save_candidate(candidate, pool.clone()).await {
        Ok(()) => 0,
        Err(e) => {
            error!(target: "index", "failed to save album candidate: {}", e);
            1
        }
    }
}

fn should_prune(audio_files_seen: usize) -> bool {
    audio_files_seen > 0
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    let config = notify::Config::default()
        .with_poll_interval(Duration::from_secs(30))
        .with_compare_contents(true);

    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        config,
    )?;

    Ok((watcher, rx))
}

pub async fn watch<P: AsRef<Path>>(
    path: P,
    pool: sqlx::Pool<Postgres>,
    dry_run: bool,
    cfg: &Config,
) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => parse_event(event, pool.clone(), dry_run, cfg).await,
            Err(e) => error!(target: "index-watcher", "watch error: {:?}", e),
        }
    }
    Ok(())
}

/// Wait until a file's size and mtime stop changing. The old fixed 75 ms sleep was not a
/// completeness guarantee; a large copy could be parsed halfway through.
async fn wait_for_stable(path: &Path) -> bool {
    let mut previous = None;
    for _ in 0..10 {
        match std::fs::metadata(path) {
            Ok(meta) => {
                let signature = (meta.len(), meta.modified().ok());
                if previous == Some(signature) {
                    return true;
                }
                previous = Some(signature);
            }
            Err(_) => previous = None,
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    false
}

async fn parse_event(
    event: notify::event::Event,
    pool: sqlx::Pool<Postgres>,
    dry_run: bool,
    cfg: &Config,
) {
    let root = PathBuf::from(std::env::var("MOUNT").unwrap_or_default());
    for path in &event.paths {
        match &event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                if path.is_dir() {
                    scan_tree(&root, path, &pool, cfg, dry_run).await;
                    continue;
                }
                if !wait_for_stable(path).await {
                    warn!(target: "index-watcher", "{} never settled; skipping", path.display());
                    continue;
                }
                if matches!(&event.kind, EventKind::Modify(_)) && !dry_run {
                    let p = path.to_str().unwrap_or_default();
                    if let Err(e) = db::invalidate_features_by_path(p, &pool).await {
                        error!(target: "index-watcher", "failed to invalidate features for {p}: {e}");
                    }
                }
                if dry_run {
                    continue;
                }
                let key = folder::album_key(path, &root);
                match metadata::scan_file(&path.to_path_buf(), cfg, Some(&key.folder)).await {
                    Ok(Some(meta)) => {
                        let candidate = folder::Candidate {
                            key,
                            tracks: vec![meta],
                            // One file is not evidence of a complete album tracklist.
                            complete: None,
                        };
                        if let Err(e) = db::save_candidate(candidate, pool.clone()).await {
                            error!(target: "index-watcher", "failed to save {}: {}", path.display(), e);
                        }
                    }
                    Ok(None) => {}
                    Err(e) => error!(target: "index-watcher", "failed to scan {}: {}", path.display(), e),
                }
            }
            EventKind::Remove(remove_kind) => {
                if dry_run {
                    info!(target: "index-watcher", "dry-run: would remove {}", path.display());
                    continue;
                }
                let p = path.to_str().unwrap_or_default();
                // The directory no longer exists when the event is handled; trust the event kind.
                let result = if matches!(remove_kind, notify::event::RemoveKind::Folder) {
                    db::delete_songs_under(p, &pool).await
                } else {
                    db::delete_song_by_path(p, &pool).await
                };
                if let Err(e) = result {
                    error!(target: "index-watcher", "failed to delete {}: {}", path.display(), e);
                }
            }
            EventKind::Access(_) | EventKind::Any | EventKind::Other => {}
        }
    }
    if !dry_run {
        analysis::enqueue(pool, cfg.clone(), false);
    }
}

#[cfg(test)]
mod tests {
    use super::should_prune;

    #[test]
    fn an_empty_scan_blocks_stale_pruning() {
        assert!(should_prune(1));
        assert!(!should_prune(0));
    }
}
