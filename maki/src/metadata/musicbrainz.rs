use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use serde::Deserialize;
use std::num::NonZeroU32;
use std::sync::OnceLock;
use tracing::debug;

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

/// MusicBrainz searches use Lucene syntax. Escape user-supplied text so an album title with
/// quotes, colons or parentheses cannot produce a malformed query (and burn all retry attempts).
fn lucene_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 8);
    for c in input.chars() {
        if matches!(c, '\\' | '+' | '-' | '!' | '(' | ')' | '{' | '}' | '[' | ']'
                   | '^' | '"' | '~' | '*' | '?' | ':' | '/' | '&' | '|') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

#[derive(Debug, Deserialize)]
struct SearchResponse<T> {
    #[serde(alias = "artists", alias = "release-groups", alias = "recordings", alias = "releases")]
    items: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct MbArtist {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
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

    let query = format!("artist:{}", lucene_escape(name));
    let url = format!(
        "https://musicbrainz.org/ws/2/artist?query={}&fmt=json",
        urlencoding::encode(&query)
    );

    debug!("Searching MusicBrainz for artist: {}", name);

    let res = client().get(&url).send().await?;

    if !res.status().is_success() {
        anyhow::bail!("MusicBrainz API error: {}", res.status());
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

    if res.status() == reqwest::StatusCode::NOT_FOUND {
        debug!("No cover art found on CAA for MBID: {}", mbid);
        return Ok(None);
    }
    if !res.status().is_success() {
        anyhow::bail!("Cover Art Archive error {} for {}", res.status(), mbid);
    }

    let bytes = res.bytes().await?;
    Ok(Some(bytes.to_vec()))
}

/// Fetch cover art for a release-group MBID (CAA has separate endpoints per entity type).
pub async fn get_release_group_cover_art_bytes(mbid: &str) -> anyhow::Result<Option<Vec<u8>>> {
    limiter().until_ready().await;
    let url = format!("https://coverartarchive.org/release-group/{}/front", urlencoding::encode(mbid));
    let res = client().get(&url).send().await?;
    if res.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !res.status().is_success() {
        anyhow::bail!("Cover Art Archive release-group error {}", res.status());
    }
    Ok(Some(res.bytes().await?.to_vec()))
}

/// Search for a recording (track) by title and artist name and return the best match MBID
pub async fn get_track_mbid(title: &str, artist_name: &str) -> anyhow::Result<Option<String>> {
    limiter().until_ready().await;

    let query = format!("recording:{} AND artist:{}", lucene_escape(title), lucene_escape(artist_name));
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
        anyhow::bail!("MusicBrainz API error: {}", res.status());
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
#[allow(dead_code)] // album-level matching (phase 5) consumes this
pub async fn get_album_info(
    title: &str,
    artist_name: &str,
) -> anyhow::Result<Option<MbReleaseGroup>> {
    limiter().until_ready().await;

    let query = format!("releasegroup:{} AND artist:{}", lucene_escape(title), lucene_escape(artist_name));
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
        anyhow::bail!("MusicBrainz API error: {}", res.status());
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
#[allow(dead_code)] // album-level matching (phase 5) consumes this
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

// ── Phase 4: curated aliases ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct MbAlias {
    pub name: String,
    #[serde(default)]
    pub locale: Option<String>,
    #[serde(default)]
    pub script: Option<String>,
    #[serde(default, rename = "type")]
    pub kind: Option<String>,
    #[serde(default)]
    pub primary: Option<bool>,
}

#[derive(Deserialize)]
struct ArtistAliasesResponse {
    #[serde(default)]
    aliases: Vec<MbAlias>,
}

/// Fetch all curated aliases for an artist. Locale/script come from MusicBrainz; this is where
/// 宇多田ヒカル and Hikaru Utada become one searchable identity without local transliteration.
pub async fn get_artist_aliases(mbid: &str) -> anyhow::Result<Vec<MbAlias>> {
    limiter().until_ready().await;
    let url = format!(
        "https://musicbrainz.org/ws/2/artist/{}?inc=aliases&fmt=json",
        urlencoding::encode(mbid)
    );
    let res = client().get(&url).send().await?;
    if !res.status().is_success() {
        anyhow::bail!("MusicBrainz artist alias lookup failed: {}", res.status());
    }
    let body: ArtistAliasesResponse = res.json().await?;
    Ok(body.aliases)
}

// ── Phase 5: album-level release matching ──────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct MbReleaseCandidate {
    pub id: String,
    pub title: String,
    /// MusicBrainz returns an integer score; f64 accepts it without a second type.
    #[serde(default)]
    pub score: Option<f64>,
    #[serde(default, rename = "release-group")]
    pub release_group: Option<MbReleaseGroup>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MbTrack {
    /// Release-track MBID (not a recording ID); kept for future release-track linking.
    #[allow(dead_code)]
    pub id: String,
    #[serde(default)]
    pub recording: Option<MbRecordingRef>,
    /// Position within the medium; MB restarts this at 1 on each disc.
    pub position: i32,
    pub title: String,
    #[serde(default)]
    pub length: Option<u32>,
    /// 1-based medium number, flattened into the detail by `get_release`; MB's track JSON
    /// does not contain this field, so it must default before the enclosing medium is applied.
    #[serde(default)]
    pub disc: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MbRecordingRef {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct MbReleaseDetail {
    pub id: String,
    pub title: String,
    pub release_group: Option<MbReleaseGroup>,
    pub tracks: Vec<MbTrack>,
}

#[derive(Deserialize)]
struct ReleaseDetailResponse {
    id: String,
    title: String,
    #[serde(rename = "release-group")]
    release_group: Option<MbReleaseGroup>,
    #[serde(default)]
    media: Vec<MbMedium>,
}

#[derive(Deserialize)]
struct MbMedium {
    #[serde(default)]
    position: Option<i32>,
    #[serde(default)]
    tracks: Vec<MbTrack>,
}

/// Search releases for a whole album, keeping MusicBrainz' own score for ranking.
pub async fn search_releases(
    title: &str,
    artist: &str,
    limit: u32,
) -> anyhow::Result<Vec<MbReleaseCandidate>> {
    limiter().until_ready().await;
    let query = format!("release:{} AND artist:{}", lucene_escape(title), lucene_escape(artist));
    let url = format!(
        "https://musicbrainz.org/ws/2/release?query={}&fmt=json&limit={}",
        urlencoding::encode(&query),
        limit
    );
    let res = client().get(&url).send().await?;
    if !res.status().is_success() {
        anyhow::bail!("MusicBrainz release search failed: {}", res.status());
    }
    let body: SearchResponse<MbReleaseCandidate> = res.json().await?;
    Ok(body.items)
}

/// Fetch one release's group and full tracklist with positions and durations.
pub async fn get_release(mbid: &str) -> anyhow::Result<MbReleaseDetail> {
    limiter().until_ready().await;
    let url = format!(
        "https://musicbrainz.org/ws/2/release/{}?inc=recordings+release-groups&fmt=json",
        urlencoding::encode(mbid)
    );
    let res = client().get(&url).send().await?;
    if !res.status().is_success() {
        anyhow::bail!("MusicBrainz release lookup failed: {}", res.status());
    }
    let detail: ReleaseDetailResponse = res.json().await?;
    let tracks = detail
        .media
        .into_iter()
        .enumerate()
        .flat_map(|(medium_index, medium)| {
            let disc = medium.position.unwrap_or(medium_index as i32 + 1);
            medium.tracks.into_iter().map(move |mut track| {
                track.disc = disc;
                track
            })
        })
        .collect();
    Ok(MbReleaseDetail {
        id: detail.id,
        title: detail.title,
        release_group: detail.release_group,
        tracks,
    })
}
