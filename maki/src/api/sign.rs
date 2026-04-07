use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{Extension, Path, Query},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;

use crate::{api::resolve_song_id, error::AppError, helpers::HmacMessage};

use super::middleware::jwt::AuthUser;

#[derive(Serialize, Deserialize)]
pub struct SignResult {
    pub id: i32,
    pub url: String,
    #[serde(with = "time::serde::rfc3339")]
    pub signed_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: OffsetDateTime,
}

#[derive(Deserialize, Default)]
pub struct SignQueryParams {
    /// When set, signs a transcode URL with this codec (e.g. "mp3", "opus", "aac")
    pub codec: Option<String>,
    /// Bitrate for transcoded URL (e.g. "128k"). Defaults to "128k".
    pub dps: Option<String>,
}

fn make_signed_url(
    base_url: &str,
    id: i32,
    hmac: &str,
    codec: Option<&str>,
    dps: Option<&str>,
) -> String {
    match codec {
        Some(c) => {
            let bitrate = dps.unwrap_or("128k");
            format!(
                "{}/api/v1/track/{}/transcode?tk={}&codec={}&dps={}",
                base_url, id, hmac, c, bitrate
            )
        }
        None => format!("{}/api/v1/track/{}/stream?tk={}", base_url, id, hmac),
    }
}

fn sign_for_id(
    id: i32,
    user_sub: &str,
    key: &[u8],
    base_url: &str,
    codec: Option<&str>,
    dps: Option<&str>,
) -> SignResult {
    let st = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let exp = st + 60 * 60; // 60 minutes
    let presigned = HmacMessage::new(st, exp, "*".to_owned(), user_sub.to_owned());
    let hmac = presigned.sign(key);
    SignResult {
        id,
        url: make_signed_url(base_url, id, &hmac, codec, dps),
        signed_at: OffsetDateTime::from_unix_timestamp(st as i64).unwrap(),
        expires_at: OffsetDateTime::from_unix_timestamp(exp as i64).unwrap(),
    }
}

/// GET /track/:id/sign — sign a single track URL.
/// Optional query params: codec (omit for raw stream), dps (bitrate, default "128k")
pub async fn sign_track_url(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
    Query(params): Query<SignQueryParams>,
    AuthUser { payload }: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let key =
        std::env::var("AUTH_SECRET").map_err(|_| anyhow::anyhow!("AUTH_SECRET is not set"))?;
    let base_url = std::env::var("EXTERNAL_MAKI_BASE_URL")
        .map_err(|_| anyhow::anyhow!("EXTERNAL_MAKI_BASE_URL is not set"))?;
    let id_parsed = resolve_song_id(&id, &pool)
        .await
        .map_err(|(_, e)| anyhow::anyhow!(e))?;

    let result = sign_for_id(
        id_parsed,
        &payload.sub,
        key.as_bytes(),
        &base_url,
        params.codec.as_deref(),
        params.dps.as_deref(),
    );

    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct BatchSignRequest {
    pub ids: Vec<String>,
    pub codec: Option<String>,
    pub dps: Option<String>,
}

/// POST /tracks/sign — sign multiple track URLs in one request.
/// Body: { ids: ["1", "abc123slug", ...], codec?: "mp3", dps?: "128k" }
/// Accepts both integer IDs and slugs. Returns signed URLs for each in the same order.
pub async fn batch_sign_track_urls(
    Extension(pool): Extension<PgPool>,
    AuthUser { payload }: AuthUser,
    Json(req): Json<BatchSignRequest>,
) -> Result<impl IntoResponse, AppError> {
    let key =
        std::env::var("AUTH_SECRET").map_err(|_| anyhow::anyhow!("AUTH_SECRET is not set"))?;
    let base_url = std::env::var("EXTERNAL_MAKI_BASE_URL")
        .map_err(|_| anyhow::anyhow!("EXTERNAL_MAKI_BASE_URL is not set"))?;

    let mut results = Vec::with_capacity(req.ids.len());
    for id_str in &req.ids {
        let id = resolve_song_id(id_str, &pool)
            .await
            .map_err(|(_, e)| anyhow::anyhow!(e))?;
        results.push(sign_for_id(
            id,
            &payload.sub,
            key.as_bytes(),
            &base_url,
            req.codec.as_deref(),
            req.dps.as_deref(),
        ));
    }

    Ok(Json(results))
}
