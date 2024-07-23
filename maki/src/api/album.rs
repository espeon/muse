use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::{PgPool, Postgres};

use crate::api::{Album, AlbumPartialRaw, AlbumRaw, TrackRaw, ArtistPartial, Track};

use super::{AlbumPartial, AllAlbumsPartial};

pub async fn get_album(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
) -> Result<axum::Json<Album>, (StatusCode, String)> {
    let id_parsed = id.split('.').collect::<Vec<&str>>()[0]
        .parse::<i32>().map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

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
        Ok(e) => {
            if e.arts.is_none() {
                e
            } else {
                AlbumRaw {
                    id: e.id,
                    name: e.name,
                    year: e.year,
                    created_at: e.created_at,
                    updated_at: e.updated_at,
                    artist_id: e.artist_id,
                    artist_name: e.artist_name,
                    artist_picture: e.artist_picture,
                    artist_bio: e.artist_bio,
                    artist_created_at: e.artist_created_at,
                    artist_updated_at: e.artist_updated_at,
                    arts: Some(e.arts.unwrap_or("".to_string()).split(',').map(|i| std::env::var("MAKI_ART_URL").expect("MAKI_ART_URL not set") + i).collect()),
                }
            }
        },
        Err(e) => return Err(internal_error(e)),
    };

    let tracks = match sqlx::query_as!(TrackRaw,
            r#"
        SELECT song.id, disc, number, song.name, song.album, song.album_artist, liked, duration, plays, lossless, song.created_at, song.updated_at, last_play, year,
            album.name as album_name,
            artist.name as artist_name
        FROM song
        LEFT JOIN album ON song.album = album.id
        LEFT JOIN artist ON song.album_artist = artist.id
        WHERE song.album = $1
        GROUP BY disc, number, song.name, song.album, song.id, artist.id, song.album_artist, liked, duration, plays, lossless, song.created_at, song.updated_at, last_play, year,
            album.name,
            artist.name
        ORDER BY disc ASC, number ASC
    "#, id_parsed
        )
        .fetch_all(&pool)
        .await{
            Ok(e) => e,
            Err(e) => return Err(internal_error(e)),
        };

    // Get the artists for each song
    let song_ids: Vec<i32> = tracks.iter().map(|track| track.id).collect();

    let song_artists = match sqlx::query!(
        r#"
        SELECT 
            song_artist.song as song_id,
            artist.id,
            artist.name,
            artist.picture,
            COUNT(album.id) AS num_albums
        FROM 
            song_artist
        LEFT JOIN 
            artist ON song_artist.artist = artist.id
        LEFT JOIN 
            album ON artist.id = album.artist
        WHERE 
            song_artist.song = ANY($1)
        GROUP BY 
            song_artist.song, artist.id
        "#, &song_ids
    )
    .fetch_all(&pool)
    .await{
        Ok(e) => e,
        Err(e) => return Err(internal_error(e)),
    };

    dbg!(song_artists.len());

    // Map the artists to each song
    let mut artist_map: std::collections::HashMap<i32, Vec<ArtistPartial>> = std::collections::HashMap::new();
    for song_artist in song_artists {
        let artist_partial = ArtistPartial {
            id: song_artist.id,
            name: song_artist.name,
            picture: song_artist.picture,
            num_albums: Some(song_artist.num_albums.unwrap_or(0)),
        };
        artist_map.entry(song_artist.song_id).or_default().push(artist_partial);
    }

    let tracks_with_artists: Vec<Track> = tracks.into_iter().map(|track| {
        let artists = artist_map.get(&track.id).cloned().unwrap_or_default();
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
    }).collect();

    Ok(Json(Album{
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
        tracks: Some(tracks_with_artists)
    }))
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
                art: i.arts.clone().unwrap_or("".to_string()).split(',').map(|i| std::env::var("MAKI_ART_URL").expect("MAKI_ART_URL not set") + i).collect(),
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
