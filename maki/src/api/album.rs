use axum::{
    debug_handler,
    extract::{Extension, Host, Path, Query},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use sqlx::{PgPool, Postgres, QueryBuilder};
use tracing::debug;

use crate::api::{
    build_default_art_url, Album, AlbumPartialRaw, AlbumRaw, ArtistPartial, Track, TrackRaw,
};

use super::{AlbumPartial, AllAlbumsPartial};

#[debug_handler]
pub async fn get_album(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
    Host(host): Host,
) -> Result<axum::Json<Album>, (StatusCode, String)> {
    let id_parsed = id.split('.').collect::<Vec<&str>>()[0]
        .parse::<i32>()
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    let art_url = build_default_art_url(host);

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
                    arts: Some(e.arts.unwrap_or("".to_string()).split(',').map(|i| art_url.clone() + i).collect()),
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
        "#,
        &song_ids
    )
    .fetch_all(&pool)
    .await
    {
        Ok(e) => e,
        Err(e) => return Err(internal_error(e)),
    };

    // Map the artists to each song
    let mut artist_map: std::collections::HashMap<i32, Vec<ArtistPartial>> =
        std::collections::HashMap::new();
    for song_artist in song_artists {
        let artist_partial = ArtistPartial {
            id: song_artist.id,
            name: song_artist.name,
            picture: song_artist.picture,
            num_albums: Some(song_artist.num_albums.unwrap_or(0)),
        };
        artist_map
            .entry(song_artist.song_id)
            .or_default()
            .push(artist_partial);
    }

    let tracks_with_artists: Vec<Track> = tracks
        .into_iter()
        .map(|track| {
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
        })
        .collect();

    Ok(Json(Album {
        id: album.id,
        name: album.name,
        art: match album.arts {
            Some(e) => e.split(',').map(|i| i.to_string()).collect(),
            None => vec![],
        },
        year: album.year,
        created_at: album.created_at,
        updated_at: album.updated_at,
        artist: ArtistPartial {
            id: album.artist_id,
            name: album.artist_name,
            picture: album.artist_picture,
            num_albums: None,
        },
        tracks: Some(tracks_with_artists),
    }))
}

#[derive(Deserialize, Default)]
pub struct GetAlbumParams {
    #[serde(default)]
    sortby: Option<SortByAlbumOptions>,
    #[serde(default)]
    dir: Option<DirOptions>,
    #[serde(default)]
    limit: Option<i32>,
    #[serde(default)]
    cursor: Option<i32>, // Single cursor based on album.id
    #[serde(default)]
    filter: Option<String>,
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

#[derive(Deserialize, PartialEq)]
enum DirOptions {
    #[serde(alias = "asc", alias = "ASC")]
    Asc,
    #[serde(alias = "desc", alias = "DESC")]
    Desc,
}

impl DirOptions {
    fn as_str(&self) -> &str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}


#[derive(Deserialize, PartialEq)]
enum DirOptions {
    #[serde(alias = "asc")]
    Asc,
    #[serde(alias = "desc")]
    Desc,
}

impl DirOptions {
    fn as_str(&self) -> &str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

enum PrimaryValue {
    Int(i32),
    String(String),
}


#[axum_macros::debug_handler]
pub async fn get_albums(
    Extension(pool): Extension<PgPool>,
    Query(GetAlbumParams {
        sortby,
        dir,
        limit,
        cursor, // Single cursor based on album.id
        filter,
    }): Query<GetAlbumParams>,
    Host(host): Host,
) -> Result<axum::Json<AllAlbumsPartial>, (StatusCode, String)> {
    let art_url = build_default_art_url(host);
    let limit_value = limit.unwrap_or(20); // Default limit

    // If no cursor is provided, use a fallback
    let cursor_value = cursor.unwrap_or(0); // Default to 0 if cursor is None

    // Step 1: Fetch the album details based on the cursor (album.id)
    let current_album = sqlx::query_as!(AlbumPartialRaw, r#"
        SELECT album.id, album.name, album.year, count(song.id), artist.id as artist_id, artist.name as artist_name, artist.picture as artist_picture,
        STRING_AGG(CAST(album_art.path AS VARCHAR), ',') as arts
        FROM album
        LEFT JOIN song ON song.album = album.id
        LEFT JOIN artist ON album.artist = artist.id
        LEFT JOIN album_art ON album.id = album_art.album
        WHERE album.id = $1
        GROUP BY album.id, album.name, artist.name, artist.id
        "#, cursor_value
    )
    .fetch_optional(&pool)
    .await.map_err(internal_error)?;

    debug!("Current album: {:?}", current_album);

    // Step 2: Determine the sorting values and where clause based on the fetched album
    let order_dir_typed = dir.unwrap_or(DirOptions::Asc);
    let order_direction = order_dir_typed.as_str(); // Default to ascending

    // Build the query
    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT album.id, album.name, album.year, count(song.id), artist.id as artist_id, artist.name as artist_name, artist.picture as artist_picture,
        STRING_AGG(CAST(album_art.path AS VARCHAR), ',') as arts
        FROM album
        LEFT JOIN song ON song.album = album.id
        LEFT JOIN artist ON album.artist = artist.id
        LEFT JOIN album_art ON album.id = album_art.album
        WHERE 1 = 1", // Always true, allows for easier dynamic clauses
    );

    let primary_value_column = match sortby {
        Some(SortByAlbumOptions::ArtistName) => "artist.name",
        Some(SortByAlbumOptions::AlbumName) => "album.name",
        Some(SortByAlbumOptions::Year) => "album.year",
        _ => "album.id", // Default to album.id
    };

    // Step 2: Determine the primary value and construct where clause
    if let Some(album) = current_album {
        if order_direction == "asc" {
            query_builder
                .push(" AND ")
                .push(primary_value_column)
                .push(" > ")
                .push_bind(album.artist_name.clone());
        } else {
            query_builder
                .push(" AND ")
                .push_bind(album.artist_name.clone())
                .push(" > ")
                .push(primary_value_column);
        }
    };
    if let Some(filter) = filter {
        query_builder
            .push(" AND (lower(album.name) ilike lower(")
            .push_bind(format!("%{}%", filter))
            .push(") OR lower(artist.name) ilike lower(")
            .push_bind(format!("%{}%", filter))
            .push("))");
    }
    query_builder.push(" GROUP BY album.id, album.name, artist.name, artist.id");

    if order_direction == "asc" {
        query_builder
            .push(" ORDER BY ")
            .push(primary_value_column)
            .push(" ASC, album.id ASC");
    } else {
        query_builder
            .push(" ORDER BY ")
            .push(primary_value_column)
            .push(" DESC, album.id DESC");
    };

    query_builder.push(" LIMIT ").push_bind(limit.unwrap_or(20));

    let query = query_builder.build_query_as::<AlbumPartialRaw>();
    let sqlres = query.fetch_all(&pool).await.map_err(internal_error)?;

    let latest_albums: Vec<AlbumPartial> = sqlres
        .iter()
        .map(|i| AlbumPartial {
            id: i.id,
            name: i.name.clone(),
            art: i
                .arts
                .clone()
                .unwrap_or("".to_string())
                .split(',')
                .map(|i| art_url.clone() + i)
                .collect(),
            year: i.year,
            count: i.count,
            artist: Some(ArtistPartial {
                id: i.artist_id,
                name: i.artist_name.clone(),
                picture: i.artist_picture.clone(),
                num_albums: None,
            }),
        })
        .collect();

    // Generate the next cursor value (the id of the last album in the fetched list)
    let new_cursor = latest_albums.last().map_or(0, |album| album.id);

    Ok(Json(AllAlbumsPartial {
        albums: latest_albums,
        limit: limit_value,
        offset: new_cursor, // The next cursor for the next page
    }))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Internal error: {:?}", err),
    )
}
