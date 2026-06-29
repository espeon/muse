use axum::{
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};

const CONTROLLER_HTML: &str = include_str!("../../../web/controller.html");

/// Serves the web controller UI. The page is a self-contained
/// vanilla-JS SPA that talks to the WebSocket at `/api/v1/remote/ws`.
/// The browser's `authjs.session-token` cookie (set by the OIDC
/// callback) authenticates the WS upgrade.
pub async fn controller_page() -> Response {
    match HeaderValue::from_str("text/html; charset=utf-8") {
        Ok(value) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, value)],
            CONTROLLER_HTML,
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "controller page: bad content-type",
        )
            .into_response(),
    }
}
