use metaflac::block::PictureType;

use crate::{
    config::Config,
    helpers::split_artists,
    metadata::{AudioMetadata, Picture, StreamInfo},
};

trait IntoStringPictureType {
    fn into_string(self) -> String;
}

impl IntoStringPictureType for PictureType {
    fn into_string(self) -> String {
        match self {
            PictureType::Other => "Other",
            PictureType::Icon => "Icon",
            PictureType::OtherIcon => "Other Icon",
            PictureType::CoverFront => "Cover (Front)",
            PictureType::CoverBack => "Cover (Back)",
            PictureType::Leaflet => "Leaflet",
            PictureType::Media => "Media",
            PictureType::LeadArtist => "Lead Artist",
            PictureType::Artist => "Artist",
            PictureType::Conductor => "Conductor",
            PictureType::Band => "Band",
            PictureType::Composer => "Composer",
            PictureType::Lyricist => "Lyricist",
            PictureType::RecordingLocation => "Recording Location",
            PictureType::DuringRecording => "During Recording",
            PictureType::DuringPerformance => "During Performance",
            PictureType::ScreenCapture => "Screen Capture",
            PictureType::BrightFish => "Bright Fish",
            PictureType::Illustration => "Illustration",
            PictureType::BandLogo => "Band Logo",
            PictureType::PublisherLogo => "Publisher Logo",
        }
        .to_owned()
    }
}

/// Scans an flac file for metadata and returns an `AudioMetadata` struct.
pub async fn scan_flac(path: &std::path::PathBuf, cfg: &Config) -> anyhow::Result<AudioMetadata> {
    let tag = metaflac::Tag::read_from_path(path)
        .map_err(|e| anyhow::anyhow!("failed to read FLAC tags from {:?}: {}", path, e))?;
    let vorbis = tag
        .vorbis_comments()
        .ok_or_else(|| anyhow::anyhow!("no vorbis comments in {:?}", path))?;

    let mut streaminfo_blocks = tag.get_blocks(metaflac::BlockType::StreamInfo);
    let stream_info = match streaminfo_blocks.next() {
        Some(metaflac::Block::StreamInfo(s)) => StreamInfo {
            total_samples: Some(s.total_samples),
            sample_rate: Some(s.sample_rate),
            bits_per_sample: Some(s.bits_per_sample),
            num_channels: Some(s.num_channels),
        },
        _ => anyhow::bail!("failed to read FLAC stream info from {:?}", path),
    };
    let duration = stream_info
        .total_samples
        .zip(stream_info.sample_rate)
        .map(|(total, rate)| (total as u32).saturating_div(rate))
        .unwrap_or(0);

    let year = vorbis.get("DATE").and_then(|d| d[0].parse::<i32>().ok());

    let picture = tag
        .pictures()
        .map(|p| Picture {
            picture_type: p.picture_type.into_string(),
            bytes: p.data.clone(),
        })
        .collect::<Vec<Picture>>();

    // artists is either ARTISTS (semicolon separated) or ARTIST (single but may be split elsewhere)
    let unk_vec = vec!["Unknown".to_string()];
    let split_prep = split_artists(
        vorbis
            .artist()
            .unwrap_or(vorbis.album_artist().unwrap_or(&unk_vec)),
        &cfg.artist_split_exceptions,
    );
    let artists = vorbis.get("ARTISTS").unwrap_or(&split_prep).to_owned();
    let _is_artists_split = artists.len() > 1;

    let mbid_artist = vorbis
        .get("MUSICBRAINZ_ALBUMARTISTID")
        .or_else(|| vorbis.get("MUSICBRAINZ_ARTISTID"))
        .and_then(|v| v.first())
        .map(|v| v.to_owned());

    let mbid_album = vorbis
        .get("MUSICBRAINZ_ALBUMID")
        .and_then(|v| v.first())
        .map(|v| v.to_owned());

    let mbid_track = vorbis
        .get("MUSICBRAINZ_TRACKID")
        .and_then(|v| v.first())
        .map(|v| v.to_owned());

    let metadata = AudioMetadata {
        name: vorbis
            .title()
            .and_then(|v| v.first().cloned())
            .unwrap_or_default(),
        number: vorbis.track().unwrap_or(0),
        duration,
        album: vorbis
            .album()
            .and_then(|v| v.first().cloned())
            .unwrap_or_default(),
        album_artist: vorbis
            .album_artist()
            .and_then(|v| v.first().cloned())
            .or_else(|| vorbis.artist().and_then(|v| v.first().cloned()))
            .unwrap_or_default(),
        album_sort: vorbis
            .get("ALBUMSORT")
            .and_then(|d| d[0].parse::<String>().ok()),
        artists,
        genre: vorbis.genre().map(|v| v.to_owned()),
        picture,
        path: path.to_owned(),
        year,
        lossless: true,
        disc: vorbis
            .get("DISCNUMBER")
            .and_then(|d| d[0].parse::<u32>().ok()),
        sample_rate: stream_info.sample_rate,
        bits_per_sample: stream_info.bits_per_sample,
        num_channels: stream_info.num_channels,
        mbid_artist,
        mbid_album,
        mbid_track,
    };

    Ok(metadata)
}
