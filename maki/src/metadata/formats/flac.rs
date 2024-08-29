use metaflac::block::PictureType;
use tracing::debug;

use crate::metadata::{AudioMetadata, Picture, StreamInfo};

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

pub async fn scan_flac(
    path: &std::path::PathBuf,
    pool: sqlx::Pool<sqlx::Postgres>,
    dry_run: bool,
) -> anyhow::Result<String> {
    // read da tag
    let tag = metaflac::Tag::read_from_path(path).unwrap();
    let vorbis = tag.vorbis_comments().ok_or(0).unwrap();
    // calculate the number of secs
    let mut streaminfo = tag.get_blocks(metaflac::BlockType::StreamInfo);
    let duration = match streaminfo.next() {
        Some(metaflac::Block::StreamInfo(s)) => Some(s.total_samples as u32 / s.sample_rate),
        _ => None,
    }
    .unwrap();
    let year = vorbis.get("DATE").and_then(|d| d[0].parse::<i32>().ok());

    let picture = tag
        .pictures()
        .map(|p| Picture {
            picture_type: p.picture_type.into_string(),
            bytes: p.data.clone(),
        })
        .collect::<Vec<Picture>>();

    let mut file_stream_info = tag.get_blocks(metaflac::BlockType::StreamInfo);
    let stream_info = match file_stream_info.next() {
        Some(metaflac::Block::StreamInfo(s)) => StreamInfo {
            _total_samples: Some(s.total_samples),
            sample_rate: Some(s.sample_rate),
            bits_per_sample: Some(s.bits_per_sample),
            num_channels: Some(s.num_channels),
        },
        _ => anyhow::bail!("Failed to read stream info"),
    };

    // artists is either ARTISTS (semicolon separated) or ARTIST (single but may be split elsewhere)
    let artists = match vorbis.get("ARTISTS") {
        Some(e) => Some(e),
        None => None,
    };
    let _is_artists_split = artists.is_some() && artists.unwrap().len() > 1;

    let metadata = AudioMetadata {
        name: vorbis
            .title()
            .map(|v| v[0].clone())
            .expect("Failed to read title"),
        number: vorbis.track().unwrap(),
        duration,
        album: vorbis.album().map(|v| v[0].clone()).unwrap(),
        album_artist: match vorbis.album_artist().map(|v| v[0].clone()) {
            Some(e) => e,
            None => vorbis.artist().map(|v| v[0].clone()).unwrap(),
        },
        album_sort: vorbis
            .get("ALBUMSORT")
            .and_then(|d| d[0].parse::<String>().ok()),
        artists: vorbis
            .get("ARTISTS")
            .unwrap_or(
                vorbis.artist().unwrap_or(
                    vorbis
                        .album_artist()
                        .unwrap_or(&["Unknown".to_owned()].to_vec()),
                ),
            )
            .to_owned(),
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
    };

    // make it pretty~!
    let secs = time::Duration::seconds(duration as i64);
    // like this: min:sec
    let mut formatted_duration = format!(
        "{}:{:02}",
        &secs.whole_minutes(),
        secs.whole_seconds() - (secs.whole_minutes() * 60)
    );
    // hours maybe? nah lol
    if secs.whole_hours() > 0 {
        formatted_duration = format!(
            "{}:{:02}:{:02}",
            secs.whole_hours(),
            &secs.whole_minutes(),
            secs.whole_seconds() - (secs.whole_minutes() * 60)
        )
    }

    let fmtd = format!(
        "{}. {} by {} ({})",
        metadata.number, metadata.name, metadata.album_artist, formatted_duration
    );

    if !dry_run {
        crate::index::db::add_song(metadata, pool).await;
    } else {
        debug!("dry run: would have added song {}", fmtd);
        debug!("Image count: {}", metadata.picture.len());
        debug!(
            "Artist count: {} - {}",
            metadata.artists.len(),
            metadata.artists.join(", ")
        );
    }

    // TODO remove this and replace with a struct containing this and more stuff
    // or keep this and commit to db somewhere else?
    Ok(fmtd)
}
