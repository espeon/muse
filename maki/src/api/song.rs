use std::collections::{HashMap, HashSet};

use axum::{
    extract::{Extension, Host, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use tracing::debug;

// ── Track list ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TrackListItem {
    pub id: i32,
    pub slug: String,
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

#[derive(Debug, Serialize, utoipa::ToSchema)]
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
    analysis::{decode_vector, FEATURE_VERSION, MIX_PROFILE_VERSION},
    api::{build_default_art_url, resolve_song_id, ArtistPartial, Track, TrackRaw},
    clients,
};

use super::middleware::jwt::{AuthUser, OptionalAuthUser};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn internal_error<E: std::error::Error>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct SimilarTracksParams {
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SimilarTrack {
    pub id: i32,
    pub name: String,
    pub album_id: i32,
    pub album_name: String,
    pub artist_name: String,
    pub distance: f32,
}

#[utoipa::path(
    get,
    path = "/api/v1/track/{id}/similar",
    tag = "tracks",
    params(
        ("id" = String, Path, description = "Seed track ID or slug"),
        ("limit" = Option<usize>, Query, description = "Number of results (default 10, max 50)"),
    ),
    responses(
        (status = 200, description = "Nearest locally analyzed tracks", body = [SimilarTrack]),
        (status = 404, description = "Track not found"),
    )
)]
/// GET /track/:id/similar — brute-force Euclidean similarity over z-scored vectors.
pub async fn get_similar_songs(
    Path(id): Path<String>,
    Query(params): Query<SimilarTracksParams>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<SimilarTrack>>, (StatusCode, String)> {
    let seed_id = resolve_song_id(&id, &pool).await?;
    let limit = params.limit.unwrap_or(10).clamp(1, 50);

    let Some(stats) = sqlx::query(
        "SELECT means, std_devs FROM audio_similarity_feature_stats WHERE version = $1",
    )
    .bind(FEATURE_VERSION)
    .fetch_optional(&pool)
    .await
    .map_err(internal_error)?
    else {
        return Ok(Json(vec![]));
    };
    let means =
        decode_vector(stats.try_get("means").map_err(internal_error)?).ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "invalid feature means".to_string(),
            )
        })?;
    let std_devs =
        decode_vector(stats.try_get("std_devs").map_err(internal_error)?).ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "invalid feature standard deviations".to_string(),
            )
        })?;

    let Some(seed_row) = sqlx::query(
        r#"
        SELECT feature.vector
        FROM song
        JOIN audio_similarity_features feature ON feature.audio_hash = song.audio_hash
        WHERE song.id = $1 AND feature.version = $2
        "#,
    )
    .bind(seed_id)
    .bind(FEATURE_VERSION)
    .fetch_optional(&pool)
    .await
    .map_err(internal_error)?
    else {
        return Ok(Json(vec![]));
    };
    let seed =
        decode_vector(seed_row.try_get("vector").map_err(internal_error)?).ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "invalid seed feature vector".to_string(),
            )
        })?;

    let rows = sqlx::query(
        r#"
        SELECT song.id, song.name, song.album AS album_id, album.name AS album_name,
               artist.name AS artist_name, features.vector
        FROM song
        JOIN audio_similarity_features features ON features.audio_hash = song.audio_hash
        JOIN album ON album.id = song.album
        JOIN artist ON artist.id = song.album_artist
        WHERE features.version = $1 AND song.id <> $2
        "#,
    )
    .bind(FEATURE_VERSION)
    .bind(seed_id)
    .fetch_all(&pool)
    .await
    .map_err(internal_error)?;

    let mut similar = rows
        .into_iter()
        .filter_map(|row| {
            let vector = decode_vector(row.try_get("vector").ok()?)?;
            let distance = z_scored_distance(&seed, &vector, &means, &std_devs)?;
            Some(SimilarTrack {
                id: row.try_get("id").ok()?,
                name: row.try_get("name").ok()?,
                album_id: row.try_get("album_id").ok()?,
                album_name: row.try_get("album_name").ok()?,
                artist_name: row.try_get("artist_name").ok()?,
                distance,
            })
        })
        .collect::<Vec<_>>();
    similar.sort_by(|left, right| left.distance.total_cmp(&right.distance));
    similar.truncate(limit);
    Ok(Json(similar))
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MixProfileResponse {
    pub content_hash: String,
    pub version: i32,
    pub provider: String,
    pub provider_version: String,
    pub bpm: Option<f32>,
    pub beat_count: Option<i32>,
    pub key_root: Option<String>,
    pub key_mode: Option<String>,
    pub key_confidence: Option<f32>,
    pub loudness_integrated_lufs: Option<f32>,
    pub loudness_range_lu: Option<f32>,
    pub true_peak_db: Option<f32>,
    pub duration_seconds: Option<f32>,
    pub leading_silence_ms: Option<i32>,
    pub trailing_silence_ms: Option<i32>,
    pub head_rms_dbfs: Option<f32>,
    pub tail_rms_dbfs: Option<f32>,
    pub payload: Value,
}

#[utoipa::path(
    get,
    path = "/api/v1/track/{id}/mix-profile",
    tag = "tracks",
    params(("id" = String, Path, description = "Track ID or slug")),
    responses(
        (status = 200, description = "Stored mix-analysis profile", body = MixProfileResponse),
        (status = 204, description = "Track exists but has not been analyzed"),
        (status = 404, description = "Track not found"),
    )
)]
pub async fn get_mix_profile(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    let song_id = resolve_song_id(&id, &pool).await?;
    let Some(row) = sqlx::query(
        r#"
        SELECT song.audio_hash, profile.version, profile.provider, profile.provider_version,
               profile.bpm, profile.beat_count, profile.key_root, profile.key_mode,
               profile.key_confidence, profile.loudness_integrated_lufs,
               profile.loudness_range_lu, profile.true_peak_db, profile.duration_seconds,
               profile.leading_silence_ms, profile.trailing_silence_ms, profile.head_rms_dbfs,
               profile.tail_rms_dbfs, profile.payload
        FROM song
        JOIN audio_mix_profiles profile ON profile.audio_hash = song.audio_hash
        WHERE song.id = $1 AND profile.version = $2
        "#,
    )
    .bind(song_id)
    .bind(MIX_PROFILE_VERSION)
    .fetch_optional(&pool)
    .await
    .map_err(internal_error)?
    else {
        return Ok(StatusCode::NO_CONTENT.into_response());
    };

    let content_hash = row
        .try_get::<Vec<u8>, _>("audio_hash")
        .map_err(internal_error)?;
    Ok(Json(MixProfileResponse {
        content_hash: content_hash
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
        version: row.try_get("version").map_err(internal_error)?,
        provider: row.try_get("provider").map_err(internal_error)?,
        provider_version: row.try_get("provider_version").map_err(internal_error)?,
        bpm: row.try_get("bpm").map_err(internal_error)?,
        beat_count: row.try_get("beat_count").map_err(internal_error)?,
        key_root: row.try_get("key_root").map_err(internal_error)?,
        key_mode: row.try_get("key_mode").map_err(internal_error)?,
        key_confidence: row.try_get("key_confidence").map_err(internal_error)?,
        loudness_integrated_lufs: row
            .try_get("loudness_integrated_lufs")
            .map_err(internal_error)?,
        loudness_range_lu: row.try_get("loudness_range_lu").map_err(internal_error)?,
        true_peak_db: row.try_get("true_peak_db").map_err(internal_error)?,
        duration_seconds: row.try_get("duration_seconds").map_err(internal_error)?,
        leading_silence_ms: row.try_get("leading_silence_ms").map_err(internal_error)?,
        trailing_silence_ms: row.try_get("trailing_silence_ms").map_err(internal_error)?,
        head_rms_dbfs: row.try_get("head_rms_dbfs").map_err(internal_error)?,
        tail_rms_dbfs: row.try_get("tail_rms_dbfs").map_err(internal_error)?,
        payload: row
            .try_get::<sqlx::types::Json<Value>, _>("payload")
            .map_err(internal_error)?
            .0,
    })
    .into_response())
}

fn z_scored_distance(
    seed: &[f32],
    candidate: &[f32],
    means: &[f32],
    std_devs: &[f32],
) -> Option<f32> {
    if seed.len() != candidate.len() || seed.len() != means.len() || seed.len() != std_devs.len() {
        return None;
    }
    Some(
        seed.iter()
            .zip(candidate)
            .zip(means.iter().zip(std_devs))
            .map(|((seed, candidate), (mean, std_dev))| {
                let std_dev = std_dev.max(f32::EPSILON);
                ((seed - mean) / std_dev - (candidate - mean) / std_dev).powi(2)
            })
            .sum::<f32>()
            .sqrt(),
    )
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

#[utoipa::path(
    get,
    path = "/api/v1/track/{id}",
    tag = "tracks",
    params(("id" = String, Path, description = "Track ID or slug")),
    responses(
        (status = 200, description = "Track data", body = [Track]),
        (status = 404, description = "Track not found"),
    ),
    security(("bearer_token" = []))
)]
pub async fn get_song(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
    Host(host): Host,
    OptionalAuthUser { payload }: OptionalAuthUser,
) -> Result<axum::Json<Vec<Track>>, (StatusCode, String)> {
    let id_parsed = resolve_song_id(&id, &pool).await?;

    let user_id = payload.and_then(|p| p.sub.parse::<i32>().ok());
    let art_base = build_default_art_url(host);

    let tracks = match sqlx::query_as!(TrackRaw,
        r#"
        SELECT song.id, song.slug, disc, number, song.name, album, song.album_artist, liked, duration, plays, lossless,
               sample_rate, bits_per_sample, num_channels, composer, song.isrc, bpm,
               song.created_at, song.updated_at, last_play, year,
               album.name as album_name,
               artist.name as artist_name,
               (SELECT album_art.path FROM album_art WHERE album_art.album = song.album LIMIT 1) AS art_path
        FROM song
        LEFT JOIN album ON song.album = album.id
        LEFT JOIN artist ON song.album_artist = artist.id
        WHERE song.id = $1
        GROUP BY song.id, disc, number, song.name, album, song.album_artist, liked, duration, plays, lossless,
                 sample_rate, bits_per_sample, num_channels, composer, song.isrc, bpm,
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
        SELECT artist.id, artist.slug, artist.name, artist.picture, COUNT(album.id) AS num_albums
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
            song_to_artists
                .entry(rel.song)
                .or_default()
                .push(artist.clone());
        }
    }

    let final_tracks = tracks
        .into_iter()
        .map(|track| {
            let track_id = track.id;
            let artists = song_to_artists.get(&track_id).cloned().unwrap_or_default();
            Track {
                id: track_id,
                slug: track.slug,
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
                composer: track.composer,
                isrc: track.isrc,
                bpm: track.bpm,
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

#[derive(Serialize, utoipa::ToSchema)]
pub struct LikedResponse {
    pub liked: bool,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PlayHistoryEntry {
    #[schema(value_type = String)]
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

#[utoipa::path(
    post,
    path = "/api/v1/track/{id}/like",
    tag = "tracks",
    params(("id" = String, Path, description = "Track ID or slug")),
    responses(
        (status = 200, description = "New liked state", body = LikedResponse),
        (status = 404, description = "Track not found"),
    ),
    security(("bearer_token" = []))
)]
/// Toggle like status for a song. Uses the per-user favorites table.
/// Returns the new liked state.
pub async fn like_song(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
    AuthUser { payload }: AuthUser,
) -> Result<Json<LikedResponse>, (StatusCode, String)> {
    let id_parsed = resolve_song_id(&id, &pool).await?;
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

#[utoipa::path(
    post,
    path = "/api/v1/track/{id}/play",
    tag = "tracks",
    params(("id" = String, Path, description = "Track ID or slug")),
    responses(
        (status = 200, description = "Now playing updated"),
        (status = 404, description = "Track not found"),
    ),
    security(("bearer_token" = []))
)]
pub async fn set_playing(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
    AuthUser { payload }: AuthUser,
) -> Result<axum::Json<()>, (StatusCode, String)> {
    let id_parsed = resolve_song_id(&id, &pool).await?;

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
        payload
            .sub
            .parse::<i32>()
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?,
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
            debug!(
                "failed to set now playing on last.fm for song {}: {}",
                id, e
            );
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/track/{id}/scrobble",
    tag = "tracks",
    params(("id" = String, Path, description = "Track ID or slug")),
    responses(
        (status = 200, description = "Play recorded and scrobbled"),
        (status = 404, description = "Track not found"),
    ),
    security(("bearer_token" = []))
)]
/// Record a completed play. Increments the global counter, writes a per-user
/// timestamped row to `plays`, and scrobbles to Last.fm.
pub async fn scrobble_song(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
    AuthUser { payload }: AuthUser,
) -> Result<axum::Json<()>, (StatusCode, String)> {
    let id_parsed = resolve_song_id(&id, &pool).await?;
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

#[utoipa::path(
    get,
    path = "/api/v1/history",
    tag = "tracks",
    params(
        ("limit" = Option<i64>, Query, description = "Max results (default 20)"),
        ("offset" = Option<i64>, Query, description = "Offset (default 0)"),
    ),
    responses(
        (status = 200, description = "Play history", body = [PlayHistoryEntry]),
    ),
    security(("bearer_token" = []))
)]
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

#[utoipa::path(
    get,
    path = "/api/v1/tracks",
    tag = "tracks",
    params(
        ("limit" = Option<i64>, Query, description = "Max results (default 50)"),
        ("cursor" = Option<i32>, Query, description = "Pagination cursor (song ID, default 0)"),
        ("lossless" = Option<bool>, Query, description = "Filter to lossless-only"),
    ),
    responses(
        (status = 200, description = "Paginated track list", body = TracksResponse),
    ),
    security(("bearer_token" = []))
)]
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
        SELECT song.id, song.slug, song.name, song.duration, song.number, song.disc,
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
                slug: r.slug,
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
