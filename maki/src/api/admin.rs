use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::info;

use super::middleware::jwt::AdminUser;

#[derive(Serialize, utoipa::ToSchema)]
pub struct RescanResponse {
    pub status: &'static str,
}

#[derive(Debug, Deserialize)]
pub struct AnalyzeParams {
    pub kind: Option<crate::analysis::AnalysisKind>,
    pub retry_failures: Option<bool>,
    pub track_id: Option<Vec<i32>>,
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

#[utoipa::path(
    post,
    path = "/api/v1/admin/analyze",
    tag = "admin",
    responses(
        (status = 202, description = "Audio analysis started", body = RescanResponse),
        (status = 403, description = "Admin access required"),
    ),
    security(("bearer_token" = []))
)]
/// POST /admin/analyze — retry failed files and analyze tracks missing features.
pub async fn post_analyze(
    Extension(pool): Extension<PgPool>,
    Extension(config): Extension<crate::config::Config>,
    Query(params): Query<AnalyzeParams>,
    AdminUser { .. }: AdminUser,
) -> Result<(StatusCode, Json<RescanResponse>), (StatusCode, String)> {
    let kind = params.kind.unwrap_or(crate::analysis::AnalysisKind::All);
    let enabled = match kind {
        crate::analysis::AnalysisKind::Similarity => config.audio_analysis_enabled,
        crate::analysis::AnalysisKind::Mix => config.mix_analysis_enabled,
        crate::analysis::AnalysisKind::All => {
            config.audio_analysis_enabled || config.mix_analysis_enabled
        }
    };
    if !enabled {
        return Err((
            StatusCode::CONFLICT,
            "requested audio analysis is disabled in config.maki.json".to_string(),
        ));
    }

    let track_ids = params.track_id.unwrap_or_default();
    info!(target: "analysis", "manual {kind:?} analysis triggered for {track_ids:?}");
    crate::analysis::enqueue_kind(
        pool,
        config,
        kind,
        params.retry_failures.unwrap_or(true),
        track_ids,
    );
    Ok((
        StatusCode::ACCEPTED,
        Json(RescanResponse {
            status: "analyzing",
        }),
    ))
}
