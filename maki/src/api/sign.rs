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

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct SignResult {
    pub id: i32,
    pub url: String,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String)]
    pub signed_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String)]
    pub expires_at: OffsetDateTime,
}

#[derive(Deserialize, Default)]
pub struct SignQueryParams {
    /// When set, signs a transcode URL with this codec (e.g. "mp3", "opus", "aac")
    pub codec: Option<String>,
    /// Bitrate for transcoded URL (e.g. "128k"). Defaults to "128k".
    pub dps: Option<String>,
    /// When set to "hls", returns a signed master.m3u8 URL instead of a stream/transcode URL.
    pub mode: Option<String>,
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

fn make_hls_url(base_url: &str, id: i32, hmac: &str) -> String {
    format!("{}/api/v1/track/{}/hls/master.m3u8?tk={}", base_url, id, hmac)
}

fn make_token(user_sub: &str, key: &[u8]) -> (String, u64, u64) {
    let st = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let exp = st + 60 * 60; // 60 minutes
    let presigned = HmacMessage::new(st, exp, "*".to_owned(), user_sub.to_owned());
    (presigned.sign(key), st, exp)
}

fn sign_for_id(
    id: i32,
    user_sub: &str,
    key: &[u8],
    base_url: &str,
    codec: Option<&str>,
    dps: Option<&str>,
) -> SignResult {
    let (hmac, st, exp) = make_token(user_sub, key);
    SignResult {
        id,
        url: make_signed_url(base_url, id, &hmac, codec, dps),
        signed_at: OffsetDateTime::from_unix_timestamp(st as i64).unwrap(),
        expires_at: OffsetDateTime::from_unix_timestamp(exp as i64).unwrap(),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/track/{id}/sign",
    tag = "tracks",
    params(
        ("id" = String, Path, description = "Track ID or slug"),
        ("codec" = Option<String>, Query, description = "Transcode codec (e.g. mp3, opus, aac); omit for raw stream"),
        ("dps" = Option<String>, Query, description = "Transcode bitrate (default 128k)"),
    ),
    responses(
        (status = 200, description = "Signed URL", body = SignResult),
        (status = 404, description = "Track not found"),
    ),
    security(("bearer_token" = []))
)]
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

    if params.mode.as_deref() == Some("hls") {
        let (hmac, st, exp) = make_token(&payload.sub, key.as_bytes());
        return Ok(Json(SignResult {
            id: id_parsed,
            url: make_hls_url(&base_url, id_parsed, &hmac),
            signed_at: OffsetDateTime::from_unix_timestamp(st as i64).unwrap(),
            expires_at: OffsetDateTime::from_unix_timestamp(exp as i64).unwrap(),
        }));
    }

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

#[derive(Deserialize, utoipa::ToSchema)]
pub struct BatchSignRequest {
    /// Track IDs or slugs to sign
    pub ids: Vec<String>,
    pub codec: Option<String>,
    pub dps: Option<String>,
    /// When "hls", returns signed master.m3u8 URLs instead of stream/transcode URLs.
    pub mode: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/tracks/sign",
    tag = "tracks",
    request_body = BatchSignRequest,
    responses(
        (status = 200, description = "Signed URLs for all requested tracks", body = [SignResult]),
    ),
    security(("bearer_token" = []))
)]
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

    let hls = req.mode.as_deref() == Some("hls");
    let mut results = Vec::with_capacity(req.ids.len());
    for id_str in &req.ids {
        let id = resolve_song_id(id_str, &pool)
            .await
            .map_err(|(_, e)| anyhow::anyhow!(e))?;
        if hls {
            let (hmac, st, exp) = make_token(&payload.sub, key.as_bytes());
            results.push(SignResult {
                id,
                url: make_hls_url(&base_url, id, &hmac),
                signed_at: OffsetDateTime::from_unix_timestamp(st as i64).unwrap(),
                expires_at: OffsetDateTime::from_unix_timestamp(exp as i64).unwrap(),
            });
        } else {
            results.push(sign_for_id(
                id,
                &payload.sub,
                key.as_bytes(),
                &base_url,
                req.codec.as_deref(),
                req.dps.as_deref(),
            ));
        }
    }

    Ok(Json(results))
}
