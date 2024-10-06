use axum::{
    http::{HeaderMap, StatusCode},
    Extension,
};
use base64::Engine;
use sqlx::PgPool;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct LogoutRequest {
    refresh_token: String,
}

pub async fn logout(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<String, (StatusCode, String)> {
    // Get the hash of the refresh token from the headers
    // TODO: make middleware to check and pre-hash this header
    let refresh_req = match headers.get("Authorization") {
        Some(value) => match value.to_str() {
            Ok(value) => match value.split_once("Bearer ") {
                Some((_, value)) => value,
                None => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        "Invalid Authorization header".to_string(),
                    ))
                }
            },
            Err(_) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Invalid Authorization header".to_string(),
                ))
            }
        },
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Missing Authorization header".to_string(),
            ))
        }
    };

    if refresh_req.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid Authorization header".to_string(),
        ));
    }
    let refresh_token_hash = ring::digest::digest(&ring::digest::SHA256, refresh_req.as_bytes());
    let refresh_token_hash = base64::engine::general_purpose::URL_SAFE.encode(refresh_token_hash);

    // Delete the session from the database
    sqlx::query!(
        "DELETE FROM sessions WHERE refresh_token = $1",
        refresh_token_hash
    )
    .execute(&pool)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok("Deleted the session from the database, if it existed.".to_string())
}
