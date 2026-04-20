use std::{path::PathBuf, process::Stdio, sync::Arc};

use axum::{
    body::Body,
    extract::{Extension, Path},
    http::{header, HeaderValue, Request, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use sqlx::PgPool;
use tokio::process::Command;
use tower::util::ServiceExt;
use tower_http::services::fs::ServeFile;
use tracing::{error, info};

use crate::{
    api::resolve_song_id,
    config::HlsProfile,
    error::AppError,
    HlsState,
};

use std::time::Instant;

const CACHE_BASE: &str = "/tmp/co.lutea.maki/hls";

fn cache_dir(song_id: i32) -> PathBuf {
    PathBuf::from(format!("{}/{}", CACHE_BASE, song_id))
}

fn touch_access(state: &HlsState, song_id: i32) {
    state.last_access.insert(song_id, Instant::now());
}

fn extract_raw_token(query: Option<&str>) -> String {
    let Some(q) = query else { return String::new() };
    let Some(after_tk) = q.split("tk=").nth(1) else { return String::new() };
    after_tk.split('&').next().unwrap_or(after_tk).to_owned()
}

fn inject_token_into_master(content: &str, token: &str) -> String {
    content
        .lines()
        .map(|line| {
            if !line.starts_with('#') && !line.is_empty() {
                format!("{}?tk={}\n", line, token)
            } else {
                format!("{}\n", line)
            }
        })
        .collect()
}

fn inject_token_into_media_playlist(content: &str, profile: &str, token: &str) -> String {
    content
        .lines()
        .map(|line| {
            if let Some(rest) = line.strip_prefix("#EXT-X-MAP:URI=\"") {
                // prefix with profile dir and inject token before closing quote
                let rest = rest.replacen('"', &format!("?tk={token}\""), 1);
                format!("#EXT-X-MAP:URI=\"{profile}/{rest}\n")
            } else if !line.starts_with('#') && !line.is_empty() {
                format!("{}/{}?tk={}\n", profile, line, token)
            } else {
                format!("{}\n", line)
            }
        })
        .collect()
}

fn build_master_m3u8(
    profiles: &[HlsProfile],
    sample_rate: Option<i32>,
    bits_per_sample: Option<i32>,
    num_channels: Option<i32>,
) -> String {
    let mut out = String::from("#EXTM3U\n#EXT-X-VERSION:7\n\n");
    for profile in profiles {
        let bandwidth = match profile.bitrate {
            Some(br) => br,
            None => {
                let sr = sample_rate.unwrap_or(44100) as u32;
                let bd = bits_per_sample.unwrap_or(16) as u32;
                let ch = num_channels.unwrap_or(2) as u32;
                sr * bd * ch * 6 / 10
            }
        };
        let codecs = if profile.codec == "flac" { "fLaC" } else { "mp4a.40.2" };
        out.push_str(&format!(
            "#EXT-X-STREAM-INF:BANDWIDTH={},CODECS=\"{}\"\n{}\n",
            bandwidth, codecs, profile.name
        ));
    }
    out
}

async fn ensure_segments(
    song_id: i32,
    state: &HlsState,
    pool: &PgPool,
) -> Result<(), AppError> {
    let dir = cache_dir(song_id);
    let done = dir.join(".done");

    if tokio::fs::metadata(&done).await.is_ok() {
        touch_access(state, song_id);
        return Ok(());
    }

    let lock = state
        .in_flight
        .entry(song_id)
        .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
        .clone();
    let _guard = lock.lock().await;

    // double-check after acquiring lock
    if tokio::fs::metadata(&done).await.is_ok() {
        touch_access(state, song_id);
        return Ok(());
    }

    let file_path = sqlx::query_scalar!("SELECT path FROM song WHERE id = $1", song_id)
        .fetch_one(pool)
        .await?;

    info!("hls: transcoding song {} ({} profiles)", song_id, state.profiles.len());
    run_ffmpeg(&file_path, &state.profiles, &dir).await?;
    tokio::fs::write(&done, b"").await?;
    touch_access(state, song_id);
    Ok(())
}

async fn run_ffmpeg(
    file_path: &str,
    profiles: &[HlsProfile],
    cache_dir: &PathBuf,
) -> Result<(), AppError> {
    let n = profiles.len();
    for profile in profiles {
        tokio::fs::create_dir_all(cache_dir.join(&profile.name)).await?;
    }

    // [0:a]asplit=N[a0][a1]...[aN-1]
    let filter = format!(
        "[0:a]asplit={}{}",
        n,
        (0..n).map(|i| format!("[a{i}]")).collect::<String>()
    );

    info!("hls: spawning ffmpeg for {:?}", file_path);
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y").arg("-i").arg(file_path);
    cmd.arg("-filter_complex").arg(&filter);

    for (i, profile) in profiles.iter().enumerate() {
        cmd.arg("-map").arg(format!("[a{i}]"));
        cmd.arg("-c:a");
        if profile.codec == "flac" {
            cmd.arg("flac");
        } else {
            cmd.arg("libfdk_aac");
        }
        if let Some(br) = profile.bitrate {
            cmd.arg("-b:a").arg(br.to_string());
        }
    }

    let segment_filename = cache_dir
        .join("%v")
        .join("seg%d.m4s")
        .to_string_lossy()
        .into_owned();
    let playlist_output = cache_dir
        .join("%v.m3u8")
        .to_string_lossy()
        .into_owned();
    let var_stream_map = profiles
        .iter()
        .enumerate()
        .map(|(i, p)| format!("a:{},name:{}", i, p.name))
        .collect::<Vec<_>>()
        .join(" ");

    cmd.arg("-f").arg("hls")
        .arg("-hls_time").arg("6")
        .arg("-hls_segment_type").arg("fmp4")
        .arg("-hls_playlist_type").arg("vod")
        .arg("-hls_segment_filename").arg(&segment_filename)
        .arg("-hls_fmp4_init_filename").arg("init.mp4")
        .arg("-var_stream_map").arg(&var_stream_map)
        .arg(&playlist_output)
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn()?;

    if let Some(stderr) = child.stderr.take() {
        tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            let mut reader = tokio::io::BufReader::new(stderr);
            let mut output = String::new();
            let _ = reader.read_to_string(&mut output).await;
            if !output.is_empty() {
                error!("ffmpeg (hls): {}", output);
            }
        });
    }

    let status = child.wait().await?;
    if !status.success() {
        return Err(anyhow::anyhow!("ffmpeg exited with status: {}", status).into());
    }
    info!("hls: ffmpeg finished successfully");

    // ffmpeg names init segments init_0.mp4, init_1.mp4, ... (ignoring -hls_fmp4_init_filename
    // for multi-variant output). Move each one into its profile subdir and rewrite the URI in
    // the playlist so segment serving works without special-casing.
    for (i, profile) in profiles.iter().enumerate() {
        let src = cache_dir.join(format!("init_{}.mp4", i));
        let dst = cache_dir.join(&profile.name).join("init.mp4");
        tokio::fs::rename(&src, &dst).await?;

        let playlist_path = cache_dir.join(format!("{}.m3u8", profile.name));
        let content = tokio::fs::read_to_string(&playlist_path).await?;
        let rewritten = content.replace(
            &format!("URI=\"init_{}.mp4\"", i),
            "URI=\"init.mp4\"",
        );
        tokio::fs::write(&playlist_path, rewritten).await?;
    }

    Ok(())
}

pub async fn get_profiles(
    Extension(state): Extension<Arc<HlsState>>,
) -> impl IntoResponse {
    axum::Json(state.profiles.clone())
}

pub async fn serve_master(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
    Extension(state): Extension<Arc<HlsState>>,

    uri: Uri,
) -> Result<Response, AppError> {
    let song_id = resolve_song_id(&id, &pool)
        .await
        .map_err(|(_, e)| anyhow::anyhow!(e))?;

    touch_access(&state, song_id);

    let row = sqlx::query!(
        "SELECT sample_rate, bits_per_sample, num_channels FROM song WHERE id = $1",
        song_id
    )
    .fetch_one(&pool)
    .await?;

    let master = build_master_m3u8(
        &state.profiles,
        row.sample_rate,
        row.bits_per_sample,
        row.num_channels,
    );

    let token = extract_raw_token(uri.query());
    let body = inject_token_into_master(&master, &token);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")
        .body(Body::from(body))
        .unwrap())
}

pub async fn serve_media_playlist(
    Path((id, profile)): Path<(String, String)>,
    Extension(pool): Extension<PgPool>,
    Extension(state): Extension<Arc<HlsState>>,

    uri: Uri,
) -> Result<Response, AppError> {
    let song_id = resolve_song_id(&id, &pool)
        .await
        .map_err(|(_, e)| anyhow::anyhow!(e))?;

    if !state.profiles.iter().any(|p| p.name == profile) {
        return Err(anyhow::anyhow!("unknown profile: {}", profile).into());
    }

    ensure_segments(song_id, &state, &pool).await?;

    let playlist_path = cache_dir(song_id).join(format!("{profile}.m3u8"));
    let content = tokio::fs::read_to_string(&playlist_path).await?;

    let token = extract_raw_token(uri.query());
    let body = inject_token_into_media_playlist(&content, &profile, &token);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")
        .body(Body::from(body))
        .unwrap())
}

pub async fn serve_init(
    Path((id, profile)): Path<(String, String)>,
    Extension(pool): Extension<PgPool>,
    Extension(state): Extension<Arc<HlsState>>,

    request: Request<Body>,
) -> Result<Response, AppError> {
    let song_id = resolve_song_id(&id, &pool)
        .await
        .map_err(|(_, e)| anyhow::anyhow!(e))?;

    if !state.profiles.iter().any(|p| p.name == profile) {
        return Err(anyhow::anyhow!("unknown profile: {}", profile).into());
    }

    ensure_segments(song_id, &state, &pool).await?;

    let path = cache_dir(song_id).join(&profile).join("init.mp4");
    let mut res = ServeFile::new(path)
        .oneshot(request)
        .await
        .map_err(|e| anyhow::anyhow!("failed to serve init.mp4: {}", e))?;
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("video/mp4"),
    );
    Ok(res.map(Body::new).into_response())
}

pub async fn serve_segment(
    Path((id, profile, segment)): Path<(String, String, String)>,
    Extension(pool): Extension<PgPool>,
    Extension(state): Extension<Arc<HlsState>>,

    request: Request<Body>,
) -> Result<Response, AppError> {
    let song_id = resolve_song_id(&id, &pool)
        .await
        .map_err(|(_, e)| anyhow::anyhow!(e))?;

    if !state.profiles.iter().any(|p| p.name == profile) {
        return Err(anyhow::anyhow!("unknown profile: {}", profile).into());
    }

    ensure_segments(song_id, &state, &pool).await?;

    let path = cache_dir(song_id).join(&profile).join(&segment);
    let mut res = ServeFile::new(path)
        .oneshot(request)
        .await
        .map_err(|e| anyhow::anyhow!("failed to serve segment: {}", e))?;
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("video/iso.segment"),
    );
    Ok(res.map(Body::new).into_response())
}
