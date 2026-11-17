use formats::{aiff::scan_aiff, mp3::scan_mp3, s2hms, wav::scan_wav};
use tracing::{debug, error};

use crate::{config::Config, metadata::formats::flac::scan_flac};

// most of this likely stolen from https://github.com/agersant/polaris/blob/master/src/index/metadata.rs
pub mod deezer;
pub mod fallback;
pub mod fm;
pub mod formats;
pub mod musicbrainz;
pub mod spotify;
pub mod theaudiodb;

#[derive(Debug, PartialEq)]
pub enum AudioFormat {
    Flac,
    Mp3,
    Wav,
    Aiff,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MetadataSource {
    Tag,
    Folder,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AudioMetadata {
    // Basic properties
    pub name: String,
    pub number: u32,
    pub duration: u32,
    pub album: String,
    pub album_artist: String,
    pub album_sort: Option<String>,
    pub artists: Vec<String>,
    pub genre: Option<Vec<String>>,
    pub picture: Vec<Picture>,
    pub path: std::path::PathBuf,
    /// Which source supplied the identity fields (name/album/artist), not the audio properties.
    pub source: MetadataSource,
    pub year: Option<i32>,
    pub disc: Option<u32>,
    // Audio properties
    pub lossless: bool,
    pub sample_rate: Option<u32>,
    pub bits_per_sample: Option<u8>,
    pub num_channels: Option<u8>,
    // MusicBrainz IDs
    pub mbid_artist: Option<String>,
    pub mbid_album: Option<String>,
    pub mbid_track: Option<String>,
    // Extended tags
    pub composer: Option<String>,
    pub isrc: Option<String>,
    pub bpm: Option<u32>,
    pub copyright: Option<String>,
    pub label: Option<String>,
}

/// A file with no ID3 chunk is normal for WAV/AIFF and common for MP3. `partial_tag_ok` only
/// recovers a *partially parsed* tag, so handle `NoTag` explicitly.
pub fn read_id3_or_empty(path: &std::path::Path) -> anyhow::Result<id3::Tag> {
    match id3::Tag::read_from_path(path) {
        Ok(tag) => Ok(tag),
        Err(e) if matches!(e.kind, id3::ErrorKind::NoTag) => Ok(id3::Tag::new()),
        Err(e) => Err(e.into()),
    }
}

pub const UNKNOWN_ARTIST: &str = "Unknown Artist";
pub const UNKNOWN_ALBUM: &str = "Unknown Album";

/// Fill identity fields that tags left empty from the file name and folder name.
/// The original tag strings are never overwritten when they are present.
fn apply_fallbacks(
    mut meta: AudioMetadata,
    path: &std::path::Path,
    album_folder: Option<&std::path::Path>,
) -> AudioMetadata {
    let track = fallback::from_path(path);
    // Folder hints come from the resolved album folder, not the file's immediate parent:
    // a bare `Disc 2` folder must not become the album name.
    let folder_name = album_folder
        .or_else(|| path.parent())
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let folder = fallback::parse_folder_name(&folder_name);

    let mut inferred = false;
    if meta.name.trim().is_empty() {
        meta.name = track.title;
        inferred = true;
    }
    if meta.album.trim().is_empty() {
        meta.album = folder
            .album
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| UNKNOWN_ALBUM.to_string());
        inferred = true;
    }
    if meta.album_artist.trim().is_empty() {
        meta.album_artist = folder
            .artist
            .filter(|s| !s.trim().is_empty())
            .or_else(|| {
                meta.artists
                    .first()
                    .cloned()
                    .filter(|s| !s.trim().is_empty())
            })
            .unwrap_or_else(|| UNKNOWN_ARTIST.to_string());
        inferred = true;
    }

    meta.artists.retain(|a| !a.trim().is_empty());
    if meta.artists.is_empty() {
        meta.artists = vec![meta.album_artist.clone()];
        inferred = true;
    }
    if meta.number == 0 {
        meta.number = track.number.unwrap_or(0);
    }
    if meta.year.is_none() {
        meta.year = folder.year;
    }
    if inferred {
        meta.source = MetadataSource::Folder;
    }
    meta
}

pub struct StreamInfo {
    pub total_samples: Option<u64>,
    pub sample_rate: Option<u32>,
    pub bits_per_sample: Option<u8>,
    pub num_channels: Option<u8>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Picture {
    pub picture_type: String,
    pub bytes: Vec<u8>,
}

pub fn get_filetype(path: &std::path::Path) -> Option<AudioFormat> {
    // get extension
    let extension = match path.extension() {
        Some(e) => e,
        _ => return None,
    };
    // format to string so we can match easily
    let extension = match extension.to_str() {
        Some(e) => e,
        _ => return None,
    };
    // match extension string to string options
    // TODO: add more formats and actual format detection past file extension
    match extension.to_lowercase().as_str() {
        "mp3" => Some(AudioFormat::Mp3),
        "wav" => Some(AudioFormat::Wav),
        "aiff" => Some(AudioFormat::Aiff),
        "flac" => Some(AudioFormat::Flac),
        _ => None,
    }
}

/// Extract metadata only. Writing rows is the caller's job, so a full album folder can be
/// read before any of it is committed.
pub async fn scan_file(
    path: &std::path::PathBuf,
    cfg: &Config,
    album_folder: Option<&std::path::Path>,
) -> anyhow::Result<Option<AudioMetadata>> {
    let m = match get_filetype(path) {
        // Scan files with vorbis tags
        Some(AudioFormat::Flac) => scan_flac(path, cfg).await,
        // Scan files with id3 tags
        Some(AudioFormat::Mp3) => scan_mp3(path, cfg).await,
        Some(AudioFormat::Wav) => scan_wav(path, cfg).await,
        Some(AudioFormat::Aiff) => scan_aiff(path, cfg).await,
        None => return Ok(None),
    };

    let meta = match m {
        Ok(meta) => meta,
        Err(e) => {
            error!("failed to scan {}: {}", path.display(), e);
            return Err(e);
        }
    };

    let meta = apply_fallbacks(meta, path, album_folder);

    debug!(
        "scanned {}. {} by {} ({})",
        meta.number,
        meta.name,
        meta.album_artist,
        s2hms(meta.duration)
    );
    Ok(Some(meta))
}
