use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use serde::Deserialize;
use std::num::NonZeroU32;
use std::sync::OnceLock;
use tracing::debug;

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
static RATE_LIMITER: OnceLock<DefaultDirectRateLimiter> = OnceLock::new();

fn client() -> &'static reqwest::Client {
    CLIENT.get_or_init(|| reqwest::Client::new())
}

fn limiter() -> &'static DefaultDirectRateLimiter {
    RATE_LIMITER.get_or_init(|| {
        RateLimiter::direct(Quota::per_second(NonZeroU32::new(10).unwrap()))
    })
}

pub struct DeezerArtist {
    pub id: u64,
    pub picture: Option<String>,
}

/// Search for an artist by name, returning their Deezer ID and best available image.
pub async fn get_artist(name: &str) -> anyhow::Result<Option<DeezerArtist>> {
    limiter().until_ready().await;

    let url = format!(
        "https://api.deezer.com/search/artist?q={}",
        urlencoding::encode(name)
    );

    debug!("Searching Deezer for artist: {}", name);

    let res = client().get(&url).send().await?;
    if !res.status().is_success() {
        return Ok(None);
    }

    let body: SearchResponse<ArtistItem> = res.json().await?;

    Ok(body.data.into_iter().next().map(|a| DeezerArtist {
        id: a.id,
        picture: a.picture_xl.or(a.picture_big).or(a.picture_medium),
    }))
}

/// Fetch related artist names for a given Deezer artist ID.
pub async fn get_related_artists(deezer_id: u64) -> anyhow::Result<Vec<String>> {
    limiter().until_ready().await;

    let url = format!(
        "https://api.deezer.com/artist/{}/related?limit=20",
        deezer_id
    );

    debug!("Fetching Deezer related artists for id: {}", deezer_id);

    let res = client().get(&url).send().await?;
    if !res.status().is_success() {
        return Ok(vec![]);
    }

    let body: SearchResponse<RelatedArtist> = res.json().await?;
    Ok(body.data.into_iter().map(|a| a.name).collect())
}

/// Search for an album by title and artist, returning the best available cover URL.
pub async fn get_album_cover(title: &str, artist: &str) -> anyhow::Result<Option<String>> {
    limiter().until_ready().await;

    let query = format!("{} {}", title, artist);
    let url = format!(
        "https://api.deezer.com/search/album?q={}",
        urlencoding::encode(&query)
    );

    debug!("Searching Deezer for album cover: {} by {}", title, artist);

    let res = client().get(&url).send().await?;
    if !res.status().is_success() {
        return Ok(None);
    }

    let body: SearchResponse<AlbumItem> = res.json().await?;

    Ok(body
        .data
        .into_iter()
        .next()
        .and_then(|a| a.cover_xl.or(a.cover_big).or(a.cover_medium)))
}


#[derive(Deserialize)]
struct SearchResponse<T> {
    data: Vec<T>,
}

#[derive(Deserialize)]
struct ArtistItem {
    id: u64,
    picture_medium: Option<String>,
    picture_big: Option<String>,
    picture_xl: Option<String>,
}

#[derive(Deserialize)]
struct AlbumItem {
    cover_medium: Option<String>,
    cover_big: Option<String>,
    cover_xl: Option<String>,
}

#[derive(Deserialize)]
struct RelatedArtist {
    name: String,
}

