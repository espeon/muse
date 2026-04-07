use serde::Deserialize;
use std::sync::OnceLock;
use tracing::debug;

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn client() -> &'static reqwest::Client {
    CLIENT.get_or_init(reqwest::Client::new)
}

fn key() -> String {
    std::env::var("TADB_KEY").unwrap_or_else(|_| "123".to_string())
}

/// Fetch artist image URL from TheAudioDB by MusicBrainz artist ID.
/// Falls back gracefully if TADB_KEY is unset (defaults to free-tier key "123").
pub async fn get_artist_image(mbid: &str) -> anyhow::Result<Option<String>> {
    let url = format!(
        "https://www.theaudiodb.com/api/v1/json/{}/artist-mb.php?i={}",
        key(),
        mbid
    );

    debug!("Fetching TheAudioDB artist image for MBID: {}", mbid);

    let res = client().get(&url).send().await?;

    if !res.status().is_success() {
        return Ok(None);
    }

    let body: ArtistResponse = res.json().await?;

    let img = body
        .artists
        .and_then(|mut v| v.drain(..).next())
        .and_then(|a| a.str_artist_thumb.or(a.str_artist_fanart));

    Ok(img)
}

#[derive(Debug, Deserialize)]
struct ArtistResponse {
    artists: Option<Vec<Artist>>,
}

#[derive(Debug, Deserialize)]
struct Artist {
    #[serde(rename = "strArtistThumb")]
    str_artist_thumb: Option<String>,
    #[serde(rename = "strArtistFanart")]
    str_artist_fanart: Option<String>,
}
