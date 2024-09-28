use std::time::{SystemTime, UNIX_EPOCH};

use axum::{extract::Path, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{error::AppError, helpers::HmacMessage};

use super::middleware::jwt::AuthUser;

#[derive(Serialize, Deserialize)]
pub struct SignResult {
    url: String,
    signed_at: OffsetDateTime,
    expires_at: OffsetDateTime,
}

/// Signs a track url given a track id. Assumes the track is valid.
pub async fn sign_track_url(
    Path(id): Path<String>,
    AuthUser { payload }: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let key =
        std::env::var("AUTH_SECRET").map_err(|_| anyhow::anyhow!("AUTH_SECRET is not set"))?;
    let base_url = std::env::var("EXTERNAL_MAKI_BASE_URL")
        .map_err(|_| anyhow::anyhow!("EXTERNAL_MAKI_BASE_URL is not set"))?;
    let id_parsed = id
        .parse::<i32>()
        .map_err(|e| anyhow::anyhow!("Failed to parse id: {}", e))?;
    // as unix timestamp
    let st = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // 60 minutes
    let exp = st + 60 * 60;
    let acl = "*";
    let presigned = HmacMessage::new(st, exp, acl.to_owned(), payload.sub);
    let hmac = presigned.sign(key.as_bytes());

    Ok(Json(SignResult {
        url: format!("{}/track/{}/stream?tk={}", base_url, id_parsed, hmac),
        signed_at: OffsetDateTime::from_unix_timestamp(st as i64).unwrap(),
        expires_at: OffsetDateTime::from_unix_timestamp(exp as i64).unwrap(),
    }))
}
