pub mod serve;
pub mod index;
pub mod song;
pub mod album;
pub mod artist;
pub mod home;

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct Track {
    id: i32,
    name: String,
    album_artist: i32,
    // comma separated list of artist IDs
    artists: Option<String>,
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
    artist_name: String
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
    artist_name: ArtistPartial
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
    tracks: Option<Vec<Track>>
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
    artist_picture: Option<String>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlbumPartial {
    id: i32,
    name: String,
    art: Vec<String>,
    year: Option<i32>,
    count: Option<i64>,
    artist: Option<ArtistPartial>
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