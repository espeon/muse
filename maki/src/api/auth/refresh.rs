use axum::{
    extract::Json,
    http::{HeaderMap, StatusCode},
    Extension,
};
use base64::Engine;
use sqlx::PgPool;
use time::Duration;

use crate::api::auth::login::UserInfoDbResponse;

use super::login::new_jwt;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct RefreshTokenRequest {
    refresh_token: String,
}

#[axum_macros::debug_handler]
pub async fn refresh_token(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<Json<RenewSessionResponse>, (StatusCode, String)> {
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
    // We hash the refresh token and compare it to hashes in the database.
    let refresh_token_hash = ring::digest::digest(&ring::digest::SHA256, refresh_req.as_bytes());
    // b64 encode the hash
    let refresh_token_hash = base64::engine::general_purpose::URL_SAFE.encode(refresh_token_hash);
    let session = sqlx::query_as!(
        SessionDbResponse,
        r#"
        SELECT sessions."userId" AS "userid"
        FROM sessions
        WHERE refresh_token = $1 AND expires > NOW();
        "#,
        refresh_token_hash
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let session = session.ok_or((
        StatusCode::UNAUTHORIZED,
        "Invalid refresh token".to_string(),
    ))?;

    // Fetch user info
    let user = sqlx::query_as!(
        UserInfoDbResponse,
        r#"
        SELECT id, name, email, image FROM users WHERE id = $1
        "#,
        session.userid
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let expires = Duration::seconds(60 * 60);

    // Return a new JWT
    let new_session = new_jwt(user, expires)?;

    Ok(Json(RenewSessionResponse {
        session_token: new_session,
        expiry: (time::OffsetDateTime::now_utc() + expires).unix_timestamp(),
    }))
}
pub struct SessionDbResponse {
    userid: i32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RenewSessionResponse {
    pub session_token: String,
    pub expiry: i64,
}
