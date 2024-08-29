use std::io::{Cursor, Read};

use base64::Engine;
use image::GenericImageView;
use md5::digest::{ExtendableOutput, Update};
use sha3::Shake128;
use sqlx::postgres::Postgres;
use tracing::{debug, error};

use crate::{
    helpers::split_artists,
    metadata::{fm, spotify, AudioMetadata},
};

pub async fn add_song(metadata: AudioMetadata, pool: sqlx::Pool<Postgres>) {
    let artist = artist_foc(metadata.clone(), pool.clone()).await.unwrap();
    let album = album_foc(metadata.clone(), artist.clone(), pool.clone())
        .await
        .unwrap();

    // check for genre data, and if so, find/create.
    let genres = if let Some(genre) = &metadata.genre {
        Some(
            genre_foc(genre, pool.clone())
                .await
                .expect("genres created or found"),
        )
    } else {
        None
    };

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

    // split artist by 'feat' and similar if there are only one
    let artists = split_artists(metadata.artists.clone());

    for arti in artists {
        let artist_exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM artist WHERE name = $1
            )
            "#,
            arti
        )
        .fetch_one(&pool)
        .await?
        .unwrap_or(false);

        if !artist_exists {
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

            let artist_id = sqlx::query!(
                r#"
                INSERT INTO artist (name, bio, picture, tags, created_at)
                VALUES ($1, $2, $3, $4, now())
                RETURNING id;
                "#,
                arti,
                fm_info.bio,
                artist_image,
                tags
            )
            .fetch_one(&pool)
            .await?
            .id;

            artist_ids.push(artist_id);
        } else {
            let artist_id = sqlx::query_scalar!(
                r#"
                SELECT id FROM artist WHERE name = $1
                "#,
                arti
            )
            .fetch_one(&pool)
            .await?;

            artist_ids.push(artist_id);
        }
    }

    Ok(artist_ids)
}
/// find or create album
async fn album_foc(
    metadata: AudioMetadata,
    artist: Vec<i32>,
    pool: sqlx::Pool<Postgres>,
) -> anyhow::Result<i32> {
    // check if album already exists
    match sqlx::query!(
        r#"
        select id
        from album
        where name = $1;
        "#,
        metadata.album,
    )
    .fetch_all(&pool)
    .await
    {
        Ok(e) => {
            // if no results, return
            if !e.is_empty() {
                Ok(e[0].id)
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

                // insert into database
                match sqlx::query!(
                    r#"
                    INSERT INTO album (name, artist, year, created_at)
                    VALUES ($1, $2, $3, $4)
                    RETURNING id;
                    "#,
                    metadata.album,
                    artist[0],
                    metadata.year,
                    time::OffsetDateTime::now_utc()
                )
                .fetch_all(&pool)
                .await
                {
                    Ok(e) => {
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
                        Ok(e[0].id)
                    }
                    Err(e) => Err(anyhow::format_err!(e)),
                }
            }
        }
        Err(e) => Err(anyhow::format_err!(e)),
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
                        // split artist by 'feat' and similar
                        let artists = split_artists(metadata.artists.clone());
                        for a in artists{
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
    let mut img = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()?
        .decode()?;

    // get current size
    let (width, height) = img.dimensions();
    img = img.resize(width, height, image::imageops::FilterType::Lanczos3);

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
