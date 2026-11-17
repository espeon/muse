//! Content-addressed audio-analysis orchestration.
//!
//! Tags and paths identify a library entry, while analysis belongs to immutable
//! source bytes. The background runner hashes files before invoking either
//! optional analyzer so a moved file can reuse its existing results.

use std::{
    convert::TryInto,
    path::Path,
    process::Stdio,
    sync::{Arc, OnceLock},
    time::{Duration, UNIX_EPOCH},
};

use dashmap::DashMap;
use futures::{stream::FuturesUnordered, StreamExt};
use serde::Deserialize;
use serde_json::Value;
use sqlx::{postgres::PgPool, types::Json, FromRow, Row};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
    sync::Mutex,
};
use tracing::{error, info, warn};

use crate::config::Config;

/// Must be bumped alongside a breaking extractor change in maki-analyzer.
pub const FEATURE_VERSION: i32 = 2;
pub const MIX_PROFILE_VERSION: i32 = 1;
const MAX_VECTOR_DIMENSIONS: usize = 512;
const HASH_BUFFER_BYTES: usize = 1024 * 1024;

static RUNNER: OnceLock<Mutex<()>> = OnceLock::new();
static ASSET_RUNNERS: OnceLock<DashMap<Vec<u8>, Arc<Mutex<()>>>> = OnceLock::new();

#[derive(Clone, Copy, Debug, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum AnalysisKind {
    Similarity,
    Mix,
    All,
}

impl AnalysisKind {
    fn includes_similarity(self) -> bool {
        matches!(self, Self::Similarity | Self::All)
    }

    fn includes_mix(self) -> bool {
        matches!(self, Self::Mix | Self::All)
    }
}

#[derive(FromRow)]
struct PendingSong {
    id: i32,
    path: String,
    audio_hash: Option<Vec<u8>>,
    audio_hash_size: Option<i64>,
    audio_hash_mtime_ns: Option<i64>,
}

struct AudioAsset {
    hash: Vec<u8>,
}

#[derive(Deserialize)]
struct WorkerOutput {
    version: i32,
    vector: Vec<f32>,
}

#[derive(Deserialize)]
struct MixSidecarResponse {
    schema_version: i32,
    provider: String,
    provider_version: String,
    profile: Value,
}

struct MixProfile {
    provider: String,
    provider_version: String,
    profile: Value,
}

/// Queue analysis without delaying the metadata scanner or HTTP server.
pub fn enqueue(pool: PgPool, config: Config, retry_failures: bool) {
    enqueue_kind(pool, config, AnalysisKind::All, retry_failures, Vec::new());
}

pub fn enqueue_kind(
    pool: PgPool,
    config: Config,
    kind: AnalysisKind,
    retry_failures: bool,
    song_ids: Vec<i32>,
) {
    let has_enabled_phase = (kind.includes_similarity() && config.audio_analysis_enabled)
        || (kind.includes_mix() && config.mix_analysis_enabled);
    if !has_enabled_phase {
        return;
    }

    tokio::spawn(async move {
        let lock = RUNNER.get_or_init(|| Mutex::new(())).lock().await;
        if let Err(err) = run(pool, config, kind, retry_failures, None, &song_ids).await {
            error!(target: "analysis", "audio analysis stopped: {err:#}");
        }
        drop(lock);
    });
}

/// Run a bounded analysis pass synchronously. Normal indexing uses [`enqueue`].
pub async fn run_now(
    pool: PgPool,
    config: Config,
    kind: AnalysisKind,
    retry_failures: bool,
    limit: usize,
    song_ids: &[i32],
) -> anyhow::Result<()> {
    let lock = RUNNER.get_or_init(|| Mutex::new(())).lock().await;
    let result = run(
        pool,
        config,
        kind,
        retry_failures,
        Some(limit.max(1)),
        song_ids,
    )
    .await;
    drop(lock);
    result
}

pub async fn prune_orphaned_assets(pool: &PgPool) -> anyhow::Result<u64> {
    Ok(sqlx::query(
        "DELETE FROM audio_assets asset WHERE NOT EXISTS (SELECT 1 FROM song WHERE song.audio_hash = asset.hash)",
    )
    .execute(pool)
    .await?
    .rows_affected())
}

async fn run(
    pool: PgPool,
    config: Config,
    kind: AnalysisKind,
    retry_failures: bool,
    limit: Option<usize>,
    song_ids: &[i32],
) -> anyhow::Result<()> {
    if kind.includes_similarity() && config.audio_analysis_enabled {
        run_similarity_phase(&pool, &config, retry_failures, limit, song_ids).await?;
    }
    if kind.includes_mix() && config.mix_analysis_enabled {
        run_mix_phase(&pool, &config, retry_failures, limit, song_ids).await?;
    }
    Ok(())
}

async fn run_similarity_phase(
    pool: &PgPool,
    config: &Config,
    retry_failures: bool,
    limit: Option<usize>,
    song_ids: &[i32],
) -> anyhow::Result<()> {
    if config.audio_analysis_command.is_empty() {
        warn!(target: "analysis", "similarity analysis is enabled but audio_analysis_command is empty");
        return Ok(());
    }
    if retry_failures {
        clear_failures(pool, "similarity", FEATURE_VERSION, song_ids).await?;
    }

    let pending = pending_songs(pool, "similarity", FEATURE_VERSION, limit, song_ids).await?;
    if pending.is_empty() {
        refresh_stats(pool).await?;
        return Ok(());
    }
    info!(target: "analysis", "queueing {} song(s) for similarity analysis", pending.len());
    run_bounded(pool, config, pending, AnalysisKind::Similarity).await?;
    refresh_stats(pool).await?;
    Ok(())
}

async fn run_mix_phase(
    pool: &PgPool,
    config: &Config,
    retry_failures: bool,
    limit: Option<usize>,
    song_ids: &[i32],
) -> anyhow::Result<()> {
    if config
        .mix_analysis_url
        .as_deref()
        .unwrap_or_default()
        .is_empty()
    {
        warn!(target: "analysis", "mix analysis is enabled but mix_analysis_url is empty");
        return Ok(());
    }
    if retry_failures {
        clear_failures(pool, "mix", MIX_PROFILE_VERSION, song_ids).await?;
    }

    let pending = pending_songs(pool, "mix", MIX_PROFILE_VERSION, limit, song_ids).await?;
    if pending.is_empty() {
        return Ok(());
    }
    info!(target: "analysis", "queueing {} song(s) for mix analysis", pending.len());
    run_bounded(pool, config, pending, AnalysisKind::Mix).await
}

async fn clear_failures(
    pool: &PgPool,
    analyzer: &str,
    version: i32,
    song_ids: &[i32],
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        DELETE FROM audio_analysis_failures failure
        WHERE analyzer = $1 AND version = $2
          AND (
            cardinality($3::int4[]) = 0
            OR EXISTS (
              SELECT 1 FROM song
              WHERE song.audio_hash = failure.audio_hash AND song.id = ANY($3)
            )
          )
        "#,
    )
        .bind(analyzer)
        .bind(version)
        .bind(song_ids)
        .execute(pool)
        .await?;
    sqlx::query(
        "DELETE FROM song_hash_failures WHERE cardinality($1::int4[]) = 0 OR song = ANY($1)",
    )
        .bind(song_ids)
        .execute(pool)
        .await?;
    Ok(())
}

async fn pending_songs(
    pool: &PgPool,
    analyzer: &str,
    version: i32,
    limit: Option<usize>,
    song_ids: &[i32],
) -> anyhow::Result<Vec<PendingSong>> {
    let feature_exists = match analyzer {
        "similarity" => "EXISTS (SELECT 1 FROM audio_similarity_features feature WHERE feature.audio_hash = song.audio_hash AND feature.version = $2)",
        "mix" => "EXISTS (SELECT 1 FROM audio_mix_profiles profile WHERE profile.audio_hash = song.audio_hash AND profile.version = $2)",
        _ => unreachable!("unknown analyzer"),
    };
    let query = format!(
        r#"
        SELECT song.id, song.path, song.audio_hash, song.audio_hash_size, song.audio_hash_mtime_ns
        FROM song
        WHERE NOT EXISTS (SELECT 1 FROM song_hash_failures hash_failure WHERE hash_failure.song = song.id)
          AND (cardinality($4::int4[]) = 0 OR song.id = ANY($4))
          AND (
            song.audio_hash IS NULL
            OR (NOT {feature_exists}
                AND NOT EXISTS (
                  SELECT 1 FROM audio_analysis_failures failure
                  WHERE failure.audio_hash = song.audio_hash
                    AND failure.analyzer = $1
                    AND failure.version = $2
                ))
          )
        ORDER BY song.id
        LIMIT $3
        "#,
    );
    // `feature_exists` is one of the static fragments above and every other value is a
    // bind parameter, so this string cannot contain user input. sqlx requires the
    // assertion because the compiler cannot see that.
    Ok(sqlx::query_as::<_, PendingSong>(sqlx::AssertSqlSafe(query))
        .bind(analyzer)
        .bind(version)
        .bind(limit.map(|value| value as i64))
        .bind(song_ids)
        .fetch_all(pool)
        .await?)
}

async fn run_bounded(
    pool: &PgPool,
    config: &Config,
    pending: Vec<PendingSong>,
    kind: AnalysisKind,
) -> anyhow::Result<()> {
    let concurrency = config.audio_analysis_threads.max(1) as usize;
    let mut running = FuturesUnordered::new();
    let mut processed = 0usize;
    let mut failed = 0usize;

    for song in pending {
        running.push(process_song(pool.clone(), config.clone(), song, kind));
        if running.len() >= concurrency {
            record_phase_result(
                running.next().await.expect("non-empty worker set"),
                &mut processed,
                &mut failed,
            );
        }
    }
    while let Some(result) = running.next().await {
        record_phase_result(result, &mut processed, &mut failed);
    }
    info!(target: "analysis", "{} analysis finished: {processed} processed, {failed} failed", analyzer_name(kind));
    Ok(())
}

fn record_phase_result(result: Result<(), String>, processed: &mut usize, failed: &mut usize) {
    match result {
        Ok(()) => *processed += 1,
        Err(message) => {
            *failed += 1;
            warn!(target: "analysis", "analysis failed: {message}");
        }
    }
}

fn analyzer_name(kind: AnalysisKind) -> &'static str {
    match kind {
        AnalysisKind::Similarity => "similarity",
        AnalysisKind::Mix => "mix",
        AnalysisKind::All => "all",
    }
}

async fn process_song(
    pool: PgPool,
    config: Config,
    song: PendingSong,
    kind: AnalysisKind,
) -> Result<(), String> {
    let asset = ensure_asset(&pool, &song).await?;
    match kind {
        AnalysisKind::Similarity => process_similarity(&pool, &config, &song, &asset).await,
        AnalysisKind::Mix => process_mix(&pool, &config, &song, &asset).await,
        AnalysisKind::All => unreachable!("all is expanded into individual phases"),
    }
}

async fn ensure_asset(pool: &PgPool, song: &PendingSong) -> Result<AudioAsset, String> {
    let before = file_signature(Path::new(&song.path))
        .await
        .map_err(|error| format!("could not inspect {}: {error}", song.path))?;
    if let Some(hash) = &song.audio_hash {
        if song.audio_hash_size == Some(before.0 as i64)
            && song.audio_hash_mtime_ns == Some(before.1)
        {
            return Ok(AudioAsset { hash: hash.clone() });
        }
    }
    let (hash, byte_size) = hash_file(Path::new(&song.path))
        .await
        .map_err(|error| format!("could not hash {}: {error}", song.path))?;
    let after = file_signature(Path::new(&song.path))
        .await
        .map_err(|error| format!("could not re-check {}: {error}", song.path))?;
    if before != after || byte_size != after.0 {
        let message = format!("file changed while hashing: {}", song.path);
        record_hash_failure(pool, song.id, &message)
            .await
            .map_err(|error| error.to_string())?;
        return Err(message);
    }

    let hash = hash.to_vec();
    sqlx::query(
        r#"
        INSERT INTO audio_assets (hash, hash_algorithm, byte_size)
        VALUES ($1, 'blake3-256', $2)
        ON CONFLICT (hash) DO NOTHING
        "#,
    )
    .bind(&hash)
    .bind(byte_size as i64)
    .execute(pool)
    .await
    .map_err(|error| error.to_string())?;
    sqlx::query(
        "UPDATE song SET audio_hash = $2, audio_hash_size = $3, audio_hash_mtime_ns = $4 WHERE id = $1",
    )
    .bind(song.id)
    .bind(&hash)
    .bind(after.0 as i64)
    .bind(after.1)
    .execute(pool)
    .await
    .map_err(|error| error.to_string())?;
    sqlx::query("DELETE FROM song_hash_failures WHERE song = $1")
        .bind(song.id)
        .execute(pool)
        .await
        .map_err(|error| error.to_string())?;

    // The direct per-song cache predates audio assets. Adopt it when a song is
    // first hashed instead of spending another analyzer run on unchanged bytes.
    sqlx::query(
        r#"
        INSERT INTO audio_similarity_features (audio_hash, version, vector, analyzed_at)
        SELECT $1, version, vector, analyzed_at
        FROM song_features
        WHERE song = $2
        ON CONFLICT (audio_hash, version) DO NOTHING
        "#,
    )
    .bind(&hash)
    .bind(song.id)
    .execute(pool)
    .await
    .map_err(|error| error.to_string())?;
    sqlx::query("DELETE FROM song_features WHERE song = $1")
        .bind(song.id)
        .execute(pool)
        .await
        .map_err(|error| error.to_string())?;
    sqlx::query("DELETE FROM song_feature_failures WHERE song = $1")
        .bind(song.id)
        .execute(pool)
        .await
        .map_err(|error| error.to_string())?;

    Ok(AudioAsset { hash })
}

async fn process_similarity(
    pool: &PgPool,
    config: &Config,
    song: &PendingSong,
    asset: &AudioAsset,
) -> Result<(), String> {
    let lock = asset_lock(&asset.hash);
    let _guard = lock.lock().await;
    if artifact_exists(pool, "similarity", FEATURE_VERSION, &asset.hash)
        .await
        .map_err(|error| error.to_string())?
        || asset_failure_exists(pool, "similarity", FEATURE_VERSION, &asset.hash)
            .await
            .map_err(|error| error.to_string())?
    {
        return Ok(());
    }
    match run_similarity_worker(config.audio_analysis_command.clone(), song).await {
        Ok(vector) => {
            sqlx::query(
                r#"
                INSERT INTO audio_similarity_features (audio_hash, version, vector, analyzed_at)
                VALUES ($1, $2, $3, now())
                ON CONFLICT (audio_hash, version) DO UPDATE SET
                  vector = EXCLUDED.vector, analyzed_at = EXCLUDED.analyzed_at
                "#,
            )
            .bind(&asset.hash)
            .bind(FEATURE_VERSION)
            .bind(encode_vector(&vector))
            .execute(pool)
            .await
            .map_err(|error| error.to_string())?;
            Ok(())
        }
        Err(message) => {
            record_asset_failure(pool, &asset.hash, "similarity", FEATURE_VERSION, &message)
                .await
                .map_err(|error| error.to_string())?;
            Err(format!("song {}: {message}", song.id))
        }
    }
}

async fn process_mix(
    pool: &PgPool,
    config: &Config,
    song: &PendingSong,
    asset: &AudioAsset,
) -> Result<(), String> {
    let lock = asset_lock(&asset.hash);
    let _guard = lock.lock().await;
    if artifact_exists(pool, "mix", MIX_PROFILE_VERSION, &asset.hash)
        .await
        .map_err(|error| error.to_string())?
        || asset_failure_exists(pool, "mix", MIX_PROFILE_VERSION, &asset.hash)
            .await
            .map_err(|error| error.to_string())?
    {
        return Ok(());
    }
    match run_mix_worker(config, &song.path).await {
        Ok(profile) => {
            let beats = profile
                .profile
                .get("beats")
                .and_then(Value::as_array)
                .map(|beats| beats.len() as i32);
            sqlx::query(
                r#"
                INSERT INTO audio_mix_profiles (
                  audio_hash, version, provider, provider_version, bpm, beat_count,
                  key_root, key_mode, key_confidence, loudness_integrated_lufs,
                  loudness_range_lu, true_peak_db, duration_seconds, leading_silence_ms,
                  trailing_silence_ms, head_rms_dbfs, tail_rms_dbfs, payload, analyzed_at
                ) VALUES (
                  $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
                  $16, $17, $18, now()
                ) ON CONFLICT (audio_hash, version) DO UPDATE SET
                  provider = EXCLUDED.provider, provider_version = EXCLUDED.provider_version,
                  bpm = EXCLUDED.bpm, beat_count = EXCLUDED.beat_count,
                  key_root = EXCLUDED.key_root, key_mode = EXCLUDED.key_mode,
                  key_confidence = EXCLUDED.key_confidence,
                  loudness_integrated_lufs = EXCLUDED.loudness_integrated_lufs,
                  loudness_range_lu = EXCLUDED.loudness_range_lu,
                  true_peak_db = EXCLUDED.true_peak_db, duration_seconds = EXCLUDED.duration_seconds,
                  leading_silence_ms = EXCLUDED.leading_silence_ms,
                  trailing_silence_ms = EXCLUDED.trailing_silence_ms,
                  head_rms_dbfs = EXCLUDED.head_rms_dbfs, tail_rms_dbfs = EXCLUDED.tail_rms_dbfs,
                  payload = EXCLUDED.payload, analyzed_at = EXCLUDED.analyzed_at
                "#,
            )
            .bind(&asset.hash)
            .bind(MIX_PROFILE_VERSION)
            .bind(profile.provider)
            .bind(profile.provider_version)
            .bind(json_f32(&profile.profile, "bpm"))
            .bind(beats)
            .bind(json_string(&profile.profile, "key"))
            .bind(json_string(&profile.profile, "mode"))
            .bind(json_f32(&profile.profile, "key_confidence"))
            .bind(json_f32(&profile.profile, "loudness_integrated"))
            .bind(json_f32(&profile.profile, "loudness_range"))
            .bind(json_f32(&profile.profile, "true_peak"))
            .bind(json_f32(&profile.profile, "duration"))
            .bind(json_i32(&profile.profile, "leading_silence_ms"))
            .bind(json_i32(&profile.profile, "trailing_silence_ms"))
            .bind(json_f32(&profile.profile, "head_rms_dbfs"))
            .bind(json_f32(&profile.profile, "tail_rms_dbfs"))
            .bind(Json(profile.profile))
            .execute(pool)
            .await
            .map_err(|error| error.to_string())?;
            Ok(())
        }
        Err(message) => {
            record_asset_failure(pool, &asset.hash, "mix", MIX_PROFILE_VERSION, &message)
                .await
                .map_err(|error| error.to_string())?;
            Err(format!("song {}: {message}", song.id))
        }
    }
}

fn asset_lock(hash: &[u8]) -> Arc<Mutex<()>> {
    ASSET_RUNNERS
        .get_or_init(DashMap::new)
        .entry(hash.to_vec())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

fn json_f32(profile: &Value, key: &str) -> Option<f32> {
    profile.get(key)?.as_f64().map(|value| value as f32)
}

fn json_i32(profile: &Value, key: &str) -> Option<i32> {
    profile.get(key)?.as_i64()?.try_into().ok()
}

fn json_string(profile: &Value, key: &str) -> Option<String> {
    profile.get(key)?.as_str().map(str::to_owned)
}

async fn artifact_exists(
    pool: &PgPool,
    analyzer: &str,
    version: i32,
    hash: &[u8],
) -> anyhow::Result<bool> {
    let query = match analyzer {
        "similarity" => "SELECT EXISTS (SELECT 1 FROM audio_similarity_features WHERE audio_hash = $1 AND version = $2)",
        "mix" => "SELECT EXISTS (SELECT 1 FROM audio_mix_profiles WHERE audio_hash = $1 AND version = $2)",
        _ => unreachable!("unknown analyzer"),
    };
    Ok(sqlx::query_scalar(query)
        .bind(hash)
        .bind(version)
        .fetch_one(pool)
        .await?)
}

async fn asset_failure_exists(
    pool: &PgPool,
    analyzer: &str,
    version: i32,
    hash: &[u8],
) -> anyhow::Result<bool> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM audio_analysis_failures WHERE audio_hash = $1 AND analyzer = $2 AND version = $3)",
    )
    .bind(hash)
    .bind(analyzer)
    .bind(version)
    .fetch_one(pool)
    .await?)
}

async fn record_hash_failure(pool: &PgPool, song: i32, message: &str) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO song_hash_failures (song, error, failed_at)
        VALUES ($1, $2, now())
        ON CONFLICT (song) DO UPDATE SET error = EXCLUDED.error, failed_at = EXCLUDED.failed_at
        "#,
    )
    .bind(song)
    .bind(message)
    .execute(pool)
    .await?;
    Ok(())
}

async fn record_asset_failure(
    pool: &PgPool,
    hash: &[u8],
    analyzer: &str,
    version: i32,
    message: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO audio_analysis_failures (audio_hash, analyzer, version, error, failed_at)
        VALUES ($1, $2, $3, $4, now())
        ON CONFLICT (audio_hash, analyzer, version) DO UPDATE SET
          error = EXCLUDED.error, failed_at = EXCLUDED.failed_at
        "#,
    )
    .bind(hash)
    .bind(analyzer)
    .bind(version)
    .bind(message)
    .execute(pool)
    .await?;
    Ok(())
}

async fn run_similarity_worker(
    command: Vec<String>,
    song: &PendingSong,
) -> Result<Vec<f32>, String> {
    let (program, arguments) = command
        .split_first()
        .ok_or_else(|| "audio_analysis_command is empty".to_owned())?;
    let output = Command::new(program)
        .args(arguments)
        .arg(&song.path)
        .output()
        .await
        .map_err(|error| format!("could not start worker: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "worker exited {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let response: WorkerOutput = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("worker returned invalid JSON: {error}"))?;
    if response.version != FEATURE_VERSION {
        return Err(format!(
            "worker returned feature version {}, expected {FEATURE_VERSION}",
            response.version
        ));
    }
    if response.vector.is_empty()
        || response.vector.len() > MAX_VECTOR_DIMENSIONS
        || response.vector.iter().any(|value| !value.is_finite())
    {
        return Err("worker returned an invalid feature vector".to_owned());
    }
    Ok(response.vector)
}

async fn run_mix_worker(config: &Config, path: &str) -> Result<MixProfile, String> {
    let base_url = config
        .mix_analysis_url
        .as_deref()
        .ok_or_else(|| "mix_analysis_url is empty".to_owned())?;
    let (authority, base_path) = parse_sidecar_url(base_url)?;
    let request_path = format!(
        "{}/v1/analyze?sample_rate=44100&sample_format=s16le&channels=1",
        base_path.trim_end_matches('/')
    );
    let mut child = Command::new("ffmpeg")
        .args([
            "-v", "error", "-i", path, "-vn", "-ac", "1", "-ar", "44100", "-f", "s16le", "pipe:1",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("could not start ffmpeg: {error}"))?;
    let response = tokio::time::timeout(
        Duration::from_secs(config.mix_analysis_timeout_seconds),
        stream_mix_request(
            &mut child,
            &authority,
            &request_path,
            config.mix_analysis_max_pcm_bytes,
        ),
    )
    .await;
    if !matches!(&response, Ok(Ok(_))) {
        let _ = child.kill().await;
    }
    let body = response.map_err(|_| "mix sidecar request timed out".to_owned())??;
    let response: MixSidecarResponse = serde_json::from_slice(&body)
        .map_err(|error| format!("mix sidecar returned invalid JSON: {error}"))?;
    if response.schema_version != MIX_PROFILE_VERSION
        || response.provider.is_empty()
        || response.provider_version.is_empty()
    {
        return Err("mix sidecar returned an unsupported profile schema".to_owned());
    }
    if !response.profile.is_object() {
        return Err("mix sidecar returned a non-object profile".to_owned());
    }
    Ok(MixProfile {
        provider: response.provider,
        provider_version: response.provider_version,
        profile: response.profile,
    })
}

fn parse_sidecar_url(url: &str) -> Result<(String, String), String> {
    let url = url.trim_end_matches('/');
    let Some(without_scheme) = url.strip_prefix("http://") else {
        return Err("mix_analysis_url must use internal http:// transport".to_owned());
    };
    let (authority, path) = without_scheme
        .split_once('/')
        .map(|(authority, path)| (authority, format!("/{path}")))
        .unwrap_or((without_scheme, String::new()));
    if authority.is_empty() {
        return Err("mix_analysis_url is missing a host".to_owned());
    }
    Ok((authority.to_owned(), path))
}

async fn stream_mix_request(
    child: &mut tokio::process::Child,
    authority: &str,
    path: &str,
    max_bytes: u64,
) -> Result<Vec<u8>, String> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "ffmpeg stdout was not available".to_owned())?;
    let mut socket = tokio::net::TcpStream::connect(authority)
        .await
        .map_err(|error| format!("could not connect to mix sidecar: {error}"))?;
    let headers = format!(
        "POST {path} HTTP/1.1\r\nHost: {authority}\r\nContent-Type: application/octet-stream\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n"
    );
    socket
        .write_all(headers.as_bytes())
        .await
        .map_err(|error| format!("could not start mix sidecar request: {error}"))?;

    let mut stdout = stdout;
    let mut buffer = vec![0; 64 * 1024];
    let mut sent = 0u64;
    loop {
        let read = stdout
            .read(&mut buffer)
            .await
            .map_err(|error| format!("could not read ffmpeg PCM: {error}"))?;
        if read == 0 {
            break;
        }
        sent = sent.saturating_add(read as u64);
        if sent > max_bytes {
            return Err(format!(
                "decoded PCM exceeds configured limit of {max_bytes} bytes"
            ));
        }
        socket
            .write_all(format!("{read:X}\r\n").as_bytes())
            .await
            .map_err(|error| format!("could not stream PCM to mix sidecar: {error}"))?;
        socket
            .write_all(&buffer[..read])
            .await
            .map_err(|error| format!("could not stream PCM to mix sidecar: {error}"))?;
        socket
            .write_all(b"\r\n")
            .await
            .map_err(|error| format!("could not stream PCM to mix sidecar: {error}"))?;
    }
    socket
        .write_all(b"0\r\n\r\n")
        .await
        .map_err(|error| format!("could not finish mix sidecar request: {error}"))?;
    let exit = child
        .wait()
        .await
        .map_err(|error| format!("could not wait for ffmpeg: {error}"))?;
    if !exit.success() {
        return Err(format!("ffmpeg exited {exit}"));
    }

    let mut response = Vec::new();
    socket
        .read_to_end(&mut response)
        .await
        .map_err(|error| format!("could not read mix sidecar response: {error}"))?;
    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| "mix sidecar returned malformed HTTP".to_owned())?;
    let headers = std::str::from_utf8(&response[..header_end])
        .map_err(|_| "mix sidecar returned non-UTF-8 HTTP headers".to_owned())?;
    if !headers.starts_with("HTTP/1.1 2") && !headers.starts_with("HTTP/1.0 2") {
        return Err(format!(
            "mix sidecar returned {}: {}",
            headers.lines().next().unwrap_or("invalid HTTP"),
            String::from_utf8_lossy(&response[header_end + 4..]).trim()
        ));
    }
    Ok(response.split_off(header_end + 4))
}

async fn file_signature(path: &Path) -> anyhow::Result<(u64, i64)> {
    let metadata = tokio::fs::metadata(path).await?;
    let modified = metadata.modified()?.duration_since(UNIX_EPOCH)?;
    let nanos = modified
        .as_secs()
        .checked_mul(1_000_000_000)
        .and_then(|seconds| seconds.checked_add(modified.subsec_nanos() as u64))
        .ok_or_else(|| anyhow::anyhow!("file modification time exceeds i64 nanoseconds"))?;
    Ok((metadata.len(), nanos.try_into()?))
}

async fn hash_file(path: &Path) -> anyhow::Result<([u8; 32], u64)> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0; HASH_BUFFER_BYTES];
    let mut byte_size = 0u64;
    loop {
        let read = file.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        byte_size += read as u64;
    }
    Ok((*hasher.finalize().as_bytes(), byte_size))
}

#[cfg(test)]
pub fn content_hash(bytes: &[u8]) -> [u8; 32] {
    *blake3::hash(bytes).as_bytes()
}

async fn refresh_stats(pool: &PgPool) -> anyhow::Result<()> {
    let rows = sqlx::query(
        r#"
        SELECT feature.vector
        FROM audio_similarity_features feature
        JOIN (SELECT DISTINCT audio_hash FROM song WHERE audio_hash IS NOT NULL) assets
          ON assets.audio_hash = feature.audio_hash
        WHERE feature.version = $1
        "#,
    )
    .bind(FEATURE_VERSION)
    .fetch_all(pool)
    .await?;
    let vectors: Vec<Vec<f32>> = rows
        .iter()
        .filter_map(|row| decode_vector(row.try_get::<Vec<u8>, _>("vector").ok()?))
        .collect();
    let Some((means, std_devs)) = calculate_stats(&vectors) else {
        sqlx::query("DELETE FROM audio_similarity_feature_stats WHERE version = $1")
            .bind(FEATURE_VERSION)
            .execute(pool)
            .await?;
        return Ok(());
    };
    sqlx::query(
        r#"
        INSERT INTO audio_similarity_feature_stats (version, vector_dimension, means, std_devs, updated_at)
        VALUES ($1, $2, $3, $4, now())
        ON CONFLICT (version) DO UPDATE SET
          vector_dimension = EXCLUDED.vector_dimension,
          means = EXCLUDED.means,
          std_devs = EXCLUDED.std_devs,
          updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(FEATURE_VERSION)
    .bind(means.len() as i32)
    .bind(encode_vector(&means))
    .bind(encode_vector(&std_devs))
    .execute(pool)
    .await?;
    Ok(())
}

pub fn encode_vector(vector: &[f32]) -> Vec<u8> {
    vector
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

pub fn decode_vector(bytes: Vec<u8>) -> Option<Vec<f32>> {
    if bytes.is_empty() || bytes.len() % std::mem::size_of::<f32>() != 0 {
        return None;
    }
    let vector: Vec<f32> = bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();
    vector
        .iter()
        .all(|value| value.is_finite())
        .then_some(vector)
}

pub fn calculate_stats(vectors: &[Vec<f32>]) -> Option<(Vec<f32>, Vec<f32>)> {
    let first = vectors.first()?;
    if first.is_empty() || vectors.iter().any(|vector| vector.len() != first.len()) {
        return None;
    }
    let count = vectors.len() as f32;
    let means: Vec<f32> = (0..first.len())
        .map(|index| vectors.iter().map(|vector| vector[index]).sum::<f32>() / count)
        .collect();
    let std_devs = (0..first.len())
        .map(|index| {
            let variance = vectors
                .iter()
                .map(|vector| (vector[index] - means[index]).powi(2))
                .sum::<f32>()
                / count;
            variance.sqrt().max(f32::EPSILON)
        })
        .collect();
    Some((means, std_devs))
}

#[cfg(test)]
mod tests {
    use super::{calculate_stats, content_hash, decode_vector, encode_vector, hash_file};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn content_hash_is_blake3_256() {
        assert_eq!(
            content_hash(b""),
            [
                0xaf, 0x13, 0x49, 0xb9, 0xf5, 0xf9, 0xa1, 0xa6, 0xa0, 0x40, 0x4d, 0xea, 0x36, 0xdc,
                0xc9, 0x49, 0x9b, 0xcb, 0x25, 0xc9, 0xad, 0xc1, 0x12, 0xb7, 0xcc, 0x9a, 0x93, 0xca,
                0xe4, 0x1f, 0x32, 0x62,
            ]
        );
    }

    #[test]
    fn content_hash_never_uses_catalog_metadata() {
        assert_ne!(
            content_hash(b"first master"),
            content_hash(b"second master")
        );
    }

    #[test]
    fn vectors_round_trip_as_little_endian_f32() {
        let vector = vec![-1.0, 0.5, 42.25];
        assert_eq!(decode_vector(encode_vector(&vector)), Some(vector));
    }

    #[tokio::test]
    async fn identical_files_at_different_paths_share_a_content_hash() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let first = std::env::temp_dir().join(format!("maki-hash-first-{suffix}"));
        let second = std::env::temp_dir().join(format!("maki-hash-second-{suffix}"));
        tokio::fs::write(&first, b"same imported audio bytes")
            .await
            .unwrap();
        tokio::fs::write(&second, b"same imported audio bytes")
            .await
            .unwrap();

        let first_hash = hash_file(&first).await.unwrap();
        let second_hash = hash_file(&second).await.unwrap();

        assert_eq!(first_hash, second_hash);
        assert_eq!(first_hash.1, 25);
        tokio::fs::remove_file(first).await.unwrap();
        tokio::fs::remove_file(second).await.unwrap();
    }

    #[test]
    fn stats_keep_constant_features_neutral() {
        let (means, std_devs) = calculate_stats(&[vec![2.0, 1.0], vec![4.0, 1.0]]).unwrap();
        assert_eq!(means, vec![3.0, 1.0]);
        assert_eq!(std_devs[0], 1.0);
        assert_eq!(std_devs[1], f32::EPSILON);
    }
}
