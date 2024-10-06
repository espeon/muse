use axum::{
    routing::{get, post},
    Router,
};

pub mod login;
pub mod logout;
pub mod pkce_store;
pub mod providers;
pub mod refresh;

pub fn router() -> Router {
    Router::new()
        .route("/login", get(login::start_auth))
        .route("/login/callback", get(login::finish_auth))
        .route("/refresh", post(refresh::refresh_token))
        .route("/logout", post(logout::logout))
}
