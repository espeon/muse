use std::process::Stdio;

use axum::{
    body::Body,
    extract::{Extension, Path, Query},
    http::{header, Request, StatusCode},
    response::{AppendHeaders, IntoResponse},
};
use futures::StreamExt;
use serde::Deserialize;
use sqlx::PgPool;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};
use tokio_util::io::ReaderStream;
use tower::util::ServiceExt;
use tower_http::services::fs::ServeFile;
use tracing::error;

use crate::error::AppError;

pub async fn serve_audio(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let res = Request::builder().uri("/").body(Body::empty()).unwrap();

    let id_parsed = id.split('.').collect::<Vec<&str>>()[0]
        .parse::<i32>()
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    match sqlx::query!(
        r#"
        SELECT path from song
        WHERE id = $1
    "#,
        id_parsed
    )
    .fetch_one(&pool)
    .await
    {
        Ok(f) => match ServeFile::new(f.path).oneshot(res).await {
            Ok(res) => Ok(res),
            Err(err) => Err((
                StatusCode::NOT_FOUND,
                format!("Something went wrong when serving a file: {}", err),
            )),
        },
        Err(err) => Err((
            StatusCode::NOT_FOUND,
            format!("Something went wrong when finding the file: {}", err),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct ServeTranscodedAudioQueryParams {
    #[serde(default)]
    codec: String,
    #[serde(default)]
    dps: String,
}

// default values
impl Default for ServeTranscodedAudioQueryParams {
    fn default() -> Self {
        Self {
            codec: "mp3".to_string(),
            dps: "128".to_string(),
        }
    }
}
#[derive(Debug, Deserialize)]
struct ServeTranscodedAudioParams {
    dir: String,
    codec: TranscodeCodec,
    dps: String,
}
#[derive(Debug, Deserialize)]
enum TranscodeCodec {
    Opus,
    M4a,
    Mp3,
    Aac,
    Ogg,
    Flac,
    Alac,
}

// note opus is in ogg container
#[derive(Debug, Deserialize)]
enum TranscodeContainerFormat {
    M4a,
    Mp3,
    Aac,
    Ogg,
    Flac,
}

impl TranscodeCodec {
    // from string
    fn from_str(s: &str) -> Self {
        match s {
            "m4a" => Self::M4a,
            "mp3" => Self::Mp3,
            "aac" => Self::Aac,
            "flac" => Self::Flac,
            "alac" => Self::Alac,
            "opus" => Self::Opus,
            "ogg" => Self::Ogg,
            _ => Self::Mp3,
        }
    }
    fn as_encoder(&self) -> &str {
        match self {
            Self::Opus => "libopus",
            Self::Mp3 => "libmp3lame",
            Self::Ogg => "libvorbis",
            Self::M4a => "libfdk_aac",
            Self::Aac => "libfdk_aac",
            Self::Flac => "flac",
            Self::Alac => "alac",
        }
    }
    // Get the container format of the codec
    fn as_container_format(&self) -> TranscodeContainerFormat {
        match self {
            Self::Opus => TranscodeContainerFormat::Ogg,
            Self::M4a => TranscodeContainerFormat::M4a,
            Self::Mp3 => TranscodeContainerFormat::Mp3,
            Self::Aac => TranscodeContainerFormat::Aac,
            Self::Ogg => TranscodeContainerFormat::Ogg,
            Self::Flac => TranscodeContainerFormat::Flac,
            Self::Alac => TranscodeContainerFormat::M4a,
        }
    }
}

impl TranscodeContainerFormat {
    fn as_str(&self) -> &str {
        match self {
            Self::M4a => "m4a",
            Self::Mp3 => "mp3",
            Self::Aac => "aac",
            Self::Ogg => "ogg",
            Self::Flac => "flac",
        }
    }
}

async fn setup_ffmpeg(
    params: &ServeTranscodedAudioParams,
) -> Result<tokio::process::Child, AppError> {
    dbg!(&params);
    let mut command = Command::new("ffmpeg");
    command
        .arg("-i")
        .arg(&params.dir)
        .arg("-map")
        .arg("0:a:0")
        .arg("-c:a")
        .arg(params.codec.as_encoder())
        .arg("-f")
        .arg(params.codec.as_container_format().as_str());

    // we need to set the bitrate only for lossy codecs
    let encoder = params.codec.as_encoder();
    if encoder == "libmp3lame"
        || encoder == "libfdk_aac"
        || encoder == "libopus"
        || encoder == "libvorbis"
    {
        command.arg("-b:a").arg(&params.dps);
    }

    command
        .arg("-")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn()?;

    // Capture and log stderr output (for debugging)
    if let Some(stderr) = child.stderr.take() {
        tokio::spawn(async move {
            let mut stderr_reader = tokio::io::BufReader::new(stderr);
            let mut stderr_output = String::new();
            while stderr_reader
                .read_to_string(&mut stderr_output)
                .await
                .is_ok()
                && !stderr_output.is_empty()
            {
                error!("FFmpeg error: {}", stderr_output);
                stderr_output.clear();
            }
        });
    }

    Ok(child)
}

pub async fn serve_transcoded_audio(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
    Query(params): Query<ServeTranscodedAudioQueryParams>,
) -> Result<impl IntoResponse, AppError> {
    let id_parsed = id.split('.').collect::<Vec<&str>>()[0]
        .parse::<i32>()
        .map_err(|_| anyhow::anyhow!("Failed to parse id"))?;

    let path = sqlx::query!(
        r#"
        SELECT path from song
        WHERE id = $1
    "#,
        id_parsed
    )
    .fetch_one(&pool)
    .await?;

    let tparams = ServeTranscodedAudioParams {
        dir: path.path,
        codec: TranscodeCodec::from_str(&params.codec),
        dps: params.dps,
    };

    let mut child = setup_ffmpeg(&tparams).await?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to get stdout"))?;

    let mut stream = ReaderStream::new(stdout).boxed();

    // Create a file to write the cache
    let cache_file_path = format!(
        "/tmp/co.lutea.maki/cache/{}.{}",
        id_parsed,
        tparams.codec.as_container_format().as_str()
    );
    // make sure the directory exists
    std::fs::create_dir_all(std::path::Path::new(&cache_file_path).parent().unwrap())?;
    let mut cache_file = File::create(&cache_file_path).await?;

    // Create a duplex stream with a 128MB buffer
    let (mut writer, reader) = tokio::io::duplex(1024 * 1024 * 128);

    tokio::spawn(async move {
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(data) => {
                    if writer.write_all(&data).await.is_err() {
                        error!("Error writing to writer.");
                        break;
                    }
                    if cache_file.write_all(&data).await.is_err() {
                        error!("Error writing to cache file.");
                        break;
                    }
                }
                Err(e) => {
                    error!("Stream read error: {}", e);
                    break;
                }
            }
        }
        drop(writer);
    });

    let stream = ReaderStream::new(reader).boxed();
    let body = Body::from_stream(stream);

    let headers = AppendHeaders([
        (
            header::CONTENT_TYPE,
            format!("audio/{}", tparams.codec.as_container_format().as_str()),
        ),
        (
            header::CONTENT_DISPOSITION,
            format!(
                "attachment; filename=\"{}.{}\"",
                id,
                tparams.codec.as_container_format().as_str()
            ),
        ),
    ]);

    Ok((headers, body))
}

/// Serve image
/// :id - album id
pub async fn serve_image(Path(id): Path<String>) -> impl IntoResponse {
    let res = Request::builder().uri("/").body(Body::empty()).unwrap();
    match ServeFile::new(format!("./art/{id}.webp"))
        .oneshot(res)
        .await
    {
        Ok(res) => Ok(res),
        Err(err) => Err((
            StatusCode::NOT_FOUND,
            format!("Something went wrong when serving a file: {}", err),
        )),
    }
}
