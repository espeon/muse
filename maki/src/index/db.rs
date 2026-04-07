use std::io::{Cursor, Read};

use base64::Engine;
use md5::{
    digest::{ExtendableOutput, Update},
    Digest, Md5,
};
use sha3::Shake128;
use sqlx::postgres::Postgres;
use time::OffsetDateTime;
use tracing::{debug, error, info, warn};

use crate::metadata::{deezer, fm, spotify, theaudiodb, AudioMetadata};

/// Stable public slug — hex-encoded MD5 of the given key string.
/// Matches the SQL backfill in the migration: `md5(key)`.
fn make_slug(key: &str) -> String {
    let mut h = Md5::new();
    Digest::update(&mut h, key.as_bytes());
    format!("{:x}", h.finalize())
}

pub async fn add_song(metadata: AudioMetadata, pool: sqlx::Pool<Postgres>) {
    let artist = artist_foc(metadata.clone(), pool.clone()).await.unwrap();

    // check for genre data, and if so, find/create.
    // we need genres for albums and songs!
    let genres = if let Some(genre) = &metadata.genre {
        Some(
            genre_foc(genre, pool.clone())
                .await
                .expect("genres created or found"),
        )
    } else {
        None
    };

    let album = album_foc(
        metadata.clone(),
        artist.clone(),
        pool.clone(),
        genres.clone(),
    )
    .await
    .unwrap();

    // finally, add our track
    if let Err(e) = song_foc(metadata.clone(), artist.clone(), album, genres, pool).await {
        error!(
            "failed to add song {} at path {}: {}",
            metadata.name,
            metadata.path.display(),
            e
        );
    };
}

/// find or create artist
async fn artist_foc(
    metadata: AudioMetadata,
    pool: sqlx::Pool<Postgres>,
) -> anyhow::Result<Vec<i32>> {
    let mut artist_ids = Vec::new();

    for arti in metadata.artists {
        // If this artist name matches the album artist, we can potentially use the MBID
        let this_mbid = if arti == metadata.album_artist {
            metadata.mbid_artist.clone()
        } else {
            None
        };

        let mut artist_id: Option<i32> = None;

        // 1. Try to find by MBID
        if let Some(mbid) = &this_mbid {
            if let Ok(Some(id)) = sqlx::query_scalar!("SELECT id FROM artist WHERE mbid = $1", mbid)
                .fetch_optional(&pool)
                .await
            {
                artist_id = Some(id);
            }
        }

        // 2. Try to find by Name
        if artist_id.is_none() {
            if let Ok(Some(id)) = sqlx::query_scalar!("SELECT id FROM artist WHERE name = $1", arti)
                .fetch_optional(&pool)
                .await
            {
                artist_id = Some(id);
                // TODO: Update MBID if we have one but the DB doesn't?
            }
        }

        // 3. Fall back to MusicBrainz name search if we still have no MBID
        let this_mbid = if this_mbid.is_none() && artist_id.is_none() {
            match crate::metadata::musicbrainz::get_artist_mbid(&arti).await {
                Ok(mbid) => mbid,
                Err(e) => {
                    tracing::warn!("MusicBrainz artist lookup failed for {}: {}", arti, e);
                    None
                }
            }
        } else {
            this_mbid
        };

        if let Some(id) = artist_id {
            artist_ids.push(id);
        } else {
            debug!("artist {} not found, searching...", arti);

            let fm_info = match fm::get_artist_info(&arti).await {
                Ok(e) => e,
                Err(_) => fm::FmArtist {
                    bio: "What a mysterious artist. No bio found.".to_string(),
                    tags: Vec::new(),
                    similar: Vec::new(),
                },
            };

            let deezer_artist = deezer::get_artist(&arti).await.unwrap_or(None);
            let deezer_id = deezer_artist.as_ref().map(|a| a.id as i64);

            // Image priority: TheAudioDB (MBID) → Deezer → Spotify
            let artist_image = if let Some(mbid) = &this_mbid {
                theaudiodb::get_artist_image(mbid).await.unwrap_or(None)
            } else {
                None
            };
            let artist_image = match artist_image.or_else(|| deezer_artist.and_then(|a| a.picture)) {
                Some(img) => Some(img),
                None => spotify::get_artist_image(&arti).await.unwrap_or(None),
            };

            let tags = fm_info.tags.join(",");
            let artist_slug = make_slug(&arti.to_lowercase());

            let id = sqlx::query!(
                r#"
                INSERT INTO artist (name, bio, picture, tags, mbid, deezer_id, slug, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, now())
                RETURNING id;
                "#,
                arti,
                fm_info.bio,
                artist_image,
                tags,
                this_mbid,
                deezer_id,
                artist_slug,
            )
            .fetch_one(&pool)
            .await?
            .id;

            // Store similar artists from all sources
            let mut similar: Vec<(String, String)> = fm_info
                .similar
                .into_iter()
                .map(|n| (n, "lastfm".to_string()))
                .collect();

            if let Some(deezer_id_val) = deezer_id {
                if let Ok(deezer_similar) =
                    deezer::get_related_artists(deezer_id_val as u64).await
                {
                    similar.extend(deezer_similar.into_iter().map(|n| (n, "deezer".to_string())));
                }
            }

            for (name, source) in similar {
                if let Err(e) = sqlx::query!(
                    r#"
                    INSERT INTO artist_similar (artist, name, source, created_at)
                    VALUES ($1, $2, $3, now())
                    "#,
                    id,
                    name,
                    source,
                )
                .execute(&pool)
                .await
                {
                    tracing::warn!("failed to insert similar artist {}: {}", name, e);
                }
            }

            artist_ids.push(id);
        }
    }

    Ok(artist_ids)
}
/// find or create album
async fn album_foc(
    metadata: AudioMetadata,
    artist: Vec<i32>,
    pool: sqlx::Pool<Postgres>,
    genres: Option<Vec<i32>>,
) -> anyhow::Result<i32> {
    let mut album_id: Option<i32> = None;

    // 1. Try to find by MBID
    if let Some(mbid) = &metadata.mbid_album {
        if let Ok(Some(id)) = sqlx::query_scalar!("SELECT id FROM album WHERE mbid = $1", mbid)
            .fetch_optional(&pool)
            .await
        {
            album_id = Some(id);
        }
    }

    // 2. Try to find by Name AND Artist
    if album_id.is_none() {
        if let Ok(Some(id)) = sqlx::query_scalar!(
            "SELECT id FROM album WHERE name = $1 AND artist = $2",
            metadata.album,
            artist[0]
        )
        .fetch_optional(&pool)
        .await
        {
            album_id = Some(id);
        }
    }

    // 3. Fall back to MusicBrainz name search if we have no MBID yet
    let album_mbid = if metadata.mbid_album.is_none() && album_id.is_none() {
        match crate::metadata::musicbrainz::get_album_mbid(&metadata.album, &metadata.album_artist).await {
            Ok(mbid) => mbid,
            Err(e) => {
                tracing::warn!("MusicBrainz album lookup failed for {}: {}", metadata.album, e);
                None
            }
        }
    } else {
        metadata.mbid_album.clone()
    };

    if let Some(id) = album_id {
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

        // if no embedded art, try Cover Art Archive then Deezer
        if images.is_empty() {
            if let Some(mbid) = &album_mbid {
                match crate::metadata::musicbrainz::get_cover_art_bytes(mbid).await {
                    Ok(Some(bytes)) => match save_image(bytes).await {
                        Ok(hash) => images.push(hash),
                        Err(e) => error!("failed to save CAA art for album {}: {}", metadata.album, e),
                    },
                    Ok(None) => debug!("no cover art on CAA for album MBID: {}", mbid),
                    Err(e) => error!("failed to fetch CAA art for album {}: {}", metadata.album, e),
                }
            }
        }
        if images.is_empty() {
            match deezer::get_album_cover(&metadata.album, &metadata.album_artist).await {
                Ok(Some(url)) => match reqwest::get(&url).await {
                    Ok(resp) => match resp.bytes().await {
                        Ok(bytes) => match save_image(bytes.to_vec()).await {
                            Ok(hash) => images.push(hash),
                            Err(e) => error!("failed to save Deezer art for album {}: {}", metadata.album, e),
                        },
                        Err(e) => error!("failed to read Deezer art bytes for album {}: {}", metadata.album, e),
                    },
                    Err(e) => error!("failed to fetch Deezer art for album {}: {}", metadata.album, e),
                },
                Ok(None) => debug!("no Deezer cover for album: {}", metadata.album),
                Err(e) => error!("Deezer album cover search failed for {}: {}", metadata.album, e),
            }
        }

        let album_slug = make_slug(&format!(
            "{}|{}",
            metadata.album.to_lowercase(),
            metadata.album_artist.to_lowercase()
        ));

        // insert into database
        match sqlx::query!(
            r#"
            INSERT INTO album (name, artist, year, mbid, slug, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id;
            "#,
            metadata.album,
            artist[0],
            metadata.year,
            album_mbid,
            album_slug,
            time::OffsetDateTime::now_utc()
        )
        .fetch_all(&pool)
        .await
        {
            Ok(e) => {
                // insert the art path into album-art
                if !images.is_empty() {
                    for image in images {
                        sqlx::query!(
                            r#"
                            INSERT INTO album_art (album, path, created_at)
                            VALUES ($1, $2, now())
                            "#,
                            e[0].id,
                            image
                        )
                        .execute(&pool)
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
                            e[0].id,
                            genre
                        )
                        .execute(&pool)
                        .await?;
                    }
                }
                Ok(e[0].id)
            }
            Err(e) => Err(anyhow::format_err!(e)),
        }
    }
}

async fn genre_foc(genres_orig: &[String], pool: sqlx::Pool<Postgres>) -> anyhow::Result<Vec<i32>> {
    let mut genre_ids = Vec::new();
    let genres = if genres_orig.len() == 1 {
        genres_orig[0]
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>()
    } else {
        // trust original
        genres_orig.to_owned()
    };
    for genre in genres {
        // check if genre exists
        let genre_exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM genre WHERE name = $1
            )
            "#,
            genre
        )
        .fetch_one(&pool)
        .await?
        .unwrap_or(false);
        if !genre_exists {
            // insert genre
            let id = sqlx::query!(
                r#"
            INSERT INTO genre (name, created_at)
            VALUES ($1, now())
            RETURNING id;
            "#,
                genre
            )
            .fetch_one(&pool)
            .await?
            .id;
            genre_ids.push(id);
        } else {
            // if genre exists, get id
            let id = sqlx::query_scalar!(
                r#"
                SELECT id FROM genre WHERE name = $1
                "#,
                genre
            )
            .fetch_one(&pool)
            .await?;
            genre_ids.push(id);
        }
    }
    Ok(genre_ids)
}

async fn song_foc(
    metadata: AudioMetadata,
    artist: Vec<i32>,
    album: i32,
    genres: Option<Vec<i32>>,
    pool: sqlx::Pool<Postgres>,
) -> anyhow::Result<i32> {
    // resolve track MBID: use tag value, or fall back to MusicBrainz lookup
    let mbid_track = if metadata.mbid_track.is_none() {
        let artist_name = metadata.artists.first().map(|s| s.as_str()).unwrap_or("");
        match crate::metadata::musicbrainz::get_track_mbid(&metadata.name, artist_name).await {
            Ok(mbid) => mbid,
            Err(e) => {
                tracing::warn!("MusicBrainz track lookup failed for {}: {}", metadata.name, e);
                None
            }
        }
    } else {
        metadata.mbid_track.clone()
    };

    // check if song exists
    match sqlx::query!(
        r#"
        select id
        from song
        where path = $1
        "#,
        metadata.path.to_str().unwrap()
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(row)) => {
            let song_id = row.id;
            // Update metadata and stamp last_scanned_at; leave plays/liked/last_play/created_at untouched
            sqlx::query!(
                r#"
                UPDATE song SET
                  number = $2, disc = $3, name = $4, album = $5, album_artist = $6,
                  duration = $7, lossless = $8, sample_rate = $9, bits_per_sample = $10,
                  num_channels = $11, mbid = $12, updated_at = now(), last_scanned_at = now()
                WHERE id = $1
                "#,
                song_id,
                metadata.number as i32,
                metadata.disc.map(|e| e as i32),
                metadata.name,
                album as i32,
                artist[0],
                metadata.duration as i32,
                metadata.lossless,
                metadata.sample_rate.map(|e| e as i32),
                metadata.bits_per_sample.map(|e| e as i32),
                metadata.num_channels.map(|e| e as i32),
                mbid_track,
            )
            .execute(&pool)
            .await?;

            // Re-sync junction tables
            sqlx::query!("DELETE FROM song_genre WHERE song = $1", song_id)
                .execute(&pool)
                .await?;
            sqlx::query!("DELETE FROM song_artist WHERE song = $1", song_id)
                .execute(&pool)
                .await?;

            if let Some(genres) = genres {
                for genre in genres {
                    sqlx::query!(
                        "INSERT INTO song_genre (song, genre, created_at) VALUES ($1, $2, now())",
                        song_id,
                        genre
                    )
                    .execute(&pool)
                    .await?;
                }
            }
            for a in &metadata.artists {
                sqlx::query!(
                    "INSERT INTO song_artist (song, artist, created_at) VALUES ($1, (SELECT id FROM artist WHERE name = $2), now())",
                    song_id,
                    a
                )
                .execute(&pool)
                .await?;
            }

            Ok(song_id)
        }
        Ok(None) => {
            // put in database
            let p = metadata.path.to_str().unwrap();
            let song_slug = make_slug(p);
            let rows = sqlx::query!(
                r#"
                INSERT INTO song (number, disc, name, path, album, album_artist, liked, duration, plays, lossless, sample_rate, bits_per_sample, num_channels, mbid, slug, last_scanned_at, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, now(), now())
                RETURNING id;
                "#,
                metadata.number as i32,
                metadata.disc.map(|e| e as i32),
                metadata.name,
                p,
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
            )
            .fetch_all(&pool)
            .await?;

            let song_id = rows[0].id;

            // insert into song-genres
            if let Some(genres) = genres {
                for genre in genres {
                    sqlx::query!(
                        "INSERT INTO song_genre (song, genre, created_at) VALUES ($1, $2, now())",
                        song_id,
                        genre
                    )
                    .execute(&pool)
                    .await?;
                }
            }

            // insert into song-artist
            for a in metadata.artists {
                sqlx::query!(
                    "INSERT INTO song_artist (song, artist, created_at) VALUES ($1, (SELECT id FROM artist WHERE name = $2), now())",
                    song_id,
                    a
                )
                .execute(&pool)
                .await?;
            }

            Ok(song_id)
        }
        Err(e) => Err(anyhow::anyhow!("{}", e)),
    }
}

/// Delete a song by its file path and clean up orphaned albums/artists.
pub async fn delete_song_by_path(path: &str, pool: &sqlx::Pool<Postgres>) -> anyhow::Result<()> {
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

    cleanup_orphans(pool).await
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

/// Delete all songs not stamped during the current scan (stale/removed files) and clean up orphans.
/// Returns the number of songs deleted.
pub async fn delete_stale_songs(
    scan_start: OffsetDateTime,
    pool: &sqlx::Pool<Postgres>,
) -> anyhow::Result<u64> {
    let stale_ids: Vec<i32> = sqlx::query_scalar!(
        "SELECT id FROM song WHERE last_scanned_at < $1 OR last_scanned_at IS NULL",
        scan_start
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
    // check if file exists
    let dest = format! {"./art/{}.webp",&hash};
    if tokio::fs::metadata(&dest).await.is_ok() {
        return Ok(hash.to_string());
    }
    debug!("saving image to {}", dest);
    // TODO: save to s3 or some sort of cdn
    let mut out = tokio::fs::File::create(&dest).await?;
    tokio::io::copy(&mut &*bytes, &mut out).await?;
    Ok(hash.to_string())
}
