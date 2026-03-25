use std::collections::HashMap;

use axum::{
    extract::{Extension, Host, Path},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;

use crate::api::build_default_art_url;
use crate::api::song::liked_ids_for_user;
use super::middleware::jwt::AuthUser;

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PlaylistSummary {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub art_path: Option<String>,
    pub track_count: i64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct PlaylistTrack {
    pub item_id: i32,
    pub song_id: i32,
    pub name: String,
    pub duration: i32,
    pub number: Option<i32>,
    pub disc: Option<i32>,
    pub liked: Option<bool>,
    pub lossless: Option<bool>,
    pub album_id: i32,
    pub album_name: String,
    pub artist_name: String,
    pub art_url: Option<String>,
    pub prev_item_id: Option<i32>,
    pub next_item_id: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct PlaylistDetail {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub art_path: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub tracks: Vec<PlaylistTrack>,
}

// ── Request types ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreatePlaylistRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePlaylistRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddTrackRequest {
    pub song_id: i32,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn internal_error<E: std::error::Error>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

fn not_found(msg: &str) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, msg.to_string())
}

fn bad_request(msg: &str) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, msg.to_string())
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /playlist — list the current user's playlists
pub async fn list_playlists(
    Extension(pool): Extension<PgPool>,
    AuthUser { payload }: AuthUser,
) -> Result<Json<Vec<PlaylistSummary>>, (StatusCode, String)> {
    let user_id = payload.sub.parse::<i32>().map_err(|e| bad_request(&e.to_string()))?;

    let rows = sqlx::query!(
        r#"
        SELECT p.id, p.name, p.description, p.art_path, p.created_at, p.updated_at,
               COUNT(pi.id) AS track_count
        FROM playlist p
        LEFT JOIN playlist_item pi ON p.id = pi.playlist_id
        WHERE p.user_id = $1
        GROUP BY p.id
        ORDER BY p.created_at DESC
        "#,
        user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(
        rows.into_iter()
            .map(|r| PlaylistSummary {
                id: r.id,
                name: r.name,
                description: r.description,
                art_path: r.art_path,
                track_count: r.track_count.unwrap_or(0),
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect(),
    ))
}

/// POST /playlist — create a new playlist
pub async fn create_playlist(
    Extension(pool): Extension<PgPool>,
    AuthUser { payload }: AuthUser,
    Json(req): Json<CreatePlaylistRequest>,
) -> Result<Json<PlaylistSummary>, (StatusCode, String)> {
    let user_id = payload.sub.parse::<i32>().map_err(|e| bad_request(&e.to_string()))?;

    let row = sqlx::query!(
        r#"
        INSERT INTO playlist (user_id, name, description)
        VALUES ($1, $2, $3)
        RETURNING id, name, description, art_path, created_at, updated_at
        "#,
        user_id,
        req.name,
        req.description
    )
    .fetch_one(&pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(PlaylistSummary {
        id: row.id,
        name: row.name,
        description: row.description,
        art_path: row.art_path,
        track_count: 0,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }))
}

/// GET /playlist/:id — get a playlist with ordered tracks
pub async fn get_playlist(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    Host(host): Host,
    AuthUser { payload }: AuthUser,
) -> Result<Json<PlaylistDetail>, (StatusCode, String)> {
    let user_id = payload.sub.parse::<i32>().map_err(|e| bad_request(&e.to_string()))?;
    let art_base = build_default_art_url(host);

    let playlist = sqlx::query!(
        "SELECT id, name, description, art_path, created_at, updated_at FROM playlist WHERE id = $1 AND user_id = $2",
        id,
        user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| not_found("Playlist not found"))?;

    // Fetch all items with their song info in one query
    let rows = sqlx::query!(
        r#"
        SELECT pi.id AS item_id, pi.song_id, pi.prev_song_id, pi.next_song_id,
               s.name, s.duration, s.number, s.disc, s.lossless,
               s.album AS album_id,
               album.name AS album_name,
               artist.name AS artist_name,
               (SELECT album_art.path FROM album_art WHERE album_art.album = s.album LIMIT 1) AS art_path
        FROM playlist_item pi
        JOIN song s ON pi.song_id = s.id
        JOIN album ON s.album = album.id
        JOIN artist ON s.album_artist = artist.id
        WHERE pi.playlist_id = $1
        "#,
        id
    )
    .fetch_all(&pool)
    .await
    .map_err(internal_error)?;

    let song_ids: Vec<i32> = rows.iter().map(|r| r.song_id).collect();
    let liked_ids = liked_ids_for_user(&pool, Some(user_id)).await;

    // Build an index so we can traverse the doubly-linked list in O(n)
    let id_to_idx: HashMap<i32, usize> = rows
        .iter()
        .enumerate()
        .map(|(idx, r)| (r.item_id, idx))
        .collect();

    let _ = song_ids; // used via liked_ids lookup below

    let head_idx = rows.iter().position(|r| r.prev_song_id.is_none());

    let mut ordered: Vec<PlaylistTrack> = Vec::with_capacity(rows.len());
    if let Some(mut idx) = head_idx {
        loop {
            let r = &rows[idx];
            ordered.push(PlaylistTrack {
                item_id: r.item_id,
                song_id: r.song_id,
                name: r.name.clone(),
                duration: r.duration,
                number: r.number,
                disc: r.disc,
                liked: Some(liked_ids.contains(&r.song_id)),
                lossless: r.lossless,
                album_id: r.album_id,
                album_name: r.album_name.clone(),
                artist_name: r.artist_name.clone(),
                art_url: r.art_path.as_ref().map(|p| format!("{}{}", art_base, p)),
                prev_item_id: r.prev_song_id,
                next_item_id: r.next_song_id,
            });
            match r.next_song_id.and_then(|nid| id_to_idx.get(&nid)) {
                Some(&next_idx) => idx = next_idx,
                None => break,
            }
        }
    }

    Ok(Json(PlaylistDetail {
        id: playlist.id,
        name: playlist.name,
        description: playlist.description,
        art_path: playlist.art_path,
        created_at: playlist.created_at,
        updated_at: playlist.updated_at,
        tracks: ordered,
    }))
}

/// PUT /playlist/:id — update playlist name / description
pub async fn update_playlist(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    AuthUser { payload }: AuthUser,
    Json(req): Json<UpdatePlaylistRequest>,
) -> Result<Json<PlaylistSummary>, (StatusCode, String)> {
    let user_id = payload.sub.parse::<i32>().map_err(|e| bad_request(&e.to_string()))?;

    let row = sqlx::query!(
        r#"
        UPDATE playlist
        SET name        = COALESCE($3, name),
            description = COALESCE($4, description),
            updated_at  = now()
        WHERE id = $1 AND user_id = $2
        RETURNING id, name, description, art_path, created_at, updated_at
        "#,
        id,
        user_id,
        req.name,
        req.description
    )
    .fetch_optional(&pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| not_found("Playlist not found"))?;

    let track_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM playlist_item WHERE playlist_id = $1",
        id
    )
    .fetch_one(&pool)
    .await
    .map_err(internal_error)?
    .unwrap_or(0);

    Ok(Json(PlaylistSummary {
        id: row.id,
        name: row.name,
        description: row.description,
        art_path: row.art_path,
        track_count,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }))
}

/// DELETE /playlist/:id — delete a playlist (cascades to items)
pub async fn delete_playlist(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    AuthUser { payload }: AuthUser,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = payload.sub.parse::<i32>().map_err(|e| bad_request(&e.to_string()))?;

    let result = sqlx::query!(
        "DELETE FROM playlist WHERE id = $1 AND user_id = $2",
        id,
        user_id
    )
    .execute(&pool)
    .await
    .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        return Err(not_found("Playlist not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// POST /playlist/:id/tracks — append a track to the end of the playlist
pub async fn add_track(
    Path(playlist_id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    Host(host): Host,
    AuthUser { payload }: AuthUser,
    Json(req): Json<AddTrackRequest>,
) -> Result<Json<PlaylistTrack>, (StatusCode, String)> {
    let user_id = payload.sub.parse::<i32>().map_err(|e| bad_request(&e.to_string()))?;
    let art_base = build_default_art_url(host);

    // Verify ownership
    sqlx::query!(
        "SELECT id FROM playlist WHERE id = $1 AND user_id = $2",
        playlist_id,
        user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| not_found("Playlist not found"))?;

    // Find the current tail (item with no next pointer)
    let tail_id = sqlx::query_scalar!(
        "SELECT id FROM playlist_item WHERE playlist_id = $1 AND next_song_id IS NULL",
        playlist_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(internal_error)?;

    // Insert the new item, linked after the tail
    let new_id = sqlx::query_scalar!(
        r#"
        INSERT INTO playlist_item (playlist_id, song_id, prev_song_id)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        playlist_id,
        req.song_id,
        tail_id
    )
    .fetch_one(&pool)
    .await
    .map_err(internal_error)?;

    // Point the old tail's next at the new item
    if let Some(tid) = tail_id {
        sqlx::query!(
            "UPDATE playlist_item SET next_song_id = $1 WHERE id = $2",
            new_id,
            tid
        )
        .execute(&pool)
        .await
        .map_err(internal_error)?;
    }

    sqlx::query!(
        "UPDATE playlist SET updated_at = now() WHERE id = $1",
        playlist_id
    )
    .execute(&pool)
    .await
    .map_err(internal_error)?;

    // Return the new item with song info
    let song = sqlx::query!(
        r#"
        SELECT s.name, s.duration, s.number, s.disc, s.lossless,
               s.album AS album_id, album.name AS album_name, artist.name AS artist_name,
               (SELECT album_art.path FROM album_art WHERE album_art.album = s.album LIMIT 1) AS art_path
        FROM song s
        JOIN album ON s.album = album.id
        JOIN artist ON s.album_artist = artist.id
        WHERE s.id = $1
        "#,
        req.song_id
    )
    .fetch_one(&pool)
    .await
    .map_err(internal_error)?;

    let liked_ids = liked_ids_for_user(&pool, Some(user_id)).await;

    Ok(Json(PlaylistTrack {
        item_id: new_id,
        song_id: req.song_id,
        name: song.name,
        duration: song.duration,
        number: song.number,
        disc: song.disc,
        liked: Some(liked_ids.contains(&req.song_id)),
        lossless: song.lossless,
        album_id: song.album_id,
        album_name: song.album_name,
        artist_name: song.artist_name,
        art_url: song.art_path.map(|p| format!("{}{}", art_base, p)),
        prev_item_id: tail_id,
        next_item_id: None,
    }))
}

/// DELETE /playlist/:id/tracks/:item_id — remove a track and re-link its neighbours
pub async fn remove_track(
    Path((playlist_id, item_id)): Path<(i32, i32)>,
    Extension(pool): Extension<PgPool>,
    AuthUser { payload }: AuthUser,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = payload.sub.parse::<i32>().map_err(|e| bad_request(&e.to_string()))?;

    // Verify ownership
    sqlx::query!(
        "SELECT id FROM playlist WHERE id = $1 AND user_id = $2",
        playlist_id,
        user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| not_found("Playlist not found"))?;

    // Fetch the item to remove so we know its neighbours
    let item = sqlx::query!(
        "SELECT id, prev_song_id, next_song_id FROM playlist_item WHERE id = $1 AND playlist_id = $2",
        item_id,
        playlist_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| not_found("Track not found in playlist"))?;

    // Splice: point prev's next → item's next
    if let Some(prev_id) = item.prev_song_id {
        sqlx::query!(
            "UPDATE playlist_item SET next_song_id = $1 WHERE id = $2",
            item.next_song_id,
            prev_id
        )
        .execute(&pool)
        .await
        .map_err(internal_error)?;
    }

    // Splice: point next's prev → item's prev
    if let Some(next_id) = item.next_song_id {
        sqlx::query!(
            "UPDATE playlist_item SET prev_song_id = $1 WHERE id = $2",
            item.prev_song_id,
            next_id
        )
        .execute(&pool)
        .await
        .map_err(internal_error)?;
    }

    sqlx::query!("DELETE FROM playlist_item WHERE id = $1", item_id)
        .execute(&pool)
        .await
        .map_err(internal_error)?;

    sqlx::query!(
        "UPDATE playlist SET updated_at = now() WHERE id = $1",
        playlist_id
    )
    .execute(&pool)
    .await
    .map_err(internal_error)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Reorder ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ReorderTrackRequest {
    /// The item_id to insert after. None = move to head of playlist.
    pub after_item_id: Option<i32>,
}

/// PUT /playlist/:id/tracks/:item_id/position
/// Move a track to a new position. `after_item_id: null` moves it to the head.
pub async fn reorder_track(
    Path((playlist_id, item_id)): Path<(i32, i32)>,
    Extension(pool): Extension<PgPool>,
    AuthUser { payload }: AuthUser,
    Json(req): Json<ReorderTrackRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = payload.sub.parse::<i32>().map_err(|e| bad_request(&e.to_string()))?;

    // Verify ownership
    sqlx::query!(
        "SELECT id FROM playlist WHERE id = $1 AND user_id = $2",
        playlist_id,
        user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| not_found("Playlist not found"))?;

    // No-op if moving after itself
    if req.after_item_id == Some(item_id) {
        return Ok(StatusCode::NO_CONTENT);
    }

    // Fetch the item being moved
    let item = sqlx::query!(
        "SELECT id, prev_song_id, next_song_id FROM playlist_item WHERE id = $1 AND playlist_id = $2",
        item_id,
        playlist_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| not_found("Track not found in playlist"))?;

    // Already in position
    if item.prev_song_id == req.after_item_id {
        return Ok(StatusCode::NO_CONTENT);
    }

    // Step 1: splice item out of its current position
    if let Some(prev_id) = item.prev_song_id {
        sqlx::query!(
            "UPDATE playlist_item SET next_song_id = $1 WHERE id = $2",
            item.next_song_id,
            prev_id
        )
        .execute(&pool)
        .await
        .map_err(internal_error)?;
    }
    if let Some(next_id) = item.next_song_id {
        sqlx::query!(
            "UPDATE playlist_item SET prev_song_id = $1 WHERE id = $2",
            item.prev_song_id,
            next_id
        )
        .execute(&pool)
        .await
        .map_err(internal_error)?;
    }

    // Step 2: find the item currently sitting after the target insertion point.
    // after_item_id = None means insert at head, so successor = current head.
    let successor_id: Option<i32> = match req.after_item_id {
        Some(after_id) => {
            let anchor = sqlx::query!(
                "SELECT next_song_id FROM playlist_item WHERE id = $1 AND playlist_id = $2",
                after_id,
                playlist_id
            )
            .fetch_optional(&pool)
            .await
            .map_err(internal_error)?
            .ok_or_else(|| not_found("after_item_id not found in playlist"))?;
            anchor.next_song_id
        }
        None => {
            sqlx::query_scalar!(
                "SELECT id FROM playlist_item WHERE playlist_id = $1 AND prev_song_id IS NULL AND id != $2",
                playlist_id,
                item_id
            )
            .fetch_optional(&pool)
            .await
            .map_err(internal_error)?
        }
    };

    // Step 3: insert item between after_item_id and successor_id
    sqlx::query!(
        "UPDATE playlist_item SET prev_song_id = $1, next_song_id = $2 WHERE id = $3",
        req.after_item_id,
        successor_id,
        item_id
    )
    .execute(&pool)
    .await
    .map_err(internal_error)?;

    if let Some(after_id) = req.after_item_id {
        sqlx::query!(
            "UPDATE playlist_item SET next_song_id = $1 WHERE id = $2",
            item_id,
            after_id
        )
        .execute(&pool)
        .await
        .map_err(internal_error)?;
    }
    if let Some(succ_id) = successor_id {
        sqlx::query!(
            "UPDATE playlist_item SET prev_song_id = $1 WHERE id = $2",
            item_id,
            succ_id
        )
        .execute(&pool)
        .await
        .map_err(internal_error)?;
    }

    sqlx::query!(
        "UPDATE playlist SET updated_at = now() WHERE id = $1",
        playlist_id
    )
    .execute(&pool)
    .await
    .map_err(internal_error)?;

    Ok(StatusCode::NO_CONTENT)
}
