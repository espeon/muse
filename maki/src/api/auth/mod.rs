use axum::{
    routing::{get, post},
    Router,
};

pub mod controller;
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
        .route("/controller/", get(controller::controller_page))
        .route("/controller", get(controller::controller_page))
}
