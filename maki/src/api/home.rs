use axum::{
    extract::{Extension, Host},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::api::{build_default_art_url, AlbumPartialRaw, AlbumPartialRawWithGenre};

use super::{AlbumPartial, ArtistPartial};

pub type Home = Vec<HomeRow>;

#[derive(Debug, Serialize, Deserialize)]
pub enum HomeRowType {
    Album,
    Artist,
    Track,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HomeRow {
    pub name: String,
    pub albums: Vec<AlbumPartial>,
    /// Type of row (e.g. Album, Artist, Track)
    /// Will be used to determine the component to render when displaying
    pub row_type: HomeRowType,
    /// Optional resource to link to in the form muse://<resource_type>/<resource_id>
    pub resource: Option<String>,
}

#[axum::debug_handler]
pub async fn home(
    Extension(pool): Extension<PgPool>,
    Host(host): Host,
) -> Result<axum::Json<Home>, (StatusCode, String)> {
    let art_url = build_default_art_url(host);
    let mut rows: Vec<HomeRow> = Vec::new();
    let latest_albums: Vec<AlbumPartial> = match sqlx::query_as!(
        AlbumPartialRaw,
        r#"
        SELECT album.id, album.name, album.year, count(song.id), artist.id as artist_id, artist.name as artist_name, artist.picture as artist_picture,
        STRING_AGG(CAST(album_art.path AS VARCHAR), ',') as arts

        FROM album
        LEFT JOIN song ON song.album = album.id
        LEFT JOIN artist ON album.artist = artist.id
        LEFT JOIN album_art ON album.id = album_art.album

        GROUP BY album.id, album.name, artist.id
        ORDER BY album.created_at DESC
        LIMIT 13
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
    rows.push(HomeRow {
        name: "Latest Albums".to_string(),
        albums: latest_albums,
        row_type: HomeRowType::Album,
        resource: None,
    });

    // random albums
    // TODO: make this configurable
    let random_albums: Vec<AlbumPartial> = match sqlx::query_as!(
        AlbumPartialRaw,
        r#"
        SELECT album.id, album.name, album.year, count(song.id), artist.id as artist_id, artist.name as artist_name, artist.picture as artist_picture,
        STRING_AGG(CAST(album_art.path AS VARCHAR), ',') as arts

        FROM album
        LEFT JOIN song ON song.album = album.id
        LEFT JOIN artist ON album.artist = artist.id
        LEFT JOIN album_art ON album.id = album_art.album

        GROUP BY album.id, album.name, artist.id
        ORDER BY RANDOM()
        LIMIT 13
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
    rows.push(HomeRow {
        name: "Random Albums".to_string(),
        albums: random_albums,
        row_type: HomeRowType::Album,
        resource: None,
    });

    // Albums from a random genre
    // TODO: make this configurable
    let selected_genre: String;
    let genre_albums: Vec<AlbumPartial> = match sqlx::query_as!(
        AlbumPartialRawWithGenre,
        r#"
        WITH random_genre AS (
            SELECT genre.id, genre.name
            FROM genre
            JOIN album_genre ON genre.id = album_genre.genre
            GROUP BY genre.id
            HAVING COUNT(album_genre.album) >= 3
            ORDER BY RANDOM()
            LIMIT 1
        )
        SELECT album.id, album.name, album.year, count(song.id), artist.id as artist_id, artist.name as artist_name, artist.picture as artist_picture, random_genre.name as genre,
        STRING_AGG(CAST(album_art.path AS VARCHAR), ',') as arts

        FROM album
        LEFT JOIN song ON song.album = album.id
        LEFT JOIN artist ON album.artist = artist.id
        LEFT JOIN album_art ON album.id = album_art.album
        LEFT JOIN album_genre ON album.id = album_genre.album
        JOIN random_genre ON album_genre.genre = random_genre.id

        GROUP BY album.id, album.name, artist.id, random_genre.id, random_genre.name
        ORDER BY RANDOM()
        LIMIT 13
"#,
    )
    .fetch_all(&pool)
    .await
    {
        Ok(e) => {
            // get the first genre
            selected_genre = e.first().unwrap().genre.clone().unwrap_or("".to_string());
            e.iter().map(|i| AlbumPartial{
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
        }).collect()},
        Err(e) => return Err(internal_error(e)),
    };
    rows.push(HomeRow {
        name: format!("Albums from the genre {}", selected_genre),
        albums: genre_albums,
        row_type: HomeRowType::Album,
        resource: None,
    });

    Ok(Json(rows))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
