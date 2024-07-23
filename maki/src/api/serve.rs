use std::process::Stdio;

use axum::{
    body::Body,
    extract::{Extension, Path, Query},
    http::{Request, StatusCode},
    response::IntoResponse,
};
use futures::StreamExt;
use serde::Deserialize;
use sqlx::PgPool;
use tokio_util::io::ReaderStream;
use tower::util::ServiceExt;
use tower_http::services::fs::ServeFile;

use crate::error::AppError;

pub async fn serve_audio(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let res = Request::builder().uri("/").body(Body::empty()).unwrap();

    let id_parsed = id.split('.').collect::<Vec<&str>>()[0]
        .parse::<i32>().map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

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
    #[serde(default)]
    format: String,
}

// default values
impl Default for ServeTranscodedAudioQueryParams {
    fn default() -> Self {
        Self {
            codec: "mp3".to_string(),
            dps: "128".to_string(),
            format: "mp3".to_string(),
        }
    }
}
#[derive(Debug, Deserialize)]
struct ServeTranscodedAudioParams {
    dir: String,
    codec: TranscodeCodec,
    dps: String,
    format: TranscodeFormat,
}
#[derive(Debug, Deserialize)]
enum TranscodeCodec{
    Opus,
    Mp3,
    Ogg,
}

// note opus is in ogg container
#[derive(Debug, Deserialize)]
enum TranscodeFormat {
    Mp3,
    Ogg,
}

impl TranscodeCodec {
    // from string
    fn from_str(s: &str) -> Self{
        match s {
            "opus" => Self::Opus,
            "mp3" => Self::Mp3,
            "ogg" => Self::Ogg,
            _ => Self::Mp3, 
        }
    }
    fn as_str(&self) -> &str {
        match self {
            Self::Opus => "libopus",
            Self::Mp3 => "libmp3lame",
            Self::Ogg => "libvorbis",
        }
    }
}

impl TranscodeFormat {
    fn from_str(s: &str) -> Self{
        match s {
            "mp3" => Self::Mp3,
            "ogg" => Self::Ogg,
            _ => Self::Mp3,
        }
    }
    fn as_str(&self) -> &str {
        match self {
            Self::Mp3 => "mp3",
            Self::Ogg => "ogg",
        }
    }
}

async fn setup_ffmpeg(params: ServeTranscodedAudioParams) -> Result<tokio::process::Child, AppError> {
    dbg!(&params);
    Ok(tokio::process::Command::new("ffmpeg")
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .arg("-i")
        .arg(params.dir)
        .arg("-map")
        .arg("0:a:0")
        .arg("-f")
        .arg(params.format.as_str())
        .arg("-c:a")
        .arg(params.codec.as_str())
        .arg("-b:a")
        .arg(params.dps)
        .arg("-")
        .spawn()?)
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
        format: TranscodeFormat::from_str(&params.format),
    };

    let mut child = setup_ffmpeg(tparams).await?;

    let stdout = match child.stdout.take(){
        Some(e) => e,
        None => return Err(anyhow::anyhow!("Failed to get stdout").into()),
    };
    let stream = ReaderStream::new(stdout).boxed();
    
    // TODO: split stream and cache it on disk
    
    let body = Body::from_stream(stream);

    Ok(body.into_response())
}

/// Serve image
/// :id - album id
pub async fn serve_image(
    Path(id): Path<String>,
) -> impl IntoResponse {
    let res = Request::builder().uri("/").body(Body::empty()).unwrap();
    match ServeFile::new(format!("./art/{id}.webp")).oneshot(res).await {
        Ok(res) => Ok(res),
        Err(err) => Err((
            StatusCode::NOT_FOUND,
            format!("Something went wrong when serving a file: {}", err),
        )),
    }
}