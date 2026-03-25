use std::collections::{HashMap, HashSet};

use axum::{
    extract::{Extension, Host, Path, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::debug;

// ── Track list ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackListItem {
    pub id: i32,
    pub name: String,
    pub duration: i32,
    pub number: Option<i32>,
    pub disc: Option<i32>,
    pub lossless: Option<bool>,
    pub sample_rate: Option<i32>,
    pub bits_per_sample: Option<i32>,
    pub num_channels: Option<i32>,
    pub album_id: i32,
    pub album_name: String,
    pub artist_id: i32,
    pub artist_name: String,
    pub art_url: Option<String>,
    pub liked: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct TracksResponse {
    pub tracks: Vec<TrackListItem>,
    pub total: i64,
    pub limit: i64,
    pub cursor: i32,
}

#[derive(Debug, Deserialize)]
pub struct TracksParams {
    pub limit: Option<i64>,
    pub cursor: Option<i32>,
    pub lossless: Option<bool>,
}

use crate::{
    api::{build_default_art_url, ArtistPartial, Track, TrackRaw},
    clients,
};

use super::middleware::jwt::{AuthUser, OptionalAuthUser};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn internal_error<E: std::error::Error>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

/// Fetch the set of song IDs that a user has liked, from the favorites table.
/// Returns an empty set for anonymous users (user_id = None).
pub async fn liked_ids_for_user(pool: &PgPool, user_id: Option<i32>) -> HashSet<i32> {
    let Some(uid) = user_id else {
        return HashSet::new();
    };
    sqlx::query_scalar!(
        "SELECT favoritable_id FROM favorites WHERE user_id = $1 AND favoritable_type = 'song'",
        uid
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .collect()
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn get_song(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
    Host(host): Host,
    OptionalAuthUser { payload }: OptionalAuthUser,
) -> Result<axum::Json<Vec<Track>>, (StatusCode, String)> {
    let id_parsed = id.split('.').collect::<Vec<&str>>()[0]
        .parse::<i32>()
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    let user_id = payload.and_then(|p| p.sub.parse::<i32>().ok());
    let art_base = build_default_art_url(host);

    let tracks = match sqlx::query_as!(TrackRaw,
        r#"
        SELECT song.id, disc, number, song.name, album, song.album_artist, liked, duration, plays, lossless,
               sample_rate, bits_per_sample, num_channels,
               song.created_at, song.updated_at, last_play, year,
               album.name as album_name,
               artist.name as artist_name,
               (SELECT album_art.path FROM album_art WHERE album_art.album = song.album LIMIT 1) AS art_path
        FROM song
        LEFT JOIN album ON song.album = album.id
        LEFT JOIN artist ON song.album_artist = artist.id
        WHERE song.id = $1
        GROUP BY song.id, disc, number, song.name, album, song.album_artist, liked, duration, plays, lossless,
                 sample_rate, bits_per_sample, num_channels,
                 song.created_at, song.updated_at, last_play, year,
                 album.name, artist.name
        "#, id_parsed
    )
    .fetch_all(&pool)
    .await {
        Ok(e) => e,
        Err(e) => return Err(internal_error(e)),
    };

    let song_ids: Vec<i32> = tracks.iter().map(|t| t.id).collect();
    let liked_ids = liked_ids_for_user(&pool, user_id).await;

    let song_artists = match sqlx::query_as!(
        ArtistPartial,
        r#"
        SELECT artist.id, artist.name, artist.picture, COUNT(album.id) AS num_albums
        FROM artist
        LEFT JOIN album ON artist.id = album.artist
        WHERE artist.id IN (SELECT artist FROM song_artist WHERE song = ANY($1))
        GROUP BY artist.id
        "#,
        &song_ids
    )
    .fetch_all(&pool)
    .await
    {
        Ok(e) => e,
        Err(e) => return Err(internal_error(e)),
    };

    let artist_map: HashMap<i32, ArtistPartial> =
        song_artists.into_iter().map(|a| (a.id, a)).collect();

    let song_artist_rels = match sqlx::query!(
        "SELECT song_artist.song, song_artist.artist FROM song_artist WHERE song_artist.song = ANY($1)",
        &song_ids
    )
    .fetch_all(&pool)
    .await
    {
        Ok(e) => e,
        Err(e) => return Err(internal_error(e)),
    };

    let mut song_to_artists: HashMap<i32, Vec<ArtistPartial>> = HashMap::new();
    for rel in song_artist_rels {
        if let Some(artist) = artist_map.get(&rel.artist) {
            song_to_artists.entry(rel.song).or_default().push(artist.clone());
        }
    }

    let final_tracks = tracks
        .into_iter()
        .map(|track| {
            let track_id = track.id;
            let artists = song_to_artists.get(&track_id).cloned().unwrap_or_default();
            Track {
                id: track_id,
                disc: track.disc,
                number: track.number,
                name: track.name,
                album: track.album,
                album_artist: track.album_artist,
                liked: user_id.map(|_| liked_ids.contains(&track_id)),
                duration: track.duration,
                plays: track.plays,
                lossless: track.lossless,
                sample_rate: track.sample_rate,
                bits_per_sample: track.bits_per_sample,
                num_channels: track.num_channels,
                created_at: track.created_at,
                updated_at: track.updated_at,
                last_play: track.last_play,
                year: track.year,
                album_name: track.album_name,
                artist_name: track.artist_name,
                art_url: track.art_path.map(|p| format!("{}{}", art_base, p)),
                artists,
            }
        })
        .collect();

    Ok(Json(final_tracks))
}

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct LikedResponse {
    pub liked: bool,
}

#[derive(Debug, Serialize)]
pub struct PlayHistoryEntry {
    pub played_at: OffsetDateTime,
    pub song_id: i32,
    pub name: String,
    pub duration: i32,
    pub album_id: i32,
    pub album_name: String,
    pub artist_id: i32,
    pub artist_name: String,
    pub liked: bool,
}

#[derive(Debug, Deserialize)]
pub struct HistoryParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ── Like / unlike ─────────────────────────────────────────────────────────────

/// Toggle like status for a song. Uses the per-user favorites table.
/// Returns the new liked state.
pub async fn like_song(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
    AuthUser { payload }: AuthUser,
) -> Result<Json<LikedResponse>, (StatusCode, String)> {
    let id_parsed = id.split('.').collect::<Vec<&str>>()[0]
        .parse::<i32>()
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    let user_id = payload
        .sub
        .parse::<i32>()
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    // Attempt to delete — if nothing was deleted it wasn't liked yet, so insert
    let deleted = sqlx::query!(
        "DELETE FROM favorites WHERE user_id = $1 AND favoritable_id = $2 AND favoritable_type = 'song'",
        user_id,
        id_parsed
    )
    .execute(&pool)
    .await
    .map_err(internal_error)?;

    let liked = if deleted.rows_affected() == 0 {
        sqlx::query!(
            "INSERT INTO favorites (user_id, favoritable_id, favoritable_type) VALUES ($1, $2, 'song')",
            user_id,
            id_parsed
        )
        .execute(&pool)
        .await
        .map_err(internal_error)?;
        true
    } else {
        false
    };

    Ok(Json(LikedResponse { liked }))
}

// ── Now playing / scrobble ────────────────────────────────────────────────────

pub async fn set_playing(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
    AuthUser { payload }: AuthUser,
) -> Result<axum::Json<()>, (StatusCode, String)> {
    let id_parsed = id.split('.').collect::<Vec<&str>>()[0]
        .parse::<i32>()
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    let song = match sqlx::query!(
        r#"
        SELECT path, number, song.name, album.name as album_name, duration, artist.name as artist_name
        FROM song
        LEFT JOIN album ON song.album = album.id
        LEFT JOIN artist ON song.album_artist = artist.id
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
        Ok(_) => {
            debug!("set now playing on last.fm for song {}", id);
            Ok(Json(()))
        }
        Err(e) => {
            debug!("failed to set now playing on last.fm for song {}: {}", id, e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

/// Record a completed play. Increments the global counter, writes a per-user
/// timestamped row to `plays`, and scrobbles to Last.fm.
pub async fn scrobble_song(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
    AuthUser { payload }: AuthUser,
) -> Result<axum::Json<()>, (StatusCode, String)> {
    let id_parsed = id.split('.').collect::<Vec<&str>>()[0]
        .parse::<i32>()
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    let user_id = payload
        .sub
        .parse::<i32>()
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let song = match sqlx::query!(
        r#"
        SELECT path, number, song.name, album.name as album_name, duration, artist.name as artist_name
        FROM song
        LEFT JOIN album ON song.album = album.id
        LEFT JOIN artist ON song.album_artist = artist.id
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

    // Last.fm scrobble (silent failure — user may not have configured it)
    match clients::lastfm::scrobble(
        user_id,
        &pool,
        &song.name,
        &song.artist_name,
        &song.album_name,
        song.duration as u32,
    )
    .await
    {
        Ok(_) => debug!("scrobbled song {} to last.fm", id),
        Err(e) => debug!("failed to scrobble song {} to last.fm: {}", id, e),
    }

    // Increment global play counter on song row
    sqlx::query!("UPDATE song SET plays = plays + 1 WHERE id = $1", id_parsed)
        .execute(&pool)
        .await
        .map_err(internal_error)?;

    // Write per-user timestamped play — this is what feeds history + future recommendations
    sqlx::query!(
        "INSERT INTO plays (user_id, song_id) VALUES ($1, $2)",
        user_id,
        id_parsed
    )
    .execute(&pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(()))
}

// ── History ───────────────────────────────────────────────────────────────────

/// GET /history — recent plays for the authenticated user, newest first.
/// Query params: limit (default 20), offset (default 0)
pub async fn get_history(
    Extension(pool): Extension<PgPool>,
    AuthUser { payload }: AuthUser,
    Query(params): Query<HistoryParams>,
) -> Result<Json<Vec<PlayHistoryEntry>>, (StatusCode, String)> {
    let user_id = payload
        .sub
        .parse::<i32>()
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);

    let rows = sqlx::query!(
        r#"
        SELECT
            p.played_at,
            s.id      AS song_id,
            s.name,
            s.duration,
            album.id  AS album_id,
            album.name AS album_name,
            artist.id  AS "artist_id: i32",
            artist.name AS artist_name,
            EXISTS (
                SELECT 1 FROM favorites f
                WHERE f.user_id = $1
                  AND f.favoritable_id = s.id
                  AND f.favoritable_type = 'song'
            ) AS "liked!: bool"
        FROM plays p
        JOIN song s    ON p.song_id  = s.id
        JOIN album     ON s.album    = album.id
        JOIN artist    ON s.album_artist = artist.id
        WHERE p.user_id = $1
        ORDER BY p.played_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        limit,
        offset
    )
    .fetch_all(&pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(
        rows.into_iter()
            .map(|r| PlayHistoryEntry {
                played_at: r.played_at,
                song_id: r.song_id,
                name: r.name,
                duration: r.duration,
                album_id: r.album_id,
                album_name: r.album_name,
                artist_id: r.artist_id,
                artist_name: r.artist_name,
                liked: r.liked,
            })
            .collect(),
    ))
}

// ── Tracks paginated listing ───────────────────────────────────────────────────

/// GET /tracks — paginated listing of all tracks.
/// Query params: limit (default 50), cursor (song.id, default 0), lossless (optional bool filter)
pub async fn get_tracks(
    Extension(pool): Extension<PgPool>,
    Host(host): Host,
    OptionalAuthUser { payload }: OptionalAuthUser,
    Query(params): Query<TracksParams>,
) -> Result<Json<TracksResponse>, (StatusCode, String)> {
    let user_id = payload.and_then(|p| p.sub.parse::<i32>().ok());
    let limit = params.limit.unwrap_or(50);
    let cursor = params.cursor.unwrap_or(0);
    let art_base = build_default_art_url(host);

    let total = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM song WHERE ($1::bool IS NULL OR lossless = $1)",
        params.lossless
    )
    .fetch_one(&pool)
    .await
    .map_err(internal_error)?
    .unwrap_or(0);

    let rows = sqlx::query!(
        r#"
        SELECT song.id, song.name, song.duration, song.number, song.disc,
               song.lossless, song.sample_rate, song.bits_per_sample, song.num_channels,
               song.album as album_id, album.name as album_name,
               song.album_artist as artist_id, artist.name as artist_name,
               (SELECT album_art.path FROM album_art WHERE album_art.album = song.album LIMIT 1) AS art_path
        FROM song
        LEFT JOIN album ON song.album = album.id
        LEFT JOIN artist ON song.album_artist = artist.id
        WHERE song.id > $1
          AND ($2::bool IS NULL OR song.lossless = $2)
        ORDER BY song.id ASC
        LIMIT $3
        "#,
        cursor,
        params.lossless,
        limit
    )
    .fetch_all(&pool)
    .await
    .map_err(internal_error)?;

    let liked_ids = liked_ids_for_user(&pool, user_id).await;

    let tracks: Vec<TrackListItem> = rows
        .into_iter()
        .map(|r| {
            let id = r.id;
            TrackListItem {
                id,
                name: r.name,
                duration: r.duration,
                number: r.number,
                disc: r.disc,
                lossless: r.lossless,
                sample_rate: r.sample_rate,
                bits_per_sample: r.bits_per_sample,
                num_channels: r.num_channels,
                album_id: r.album_id,
                album_name: r.album_name,
                artist_id: r.artist_id,
                artist_name: r.artist_name,
                art_url: r.art_path.map(|p| format!("{}{}", art_base, p)),
                liked: user_id.map(|_| liked_ids.contains(&id)),
            }
        })
        .collect();

    let next_cursor = tracks.last().map(|t| t.id).unwrap_or(cursor);

    Ok(Json(TracksResponse {
        tracks,
        total,
        limit,
        cursor: next_cursor,
    }))
}
