use std::io::{Cursor, Read};

use base64::Engine;
use md5::digest::{ExtendableOutput, Update};
use sha3::Shake128;
use sqlx::postgres::Postgres;
use tracing::{debug, error};

use crate::metadata::{fm, spotify, AudioMetadata};

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

        if let Some(id) = artist_id {
            artist_ids.push(id);
        } else {
            debug!("artist {} not found, searching...", arti);
            let fm_info = match fm::get_artist_info(&arti).await {
                Ok(e) => e,
                Err(_) => {
                    // if we can't find it return a blank artist
                    fm::FmArtist {
                        bio: "What a mysterious artist. No bio found.".to_string(),
                        tags: Vec::new(),
                        similar: Vec::new(),
                    }
                }
            };
            let artist_image = spotify::get_artist_image(&arti).await?;
            let tags = fm_info.tags.join(",");

            let id = sqlx::query!(
                r#"
                INSERT INTO artist (name, bio, picture, tags, mbid, created_at)
                VALUES ($1, $2, $3, $4, $5, now())
                RETURNING id;
                "#,
                arti,
                fm_info.bio,
                artist_image,
                tags,
                this_mbid
            )
            .fetch_one(&pool)
            .await?
            .id;

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

        // if no embedded art, try the Cover Art Archive
        if images.is_empty() {
            if let Some(mbid) = &metadata.mbid_album {
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

        // insert into database
        match sqlx::query!(
            r#"
            INSERT INTO album (name, artist, year, mbid, created_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id;
            "#,
            metadata.album,
            artist[0],
            metadata.year,
            metadata.mbid_album,
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
    // check if song exists
    match sqlx::query!(
        r#"
        select id
        from song
        where name = $1
        and path = $2
        "#,
        metadata.name,
        metadata.path.to_str().unwrap()
    )
    .fetch_all(&pool)
    .await
    {
        Ok(e) => {
            if !e.is_empty() {
                Ok(e[0].id)
            } else {
                // put in database
                let p = metadata.path.to_str().unwrap();
                match sqlx::query!(
                    r#"
                    INSERT INTO song (number, disc, name, path, album, album_artist, liked, duration, plays, lossless, sample_rate, bits_per_sample, num_channels, created_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, now())
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
                )
                .fetch_all(&pool)
                .await
                {
                    Ok(e) => {
                        // insert into song-genres
                        if let Some(genres) = genres {
                            for genre in genres {
                                sqlx::query!(
                                    r#"
                                    INSERT INTO song_genre (song, genre, created_at)
                                    VALUES ($1, $2, now())
                                    "#,
                                    e[0].id,
                                    genre
                                )
                                .execute(&pool)
                                .await?;
                            }
                        }

                        // insert into song-artist
                        for a in metadata.artists {
                            sqlx::query!(
                                r#"
                                INSERT INTO song_artist (song, artist, created_at)
                                VALUES ($1, (SELECT id FROM artist WHERE name = $2), now())
                                "#,
                                e[0].id,
                                // parse to i32
                                a
                            )
                            .execute(&pool)
                            .await?;
                        }
                        Ok(e[0].id)
                    },
                    Err(e) => {
                        error!("failed to insert song genre: {}", e);
                        Err(anyhow::format_err!(e))
                    }
                }
            }
        }
        Err(e) => {
            error!("failed to insert song: {}", e);
            Err(anyhow::format_err!(e))
        }
    }
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
