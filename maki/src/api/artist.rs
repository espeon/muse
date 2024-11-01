use super::{build_default_art_url, AlbumPartial, AlbumPartialRaw, Artist, ArtistPartial};
use crate::api::ArtistRaw;
use axum::{
    extract::{Host, Path, Query},
    http::StatusCode,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, QueryBuilder};

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
    #[serde(default)]
    filter: Option<String>,

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
            Self::Id => "artist.id",
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
        filter,
    }): Query<GetArtistParams>,
) -> Result<axum::Json<AllArtistsPartial>, (StatusCode, String)> {
    let cursor_val: i32 = cursor.unwrap_or(0); // Default cursor to 0 if None

    let current_artist = sqlx::query_as!(
        ArtistPartial,
        "SELECT artist.id, artist.name, artist.picture, COUNT(album) as num_albums
        FROM artist
        LEFT JOIN album ON album.artist = artist.id
        WHERE artist.id = $1
        GROUP BY artist.id, artist.name",
        cursor_val
    )
    .fetch_optional(&pool)
    .await
    .map_err(internal_error)?;

    debug!("Artist: {:?}", current_artist);
    let order_dir = dir.unwrap_or(DirOptions::Asc);
    let sort_column_typed = sortby.unwrap_or(SortByArtistOptions::ArtistName);
    let order_dir = order_dir.as_str(); // Default to ascending order
    let sort_column = sort_column_typed.as_str(); // Default sorting by artist name

    // Begin constructing query
    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT artist.id, artist.name, artist.picture, COUNT(album) as num_albums
        FROM artist
        LEFT JOIN album ON album.artist = artist.id
        WHERE (SELECT(COUNT(album) > 0) FROM album WHERE artist.id = album.artist)", // Only show artists with albums
    );

    // Add dynamic WHERE clause if current_artist exists
    if let Some(e) = current_artist {
        debug!("Artist: {:?}", &e);
        if order_dir == "asc" {
            query_builder
                .push(" AND ")
                // we can't bind a column here b/c autoescape :(
                .push(sort_column)
                .push(" > ")
                .push_bind(e.name);
        } else {
            query_builder
                .push(" AND ")
                .push_bind(e.name)
                .push(" > ")
                .push(sort_column);
        };
    }

    // where lower(name) ilike lower($1)
    if let Some(filter) = filter {
        query_builder
            .push(" AND lower(artist.name) ilike lower(")
            .push_bind(format!("%{}%", filter))
            .push(")");
    }

    // Add GROUP BY clause
    query_builder.push(" GROUP BY artist.id, artist.name");

    if order_dir == "asc" {
        query_builder
            .push(" ORDER BY ")
            // again, we can't bind columns b/c autoescape
            .push(sort_column)
            .push(" ASC");
    } else {
        query_builder
            .push(" ORDER BY ")
            .push(sort_column)
            .push(" DESC");
    };

    // Add LIMIT clause
    query_builder.push(" LIMIT ").push_bind(limit.unwrap_or(20));

    // Build query and fetch results
    let query = query_builder.build_query_as::<ArtistPartial>();
    let latest_artists = query.fetch_all(&pool).await.map_err(internal_error)?;

    let cursor = latest_artists.last().map_or(0, |artist| artist.id);
    Ok(Json(AllArtistsPartial {
        artists: latest_artists,
        limit: limit.unwrap_or(20),
        cursor,
    }))
}

fn internal_error<E: std::fmt::Debug>(err: E) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Internal error: {:?}", err),
    )
}
