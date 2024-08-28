use axum::{
    extract::{Extension, Host},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::api::{build_default_art_url, AlbumPartialRaw};

use super::{AlbumPartial, ArtistPartial};

#[derive(Debug, Serialize, Deserialize)]
pub struct Home {
    latest_albums: Vec<AlbumPartial>,
}

#[axum::debug_handler]
pub async fn home(
    Extension(pool): Extension<PgPool>,
    Host(host): Host,
) -> Result<axum::Json<Home>, (StatusCode, String)> {
    let art_url = build_default_art_url(host);
    let latest_albums = match sqlx::query_as!(
        AlbumPartialRaw,
        r#"
        SELECT album.id, album.name, album.year, count(song.id), artist.id as artist_id, artist.name as artist_name, artist.picture as artist_picture,
        STRING_AGG(CAST(album_art.path AS VARCHAR), ',') as arts

        FROM album
        LEFT JOIN song ON song.album = album.id
        LEFT JOIN artist ON album.artist = artist.id
        LEFT JOIN album_art ON album.id = album_art.album

        group by album.id, album.name, artist.id
        order by album.created_at desc
        limit 13
"#,
    )
    .fetch_all(&pool)
    .await
    {
        Ok(e) => e.iter().map(|i| AlbumPartial{
            id:i.id,
            name:i.name.clone(),
            art: i.arts.clone().unwrap_or("".to_string()).split(',').map(|i| art_url.clone() + i).collect(),
            year:i.year,
            count:i.count,
            artist:Some(ArtistPartial{
                id: i.artist_id,
                name: i.artist_name.clone(),
                picture: i.artist_picture.clone(),
                num_albums: None
            })
        }).collect(),
        Err(e) => return Err(internal_error(e)),
    };
    Ok(Json(Home { latest_albums }))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
