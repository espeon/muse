use api::{
    auth::providers::{create_shared_auth_provider, generic::GenericOidcPkceProvider},
    middleware::jwt::AuthUser,
};
use axum::{
    http::{self, HeaderValue, Method},
    response::Html,
    routing::get,
    Extension, Router,
};
use sqlx::{postgres::Postgres, Pool};
use std::net::SocketAddr;
use tracing::info;

use tower_http::cors::CorsLayer;

mod api;
pub mod clients;
mod config;
mod db;
mod error;
mod helpers;
mod index;
mod metadata;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt::init();

    let cfg = config::load_or_create_config("config/config.maki.json");

    let path = std::env::var("MOUNT").unwrap();

    let pool = db::get_pool().await?;
    let p_cloned = pool.clone();

    // detect dry run flag (overrides NO_SCAN)
    let dry_run = std::env::var("DRY_RUN").is_ok();
    // detect no scan flag
    let no_scan = std::env::var("NO_SCAN").is_ok();

    // start up our web server
    // dunno if i want this in a separate thread or not
    if !dry_run && !no_scan {
        // start indexing/scanning in new thread
        tokio::spawn(async move {
            index::start(path.clone(), &path, p_cloned, dry_run, &cfg).await;
        });
    } else if dry_run {
        index::start(path.clone(), &path, p_cloned, dry_run, &cfg).await;
        return Ok(());
    }
    serve(pool).await?;

    Ok(())
}

async fn serve(pool: Pool<Postgres>) -> anyhow::Result<()> {
    let auth = GenericOidcPkceProvider::new(
        reqwest::Client::new(),
        std::env::var("OIDC_CLIENT_ID").unwrap(),
        std::env::var("OIDC_CLIENT_SECRET").unwrap(),
        std::env::var("OIDC_AUTHORIZE").unwrap(),
        std::env::var("OIDC_ISSUER").unwrap(),
        std::env::var("OIDC_TOKEN").unwrap(),
        std::env::var("OIDC_REDIRECT").unwrap(),
    )
    .await?;
    let authcfg = create_shared_auth_provider(auth);
    // build our application with a route
    let app = Router::new()
        .route("/", get(handler))
        .nest("/api/v1", api::router())
        .nest("/auth", api::auth::router())
        .layer(Extension(pool))
        .layer(Extension(authcfg))
        .layer(
            CorsLayer::new()
                .allow_origin("*".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers([http::header::CONTENT_TYPE]),
        );

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 3033));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {}", &addr));
    info!("Maki is listening on {}", addr);
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn handler(AuthUser { payload }: AuthUser) -> Html<String> {
    let pl = format!(
        "<html><body><h1>Hello {}</h1><img src=\"{}\"></body></html>",
        payload.name.unwrap_or("Anonymous".to_string()),
        payload
            .picture
            .unwrap_or("https://i.imgur.com/0e0u4wH.png".to_string())
    );
    Html(pl)
}
