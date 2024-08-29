use formats::mp3::scan_mp3;
use tracing::{debug, error};

use crate::metadata::formats::flac::scan_flac;

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
    //match extension string to string options
    match extension.to_lowercase().as_str() {
        "mp3" => Some(AudioFormat::Mp3),
        "wav" => Some(AudioFormat::Wav),
        "aiff" => Some(AudioFormat::Aiff),
        "flac" => Some(AudioFormat::Flac),
        _ => None,
    }
}

pub async fn scan_file(path: &std::path::PathBuf, pool: sqlx::Pool<sqlx::Postgres>, dry_run: bool) {
    let data = match get_filetype(path) {
        Some(AudioFormat::Flac) => scan_flac(path, pool, dry_run).await,
        Some(AudioFormat::Mp3) | Some(AudioFormat::Wav) | Some(AudioFormat::Aiff) => {
            scan_mp3(path, pool, dry_run).await
        }
        None => return,
    };
    match data {
        Ok(data) => debug!("sucessfully scanned {}", data),
        Err(e) => error!("failed to scan {}: {}", path.display(), e),
    }
}
