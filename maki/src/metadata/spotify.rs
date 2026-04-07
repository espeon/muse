use base64::Engine;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
static TOKEN_CACHE: OnceLock<Mutex<Option<CachedToken>>> = OnceLock::new();

fn client() -> &'static reqwest::Client {
    CLIENT.get_or_init(reqwest::Client::new)
}

fn token_cache() -> &'static Mutex<Option<CachedToken>> {
    TOKEN_CACHE.get_or_init(|| Mutex::new(None))
}

struct CachedToken {
    token: String,
    expires_at: Instant,
}

pub async fn get_artist_image(query: &str) -> anyhow::Result<Option<String>> {
    let key = match authorize_spotify().await {
        Ok(k) => k,
        Err(_) => return Ok(None),
    };
    let res: SpotifyArtistResponse = client()
        .get(format!(
            "https://api.spotify.com/v1/search?type=artist&q={}",
            query
        ))
        .bearer_auth(key)
        .send()
        .await?
        .json()
        .await?;
    let img = if !res.artists.items.is_empty() && !res.artists.items[0].images.is_empty() {
        Some(res.artists.items[0].images[0].clone().url)
    } else {
        None
    };
    Ok(img)
}

async fn authorize_spotify() -> anyhow::Result<String> {
    let mut cache = token_cache().lock().await;

    if let Some(ref cached) = *cache {
        if cached.expires_at > Instant::now() {
            return Ok(cached.token.clone());
        }
    }

    let id = env::var("SPOTIFY_ID")
        .map_err(|_| anyhow::anyhow!("SPOTIFY_ID not set, skipping Spotify lookup"))?;
    let secret = env::var("SPOTIFY_SECRET")
        .map_err(|_| anyhow::anyhow!("SPOTIFY_SECRET not set, skipping Spotify lookup"))?;

    let form = [("grant_type", "client_credentials")];
    let hash = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(format!("{}:{}", id, secret));

    let resp: TokenResponse = client()
        .post("https://accounts.spotify.com/api/token")
        .form(&form)
        .header("Authorization", format!("Basic {}", hash))
        .send()
        .await?
        .json()
        .await?;

    // Expire 60s early to avoid using a token right at the edge
    let expires_at = Instant::now() + Duration::from_secs(resp.expires_in as u64 - 60);
    *cache = Some(CachedToken {
        token: resp.access_token.clone(),
        expires_at,
    });

    Ok(resp.access_token)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpotifyArtistResponse {
    artists: Artists,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Artists {
    href: String,
    items: Vec<Item>,
    limit: i64,
    next: Option<serde_json::Value>,
    offset: i64,
    previous: Option<serde_json::Value>,
    total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    external_urls: ExternalUrls,
    genres: Vec<Option<serde_json::Value>>,
    href: String,
    id: String,
    images: Vec<Image>,
    name: String,
    popularity: i64,
    #[serde(rename = "type")]
    item_type: String,
    uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalUrls {
    spotify: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Image {
    height: i64,
    url: String,
    width: i64,
}
