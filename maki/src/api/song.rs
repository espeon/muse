use axum::{extract::{Extension, Path}, http::StatusCode, Json};
use sqlx::PgPool;

use crate::api::Track;

pub async fn get_song(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
) -> Result<axum::Json<Vec<Track>>, (StatusCode, String)> {
    // internal_error
    let id_parsed = match id.split('.').collect::<Vec<&str>>()[0].parse::<i32>() {
        Ok(e) => e,
        Err(e) => return Err(internal_error(e)),
    };
        match sqlx::query_as!(Track,
            r#"
        SELECT song.id, number, disc, song.name, album, song.album_artist, liked, duration, plays, lossless, song.created_at, song.updated_at, last_play, year,
        	album.name as album_name,
            artist.name as artist_name,
            STRING_AGG(CAST(song_artist.artist AS VARCHAR), ',') as artists

        FROM song
        
        LEFT JOIN album ON song.album = album.id
        LEFT JOIN artist ON song.album_artist = artist.id
        LEFT JOIN song_artist ON song.id = song_artist.song

        WHERE song.id = $1
        GROUP BY song.id, number, song.name, album, song.album_artist, liked, duration, plays, lossless, song.created_at, song.updated_at, last_play, year,
        	album.name, artist.name
    "#, id_parsed
        )
        .fetch_all(&pool)
        .await{
            Ok(e) => Ok(Json(e)),
            Err(e) => Err(internal_error(e)),
        }
}

/// like song
/// 
pub async fn like_song(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
) -> Result<axum::Json<()>, (StatusCode, String)> {
    let id_parsed = id.split('.').collect::<Vec<&str>>()[0].parse::<i32>().unwrap();
    match sqlx::query!("UPDATE song SET liked = true WHERE id = $1", id_parsed)
        .execute(&pool)
        .await
    {
        Ok(_) => Ok(Json(())),
        Err(e) => Err(internal_error(e)),
    }
}

/// scrobble song
/// TODO: scrobble song to last.fm as well, if configured
/// 
/// ?finished - mark song as finished
pub async fn scrobble_song(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
) -> Result<axum::Json<()>, (StatusCode, String)> {
    let id_parsed = id.split('.').collect::<Vec<&str>>()[0].parse::<i32>().unwrap();
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