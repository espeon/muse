use axum::{extract::Extension, http::StatusCode, Json};
use serde::Serialize;
use sqlx::PgPool;

use super::middleware::jwt::AuthUser;

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub id: i32,
    pub name: Option<String>,
    pub email: Option<String>,
    pub picture: Option<String>,
    pub is_admin: bool,
    pub lastfm_connected: bool,
}

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
        SELECT users.id, users.name, users.email, users.image, users.is_admin,
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
    }))
}
