use axum::{extract::Extension, http::StatusCode, Json};
use serde::Serialize;
use sqlx::PgPool;
use tracing::info;

use super::middleware::jwt::AdminUser;

#[derive(Serialize, utoipa::ToSchema)]
pub struct RescanResponse {
    pub status: &'static str,
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/rescan",
    tag = "admin",
    responses(
        (status = 202, description = "Rescan started", body = RescanResponse),
        (status = 403, description = "Admin access required"),
    ),
    security(("bearer_token" = []))
)]
/// POST /admin/rescan — trigger a full library rescan in the background.
/// Requires admin privileges. Returns 202 Accepted immediately.
pub async fn post_rescan(
    Extension(pool): Extension<PgPool>,
    AdminUser { .. }: AdminUser,
) -> Result<(StatusCode, Json<RescanResponse>), (StatusCode, String)> {
    let mount = std::env::var("MOUNT")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "MOUNT not set".to_string()))?;

    info!(target: "admin", "manual rescan triggered");

    tokio::spawn(async move {
        let cfg = crate::config::load_or_create_config("config/config.maki.json");
        crate::index::scan(&mount, pool, false, &cfg).await;
    });

    Ok((StatusCode::ACCEPTED, Json(RescanResponse { status: "scanning" })))
}
