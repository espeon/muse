use base64::Engine;
use serde::{Deserialize, Serialize};
use std::env;

pub async fn get_artist_image(query: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    // TODO: store key so we dont need to do extra request on every single image request
    let key = authorize_spotify().await?;
    let res: SpotifyArtistResponse = client
        .get(format!(
            "https://api.spotify.com/v1/search?type=artist&q={}",
            query
        ))
        .bearer_auth(key)
        .send()
        .await?
        .json()
        .await?;
    let img = if !res.artists.items.is_empty() {
        if !res.artists.items[0].images.is_empty() {
            res.artists.items[0].images[0].clone().url
        } else {
            "https://http.cat/404.jpg".to_string()
        }
    } else {
        // TODO look somewhere else for this
        "https://http.cat/450.jpg".to_string()
    };
    Ok(img)
}

async fn authorize_spotify() -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let form = [("grant_type", "client_credentials")];

    let hash = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(format!(
        "{}:{}",
        env::var("SPOTIFY_ID").unwrap(),
        env::var("SPOTIFY_SECRET").unwrap()
    ));

    let resp: TokenResponse = client
        .post("https://accounts.spotify.com/api/token".to_string())
        .form(&form)
        .header("Authorization", format!("Basic {}", hash))
        .send()
        .await?
        .json()
        .await?;
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
