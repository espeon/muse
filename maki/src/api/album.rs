use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::{PgPool, Postgres};

use crate::api::{Album, AlbumPartialRaw, AlbumRaw, ArtistPartial, Track};

use super::{AlbumPartial, AllAlbumsPartial};

pub async fn get_album(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
) -> Result<axum::Json<Album>, (StatusCode, String)> {
    let id_parsed = id.split('.').collect::<Vec<&str>>()[0]
        .parse::<i32>()
        .unwrap();

    let album = match sqlx::query_as!(AlbumRaw, r#"
        SELECT album.id, album.name, year, album.created_at, album.updated_at, 
            artist.id as artist_id, artist.name as artist_name, artist.picture as artist_picture, artist.bio as artist_bio, 
            artist.created_at as artist_created_at, artist.updated_at as artist_updated_at,
            STRING_AGG(CAST(album_art.path AS VARCHAR), ',') as arts
        FROM album
        LEFT JOIN artist ON album.artist = artist.id
        LEFT JOIN album_art ON album.id = album_art.album
        
        WHERE album.id = $1

        GROUP BY album.id, artist.id
        "#, id_parsed
    )
    .fetch_one(&pool)
    .await{
        Ok(e) => e,
        Err(e) => return Err(internal_error(e)),
    };

    match  sqlx::query_as!(Track,
            r#"
        SELECT song.id, disc, number, song.name, song.album, song.album_artist, liked, duration, plays, lossless, song.created_at, song.updated_at, last_play, year,
        	album.name as album_name,
            artist.name as artist_name,
            STRING_AGG(CAST(song_artist.artist AS VARCHAR), ',') as artists
        FROM song
        
        LEFT JOIN album ON song.album = album.id
        LEFT JOIN artist ON song.album_artist = artist.id
        LEFT JOIN song_artist ON song.id = song_artist.song
        

        WHERE song.album = $1
        GROUP BY disc, number, song.name, song.album, song.id, artist.id, song.album_artist, liked, duration, plays, lossless, song.created_at, song.updated_at, last_play, year,
        	album.name,
            artist.name
        ORDER BY disc ASC, number ASC
    "#, id_parsed
        )
        .fetch_all(&pool)
        .await{
            Ok(e) => Ok(Json(Album{
                id: album.id, 
                name: album.name,
                art: match album.arts {
                    Some(e) => e.split(',').map(|i| i.to_string()).collect(),
                    None => vec![],
                },
                year: album.year,
                created_at: album.created_at,
                updated_at: album.updated_at,
                artist: ArtistPartial{
                    id: album.artist_id,
                    name: album.artist_name,
                    picture: album.artist_picture,
                    num_albums: None,
                },
                tracks: Some(e)
            })),
            Err(e) => Err(internal_error(e)),
        }
}

#[derive(Deserialize, Default)]
pub struct GetAlbumParams {
    #[serde(default)]
    sortby: Option<SortByAlbumOptions>,
    #[serde(default)]
    dir: Option<String>,
    #[serde(default)]
    limit: Option<i32>,
    #[serde(default)]
    offset: Option<i32>,
}

#[derive(Deserialize)]
enum SortByAlbumOptions {
    #[serde(alias = "id")]
    Id,
    #[serde(alias = "artist")]
    ArtistName,
    #[serde(alias = "album")]
    AlbumName,
    #[serde(alias = "year")]
    Year,
}

impl SortByAlbumOptions {
    fn as_str(&self) -> &str {
        match self {
            Self::Id => "album.id",
            Self::ArtistName => "artist.name",
            Self::AlbumName => "album.name",
            Self::Year => "album.year",
        }
    }
}

#[derive(Deserialize)]
enum DirOptions {
    #[serde(alias = "asc")]
    Asc,
    #[serde(alias = "desc")]
    Desc,
}

#[axum_macros::debug_handler]
pub async fn get_albums(
    Extension(pool): Extension<PgPool>,
    Query(GetAlbumParams { sortby, dir, limit, offset }): Query<GetAlbumParams>,
) -> Result<axum::Json<AllAlbumsPartial>, (StatusCode, String)> {
        let latest_albums = match sqlx::query_as::<Postgres, AlbumPartialRaw>(
            &format!("
            SELECT album.id, album.name, album.year, count(song.id), artist.id as artist_id, artist.name as artist_name, artist.picture as artist_picture,
            STRING_AGG(CAST(album_art.path AS VARCHAR), ',') as arts
    
            FROM album
            LEFT JOIN song ON song.album = album.id
            LEFT JOIN artist ON album.artist = artist.id
            LEFT JOIN album_art ON album.id = album_art.album
            
            group by album.id, album.name, artist.name, artist.id
            order by {} {}
            limit {}
            offset {}
    ", sortby.unwrap_or(SortByAlbumOptions::ArtistName).as_str(), dir.unwrap_or("desc".to_string()).as_str(), limit.unwrap_or(20), offset.unwrap_or(0)),
        )
        .fetch_all(&pool)
        .await
        {
            Ok(e) => e.iter().map(|i| AlbumPartial{
                id:i.id,
                name:i.name.clone(),
                art: i.arts.clone().unwrap_or("".to_string()).split(',').map(|i| i.to_string()).collect(),
                year:i.year,
                count:i.count,
                artist:Some(ArtistPartial{
                    id: i.artist_id,
                    name: i.artist_name.clone(),
                    picture: i.artist_picture.clone(),
                    num_albums: None,
                })
            }).collect(),
            Err(e) => return Err(internal_error(e)),
        };
        Ok(Json(AllAlbumsPartial{ albums: latest_albums, limit: limit.unwrap_or(20), offset: offset.unwrap_or(0) }))
    }

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
