pub mod album;
pub mod artist;
pub mod home;
pub mod index;
pub mod serve;
pub mod sign;
pub mod song;

pub mod auth;

pub mod connect;
pub mod middleware;

use axum::{
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use time::OffsetDateTime;

pub fn router() -> Router {
    Router::new()
        .route("/lastfm/token", get(connect::lastfm::get_lastfm_token))
        .route(
            "/lastfm/session",
            post(connect::lastfm::post_lastfm_session),
        )
        // Search routes
        .route("/search/:slug", get(index::search_songs))
        // Track routes
        .route("/track/:id", get(song::get_song))
        .route("/track/:id/sign", get(sign::sign_track_url))
        .route("/track/:id/stream", get(serve::serve_audio))
        .route("/track/:id/transcode", get(serve::serve_transcoded_audio))
        .route("/track/:id/like", get(song::like_song))
        .route("/track/:id/scrobble", get(song::scrobble_song))
        .route("/track/:id/play", get(song::set_playing))
        // Album routes
        .route("/album/:id", get(album::get_album))
        .route("/album", get(album::get_albums))
        .route("/art/:id", get(serve::serve_image))
        // Artist Routes
        .route("/artist/:id", get(artist::get_artist))
        .route("/artist", get(artist::get_artists))
        // Index routes
        .route("/index-q0b3.json", get(index::index_songs))
        .route("/home/", get(home::home))
        .nest("/auth", auth::router())
}

pub fn build_default_art_url(host: String) -> String {
    let art_path = "api/v1/art/";
    // build default art url base
    let art_url: String = if std::env::var("EXTERNAL_MAKI_BASE_URL").is_ok() {
        let u = std::env::var("EXTERNAL_MAKI_BASE_URL").unwrap().to_string() + "/" + art_path;
        if u.ends_with('/') {
            u
        } else {
            u + "/"
        }
    } else {
        // if host has a port or is not localhost, use http
        if host.contains(':') || host.contains("localhost") {
            format!("http://{host}/{art_path}")
        } else {
            format!("https://{host}/{art_path}")
        }
    };
    art_url
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Track {
    id: i32,
    name: String,
    album_artist: i32,
    // comma separated list of artist IDs
    artists: Vec<ArtistPartial>,
    plays: Option<i32>,
    duration: i32,
    liked: Option<bool>,
    last_play: Option<OffsetDateTime>, // was serde_json::Value
    year: Option<i32>,
    number: Option<i32>,
    disc: Option<i32>,
    lossless: Option<bool>,
    created_at: OffsetDateTime,
    updated_at: Option<OffsetDateTime>,
    album: i32,
    album_name: String,
    artist_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrackRaw {
    id: i32,
    number: Option<i32>,
    disc: Option<i32>,
    name: String,
    album: i32,
    album_artist: i32,
    liked: Option<bool>,
    duration: i32,
    plays: Option<i32>,
    lossless: Option<bool>,
    created_at: OffsetDateTime,
    updated_at: Option<OffsetDateTime>,
    last_play: Option<OffsetDateTime>,
    year: Option<i32>,
    album_name: String,
    artist_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackPartial {
    id: i32,
    name: String,
    duration: i32,
    liked: Option<bool>,
    number: Option<i32>,
    disc: Option<i32>,
    lossless: Option<bool>,
    album_name: AlbumPartial,
    artist_name: ArtistPartial,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Album {
    id: i32,
    name: String,
    art: Vec<String>,
    year: Option<i32>,
    created_at: OffsetDateTime,
    updated_at: Option<OffsetDateTime>,
    artist: ArtistPartial,
    tracks: Option<Vec<Track>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AlbumPartialRaw {
    id: i32,
    name: String,
    count: Option<i64>,
    arts: Option<String>,
    year: Option<i32>,
    artist_id: i32,
    artist_name: String,
    artist_picture: Option<String>,
}

pub struct AlbumPartialRawWithGenre {
    id: i32,
    name: String,
    count: Option<i64>,
    arts: Option<String>,
    year: Option<i32>,
    artist_id: i32,
    artist_name: String,
    artist_picture: Option<String>,
    genre: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlbumPartial {
    id: i32,
    name: String,
    art: Vec<String>,
    year: Option<i32>,
    count: Option<i64>,
    artist: Option<ArtistPartial>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AllAlbumsPartial {
    albums: Vec<AlbumPartial>,
    limit: i32,
    offset: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlbumRaw {
    id: i32,
    name: String,
    year: Option<i32>,
    arts: Option<String>,
    created_at: OffsetDateTime,
    updated_at: Option<OffsetDateTime>,
    artist_id: i32,
    artist_name: String,
    artist_picture: Option<String>,
    artist_bio: Option<String>,
    artist_created_at: OffsetDateTime,
    artist_updated_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Artist {
    id: i64,
    name: String,
    picture: Option<String>,
    tags: Option<String>,
    bio: Option<String>,
    created_at: OffsetDateTime,
    updated_at: Option<OffsetDateTime>,
    albums: Vec<AlbumPartial>,
}

pub struct ArtistRaw {
    id: i64,
    name: String,
    picture: Option<String>,
    tags: Option<String>,
    bio: Option<String>,
    created_at: OffsetDateTime,
    updated_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct ArtistPartial {
    id: i32,
    name: String,
    picture: Option<String>,
    num_albums: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Genre {
    id: i64,
    name: String,
    created_at: String,
    updated_at: Option<String>,
}
