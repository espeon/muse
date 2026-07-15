use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use serde::Deserialize;
use std::num::NonZeroU32;
use std::sync::OnceLock;
use tracing::{debug, error};

static USER_AGENT: &str = "Muse/0.1.0 ( contact@muse.moe )";

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
static RATE_LIMITER: OnceLock<DefaultDirectRateLimiter> = OnceLock::new();

fn client() -> &'static reqwest::Client {
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("failed to build reqwest client")
    })
}

fn limiter() -> &'static DefaultDirectRateLimiter {
    RATE_LIMITER.get_or_init(|| RateLimiter::direct(Quota::per_second(NonZeroU32::new(1).unwrap())))
}

#[derive(Debug, Deserialize)]
struct SearchResponse<T> {
    #[serde(alias = "artists", alias = "release-groups", alias = "recordings")]
    items: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct MbArtist {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct MbReleaseGroup {
    pub id: String,
    pub title: String,
    pub disambiguation: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MbRecording {
    pub id: String,
    pub title: String,
}

/// Search for an artist by name and return the best match MBID
pub async fn get_artist_mbid(name: &str) -> anyhow::Result<Option<String>> {
    limiter().until_ready().await;

    let query = format!("artist:{}", name);
    let url = format!(
        "https://musicbrainz.org/ws/2/artist?query={}&fmt=json",
        urlencoding::encode(&query)
    );

    debug!("Searching MusicBrainz for artist: {}", name);

    let res = client().get(&url).send().await?;

    if !res.status().is_success() {
        error!("MusicBrainz API error: {}", res.status());
        return Ok(None);
    }

    let body: SearchResponse<MbArtist> = res.json().await?;

    if let Some(artist) = body.items.first() {
        debug!("Found MusicBrainz match: {} ({})", artist.name, artist.id);
        Ok(Some(artist.id.clone()))
    } else {
        debug!("No MusicBrainz match for artist: {}", name);
        Ok(None)
    }
}

/// Fetch the front cover art bytes for a release from the Cover Art Archive
pub async fn get_cover_art_bytes(mbid: &str) -> anyhow::Result<Option<Vec<u8>>> {
    limiter().until_ready().await;

    let url = format!("https://coverartarchive.org/release/{}/front", mbid);

    debug!("Fetching cover art from CAA for release MBID: {}", mbid);

    let res = client().get(&url).send().await?;

    if !res.status().is_success() {
        debug!("No cover art found on CAA for MBID: {}", mbid);
        return Ok(None);
    }

    let bytes = res.bytes().await?;
    Ok(Some(bytes.to_vec()))
}

/// Search for a recording (track) by title and artist name and return the best match MBID
pub async fn get_track_mbid(title: &str, artist_name: &str) -> anyhow::Result<Option<String>> {
    limiter().until_ready().await;

    let query = format!("recording:{} AND artist:{}", title, artist_name);
    let url = format!(
        "https://musicbrainz.org/ws/2/recording?query={}&fmt=json",
        urlencoding::encode(&query)
    );

    debug!(
        "Searching MusicBrainz for track: {} by {}",
        title, artist_name
    );

    let res = client().get(&url).send().await?;

    if !res.status().is_success() {
        error!("MusicBrainz API error: {}", res.status());
        return Ok(None);
    }

    let body: SearchResponse<MbRecording> = res.json().await?;

    if let Some(recording) = body.items.first() {
        debug!(
            "Found MusicBrainz match: {} ({})",
            recording.title, recording.id
        );
        Ok(Some(recording.id.clone()))
    } else {
        debug!("No MusicBrainz match for track: {}", title);
        Ok(None)
    }
}

/// Search for a release group (album) by title and artist name.
/// Returns the best match including its disambiguation string if present.
pub async fn get_album_info(
    title: &str,
    artist_name: &str,
) -> anyhow::Result<Option<MbReleaseGroup>> {
    limiter().until_ready().await;

    let query = format!("releasegroup:{} AND artist:{}", title, artist_name);
    let url = format!(
        "https://musicbrainz.org/ws/2/release-group?query={}&fmt=json",
        urlencoding::encode(&query)
    );

    debug!(
        "Searching MusicBrainz for album: {} by {}",
        title, artist_name
    );

    let res = client().get(&url).send().await?;

    if !res.status().is_success() {
        error!("MusicBrainz API error: {}", res.status());
        return Ok(None);
    }

    let body: SearchResponse<MbReleaseGroup> = res.json().await?;

    if let Some(rg) = body.items.into_iter().next() {
        debug!(
            "Found MusicBrainz match: {} ({}) disambiguation={:?}",
            rg.title, rg.id, rg.disambiguation
        );
        Ok(Some(rg))
    } else {
        debug!("No MusicBrainz match for album: {}", title);
        Ok(None)
    }
}

/// Fetch release group info (including disambiguation) for a release MBID.
/// File tags store the release MBID (MUSICBRAINZ_ALBUMID), not the release-group MBID,
/// so we look up the release and extract the embedded release-group.
pub async fn get_release_group_info(release_mbid: &str) -> anyhow::Result<Option<MbReleaseGroup>> {
    limiter().until_ready().await;

    #[derive(Deserialize)]
    struct ReleaseResponse {
        #[serde(rename = "release-group")]
        release_group: Option<MbReleaseGroup>,
    }

    let url = format!(
        "https://musicbrainz.org/ws/2/release/{}?inc=release-groups&fmt=json",
        urlencoding::encode(release_mbid)
    );

    debug!("Fetching MusicBrainz release: {}", release_mbid);

    let res = client().get(&url).send().await?;

    if !res.status().is_success() {
        debug!("MusicBrainz release not found for MBID: {}", release_mbid);
        return Ok(None);
    }

    let release: ReleaseResponse = res.json().await?;
    Ok(release.release_group)
}
