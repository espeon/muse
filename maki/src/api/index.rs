use axum::{
    extract::{Extension, Host, Path, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool, Postgres};

use crate::api::build_default_art_url;

#[derive(Serialize)]
pub struct IndexSong {
    id: i32,
    artist_name: Option<String>,
    song_name: String,
    album_name: Option<String>,
}

#[derive(Serialize, FromRow)]
pub struct SearchSong {
    id: i32,
    artist_name: String,
    song_name: String,
    album_name: String,
    picture: Option<String>,
}

pub async fn index_songs(
    Extension(pool): Extension<PgPool>,
) -> Result<axum::Json<Vec<IndexSong>>, (StatusCode, String)> {
    match sqlx::query_as!(
        IndexSong,
        r#"
    SELECT
    song.id,
    song.name as song_name,
    artist.name as artist_name,
    album.name as album_name
    FROM
    song
    LEFT JOIN album ON song.album = album.id
    LEFT JOIN artist ON song.album_artist = artist.id
"#,
    )
    .fetch_all(&pool)
    .await
    {
        Ok(e) => Ok(Json(e)),
        Err(e) => Err(internal_error(e)),
    }
}

// query params for search songs
#[derive(Deserialize)]
pub struct SearchQueryParams {
    #[serde(default)]
    sortby: Option<SortByOptions>,
    #[serde(default)]
    dir: Option<String>,
}

#[derive(Deserialize)]
enum SortByOptions {
    #[serde(alias = "id")]
    Id,
    #[serde(alias = "song")]
    SongName,
    #[serde(alias = "artist")]
    ArtistName,
    #[serde(alias = "album")]
    AlbumName,
}

impl SortByOptions {
    fn as_str(&self) -> &str {
        match self {
            Self::Id => "song.id",
            Self::SongName => "song.name",
            Self::ArtistName => "artist.name",
            Self::AlbumName => "album.name",
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

// impl DirOptions {
//     fn from_str(s: &str) -> Option<DirOptions> {
//         match s {
//             "asc" => Some(DirOptions::Asc),
//             "desc" => Some(DirOptions::Desc),
//             _ => Some(DirOptions::Asc),
//         }
//     }

//     fn as_str(&self) -> &str {
//         match self {
//             Self::Asc => "asc",
//             Self::Desc => "desc",
//         }
//     }
// }

/// search songs
/// TODO: move to tantivy or meilisearch
///
pub async fn search_songs(
    Path(slug): Path<String>,
    Extension(pool): Extension<PgPool>,
    Query(params): Query<SearchQueryParams>,
    Host(host): Host,
) -> Result<axum::Json<Vec<SearchSong>>, (StatusCode, String)> {
    let sort_by = params
        .sortby
        .as_ref()
        .map(|e| e.as_str())
        .unwrap_or("song.id");
    let dir = params.dir.as_deref().unwrap_or("asc");

    let art_url = build_default_art_url(host);

    match sqlx::query_as::<Postgres, SearchSong>(
        // SearchSong,
        &format!(
            r#"
            SELECT
            song.id,
            song.name as song_name,
            artist.name as artist_name,
            album.name as album_name,
            artist.id as artist_id,
            album.id as album_id,
            STRING_AGG(CAST(album_art.path AS VARCHAR), ',') as picture
            FROM song
            LEFT JOIN album ON song.album = album.id
            LEFT JOIN artist ON song.album_artist = artist.id
            LEFT JOIN song_artist ON song.id = song_artist.song
            LEFT JOIN album_art ON album.id = album_art.album

            WHERE unaccent(song.name) ILIKE ('%' || unaccent('{0}') || '%')
            OR unaccent(artist.name) ILIKE ('%' || unaccent('{0}') || '%')
            OR unaccent(album.name) ILIKE ('%' || unaccent('{0}') || '%')

            GROUP BY song.id, song.name, artist.name, album.name

            ORDER BY {1} {2}
        "#,
            slug, sort_by, dir
        ),
    )
    .fetch_all(&pool)
    .await
    {
        Ok(e) => {
            let e = e
                .into_iter()
                .map(|mut i| {
                    if i.picture.is_some() {
                        i.picture = Some(art_url.clone() + i.picture.as_ref().unwrap());
                        i
                    } else {
                        i
                    }
                })
                .collect();

            Ok(Json(e))
        }
        Err(e) => Err(internal_error(e)),
    }
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
