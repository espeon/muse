use formats::{mp3::scan_mp3, s2hms};
use tracing::{debug, error, info};

use crate::{config::Config, metadata::formats::flac::scan_flac};

// most of this likely stolen from https://github.com/agersant/polaris/blob/master/src/index/metadata.rs
pub mod fm;
pub mod formats;
pub mod spotify;

#[derive(Debug, PartialEq)]
pub enum AudioFormat {
    Flac,
    Mp3,
    Wav,
    Aiff,
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
    pub year: Option<i32>,
    pub disc: Option<u32>,
    // Audio properties
    pub lossless: bool,
    pub sample_rate: Option<u32>,
    pub bits_per_sample: Option<u8>,
    pub num_channels: Option<u8>,
}

pub struct StreamInfo {
    _total_samples: Option<u64>,
    sample_rate: Option<u32>,
    bits_per_sample: Option<u8>,
    /*     bit_rate: Option<i64>,
     */ num_channels: Option<u8>,
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

pub async fn scan_file(
    path: &std::path::PathBuf,
    pool: sqlx::Pool<sqlx::Postgres>,
    dry_run: bool,
    cfg: &Config,
) {
    let m = match get_filetype(path) {
        // Scan files with vorbis tags
        Some(AudioFormat::Flac) => scan_flac(path, cfg).await,
        // Scan files with id3 tags
        Some(AudioFormat::Mp3) => scan_mp3(path, cfg).await,
        // TODO: Scan files with wav tags
        Some(AudioFormat::Wav) | Some(AudioFormat::Aiff) => todo!(),
        None => return,
    };

    let meta = match m {
        Ok(meta) => meta,
        Err(e) => return error!("failed to scan {}: {}", path.display(), e),
    };

    let fmtd = format!(
        "{}. {} by {} ({})",
        meta.number,
        meta.name,
        meta.album_artist,
        s2hms(meta.duration)
    );

    if !dry_run {
        crate::index::db::add_song(meta, pool).await;
    } else {
        info!("dry run: would have added song {}", fmtd);
        debug!("Image count: {}", meta.picture.len());
        debug!(
            "Artist count: {} - {}",
            meta.artists.len(),
            meta.artists.join(", ")
        );
    }

    debug!("sucessfully scanned {}", fmtd);
}
