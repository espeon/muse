use api::auth::providers::{create_shared_auth_provider, generic::GenericOidcPkceProvider};
use axum::{
    http::{self, HeaderValue, Method},
    routing::get,
    Extension, Router,
};
use clap::{Parser, Subcommand};
use dashmap::DashMap;
use sqlx::{postgres::Postgres, Pool};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tracing::info;

use tower_http::cors::CorsLayer;

pub struct HlsState {
    pub profiles: Vec<config::HlsProfile>,
    pub in_flight: DashMap<i32, Arc<tokio::sync::Mutex<()>>>,
    pub last_access: DashMap<i32, std::time::Instant>,
    pub max_cache_bytes: u64,
}

mod analysis;
mod api;
pub mod clients;
mod config;
mod db;
mod enrich;
mod error;
mod helpers;
mod index;
mod match_album;
mod metadata;
mod web;

#[derive(Parser)]
#[command(name = "kyoku", about = "Maki music server")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Read and display all tags for a given audio file
    Tags {
        /// Path to the audio file
        path: PathBuf,
    },
    /// Analyze a bounded number of tracks without starting the web server
    Analyze {
        /// Maximum tracks to analyze in this run
        #[arg(long, default_value_t = 10)]
        limit: usize,
        /// Analyze only this track ID (repeat the flag for more tracks)
        #[arg(long = "track-id")]
        track_ids: Vec<i32>,
        /// Retry tracks whose last analysis failed
        #[arg(long)]
        retry_failures: bool,
        /// Analyzer pass to run
        #[arg(long, value_enum, default_value = "all")]
        kind: analysis::AnalysisKind,
        /// Remove cached assets no song currently references
        #[arg(long)]
        prune_orphaned_assets: bool,
    },
    /// Recompute normalized search keys for all existing rows
    ReindexSearch,
    /// Explicitly accept a library root as trusted (required after a root change)
    SetLibraryRoot {
        #[arg(long)]
        path: PathBuf,
    },
}

/// Command that runs without starting the HTTP server.
enum EarlyCommand {
    Analyze(
        usize,
        Vec<i32>,
        bool,
        analysis::AnalysisKind,
        bool,
    ),
    ReindexSearch,
    SetLibraryRoot(PathBuf),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    let early = match cli.command {
        Some(Command::Tags { path }) => return cmd_tags(&path).await,
        Some(Command::Analyze {
            limit,
            track_ids,
            retry_failures,
            kind,
            prune_orphaned_assets,
        }) => Some(EarlyCommand::Analyze(
            limit,
            track_ids,
            retry_failures,
            kind,
            prune_orphaned_assets,
        )),
        Some(Command::ReindexSearch) => Some(EarlyCommand::ReindexSearch),
        Some(Command::SetLibraryRoot { path }) => Some(EarlyCommand::SetLibraryRoot(path)),
        None => None,
    };

    tracing_subscriber::fmt::init();

    let cfg = config::load_or_create_config("config/config.maki.json");

    let path = if early.is_none() {
        Some(std::env::var("MOUNT")?)
    } else {
        None
    };

    let pool = db::get_pool().await?;
    if let Ok(stale) = sqlx::query_scalar!(
        "SELECT (SELECT count(*) FROM artist WHERE search_key_version < 1) + \
                (SELECT count(*) FROM album WHERE search_key_version < 1) + \
                (SELECT count(*) FROM song WHERE search_key_version < 1)"
    )
    .fetch_one(&pool)
    .await
    {
        let stale: i64 = stale.unwrap_or(0);
        if stale > 0 {
            tracing::warn!("{stale} row(s) have approximate search keys; run `maki reindex-search`");
        }
    }
    match early {
        Some(EarlyCommand::SetLibraryRoot(path)) => {
            let canonical = std::fs::canonicalize(&path)?
                .to_string_lossy()
                .into_owned();
            index::db::set_library_root(&pool, &canonical).await?;
            println!(
                "library root set to {canonical}; run a full scan next (accepting the root does not rescan or prune)"
            );
            return Ok(());
        }
        Some(EarlyCommand::ReindexSearch) => {
            let (artists, albums, songs) = index::db::reindex_search_keys(&pool).await?;
            println!("reindexed {artists} artist(s), {albums} album(s), {songs} song(s)");
            return Ok(());
        }
        Some(EarlyCommand::Analyze(
            limit,
            track_ids,
            retry_failures,
            kind,
            prune_orphaned_assets,
        )) => {
            if prune_orphaned_assets {
                let pruned = analysis::prune_orphaned_assets(&pool).await?;
                println!("pruned {pruned} orphaned audio asset(s)");
            } else {
                analysis::run_now(pool, cfg, kind, retry_failures, limit, &track_ids).await?;
            }
            return Ok(());
        }
        None => {}
    }
    let path = path.expect("MOUNT is required when serving Maki");
    let p_cloned = pool.clone();

    // detect dry run flag (overrides NO_SCAN)
    let dry_run = std::env::var("DRY_RUN").is_ok();
    // detect no scan flag
    let no_scan = std::env::var("NO_SCAN").is_ok();

    if cfg.enrichment_enabled && !dry_run {
        enrich::spawn(pool.clone(), &cfg);
    }

    // start up our web server
    // dunno if i want this in a separate thread or not
    if !dry_run && !no_scan {
        let cfg_for_index = cfg.clone();
        // start indexing/scanning in new thread
        tokio::spawn(async move {
            index::start(path.clone(), &path, p_cloned, dry_run, &cfg_for_index).await;
        });
    } else if dry_run {
        index::start(path.clone(), &path, p_cloned, dry_run, &cfg).await;
        return Ok(());
    }
    serve(pool, cfg).await?;

    Ok(())
}

/// The route table, without the per-request extension layers.
///
/// Split out of [`serve`] so it can be built in a test: axum validates route path syntax
/// at construction time and panics on malformed paths, which the compiler cannot catch.
fn routes() -> Router {
    Router::new()
        .route("/", get(web::spa_fallback))
        .nest("/api/v1", api::router())
        .nest("/auth", api::auth::router())
        .route("/assets/{*path}", get(web::static_handler))
        .route("/favicon.ico", get(web::static_handler))
        .route("/favicon.svg", get(web::static_handler))
        .route("/manifest.webmanifest", get(web::static_handler))
        .route("/apple-touch-icon.png", get(web::static_handler))
        .fallback(web::spa_fallback)
}

async fn serve(pool: Pool<Postgres>, cfg: config::Config) -> anyhow::Result<()> {
    let userinfo_url = std::env::var("OIDC_USERINFO_URL").ok();
    let auth = GenericOidcPkceProvider::new(
        reqwest::Client::new(),
        std::env::var("OIDC_CLIENT_ID").unwrap(),
        std::env::var("OIDC_CLIENT_SECRET").unwrap(),
        std::env::var("OIDC_AUTHORIZE").unwrap(),
        std::env::var("OIDC_ISSUER").unwrap(),
        std::env::var("OIDC_TOKEN").unwrap(),
        std::env::var("OIDC_REDIRECT").unwrap(),
        userinfo_url,
    )
    .await?;
    let authcfg = create_shared_auth_provider(auth);
    let spa_origins = api::auth::login::SpaOrigins::parse(
        &std::env::var("WEB_ORIGINS").unwrap_or_default(),
    )?;
    let hls_state = Arc::new(HlsState {
        profiles: cfg.hls_profiles.clone(),
        in_flight: DashMap::new(),
        last_access: DashMap::new(),
        max_cache_bytes: cfg.hls_max_cache_bytes,
    });

    {
        let sweep_state = hls_state.clone();
        tokio::spawn(async move {
            hls_cache_sweep(sweep_state).await;
        });
    }

    let remote_hub = Arc::new(api::remote::hub::Hub::new());
    let _remote_sweeper = remote_hub.start_sweeper();

    // build our application with a route
    let app = routes()
        .layer(Extension(pool))
        .layer(Extension(cfg.clone()))
        .layer(Extension(authcfg))
        .layer(Extension(spa_origins))
        .layer(Extension(hls_state))
        .layer(Extension(remote_hub))
        .layer(
            CorsLayer::new()
                .allow_origin("*".parse::<HeaderValue>().unwrap())
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION]),
        );

    // run it
    let port = std::env::var("MAKI_PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3033);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {}", &addr));
    info!("Maki is listening on {}", addr);
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn cmd_tags(path: &std::path::Path) -> anyhow::Result<()> {
    use metadata::{
        formats::{aiff::scan_aiff, flac::scan_flac, mp3::scan_mp3, wav::scan_wav},
        get_filetype, AudioFormat,
    };

    let path_buf = path.to_path_buf();
    let format = get_filetype(path)
        .ok_or_else(|| anyhow::anyhow!("unsupported or unrecognized file format"))?;

    let cfg = config::Config {
        artist_split_exceptions: vec![],
        ..config::Config::default()
    };

    let meta = match format {
        AudioFormat::Flac => scan_flac(&path_buf, &cfg).await?,
        AudioFormat::Mp3 => scan_mp3(&path_buf, &cfg).await?,
        AudioFormat::Wav => scan_wav(&path_buf, &cfg).await?,
        AudioFormat::Aiff => scan_aiff(&path_buf, &cfg).await?,
    };

    println!("file:          {}", meta.path.display());
    println!("title:         {}", meta.name);
    println!("track:         {}", meta.number);
    if let Some(disc) = meta.disc {
        println!("disc:          {}", disc);
    }
    println!("album:         {}", meta.album);
    println!("album artist:  {}", meta.album_artist);
    println!("artists:       {}", meta.artists.join(", "));
    if let Some(sort) = &meta.album_sort {
        println!("album sort:    {}", sort);
    }
    if let Some(year) = meta.year {
        println!("year:          {}", year);
    }
    if let Some(genres) = &meta.genre {
        println!("genre:         {}", genres.join(", "));
    }
    println!("duration:      {}s", meta.duration);
    println!("lossless:      {}", meta.lossless);
    if let Some(sr) = meta.sample_rate {
        println!("sample rate:   {} Hz", sr);
    }
    if let Some(bps) = meta.bits_per_sample {
        println!("bit depth:     {}-bit", bps);
    }
    if let Some(ch) = meta.num_channels {
        println!("channels:      {}", ch);
    }
    if let Some(id) = &meta.mbid_track {
        println!("mbid track:    {}", id);
    }
    if let Some(id) = &meta.mbid_album {
        println!("mbid album:    {}", id);
    }
    if let Some(id) = &meta.mbid_artist {
        println!("mbid artist:   {}", id);
    }
    if let Some(composer) = &meta.composer {
        println!("composer:      {}", composer);
    }
    if let Some(isrc) = &meta.isrc {
        println!("isrc:          {}", isrc);
    }
    if let Some(bpm) = meta.bpm {
        println!("bpm:           {}", bpm);
    }
    if let Some(copyright) = &meta.copyright {
        println!("copyright:     {}", copyright);
    }
    if let Some(label) = &meta.label {
        println!("label:         {}", label);
    }
    if let Some(genre) = &meta.genre {
        println!("genre:         {:?}", genre);
    }
    if !meta.picture.is_empty() {
        for pic in &meta.picture {
            println!(
                "picture:       {} ({} bytes)",
                pic.picture_type,
                pic.bytes.len()
            );
        }
    }

    Ok(())
}

const HLS_CACHE_BASE: &str = "/tmp/co.lutea.maki/hls";
const SWEEP_INTERVAL: std::time::Duration = std::time::Duration::from_secs(300);

async fn hls_cache_sweep(state: Arc<HlsState>) {
    loop {
        tokio::time::sleep(SWEEP_INTERVAL).await;

        let total_size = match dir_size(HLS_CACHE_BASE).await {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("hls cache sweep: failed to measure cache size: {}", e);
                continue;
            }
        };

        if total_size <= state.max_cache_bytes {
            continue;
        }

        tracing::info!(
            "hls cache sweep: {} bytes used, limit is {}. pruning...",
            total_size,
            state.max_cache_bytes
        );

        let mut entries: Vec<(i32, std::time::Instant)> = state
            .last_access
            .iter()
            .map(|r| (*r.key(), *r.value()))
            .collect();
        entries.sort_by_key(|(_, t)| *t);

        let mut freed = 0u64;
        let target = total_size.saturating_sub(state.max_cache_bytes);

        for (song_id, _) in entries {
            if freed >= target {
                break;
            }
            let dir = std::path::PathBuf::from(format!("{}/{}", HLS_CACHE_BASE, song_id));
            match dir_size(dir.to_string_lossy().as_ref()).await {
                Ok(size) => {
                    if tokio::fs::remove_dir_all(&dir).await.is_ok() {
                        state.last_access.remove(&song_id);
                        state.in_flight.remove(&song_id);
                        freed += size;
                        tracing::info!("hls cache sweep: evicted song {} ({} bytes)", song_id, size);
                    }
                }
                Err(_) => {
                    state.last_access.remove(&song_id);
                }
            }
        }

        tracing::info!("hls cache sweep: freed {} bytes", freed);
    }
}

fn dir_size(path: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<u64>> + Send + '_>> {
    Box::pin(async move {
        let mut total: u64 = 0;
        let mut entries = tokio::fs::read_dir(path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let meta = entry.metadata().await?;
            if meta.is_dir() {
                total += dir_size(&entry.path().to_string_lossy()).await?;
            } else {
                total += meta.len();
            }
        }
        Ok(total)
    })
}

#[cfg(test)]
mod cli_tests {
    use clap::Parser;

    use super::{analysis::AnalysisKind, Cli, Command};

    #[test]
    fn analyze_accepts_repeated_track_ids() {
        let cli = Cli::try_parse_from([
            "kyoku",
            "analyze",
            "--kind",
            "mix",
            "--track-id",
            "7",
            "--track-id",
            "11",
        ])
        .unwrap();
        let Some(Command::Analyze {
            track_ids, kind, ..
        }) = cli.command
        else {
            panic!("expected analyze command");
        };

        assert_eq!(track_ids, vec![7, 11]);
        assert!(matches!(kind, AnalysisKind::Mix));
    }
}

#[cfg(test)]
mod route_tests {
    /// axum panics while building a router if a path uses malformed syntax, so this
    /// catches route-table mistakes (e.g. 0.7-style `/track/:id`) that compile fine.
    #[test]
    fn route_table_builds() {
        let _ = super::routes();
    }
}
