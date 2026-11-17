use axum::{extract::Extension, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::middleware::jwt::AuthUser;

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MeResponse {
    pub id: i32,
    pub name: Option<String>,
    pub email: Option<String>,
    pub picture: Option<String>,
    pub is_admin: bool,
    pub lastfm_connected: bool,
    pub name_display: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct DisplayPreference {
    pub name_display: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/me",
    tag = "user",
    responses(
        (status = 200, description = "Authenticated user info", body = MeResponse),
    ),
    security(("bearer_token" = []))
)]
pub async fn get_me(
    Extension(pool): Extension<PgPool>,
    AuthUser { payload }: AuthUser,
) -> Result<Json<MeResponse>, (StatusCode, String)> {
    let user_id = payload
        .sub
        .parse::<i32>()
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let row = sqlx::query!(
        r#"
        SELECT users.id, users.name, users.email, users.image, users.is_admin, users.name_display,
               EXISTS (
                   SELECT 1 FROM user_lastfm WHERE user_lastfm."userId" = users.id
               ) AS "lastfm_connected!: bool"
        FROM users
        WHERE users.id = $1
        "#,
        user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(MeResponse {
        id: row.id,
        name: row.name,
        email: row.email,
        picture: row.image,
        is_admin: row.is_admin,
        lastfm_connected: row.lastfm_connected,
        name_display: row.name_display,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/me/display",
    tag = "user",
    request_body = DisplayPreference,
    responses((status = 200, body = MeResponse), (status = 400, description = "Invalid preference")),
    security(("bearer_token" = []))
)]
pub async fn post_display_preference(
    Extension(pool): Extension<PgPool>,
    AuthUser { payload }: AuthUser,
    axum::Json(body): axum::Json<DisplayPreference>,
) -> Result<Json<MeResponse>, (StatusCode, String)> {
    let user_id = payload.sub.parse::<i32>().map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let preference = match body.name_display.as_str() {
        "original" | "romanised" | "english" => body.name_display.as_str(),
        _ => return Err((StatusCode::BAD_REQUEST, "name_display must be original, romanised or english".into())),
    };
    sqlx::query!("UPDATE users SET name_display = $1 WHERE id = $2", preference, user_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    get_me(Extension(pool), AuthUser { payload }).await
}
