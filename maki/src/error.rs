use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum AppError {
    // Generic internal error
    #[error("internal error")]
    InternalError(#[from] anyhow::Error),
    #[error("failed to authenticate")]
    SqlError(#[from] sqlx::Error),
    #[error("failed to authenticate")]
    RequestError(#[from] reqwest::Error),
    #[error("Unable to read file or start subprocess")]
    UnableToRead(#[from] std::io::Error),
    #[error("not found")]
    NotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::InternalError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("500 internal server error: {}", err),
            ),
            Self::SqlError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("500 database error: {}", err),
            ),
            Self::RequestError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("500 request error: {}", err),
            ),
            Self::UnableToRead(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("500 unable to read file or start subprocess: {}", err),
            ),
            Self::NotFound => (StatusCode::NOT_FOUND, "404 not found".into()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
