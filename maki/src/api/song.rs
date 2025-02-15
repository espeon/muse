use std::collections::HashMap;

use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use tracing::debug;

use crate::{
    api::{ArtistPartial, Track, TrackRaw},
    clients,
};

use super::middleware::jwt::AuthUser;

pub async fn get_song(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
) -> Result<axum::Json<Vec<Track>>, (StatusCode, String)> {
    let id_parsed = id.split('.').collect::<Vec<&str>>()[0]
        .parse::<i32>()
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    // Fetch raw tracks
    let tracks = match sqlx::query_as!(TrackRaw,
        r#"
        SELECT song.id, disc, number, song.name, album, song.album_artist, liked, duration, plays, lossless, song.created_at, song.updated_at, last_play, year,
               album.name as album_name,
               artist.name as artist_name
        FROM song
        LEFT JOIN album ON song.album = album.id
        LEFT JOIN artist ON song.album_artist = artist.id
        WHERE song.id = $1
        GROUP BY song.id, disc, number, song.name, album, song.album_artist, liked, duration, plays, lossless, song.created_at, song.updated_at, last_play, year,
                 album.name, artist.name
        "#, id_parsed
    )
    .fetch_all(&pool)
    .await {
        Ok(e) => e,
        Err(e) => return Err(internal_error(e)),
    };

    // Get the song IDs for the fetched tracks
    let song_ids: Vec<i32> = tracks.iter().map(|track| track.id).collect();

    // Fetch the artists related to each song
    let song_artists = match sqlx::query_as!(
        ArtistPartial,
        r#"
        SELECT
            artist.id,
            artist.name,
            artist.picture,
            COUNT(album.id) AS num_albums
        FROM
            artist
        LEFT JOIN
            album ON artist.id = album.artist
        WHERE
            artist.id IN (
                SELECT artist FROM song_artist WHERE song = ANY($1)
            )
        GROUP BY
            artist.id;
        "#,
        &song_ids
    )
    .fetch_all(&pool)
    .await
    {
        Ok(e) => e,
        Err(e) => return Err(internal_error(e)),
    };

    // Map the artists by their ID for quick lookup
    let artist_map: HashMap<i32, ArtistPartial> = song_artists
        .into_iter()
        .map(|artist| (artist.id, artist))
        .collect();

    // Fetch the song-artist relationships
    let song_artist_relationships = match sqlx::query!(
        r#"
        SELECT song_artist.song, song_artist.artist
        FROM song_artist
        WHERE song_artist.song = ANY($1)
        "#,
        &song_ids
    )
    .fetch_all(&pool)
    .await
    {
        Ok(e) => e,
        Err(e) => return Err(internal_error(e)),
    };

    // Map the relationships by song ID
    let mut song_to_artists: HashMap<i32, Vec<ArtistPartial>> = HashMap::new();
    for relationship in song_artist_relationships {
        if let Some(artist) = artist_map.get(&relationship.artist) {
            song_to_artists
                .entry(relationship.song)
                .or_default()
                .push(artist.clone());
        }
    }

    // Convert the raw tracks into the final Track struct
    let final_tracks: Vec<Track> = tracks
        .into_iter()
        .map(|track| {
            let artists = song_to_artists.get(&track.id).cloned().unwrap_or_default();
            Track {
                id: track.id,
                disc: track.disc,
                number: track.number,
                name: track.name,
                album: track.album,
                album_artist: track.album_artist,
                liked: track.liked,
                duration: track.duration,
                plays: track.plays,
                lossless: track.lossless,
                created_at: track.created_at,
                updated_at: track.updated_at,
                last_play: track.last_play,
                year: track.year,
                album_name: track.album_name,
                artist_name: track.artist_name,
                artists,
            }
        })
        .collect();

    Ok(Json(final_tracks))
}
/// like song
///
pub async fn like_song(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
) -> Result<axum::Json<()>, (StatusCode, String)> {
    let id_parsed = id.split('.').collect::<Vec<&str>>()[0]
        .parse::<i32>()
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    match sqlx::query!("UPDATE song SET liked = true WHERE id = $1", id_parsed)
        .execute(&pool)
        .await
    {
        Ok(_) => Ok(Json(())),
        Err(e) => Err(internal_error(e)),
    }
}

/// Set playing status of song on external services
pub async fn set_playing(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
    AuthUser { payload }: AuthUser,
) -> Result<axum::Json<()>, (StatusCode, String)> {
    let id_parsed = id.split('.').collect::<Vec<&str>>()[0]
        .parse::<i32>()
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    // get song info
    let song = match sqlx::query!(
        r#"
        SELECT path, number, song.name, album.name as album_name, duration, artist.name as artist_name from song
                LEFT JOIN album on song.album = album.id
                LEFT JOIN artist on song.album_artist = artist.id
                WHERE song.id = $1
    "#,
        id_parsed
    )
    .fetch_one(&pool)
    .await
    {
        Ok(e) => e,
        Err(e) => return Err(internal_error(e)),
    };
    match clients::lastfm::set_now_playing(
        payload.sub.parse::<i32>().unwrap(),
        &pool,
        &song.name,
        &song.artist_name,
        &song.album_name,
        song.duration as u32,
    )
    .await
    {
        // silently fail, assume scrobble failed b/c user hasn't set up last.fm
        Ok(_) => {
            debug!("set now playing on last.fm for song {}", id);
            Ok(Json(()))
        }
        Err(e) => {
            debug!(
                "failed to set now playing on last.fm for song {}: {}",
                id, e
            );
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

/// scrobble song
/// TODO: scrobble song to last.fm as well, if configured
///
/// ?finished - mark song as finished
pub async fn scrobble_song(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
    AuthUser { payload }: AuthUser,
) -> Result<axum::Json<()>, (StatusCode, String)> {
    let id_parsed = id.split('.').collect::<Vec<&str>>()[0]
        .parse::<i32>()
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    // get song info
    let song = match sqlx::query!(
        r#"
        SELECT path, number, song.name, album.name as album_name, duration, artist.name as artist_name from song
                LEFT JOIN album on song.album = album.id
                LEFT JOIN artist on song.album_artist = artist.id
                WHERE song.id = $1
    "#,
        id_parsed
    )
    .fetch_one(&pool)
    .await
    {
        Ok(e) => e,
        Err(e) => return Err(internal_error(e)),
    };
    match clients::lastfm::scrobble(
        payload.sub.parse::<i32>().unwrap(),
        &pool,
        &song.name,
        &song.artist_name,
        &song.album_name,
        song.duration as u32,
    )
    .await
    {
        // silently fail, assume scrobble failed b/c user hasn't set up last.fm
        Ok(_) => debug!("scrobbled song {} to last.fm", id),
        Err(e) => debug!("failed to scrobble song {} to last.fm: {}", id, e),
    }
    match sqlx::query!("UPDATE song SET plays = plays + 1 WHERE id = $1", id_parsed)
        .execute(&pool)
        .await
    {
        Ok(_) => Ok(Json(())),
        Err(e) => Err(internal_error(e)),
    }
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
