use super::{build_default_art_url, AlbumPartial, AlbumPartialRaw, Artist, ArtistPartial};
use crate::api::ArtistRaw;
use axum::{
    extract::{Host, Path, Query},
    http::StatusCode,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres};
use tracing::debug;

pub async fn get_artist(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    Host(host): Host,
) -> Result<Json<Artist>, (StatusCode, String)> {
    match sqlx::query_as!(ArtistRaw, r#"
    SELECT artist.id, artist.name, artist.picture, artist.tags, artist.bio, artist.created_at, artist.updated_at

    FROM artist

    WHERE artist.id = $1

    GROUP BY artist.name, artist.id
    "#, id
    )
    .fetch_one(&pool)
    .await
    {
        Ok(e) => {
                    // fetch albums
                    let albums_raw: Vec<AlbumPartialRaw> = sqlx::query_as!(AlbumPartialRaw, r#"
                        SELECT album.id, album.name, album.year, count(song.id), artist.id as artist_id, artist.name as artist_name, artist.picture as artist_picture,
                        STRING_AGG(CAST(album_art.path AS VARCHAR), ',') as arts

                        FROM album
                        LEFT JOIN song ON song.album = album.id
                        LEFT JOIN artist ON album.artist = artist.id
                        LEFT JOIN album_art ON album.id = album_art.album

                        WHERE album.artist = $1
                        GROUP BY album.id, album.name, artist.id
                        order by album.created_at desc
                        "#, id
                    )
                    .fetch_all(&pool)
                    .await
                    .map_err(internal_error)?;

                    let art_url = build_default_art_url(host);

                    let albums = albums_raw.iter().map(|i| {
                        return AlbumPartial {
                            id: i.id,
                            name: i.name.clone(),
                            art: i.arts.clone().unwrap_or("".to_string()).split(',').map(|i| art_url.clone() + i).collect(),
                            year: i.year,
                            count: i.count,
                            artist: Some(ArtistPartial {
                                id: i.artist_id,
                                name: i.artist_name.clone(),
                                picture: i.artist_picture.clone(),
                                num_albums: None
                            })
                        }
                        }).collect();

                    Ok(Json(Artist {
                        id: e.id,
                        name: e.name,
                        picture: e.picture,
                        tags: e.tags,
                        bio: e.bio,
                        created_at: e.created_at,
                        updated_at: e.updated_at,
                        albums,
                    }))
        },
        Err(e) => Err(internal_error(e)),
    }
}

#[derive(Deserialize, Default)]
pub struct GetArtistParams {
    #[serde(default)]
    sortby: Option<SortByArtistOptions>,
    #[serde(default)]
    dir: Option<DirOptions>,
    #[serde(default)]
    limit: Option<i32>,
    #[serde(default)]
    cursor: Option<i32>,
}

#[derive(Deserialize)]
enum SortByArtistOptions {
    #[serde(alias = "id")]
    Id,
    #[serde(alias = "artist")]
    ArtistName,
}

impl SortByArtistOptions {
    fn as_str(&self) -> &str {
        match self {
            Self::Id => "album.id",
            Self::ArtistName => "artist.name",
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

#[derive(Serialize)]
pub struct AllArtistsPartial {
    artists: Vec<ArtistPartial>,
    limit: i32,
    cursor: i32,
}

#[axum_macros::debug_handler]
pub async fn get_artists(
    Extension(pool): Extension<PgPool>,
    Query(GetArtistParams {
        sortby,
        dir,
        limit,
        cursor,
    }): Query<GetArtistParams>,
) -> Result<axum::Json<AllArtistsPartial>, (StatusCode, String)> {
    let cursor_val: i32 = cursor.unwrap_or(0); // Default to 0 if cursor is None
    let current_artist = sqlx::query_as!(
        ArtistPartial,
        "SELECT artist.id, artist.name, artist.picture, COUNT(album) as num_albums
        FROM artist

        LEFT JOIN album ON album.artist = artist.id

        WHERE artist.id = $1

        GROUP BY artist.name, artist.id",
        cursor_val
    )
    .fetch_optional(&pool)
    .await
    .map_err(internal_error)?;

    debug!("Artist: {:?}", current_artist);

    let order_dir_typed = dir.unwrap_or(DirOptions::Asc);
    let order_direction = order_dir_typed.as_str(); // Default to ascending

    let comparison_operator = if order_direction == "asc" { ">" } else { "<" };

    let where_clause = match current_artist {
        Some(e) => match sortby {
            Some(SortByArtistOptions::ArtistName) => {
                format!("AND artist.name {comparison_operator} '{}'", e.name)
            }
            Some(SortByArtistOptions::Id) => {
                format!("AND artist.id {comparison_operator} {}", e.id)
            }
            None => "".to_string(),
        },
        None => "".to_string(),
    };

    let latest_artists = match sqlx::query_as::<Postgres, ArtistPartial>(&format!(
        "
            SELECT artist.id, artist.name, artist.picture, COUNT(album) as num_albums
            FROM artist

            LEFT JOIN album ON album.artist = artist.id

            WHERE (SELECT(COUNT(album) > 0) FROM album WHERE artist.id = album.artist) {}

            GROUP BY artist.name, artist.id
            order by {} {}
            limit {}
    ",
        where_clause,
        sortby.unwrap_or(SortByArtistOptions::ArtistName).as_str(),
        order_direction,
        limit.unwrap_or(20)
    ))
    .fetch_all(&pool)
    .await
    {
        Ok(e) => e,
        Err(e) => return Err(internal_error(e)),
    };

    let cursor = latest_artists.last().map_or(0, |album| album.id);
    Ok(Json(AllArtistsPartial {
        artists: latest_artists,
        limit: limit.unwrap_or(20),
        cursor,
    }))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
