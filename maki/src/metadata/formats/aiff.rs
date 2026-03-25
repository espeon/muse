use std::io::{Read, Seek, SeekFrom};

use crate::{
    config::Config,
    helpers::{sort_string, split_artists},
    metadata::{AudioMetadata, Picture},
};
use id3::{partial_tag_ok, TagLike};

/// Parse AIFF/AIFC COMM chunk to get stream info.
/// Returns (num_channels, num_sample_frames, bits_per_sample, sample_rate_hz).
fn read_aiff_comm(path: &std::path::Path) -> Option<(u16, u32, u16, u32)> {
    let mut f = std::fs::File::open(path).ok()?;
    let mut header = [0u8; 12];
    f.read_exact(&mut header).ok()?;

    // Verify FORM....AIFF or FORM....AIFC
    if &header[0..4] != b"FORM" {
        return None;
    }
    if &header[8..12] != b"AIFF" && &header[8..12] != b"AIFC" {
        return None;
    }

    // Scan chunks until we find COMM
    loop {
        let mut chunk_id = [0u8; 4];
        let mut size_bytes = [0u8; 4];
        if f.read_exact(&mut chunk_id).is_err() {
            break;
        }
        if f.read_exact(&mut size_bytes).is_err() {
            break;
        }
        let chunk_size = i32::from_be_bytes(size_bytes) as u32;

        if &chunk_id == b"COMM" && chunk_size >= 18 {
            let mut comm = vec![0u8; chunk_size as usize];
            f.read_exact(&mut comm).ok()?;

            let num_channels = u16::from_be_bytes([comm[0], comm[1]]);
            let num_sample_frames = u32::from_be_bytes([comm[2], comm[3], comm[4], comm[5]]);
            let sample_size = u16::from_be_bytes([comm[6], comm[7]]);
            // Bytes 8–17: IEEE 754 80-bit extended sample rate
            let sample_rate = extended80_to_u32(&comm[8..18]);
            return Some((num_channels, num_sample_frames, sample_size, sample_rate));
        }

        // Skip this chunk (AIFF chunks are padded to even byte boundaries)
        let skip = if chunk_size % 2 != 0 {
            chunk_size + 1
        } else {
            chunk_size
        };
        if f.seek(SeekFrom::Current(skip as i64)).is_err() {
            break;
        }
    }
    None
}

/// Convert an 80-bit IEEE 754 extended precision float (big-endian) to u32.
/// Used for AIFF COMM chunk sample rate field.
fn extended80_to_u32(bytes: &[u8]) -> u32 {
    if bytes.len() < 10 {
        return 0;
    }
    let exponent = (((bytes[0] as i32 & 0x7f) << 8) | bytes[1] as i32) - 16383;
    let mantissa = u64::from_be_bytes([
        bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8], bytes[9],
    ]);
    let rate = (mantissa as f64) * 2.0_f64.powi(exponent - 63);
    rate as u32
}

/// Scans an AIFF file for metadata. Reads ID3 tags embedded in the AIFF "ID3 " chunk
/// and stream info (sample rate, bit depth, duration) from the AIFF COMM chunk.
pub async fn scan_aiff(path: &std::path::PathBuf, cfg: &Config) -> anyhow::Result<AudioMetadata> {
    let tag = partial_tag_ok(id3::Tag::read_from_path(path))?;

    let (sample_rate, bits_per_sample, num_channels, duration) =
        match read_aiff_comm(path) {
            Some((channels, frames, bits, rate)) => {
                let secs = if rate > 0 { frames / rate } else { 0 };
                (Some(rate), Some(bits as u8), Some(channels as u8), secs)
            }
            None => (None, None, None, tag.duration().unwrap_or(0)),
        };

    let artists: Vec<String> = tag
        .artists()
        .map(|a| a.into_iter().map(|s| s.to_string()).collect())
        .unwrap_or_else(|| {
            split_artists(
                &[tag.artist().unwrap_or_default().to_owned()].to_vec(),
                &cfg.artist_split_exceptions,
            )
        });

    let meta = AudioMetadata {
        name: tag.title().unwrap_or_default().to_string(),
        number: tag.track().unwrap_or(25565),
        duration,
        album: tag.album().unwrap_or_default().to_string(),
        album_artist: tag.album_artist().unwrap_or_default().to_string(),
        album_sort: sort_string(tag.album()),
        artists,
        genre: tag
            .genre()
            .map(|g| vec![g.to_string()])
            .or_else(|| tag.genres().map(|gs| gs.into_iter().map(|s: &str| s.to_string()).collect())),
        picture: tag
            .pictures()
            .map(|p| Picture {
                picture_type: p.picture_type.to_string(),
                bytes: p.data.clone(),
            })
            .collect(),
        path: path.to_path_buf(),
        lossless: true,
        sample_rate,
        bits_per_sample,
        num_channels,
        year: tag.year(),
        disc: tag.disc(),
    };

    Ok(meta)
}
