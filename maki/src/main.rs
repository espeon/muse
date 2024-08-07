use axum::{http::{self, HeaderValue, Method}, response::Html, routing::get, Extension, Router};
use sqlx::{Pool, postgres::Postgres};
use std::net::SocketAddr;

use tower_http::cors::CorsLayer;

mod api;
mod error;
mod db;
mod index;
mod metadata;
mod helpers;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok(); // ok does NOT return an error if there is no env file, which is what we want here

    let path = std::env::var("MOUNT").unwrap();

    let pool = db::get_pool().await?;
    let p_cloned = pool.clone();

    // start indexing/scanning in new thread
    tokio::spawn(async move {
        index::start(path.clone(), &path, p_cloned).await;
    });

    // start up our web server
    // dunno if i want this in a separate thread or not
    serve(pool).await?;

    Ok(())
}

async fn serve(pool: Pool<Postgres>) -> anyhow::Result<()> {
    // build our application with a route
    let app = Router::new()
        .route("/", get(handler))
        // Search routes
        .route("/search/:slug", get(api::index::search_songs))
        // Track routes
        .route("/track/:id", get(api::song::get_song))
        .route("/track/:id/stream", get(api::serve::serve_audio))
        .route("/track/:id/transcode", get(api::serve::serve_transcoded_audio))
        .route("/track/:id/like", get(api::song::like_song))
        .route("/track/:id/scrobble", get(api::song::scrobble_song))
        // Album routes
        .route("/album/:id", get(api::album::get_album))
        .route("/album", get(api::album::get_albums))
        .route("/art/:id", get(api::serve::serve_image))
        // Artist Routes
        .route("/artist/:id", get(api::artist::get_artist))
        .route("/artist", get(api::artist::get_artists))

        // Index routes
        .route("/index-q0b3.json", get(api::index::index_songs))
        .route("/home/", get(api::home::home))
        .layer(Extension(pool))
        .layer(
            CorsLayer::new()
                .allow_origin("*".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers([http::header::CONTENT_TYPE])
        );

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 3033));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap_or_else(|_| panic!("Failed to bind to {}", &addr));
    println!("listening on {}", addr);
    axum::serve(listener, app.into_make_service())
    .await?;

    Ok(())
}

async fn handler() -> Html<&'static str> {
    Html("<code>sh.kanbaru.kyoku</code>")
}
