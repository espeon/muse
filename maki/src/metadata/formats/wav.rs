use crate::{
    config::Config,
    helpers::{sort_string, split_artists},
    metadata::{AudioMetadata, Picture},
};
use id3::{partial_tag_ok, TagLike};

/// Scans a WAV file for metadata. Reads ID3 tags embedded in the WAV "id3 " chunk
/// and stream info (sample rate, bit depth, duration) from the WAV fmt header via hound.
pub async fn scan_wav(path: &std::path::PathBuf, cfg: &Config) -> anyhow::Result<AudioMetadata> {
    let tag = partial_tag_ok(id3::Tag::read_from_path(path))?;

    let (sample_rate, bits_per_sample, num_channels, duration) =
        match hound::WavReader::open(path) {
            Ok(reader) => {
                let spec = reader.spec();
                // duration() returns total sample frames (samples / channels)
                let frames = reader.duration();
                let secs = if spec.sample_rate > 0 {
                    frames / spec.sample_rate
                } else {
                    0
                };
                (
                    Some(spec.sample_rate),
                    Some(spec.bits_per_sample as u8),
                    Some(spec.channels as u8),
                    secs,
                )
            }
            Err(_) => (None, None, None, tag.duration().unwrap_or(0)),
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
