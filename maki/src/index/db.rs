use std::{
    convert::TryInto,
    io::{Cursor, Read},
    time::UNIX_EPOCH,
};

use base64::Engine;
use md5::{
    digest::{ExtendableOutput, Update},
    Digest, Md5,
};
use sha3::Shake128;
use sqlx::{postgres::Postgres, Row};
use time::OffsetDateTime;
use tracing::{debug, error, info, warn};

use crate::{
    helpers::normalize::search_key,
    metadata::{deezer, fm, spotify, theaudiodb, AudioMetadata},
};

/// Stable public slug — hex-encoded MD5 of the given key string.
/// Matches the SQL backfill in the migration: `md5(key)`.
fn make_slug(key: &str) -> String {
    let mut h = Md5::new();
    Digest::update(&mut h, key.as_bytes());
    format!("{:x}", h.finalize())
}

/// Strip null bytes from a string — PostgreSQL text columns reject 0x00,
/// which frequently appears in corrupted or poorly-encoded metadata tags.
fn sanitize_str(s: &str) -> String {
    s.replace('\0', "")
}

/// Clean and validate one track's text fields before it reaches the DB.
fn prepare_metadata(mut metadata: AudioMetadata) -> anyhow::Result<AudioMetadata> {
    metadata.name = sanitize_str(&metadata.name);
    metadata.album = sanitize_str(&metadata.album);
    metadata.album_artist = sanitize_str(&metadata.album_artist);
    if let Some(s) = &mut metadata.album_sort {
        *s = sanitize_str(s);
    }
    metadata.artists = metadata.artists.iter().map(|s| sanitize_str(s)).collect();
    if let Some(g) = &mut metadata.genre {
        *g = g.iter().map(|s| sanitize_str(s)).collect();
    }
    metadata.composer = metadata.composer.as_deref().map(sanitize_str);
    metadata.isrc = metadata.isrc.as_deref().map(sanitize_str);
    metadata.copyright = metadata.copyright.as_deref().map(sanitize_str);
    metadata.label = metadata.label.as_deref().map(sanitize_str);

    // With path fallbacks applied, an empty field means the path itself had no usable identity.
    if metadata.name.trim().is_empty()
        || metadata.album.trim().is_empty()
        || metadata.album_artist.trim().is_empty()
    {
        anyhow::bail!("empty name/album/album_artist for {}", metadata.path.display());
    }
    Ok(metadata)
}

/// Save one fully-read album folder. All rows in the candidate are written by this call;
/// nothing is visible to matching while the folder is still being read.
pub async fn save_candidate(
    candidate: crate::index::folder::Candidate,
    pool: sqlx::Pool<Postgres>,
) -> anyhow::Result<()> {
    if candidate.tracks.is_empty() {
        return Ok(());
    }
    let tracks: Vec<AudioMetadata> = candidate
        .tracks
        .into_iter()
        .map(prepare_metadata)
        .collect::<anyhow::Result<Vec<_>>>()?;

    let mut tx = pool.begin().await?;
    let conn = &mut *tx;

    // Resolve the album artist once, not once per file.
    let mut album_meta = tracks[0].clone();
    album_meta.artists = vec![album_meta.album_artist.clone()];
    let album_artists = artist_foc(album_meta, conn).await?;
    let album_artist_id = *album_artists
        .first()
        .ok_or_else(|| anyhow::anyhow!("no album artist resolved"))?;

    let genres = tracks.iter().find_map(|t| t.genre.clone());
    let genre_ids = match &genres {
        Some(g) => Some(genre_foc(g, conn).await?),
        None => None,
    };

    let album_id = album_foc(
        tracks[0].clone(),
        vec![album_artist_id],
        conn,
        genre_ids.clone(),
        &candidate.key,
        candidate.complete,
    )
    .await?;

    for mut track in tracks {
        if track.disc.is_none() {
            track.disc = candidate.key.disc;
        }
        let artists = artist_foc(track.clone(), conn).await?;
        let track_genres = match track.genre.clone() {
            Some(g) => Some(genre_foc(&g, conn).await?),
            None => None,
        };
        song_foc(
            track,
            artists,
            album_id,
            track_genres.or_else(|| genre_ids.clone()),
            conn,
        )
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Find or create artist. Deliberately DB-only: every network call is queued for the worker.
/// Record an identity conflict for later review. `target` is stable per (kind, artist/alias).
async fn record_identity_conflict(
    conn: &mut sqlx::PgConnection,
    kind: &str,
    target: i32,
    detail: &str,
) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO identity_review (kind, target, detail) VALUES ($1, $2, $3) \
         ON CONFLICT (kind, target) DO UPDATE SET detail = EXCLUDED.detail, created_at = now()",
        kind,
        target,
        detail
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn artist_foc(
    metadata: AudioMetadata,
    conn: &mut sqlx::PgConnection,
) -> anyhow::Result<Vec<i32>> {
    let mut artist_ids = Vec::new();

    for arti in metadata.artists {
        // A track artist inherits the album artist's MBID only when the names agree.
        let this_mbid = if arti == metadata.album_artist {
            metadata.mbid_artist.clone()
        } else {
            None
        };
        let mut artist_id: Option<i32> = None;
        let mut unresolved = false;

        // 1. Exact MBID match.
        if let Some(mbid) = &this_mbid {
            if let Some(id) = sqlx::query_scalar!("SELECT id FROM artist WHERE mbid = $1", mbid)
                .fetch_optional(&mut *conn)
                .await?
            {
                artist_id = Some(id);
            }
        }

        // 2. Exact name. A tag MBID that conflicts with the row's MBID is recorded, not merged.
        if artist_id.is_none() {
            let rows = sqlx::query!("SELECT id, mbid FROM artist WHERE name = $1 ORDER BY id", arti)
                .fetch_all(&mut *conn)
                .await?;
            if rows.len() > 1 {
                let ids: Vec<i32> = rows.iter().map(|r| r.id).collect();
                warn!("artist name {arti} is ambiguous: {ids:?}; leaving unresolved");
                record_identity_conflict(&mut *conn, "artist_name", ids[0],
                    &format!("name={arti} candidates={ids:?}")).await?;
                unresolved = true;
            } else if let Some(row) = rows.first() {
                if let (Some(tag), Some(existing)) = (&this_mbid, row.mbid.as_deref()) {
                    if tag != existing {
                        record_identity_conflict(&mut *conn, "artist_mbid_conflict", row.id,
                            &format!("tag {tag} conflicts with stored {existing} for {arti}")).await?;
                        unresolved = true;
                    }
                }
                if !unresolved {
                    artist_id = Some(row.id);
                }
            }
        }

        // 3. MusicBrainz-synced aliases. Ambiguity is recorded and left unresolved.
        if artist_id.is_none() && !unresolved {
            let key = search_key(&arti);
            let matches = sqlx::query!(
                "SELECT a.artist AS id, ar.mbid FROM artist_alias a \
                 JOIN artist ar ON ar.id = a.artist WHERE a.alias_key = $1 ORDER BY a.artist",
                key
            )
            .fetch_all(&mut *conn)
            .await?;
            match matches.len() {
                0 => {}
                1 => {
                    if let (Some(tag), Some(existing)) = (&this_mbid, matches[0].mbid.as_deref()) {
                        if tag != existing {
                            record_identity_conflict(&mut *conn, "artist_mbid_conflict", matches[0].id,
                                &format!("tag {tag} conflicts with stored {existing} for alias {arti}")).await?;
                            unresolved = true;
                        }
                    }
                    if !unresolved {
                        artist_id = Some(matches[0].id);
                    }
                }
                _ => {
                    let ids: Vec<i32> = matches.iter().map(|m| m.id).collect();
                    warn!("alias {arti} maps to multiple artists: {ids:?}; leaving unresolved");
                    let target = i32::from_str_radix(&make_slug(&key)[..8], 16)
                        .unwrap_or(0);
                    record_identity_conflict(&mut *conn, "artist_alias", target,
                        &format!("alias_key={key} candidates={ids:?}")).await?;
                    unresolved = true;
                }
            }
        }

        let id = if unresolved {
            // Do not guess: attach to a single Unknown Artist row, with the conflict on record.
            let slug = make_slug(&crate::metadata::UNKNOWN_ARTIST.to_lowercase());
            sqlx::query!(
                "INSERT INTO artist (name, slug, search_key, search_key_version, created_at) \
                 VALUES ($1, $2, $3, 1, now()) \
                 ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name RETURNING id",
                crate::metadata::UNKNOWN_ARTIST,
                slug,
                search_key(crate::metadata::UNKNOWN_ARTIST),
            )
            .fetch_one(&mut *conn)
            .await?
            .id
        } else {
            match artist_id {
                Some(id) => {
                    if let Some(mbid) = &this_mbid {
                        sqlx::query!(
                            "UPDATE artist SET mbid = $1 WHERE id = $2 AND mbid IS NULL",
                            mbid,
                            id
                        )
                        .execute(&mut *conn)
                        .await?;
                    }
                    id
                }
                None => {
                    let artist_slug = make_slug(&arti.to_lowercase());
                    sqlx::query!(
                        r#"
                        INSERT INTO artist (name, mbid, slug, search_key, search_key_version, created_at)
                        VALUES ($1, $2, $3, $4, $5, now())
                        ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name,
                            search_key = EXCLUDED.search_key,
                            search_key_version = EXCLUDED.search_key_version
                        RETURNING id;
                        "#,
                        arti,
                        this_mbid,
                        artist_slug,
                        search_key(&arti),
                        1i16,
                    )
                    .fetch_one(&mut *conn)
                    .await?
                    .id
                }
            }
        };

        if this_mbid.is_some() {
            crate::enrich::invalidate(&mut *conn, crate::enrich::ARTIST_MBID, id).await?;
        }
        crate::enrich::enqueue(&mut *conn, crate::enrich::ARTIST_MBID, id).await?;
        crate::enrich::enqueue(&mut *conn, crate::enrich::ARTIST_ALIAS, id).await?;
        crate::enrich::enqueue(&mut *conn, crate::enrich::ARTIST_PROFILE, id).await?;
        artist_ids.push(id);
    }

    Ok(artist_ids)
}
/// find or create album
async fn album_foc(
    metadata: AudioMetadata,
    artist: Vec<i32>,
    conn: &mut sqlx::PgConnection,
    genres: Option<Vec<i32>>,
    key: &crate::index::folder::AlbumKey,
    complete: Option<bool>,
) -> anyhow::Result<i32> {
    let mut album_id: Option<i32> = None;
    let edition_key = key.folder.to_string_lossy().into_owned();
    let album_name = key
        .name_override
        .clone()
        .unwrap_or_else(|| metadata.album.clone());

    // 1. Try to find by MBID
    if let Some(mbid) = &metadata.mbid_album {
        if let Ok(Some(id)) = sqlx::query_scalar!("SELECT id FROM album WHERE mbid = $1", mbid)
            .fetch_optional(&mut *conn)
            .await
        {
            album_id = Some(id);
        }
    }

    // 1b. Folder identity. An album without an MBID is identified by the folder it came from,
    // which keeps two editions of one title in separate rows.
    if album_id.is_none() {
        if let Ok(Some(id)) =
            sqlx::query_scalar!("SELECT id FROM album WHERE edition_key = $1", edition_key)
                .fetch_optional(&mut *conn)
                .await
        {
            album_id = Some(id);
        }
    }

    // 2. Try to find by Name AND Artist — only when the file has no MBID and no folder match.
    if album_id.is_none() && metadata.mbid_album.is_none() {
        if let Ok(Some(id)) = sqlx::query_scalar!(
            "SELECT id FROM album WHERE name = $1 AND artist = $2 AND edition_key IS NULL",
            album_name,
            artist[0]
        )
        .fetch_optional(&mut *conn)
        .await
        {
            album_id = Some(id);
        }
    }
    // Tags carry a *release* MBID. Resolving it to a release-group (and finding one when the
    // tags have none) is album-level matching, which now happens in the enrichment worker.
    let album_mbid = metadata.mbid_album.clone();
    let album_disambiguation: Option<String> = None;

    if let Some(id) = album_id {
        if let Some(year) = metadata.year {
            sqlx::query!(
                "UPDATE album SET year = $1 WHERE id = $2 AND year IS NULL",
                year,
                id
            )
            .execute(&mut *conn)
            .await?;
        }
        sqlx::query!(
            "UPDATE album SET edition_key = COALESCE(edition_key, $1), scan_complete = COALESCE(scan_complete, $2) WHERE id = $3",
            edition_key,
            complete,
            id
        )
        .execute(&mut *conn)
        .await?;
        crate::enrich::enqueue(&mut *conn, crate::enrich::ALBUM_ART, id).await?;
        crate::enrich::enqueue(&mut *conn, crate::enrich::ALBUM_MATCH, id).await?;
        Ok(id)
    } else {
        // else we insert the allbum
        // save image as <md5> OR None
        let mut images: Vec<String> = vec![];
        for i in metadata.picture {
            match save_image(i.bytes).await {
                Ok(e) => images.push(e),
                Err(e) => {
                    error!(
                        "failed to save image for album {} to disk so skipping: {}",
                        metadata.name, e
                    );
                    continue;
                }
            };
        }
        // dedupe images
        images.dedup();

        // Art from external services is fetched by the enrichment worker, after the scan.

        // Include the MBID in the slug key when available so that identically-named
        // albums (e.g. self-titled LPs) each get a stable, unique slug.  When there
        // is no MBID we fall back to name|artist and bump a numeric suffix on collision.
        let base_slug_key = format!(
            "{}|{}",
            album_name.to_lowercase(),
            metadata.album_artist.to_lowercase()
        );
        let album_slug = if let Some(mbid) = &album_mbid {
            make_slug(&format!("{}|{}", base_slug_key, mbid))
        } else {
            // Walk until we find a free slug
            let base = make_slug(&base_slug_key);
            let taken: bool =
                sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM album WHERE slug = $1)", base)
                    .fetch_one(&mut *conn)
                    .await?
                    .unwrap_or(false);

            if !taken {
                base
            } else {
                let mut n = 2u32;
                loop {
                    let candidate = format!("{}_{}", base, n);
                    let taken: bool = sqlx::query_scalar!(
                        "SELECT EXISTS(SELECT 1 FROM album WHERE slug = $1)",
                        candidate
                    )
                    .fetch_one(&mut *conn)
                    .await?
                    .unwrap_or(false);
                    if !taken {
                        break candidate;
                    }
                    n += 1;
                }
            }
        };

        // insert into database — use ON CONFLICT to handle the race where two
        // concurrent files from the same album both try to insert at once.
        let row = sqlx::query!(
            r#"
            INSERT INTO album (name, artist, year, mbid, mbid_source, slug, disambiguation, copyright, label, edition_key, scan_complete, search_key, search_key_version, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, now())
            ON CONFLICT (slug) DO NOTHING
            RETURNING id;
            "#,
            album_name,
            artist[0],
            metadata.year,
            album_mbid,
            album_mbid.as_ref().map(|_| "tag-release"),
            album_slug,
            album_disambiguation,
            metadata.copyright,
            metadata.label,
            edition_key,
            complete.unwrap_or(false),
            search_key(&album_name),
            1i16,
        )
        .fetch_optional(&mut *conn)
        .await?;

        // If None, another concurrent insert won the race — look up the existing id.
        let album_id = match row {
            Some(r) => r.id,
            None => {
                sqlx::query_scalar!("SELECT id FROM album WHERE slug = $1", album_slug)
                    .fetch_one(&mut *conn)
                    .await?
            }
        };

        // insert the art path into album-art
        if !images.is_empty() {
            for image in images {
                sqlx::query!(
                    r#"
                    INSERT INTO album_art (album, path, created_at)
                    VALUES ($1, $2, now())
                    ON CONFLICT DO NOTHING
                    "#,
                    album_id,
                    image
                )
                .execute(&mut *conn)
                .await?;
            }
        }
        // insert into genre-album
        // Note: albums themselves don't have 'genres' so this is based on all the genres in all the songs
        // SO we should do an upsert here
        if let Some(genres) = genres {
            for genre in genres {
                sqlx::query!(
                    r#"
                    INSERT INTO album_genre (album, genre, created_at)
                    VALUES ($1, $2, now())
                    ON CONFLICT DO NOTHING
                    "#,
                    album_id,
                    genre
                )
                .execute(&mut *conn)
                .await?;
            }
        }
        crate::enrich::enqueue(&mut *conn, crate::enrich::ALBUM_ART, album_id).await?;
        crate::enrich::enqueue(&mut *conn, crate::enrich::ALBUM_MATCH, album_id).await?;
        Ok(album_id)
    }
}

async fn genre_foc(genres_orig: &[String], conn: &mut sqlx::PgConnection) -> anyhow::Result<Vec<i32>> {
    let mut genre_ids = Vec::new();
    // Genres were already sanitized by prepare_metadata; only split a single comma-separated tag.
    let genres: Vec<String> = if genres_orig.len() == 1 {
        genres_orig[0]
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        genres_orig.to_vec()
    };
    for genre in genres {
        // SELECT-first to avoid the check-then-insert race.
        if let Ok(Some(id)) = sqlx::query_scalar!(
            "SELECT id FROM genre WHERE name = $1 ORDER BY id LIMIT 1",
            genre
        )
        .fetch_optional(&mut *conn)
        .await
        {
            genre_ids.push(id);
            continue;
        }
        // Not found — insert. If a concurrent insert races us, fall back to SELECT.
        let id = match sqlx::query!(
            r#"
            INSERT INTO genre (name, created_at)
            VALUES ($1, now())
            RETURNING id;
            "#,
            genre
        )
        .fetch_one(&mut *conn)
        .await
        {
            Ok(row) => row.id,
            Err(e) => {
                debug!("genre insert failed for '{}' (likely race): {}", genre, e);
                sqlx::query_scalar!(
                    "SELECT id FROM genre WHERE name = $1 ORDER BY id LIMIT 1",
                    genre
                )
                .fetch_one(&mut *conn)
                .await?
            }
        };
        genre_ids.push(id);
    }
    Ok(genre_ids)
}

async fn song_foc(
    metadata: AudioMetadata,
    artist: Vec<i32>,
    album: i32,
    genres: Option<Vec<i32>>,
    conn: &mut sqlx::PgConnection,
) -> anyhow::Result<i32> {
    // The track MBID comes from tags; a lookup when it is missing is queued for the worker.
    let mbid_track = metadata.mbid_track.clone();
    let name_key = search_key(&metadata.name);

    let source_signature = source_file_signature(&metadata.path)?;

    let path_str = metadata.path.to_str().ok_or_else(|| {
        anyhow::anyhow!("path is not valid UTF-8: {}", metadata.path.display())
    })?;

    // check if song exists
    match sqlx::query("SELECT id, audio_hash_size, audio_hash_mtime_ns, mbid FROM song WHERE path = $1")
        .bind(path_str)
        .fetch_optional(&mut *conn)
        .await
    {
        Ok(Some(row)) => {
            let song_id: i32 = row.try_get("id")?;
            // A tag without an MBID must not erase an ID the enrichment worker already found.
            let unchanged_source = row.try_get::<Option<i64>, _>("audio_hash_size")?
                == Some(source_signature.0 as i64)
                && row.try_get::<Option<i64>, _>("audio_hash_mtime_ns")?
                    == Some(source_signature.1);
            // Preserve the current DB value in SQL (not a value read earlier) so a concurrent
            // enrichment write cannot be clobbered. A changed source clears the old MBID so the
            // track is re-identified.
            let preserve_db_mbid = metadata.mbid_track.is_none() && unchanged_source;
            sqlx::query(
                r#"
                UPDATE song SET
                  number = $2, disc = $3, name = $4, album = $5, album_artist = $6,
                  duration = $7, lossless = $8, sample_rate = $9, bits_per_sample = $10,
                  num_channels = $11,
                  mbid = CASE WHEN $12 THEN mbid ELSE $13 END,
                  composer = $14, isrc = $15, bpm = $16,
                  audio_hash = CASE WHEN $17 THEN audio_hash ELSE NULL END,
                  audio_hash_size = $18, audio_hash_mtime_ns = $19,
                  search_key = $20, search_key_version = $21,
                  updated_at = now(), last_scanned_at = now()
                WHERE id = $1
                "#,
            )
            .bind(song_id)
            .bind(metadata.number as i32)
            .bind(metadata.disc.map(|e| e as i32))
            .bind(metadata.name)
            .bind(album as i32)
            .bind(artist[0])
            .bind(metadata.duration as i32)
            .bind(metadata.lossless)
            .bind(metadata.sample_rate.map(|e| e as i32))
            .bind(metadata.bits_per_sample.map(|e| e as i32))
            .bind(metadata.num_channels.map(|e| e as i32))
            .bind(preserve_db_mbid)
            .bind(metadata.mbid_track.clone())
            .bind(metadata.composer)
            .bind(metadata.isrc)
            .bind(metadata.bpm.map(|b| b as i32))
            .bind(unchanged_source)
            .bind(source_signature.0 as i64)
            .bind(source_signature.1)
            .bind(&name_key)
            .bind(1i16)
            .execute(&mut *conn)
            .await?;

            if !unchanged_source {
                sqlx::query("DELETE FROM song_hash_failures WHERE song = $1")
                    .bind(song_id)
                    .execute(&mut *conn)
                    .await?;
            }

            // Nuke HLS cache so stale segments aren't served after a file update
            tokio::fs::remove_dir_all(format!("/tmp/co.lutea.maki/hls/{}", song_id))
                .await
                .ok();

            // Re-sync junction tables
            sqlx::query!("DELETE FROM song_genre WHERE song = $1", song_id)
                .execute(&mut *conn)
                .await?;
            sqlx::query!("DELETE FROM song_artist WHERE song = $1", song_id)
                .execute(&mut *conn)
                .await?;

            if let Some(genres) = genres {
                for genre in genres {
                    sqlx::query!(
                        "INSERT INTO song_genre (song, genre, created_at) VALUES ($1, $2, now())",
                        song_id,
                        genre
                    )
                    .execute(&mut *conn)
                    .await?;
                }
            }
            for &aid in &artist {
                sqlx::query!(
                    "INSERT INTO song_artist (song, artist, created_at) VALUES ($1, $2, now()) ON CONFLICT DO NOTHING",
                    song_id,
                    aid
                )
                .execute(&mut *conn)
                .await?;
            }

            if mbid_track.is_none() {
                crate::enrich::enqueue(&mut *conn, crate::enrich::TRACK_MBID, song_id).await?;
            }
            Ok(song_id)
        }
        Ok(None) => {
            // put in database
            let song_slug = make_slug(path_str);
            let song_id = sqlx::query!(
                r#"
                INSERT INTO song (number, disc, name, path, album, album_artist, liked, duration, plays, lossless, sample_rate, bits_per_sample, num_channels, mbid, slug, composer, isrc, bpm, search_key, search_key_version, last_scanned_at, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, now(), now())
                RETURNING id;
                "#,
                metadata.number as i32,
                metadata.disc.map(|e| e as i32),
                metadata.name,
                path_str,
                album as i32,
                artist[0],
                false,
                metadata.duration as i32,
                0 as i32,
                metadata.lossless,
                metadata.sample_rate.map(|e| e as i32),
                metadata.bits_per_sample.map(|e| e as i32),
                metadata.num_channels.map(|e| e as i32),
                mbid_track,
                song_slug,
                metadata.composer,
                metadata.isrc,
                metadata.bpm.map(|b| b as i32),
                name_key,
                1i16,
            )
            .fetch_one(&mut *conn)
            .await?
            .id;

            // insert into song-genres
            if let Some(genres) = genres {
                for genre in genres {
                    sqlx::query!(
                        "INSERT INTO song_genre (song, genre, created_at) VALUES ($1, $2, now())",
                        song_id,
                        genre
                    )
                    .execute(&mut *conn)
                    .await?;
                }
            }

            // insert into song-artist
            for &aid in &artist {
                sqlx::query!(
                    "INSERT INTO song_artist (song, artist, created_at) VALUES ($1, $2, now()) ON CONFLICT DO NOTHING",
                    song_id,
                    aid
                )
                .execute(&mut *conn)
                .await?;
            }

            if mbid_track.is_none() {
                crate::enrich::enqueue(&mut *conn, crate::enrich::TRACK_MBID, song_id).await?;
            }
            Ok(song_id)
        }
        Err(e) => Err(anyhow::anyhow!("{}", e)),
    }
}

/// Recompute exact search keys after the SQL migration's approximation. Safe to re-run:
/// rows with version 1 are skipped.
pub async fn reindex_search_keys(pool: &sqlx::Pool<Postgres>) -> anyhow::Result<(u64, u64, u64)> {
    const BATCH: i64 = 500;
    let mut counts = (0u64, 0u64, 0u64);

    loop {
        let rows = sqlx::query!(
            "SELECT id, name FROM artist WHERE search_key_version < 1 OR search_key IS NULL              ORDER BY id LIMIT $1",
            BATCH
        )
        .fetch_all(pool)
        .await?;
        if rows.is_empty() {
            break;
        }
        for row in rows {
            sqlx::query!(
                "UPDATE artist SET search_key = $1, search_key_version = 1 WHERE id = $2",
                search_key(&row.name),
                row.id
            )
            .execute(pool)
            .await?;
            counts.0 += 1;
        }
    }

    loop {
        let rows = sqlx::query!(
            "SELECT id, name FROM album WHERE search_key_version < 1 OR search_key IS NULL              ORDER BY id LIMIT $1",
            BATCH
        )
        .fetch_all(pool)
        .await?;
        if rows.is_empty() {
            break;
        }
        for row in rows {
            sqlx::query!(
                "UPDATE album SET search_key = $1, search_key_version = 1 WHERE id = $2",
                search_key(&row.name),
                row.id
            )
            .execute(pool)
            .await?;
            counts.1 += 1;
        }
    }

    loop {
        let rows = sqlx::query!(
            "SELECT id, name FROM song WHERE search_key_version < 1 OR search_key IS NULL              ORDER BY id LIMIT $1",
            BATCH
        )
        .fetch_all(pool)
        .await?;
        if rows.is_empty() {
            break;
        }
        for row in rows {
            sqlx::query!(
                "UPDATE song SET search_key = $1, search_key_version = 1 WHERE id = $2",
                search_key(&row.name),
                row.id
            )
            .execute(pool)
            .await?;
            counts.2 += 1;
        }
    }
    Ok(counts)
}

/// Remove every song whose path lies under a deleted directory.
pub async fn delete_songs_under(prefix: &str, pool: &sqlx::Pool<Postgres>) -> anyhow::Result<()> {
    // `LIKE` would treat underscores and percent signs in real folder names as wildcards.
    let paths: Vec<String> = sqlx::query_scalar!(
        "SELECT path FROM song WHERE starts_with(path, $1)",
        format!("{prefix}/")
    )
    .fetch_all(pool)
    .await?;
    for path in paths {
        delete_song_by_path_inner(&path, pool, false).await?;
    }
    cleanup_orphans(pool).await
}

pub async fn delete_song_by_path(path: &str, pool: &sqlx::Pool<Postgres>) -> anyhow::Result<()> {
    delete_song_by_path_inner(path, pool, true).await
}

async fn delete_song_by_path_inner(
    path: &str,
    pool: &sqlx::Pool<Postgres>,
    cleanup: bool,
) -> anyhow::Result<()> {
    let song_id = match sqlx::query_scalar!("SELECT id FROM song WHERE path = $1 LIMIT 1", path)
        .fetch_optional(pool)
        .await?
    {
        Some(id) => id,
        None => return Ok(()),
    };

    // Nullify LL pointers in sibling playlist_item rows before DELETE fires
    sqlx::query!(
        "UPDATE playlist_item SET prev_song_id = NULL WHERE prev_song_id IN (SELECT id FROM playlist_item WHERE song_id = $1)",
        song_id
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        "UPDATE playlist_item SET next_song_id = NULL WHERE next_song_id IN (SELECT id FROM playlist_item WHERE song_id = $1)",
        song_id
    )
    .execute(pool)
    .await?;

    // favorites has no FK so must be cleaned manually
    sqlx::query!(
        "DELETE FROM favorites WHERE favoritable_id = $1 AND favoritable_type = 'song'",
        song_id
    )
    .execute(pool)
    .await?;

    sqlx::query!("DELETE FROM song WHERE id = $1", song_id)
        .execute(pool)
        .await?;

    if cleanup {
        cleanup_orphans(pool).await?;
    }
    Ok(())
}

/// Invalidate cached audio features when a watcher reports a file replacement.
/// Full scans do not call this, so unchanged files keep their prior analysis.
pub async fn invalidate_features_by_path(
    path: &str,
    pool: &sqlx::Pool<Postgres>,
) -> anyhow::Result<()> {
    let song_id = sqlx::query("SELECT id FROM song WHERE path = $1 LIMIT 1")
        .bind(path)
        .fetch_optional(pool)
        .await?
        .and_then(|row| row.try_get::<i32, _>("id").ok());
    if let Some(song_id) = song_id {
        sqlx::query(
            "UPDATE song SET audio_hash = NULL, audio_hash_size = NULL, audio_hash_mtime_ns = NULL WHERE id = $1",
        )
        .bind(song_id)
        .execute(pool)
        .await?;
        sqlx::query("DELETE FROM song_hash_failures WHERE song = $1")
            .bind(song_id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

fn source_file_signature(path: &std::path::Path) -> anyhow::Result<(u64, i64)> {
    let metadata = std::fs::metadata(path)?;
    let modified = metadata.modified()?.duration_since(UNIX_EPOCH)?;
    let nanos = modified
        .as_secs()
        .checked_mul(1_000_000_000)
        .and_then(|seconds| seconds.checked_add(modified.subsec_nanos() as u64))
        .ok_or_else(|| anyhow::anyhow!("file modification time exceeds i64 nanoseconds"))?;
    Ok((metadata.len(), nanos.try_into()?))
}

/// Remove albums with no songs and artists with no albums or songs.
pub async fn cleanup_orphans(pool: &sqlx::Pool<Postgres>) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM album WHERE id NOT IN (SELECT DISTINCT album FROM song)")
        .execute(pool)
        .await?;

    sqlx::query!(
        r#"
        DELETE FROM artist
        WHERE id NOT IN (SELECT DISTINCT artist FROM album)
          AND id NOT IN (SELECT DISTINCT album_artist FROM song)
          AND id NOT IN (SELECT DISTINCT artist FROM song_artist)
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query!("DELETE FROM artist_similar WHERE artist NOT IN (SELECT id FROM artist)")
        .execute(pool)
        .await?;

    Ok(())
}

/// Count songs that would be pruned by a scan started at `scan_start`.
pub async fn song_count(pool: &sqlx::Pool<Postgres>) -> anyhow::Result<i64> {
    Ok(sqlx::query_scalar!("SELECT count(*) FROM song")
        .fetch_one(pool)
        .await?
        .unwrap_or(0))
}

/// Persist the last prune decision in one statement so the tuple is internally consistent.
pub async fn record_prune(pool: &sqlx::Pool<Postgres>, outcome: &str, detail: &str) {
    if let Err(e) = sqlx::query!(
        "INSERT INTO index_state (key, value) VALUES          ('last_prune_result', $1), ('last_prune_detail', $2), ('last_prune_at', now()::text)          ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value",
        outcome,
        detail
    )
    .execute(pool)
    .await
    {
        warn!("failed to record prune outcome: {e}");
    }
}

/// Establish the library root explicitly. Never inferred from a scan of a populated DB:
/// that would let a subtree scan poison the guard.
pub async fn set_library_root(pool: &sqlx::Pool<Postgres>, root: &str) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO index_state (key, value) VALUES ('library_root', $1)          ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value",
        root
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn stale_count(
    scan_start: OffsetDateTime,
    pool: &sqlx::Pool<Postgres>,
    excluded_folders: &[String],
) -> anyhow::Result<i64> {
    Ok(sqlx::query_scalar!(
        "SELECT count(*) FROM song WHERE (last_scanned_at < $1 OR last_scanned_at IS NULL)          AND NOT EXISTS (SELECT 1 FROM unnest($2::text[]) f WHERE starts_with(song.path, f || '/'))",
        scan_start,
        excluded_folders
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0))
}

/// Delete all songs not stamped during the current scan (stale/removed files) and clean up orphans.
/// `max_fraction` is a fuse: a mount that is empty or points at a subtree must never be able to
/// delete the whole library in one pass.
pub async fn delete_stale_songs(
    scan_start: OffsetDateTime,
    pool: &sqlx::Pool<Postgres>,
    max_fraction: f64,
    excluded_folders: &[String],
) -> anyhow::Result<u64> {
    let stale = stale_count(scan_start, pool, excluded_folders).await?;
    let total: i64 = sqlx::query_scalar!("SELECT count(*) FROM song")
        .fetch_one(pool)
        .await?
        .unwrap_or(0);
    if total > 0 && (stale as f64) > (total as f64 * max_fraction) {
        anyhow::bail!(
            "refusing to prune {stale} eligible of {total} songs; max_prune_fraction={max_fraction} (raise it in config/config.maki.json or delete the stale files explicitly)"
        );
    }
    let stale_ids: Vec<i32> = sqlx::query_scalar!(
        "SELECT id FROM song WHERE (last_scanned_at < $1 OR last_scanned_at IS NULL)          AND NOT EXISTS (SELECT 1 FROM unnest($2::text[]) f WHERE starts_with(song.path, f || '/'))",
        scan_start,
        excluded_folders
    )
    .fetch_all(pool)
    .await?;

    if stale_ids.is_empty() {
        return Ok(0);
    }

    info!("pruning {} stale song(s)", stale_ids.len());

    sqlx::query!(
        "UPDATE playlist_item SET prev_song_id = NULL WHERE prev_song_id IN (SELECT id FROM playlist_item WHERE song_id = ANY($1))",
        &stale_ids
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        "UPDATE playlist_item SET next_song_id = NULL WHERE next_song_id IN (SELECT id FROM playlist_item WHERE song_id = ANY($1))",
        &stale_ids
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        "DELETE FROM favorites WHERE favoritable_type = 'song' AND favoritable_id = ANY($1)",
        &stale_ids
    )
    .execute(pool)
    .await?;

    let count = stale_ids.len() as u64;

    sqlx::query!("DELETE FROM song WHERE id = ANY($1)", &stale_ids)
        .execute(pool)
        .await?;

    if let Err(e) = cleanup_orphans(pool).await {
        warn!("orphan cleanup after stale prune failed: {}", e);
    }

    Ok(count)
}

/// Converts to webp and saves an image to disk under its SHAKE128 hash
async fn save_image(bytes: Vec<u8>) -> anyhow::Result<String> {
    // convert to webp via image crate
    let img = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()?
        .decode()?;

    let format = image::ImageFormat::WebP;
    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), format)?;

    // get the SHAKE128 hash
    let mut hasher = Shake128::default();
    hasher.update(&bytes);
    let mut reader = hasher.finalize_xof();
    let mut buf = [0u8; 10];
    let _ = reader.read(&mut buf);

    // format hash to base 64 urlsafe
    let hash = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf);

    // save image under <hash>.webp
    let dest = format!("./art/{}.webp", &hash);
    if tokio::fs::metadata(&dest).await.is_ok() {
        return Ok(hash.to_string());
    }
    debug!("saving image to {}", dest);
    let mut out = tokio::fs::File::create(&dest).await?;
    tokio::io::copy(&mut &*bytes, &mut out).await?;
    Ok(hash.to_string())
}

/// Download an image from an external URL and cache it as a local webp file.
/// Returns the cache key (filename without .webp) that can be served via
/// `/api/v1/art/{key}`.
async fn cache_external_image(url: &str) -> anyhow::Result<String> {
    debug!("caching external image: {}", url);
    let res = reqwest::get(url).await?;
    if !res.status().is_success() {
        anyhow::bail!("HTTP {} from {}", res.status(), url);
    }
    let bytes = res.bytes().await?.to_vec();
    save_image(bytes).await
}

// ── Enrichment handlers ────────────────────────────────────────────────────────────
// These are the only places in the index that call external services. Everything above
// this line is DB-only so a scan never blocks on the network.

pub async fn enrich_artist_mbid(id: i32, pool: &sqlx::Pool<Postgres>) -> anyhow::Result<()> {
    let row = sqlx::query!("SELECT name, mbid FROM artist WHERE id = $1", id)
        .fetch_optional(pool)
        .await?;
    let Some(row) = row else { return Ok(()) }; // deleted since the job was queued
    if row.mbid.is_some() {
        return Ok(());
    }
    let Some(mbid) = crate::metadata::musicbrainz::get_artist_mbid(&row.name).await? else {
        return Ok(()); // no MB match is not a failure worth retrying
    };
    sqlx::query!(
        "UPDATE artist SET mbid = $1 WHERE id = $2 AND mbid IS NULL",
        mbid,
        id
    )
    .execute(pool)
    .await?;
    // Alias import depends on the MBID. If it already ran with no MBID, reset and requeue it.
    crate::enrich::invalidate(pool, crate::enrich::ARTIST_ALIAS, id).await?;
    crate::enrich::enqueue(pool, crate::enrich::ARTIST_ALIAS, id).await?;
    Ok(())
}

pub async fn enrich_artist_profile(id: i32, pool: &sqlx::Pool<Postgres>) -> anyhow::Result<()> {
    let row = sqlx::query!("SELECT name, mbid, picture FROM artist WHERE id = $1", id)
        .fetch_optional(pool)
        .await?;
    let Some(row) = row else { return Ok(()) };

    // Lazy migration of a hotlinked picture, previously done inline while scanning.
    let mut picture = row.picture;
    if let Some(url) = picture.as_deref().filter(|u| u.starts_with("http")) {
        match cache_external_image(url).await {
            Ok(hash) => {
                sqlx::query!("UPDATE artist SET picture = $1 WHERE id = $2", hash, id)
                    .execute(pool)
                    .await?;
                picture = Some(hash);
            }
            Err(e) => warn!("failed to cache artist image for {}: {}", row.name, e),
        }
    }

    // A missing Last.fm profile is normal; only transport/API errors are propagated.
    let fm_info = fm::get_artist_info(&row.name).await?;
    let deezer_artist = deezer::get_artist(&row.name).await?;
    let deezer_id = deezer_artist.as_ref().map(|a| a.id as i64);

    // Image priority: TheAudioDB (MBID) → Deezer → Spotify. Do not overwrite a cached image.
    let mut image_url = match row.mbid.as_deref() {
        Some(mbid) => theaudiodb::get_artist_image(mbid).await.unwrap_or(None),
        None => None,
    };
    if image_url.is_none() {
        image_url = deezer_artist.and_then(|a| a.picture);
    }
    if image_url.is_none() {
        image_url = spotify::get_artist_image(&row.name).await.unwrap_or(None);
    }
    if picture.is_none() {
        if let Some(url) = image_url {
            picture = cache_external_image(&url).await.ok();
        }
    }

    let bio = fm_info.as_ref().map(|info| info.bio.clone());
    let tags = fm_info.as_ref().map(|info| info.tags.join(","));
    sqlx::query!(
        "UPDATE artist SET bio = COALESCE($1, bio), tags = COALESCE($2, tags), \
         picture = $3, deezer_id = $4 WHERE id = $5",
        bio,
        tags,
        picture,
        deezer_id,
        id
    )
    .execute(pool)
    .await?;

    // Replace rather than append: a retry must not duplicate `similar` rows (no unique key).
    sqlx::query!("DELETE FROM artist_similar WHERE artist = $1", id)
        .execute(pool)
        .await?;
    let mut similar: Vec<(String, String)> = fm_info
        .map(|info| {
            info.similar
                .into_iter()
                .map(|n| (n, "lastfm".to_string()))
                .collect()
        })
        .unwrap_or_default();
    if let Some(deezer_id) = deezer_id {
        if let Ok(found) = deezer::get_related_artists(deezer_id as u64).await {
            similar.extend(found.into_iter().map(|n| (n, "deezer".to_string())));
        }
    }
    for (name, source) in similar {
        sqlx::query!(
            "INSERT INTO artist_similar (artist, name, source, created_at) VALUES ($1, $2, $3, now())",
            id,
            name,
            source
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn enrich_album_art(id: i32, pool: &sqlx::Pool<Postgres>) -> anyhow::Result<()> {
    let has_art: bool = sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM album_art WHERE album = $1)", id)
        .fetch_one(pool)
        .await?
        .unwrap_or(false);
    if has_art {
        return Ok(());
    }
    let album = sqlx::query!("SELECT name, mbid, release_group_mbid, artist FROM album WHERE id = $1", id)
        .fetch_optional(pool)
        .await?;
    let Some(album) = album else { return Ok(()) };

    let mut images: Vec<String> = Vec::new();
    if let Some(mbid) = album.mbid.as_deref() {
        if let Some(bytes) = crate::metadata::musicbrainz::get_cover_art_bytes(mbid).await? {
            images.push(save_image(bytes).await?);
        }
    }
    if images.is_empty() {
        if let Some(rg_mbid) = album.release_group_mbid.as_deref() {
            if let Some(bytes) =
                crate::metadata::musicbrainz::get_release_group_cover_art_bytes(rg_mbid).await?
            {
                images.push(save_image(bytes).await?);
            }
        }
    }
    if images.is_empty() {
        let artist_name: Option<String> =
            sqlx::query_scalar!("SELECT name FROM artist WHERE id = $1", album.artist)
                .fetch_optional(pool)
                .await?;
        if let Some(artist_name) = artist_name {
            if let Some(url) = deezer::get_album_cover(&album.name, &artist_name).await? {
                let bytes = reqwest::get(&url).await?.bytes().await?;
                images.push(save_image(bytes.to_vec()).await?);
            }
        }
    }
    for image in images {
        sqlx::query!(
            "INSERT INTO album_art (album, path, created_at) VALUES ($1, $2, now()) ON CONFLICT DO NOTHING",
            id,
            image
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn enrich_track_mbid(id: i32, pool: &sqlx::Pool<Postgres>) -> anyhow::Result<()> {
    let song = sqlx::query!("SELECT name, mbid, album_artist FROM song WHERE id = $1", id)
        .fetch_optional(pool)
        .await?;
    let Some(song) = song else { return Ok(()) };
    if song.mbid.is_some() {
        return Ok(());
    }
    let artist_name: Option<String> =
        sqlx::query_scalar!("SELECT name FROM artist WHERE id = $1", song.album_artist)
            .fetch_optional(pool)
            .await?;
    let Some(artist_name) = artist_name else { return Ok(()) };
    let Some(mbid) = crate::metadata::musicbrainz::get_track_mbid(&song.name, &artist_name).await?
    else {
        return Ok(());
    };
    sqlx::query!(
        "UPDATE song SET mbid = $1 WHERE id = $2 AND mbid IS NULL",
        mbid,
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}


// ── Identity and album matching enrichment handlers ────────────────────────────────────

/// Import MusicBrainz' curated aliases so romanised and native-script names resolve to one artist.
pub async fn enrich_artist_aliases(id: i32, pool: &sqlx::Pool<Postgres>) -> anyhow::Result<()> {
    let row = sqlx::query!("SELECT mbid FROM artist WHERE id = $1", id)
        .fetch_optional(pool)
        .await?;
    let Some(mbid) = row.and_then(|r| r.mbid) else {
        return Ok(()); // MBID job has not found one yet; it will be retried on the next change
    };
    let aliases = crate::metadata::musicbrainz::get_artist_aliases(&mbid).await?;
    for alias in aliases {
        sqlx::query!(
            "INSERT INTO artist_alias (artist, alias, alias_key, locale, script, kind, source, is_primary) \
             VALUES ($1, $2, $3, $4, $5, $6, 'musicbrainz', $7) \
             ON CONFLICT (artist, alias_key) DO UPDATE SET \
               alias = EXCLUDED.alias, locale = EXCLUDED.locale, script = EXCLUDED.script, \
               kind = EXCLUDED.kind, is_primary = EXCLUDED.is_primary",
            id,
            alias.name,
            search_key(&alias.name),
            alias.locale,
            alias.script,
            alias.kind,
            alias.primary.unwrap_or(false)
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

/// Apply a reviewed or confidently matched release without touching local titles.
/// Recording MBIDs (not release-track MBIDs) are written to songs, keyed by disc and position.
pub async fn apply_release_match(
    album_id: i32,
    release_mbid: &str,
    release_group_mbid: Option<&str>,
    detail: &crate::metadata::musicbrainz::MbReleaseDetail,
    pool: &sqlx::Pool<Postgres>,
) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query!(
        "UPDATE album SET mbid = $1, release_group_mbid = COALESCE(album.release_group_mbid, $2), \
         mbid_source = 'match' WHERE id = $3",
        release_mbid,
        release_group_mbid,
        album_id
    )
    .execute(&mut *tx)
    .await?;
    let mut covered = 0u64;
    for track in &detail.tracks {
        let Some(recording) = &track.recording else { continue };
        let result = sqlx::query(
            "UPDATE song SET mbid = $1 WHERE album = $2 AND number = $3 \
             AND COALESCE(disc, 1) = $4",
        )
        .bind(&recording.id)
        .bind(album_id)
        .bind(track.position)
        .bind(track.disc)
        .execute(&mut *tx)
        .await?;
        covered += result.rows_affected();
    }
    let expected = detail
        .tracks
        .iter()
        .filter(|t| t.recording.is_some())
        .count() as u64;
    if covered != expected {
        anyhow::bail!(
            "match would cover only {covered} of {expected} tracks; refusing partial application"
        );
    }
    sqlx::query!("DELETE FROM album_match_candidate WHERE album = $1", album_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

/// Match a whole album folder against MusicBrainz releases using track count, durations and title.
pub async fn enrich_album_match(id: i32, pool: &sqlx::Pool<Postgres>) -> anyhow::Result<()> {
    let cfg = crate::config::load_or_create_config("config/config.maki.json");
    let album = sqlx::query!(
        "SELECT name, artist, scan_complete, mbid_source FROM album WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await?;
    let Some(album) = album else { return Ok(()) };
    if !album.scan_complete {
        return Ok(()); // never match an album whose tracklist we know is incomplete
    }
    // Tagged release IDs and admin selections are authoritative; do not overwrite them.
    if matches!(album.mbid_source.as_deref(), Some("tag-release") | Some("match")) {
        return Ok(());
    }

    let artist_name: Option<String> =
        sqlx::query_scalar!("SELECT name FROM artist WHERE id = $1", album.artist)
            .fetch_optional(pool)
            .await?;
    let rows = sqlx::query!(
        "SELECT name, duration, number, disc FROM song WHERE album = $1 ORDER BY disc NULLS FIRST, number",
        id
    )
    .fetch_all(pool)
    .await?;
    if rows.is_empty() {
        return Ok(());
    }
    let input = crate::match_album::MatchInput {
        title: album.name.clone(),
        artist: artist_name.unwrap_or_default(),
        tracks: rows
            .iter()
            .map(|r| crate::match_album::LocalTrack {
                disc: r.disc.map(|d| d as u32),
                position: r.number.unwrap_or(0),
                duration: r.duration as u32,
                title: r.name.clone(),
            })
            .collect(),
    };

    let candidates =
        crate::metadata::musicbrainz::search_releases(&input.title, &input.artist, 5).await?;
    let mut ranked: Vec<(
        crate::metadata::musicbrainz::MbReleaseDetail,
        crate::match_album::AlbumScore,
    )> = Vec::new();
    let mut surviving: Vec<String> = Vec::new();

    for candidate in candidates {
        // A weak search score is unlikely to beat the evidence threshold; skip its 1 req/s
        // release lookup entirely on a small, rate-limited box.
        if candidate.score.unwrap_or(0.0) < 50.0 {
            continue;
        }
        let detail = crate::metadata::musicbrainz::get_release(&candidate.id).await?;
        let result = crate::match_album::score(&input, &detail, candidate.score);
        if result.value >= cfg.album_match_candidate_floor {
            let payload = serde_json::json!({
                "median_duration_delta_ms": result.median_duration_delta_ms,
                "track_count_delta": result.track_count_delta,
                "comparable_tracks": result.comparable_tracks,
                "mb_score": candidate.score,
                "candidate_title": candidate.title,
                "candidate_release_group": candidate.release_group.as_ref().map(|rg| rg.id.clone()),
            });
            sqlx::query!(
                "INSERT INTO album_match_candidate (album, release_mbid, release_group_mbid, score, payload) \
                 VALUES ($1, $2, $3, $4, $5) \
                 ON CONFLICT (album, release_mbid) DO UPDATE SET \
                   score = EXCLUDED.score, payload = EXCLUDED.payload, \
                   release_group_mbid = EXCLUDED.release_group_mbid",
                id,
                detail.id,
                detail.release_group.as_ref().map(|rg| rg.id.clone()),
                result.value,
                payload
            )
            .execute(pool)
            .await?;
            surviving.push(detail.id.clone());
            ranked.push((detail, result));
        }
    }
    sqlx::query!(
        "DELETE FROM album_match_candidate WHERE album = $1 AND release_mbid <> ALL($2)",
        id,
        &surviving
    )
    .execute(pool)
    .await?;

    ranked.sort_by(|a, b| b.1.value.total_cmp(&a.1.value));
    let Some((detail, best)) = ranked.first() else { return Ok(()) };
    if best.value < cfg.album_match_min_score || best.comparable_tracks < 2 {
        return Ok(()); // insufficient evidence; left for the review surface
    }
    let runner_up = ranked.get(1).map(|(_, s)| s.value).unwrap_or(0.0);
    if best.value - runner_up < cfg.album_match_min_margin {
        return Ok(()); // indistinguishable candidates; do not guess
    }
    apply_release_match(
        id,
        &detail.id,
        detail.release_group.as_ref().map(|rg| rg.id.as_str()),
        detail,
        pool,
    )
    .await
}
