use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone)]
pub struct FmArtist {
    pub bio: String,
    pub tags: Vec<String>,
    pub similar: Vec<String>,
}
pub async fn get_artist_info(artist: &str) -> anyhow::Result<FmArtist> {
    let result: FmSearchResult = reqwest::get(&format!(
        "http://ws.audioscrobbler.com/2.0/?method=artist.getinfo&artist={}&api_key={}&format=json",
        artist,
        std::env::var("FM_KEY").unwrap()
    ))
    .await?
    .json()
    .await?;
    Ok(FmArtist {
        bio: result.artist.bio.summary,
        tags: result
            .artist
            .tags
            .tag
            .into_iter()
            .map(|t| t.name)
            .collect::<Vec<_>>(),
        similar: result
            .artist
            .similar
            .artist
            .into_iter()
            .map(|t| t.name)
            .collect::<Vec<_>>(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FmSearchResult {
    artist: SearchResultArtist,
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
