use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use serde::{Deserialize, Serialize};
use tracing::debug;
use std::num::NonZeroU32;
use std::sync::OnceLock;

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
static RATE_LIMITER: OnceLock<DefaultDirectRateLimiter> = OnceLock::new();

fn client() -> &'static reqwest::Client {
    CLIENT.get_or_init(reqwest::Client::new)
}

fn limiter() -> &'static DefaultDirectRateLimiter {
    RATE_LIMITER.get_or_init(|| RateLimiter::direct(Quota::per_second(NonZeroU32::new(5).unwrap())))
}

#[derive(Debug, PartialEq, Clone)]
pub struct FmArtist {
    pub bio: String,
    pub tags: Vec<String>,
    pub similar: Vec<String>,
}
pub async fn get_artist_info(artist: &str) -> anyhow::Result<Option<FmArtist>> {
    let key = std::env::var("FM_KEY")
        .map_err(|_| anyhow::anyhow!("FM_KEY not set, skipping Last.fm lookup"))?;

    limiter().until_ready().await;

    // The artist name must be form-encoded: names like "Paul McCartney & Wings" otherwise
    // inject extra query parameters and Last.fm replies with an error document.
    let url = format!(
        "http://ws.audioscrobbler.com/2.0/?method=artist.getinfo&artist={}&api_key={}&format=json",
        urlencoding::encode(artist),
        urlencoding::encode(&key)
    );
    let response = client().get(url).send().await?;
    if !response.status().is_success() {
        anyhow::bail!("Last.fm API error: {}", response.status());
    }
    let result: FmSearchResult = response.json().await?;
    let Some(artist) = result.artist else {
        // "Artist not found" is a normal Last.fm response; it is not a retryable failure.
        debug!("Last.fm has no profile for {artist_name}", artist_name = artist);
        return Ok(None);
    };
    Ok(Some(FmArtist {
        bio: artist.bio.summary,
        tags: artist.tags.tag.into_iter().map(|t| t.name).collect(),
        similar: artist
            .similar
            .artist
            .into_iter()
            .map(|t| t.name)
            .collect(),
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FmSearchResult {
    #[serde(default)]
    artist: Option<SearchResultArtist>,
    #[serde(default)]
    error: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResultArtist {
    name: String,
    url: String,
    image: Vec<Image>,
    streamable: String,
    ontour: String,
    stats: Stats,
    similar: Similar,
    tags: Tags,
    bio: Bio,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bio {
    links: Links,
    published: String,
    summary: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Links {
    link: Link,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    #[serde(rename = "#text")]
    text: String,
    rel: String,
    href: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    #[serde(rename = "#text")]
    text: String,
    size: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Similar {
    artist: Vec<ArtistElement>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtistElement {
    name: String,
    url: String,
    image: Vec<Image>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stats {
    listeners: String,
    playcount: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tags {
    tag: Vec<Tag>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    name: String,
    url: String,
}

#[cfg(test)]
mod tests {
    use super::FmSearchResult;

    #[test]
    fn lastfm_error_document_deserializes_without_an_artist() {
        let body = r#"{"error":6,"message":"Artist not found","error_code":6}"#;
        let parsed: FmSearchResult = serde_json::from_str(body).unwrap();
        assert!(parsed.artist.is_none());
        assert_eq!(parsed.error, Some(6));
    }

    #[test]
    fn amp_in_artist_is_form_encoded() {
        assert_eq!(
            urlencoding::encode("Paul McCartney & Wings"),
            "Paul%20McCartney%20%26%20Wings"
        );
    }
}
