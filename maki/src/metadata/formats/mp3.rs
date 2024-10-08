use std::time::Duration;

use crate::{
    config::Config,
    helpers::{sort_string, split_artists},
    metadata::{AudioMetadata, Picture},
};

use id3::{partial_tag_ok, Tag, TagLike};

/// Scans an mp3 file for metadata and returns an `AudioMetadata` struct.
pub async fn scan_mp3(path: &std::path::PathBuf, cfg: &Config) -> anyhow::Result<AudioMetadata> {
    let tag_rs = Tag::read_from_path(path);

    // partial tags are okay
    // assume it has at least a title, number, album, artist.
    let tag = partial_tag_ok(tag_rs)?;

    let genre = match_id3v1_genres(&tag.genres().unwrap_or_default());

    let artists: Vec<String> = tag
        .artists()
        .map(|a| a.into_iter().map(|a| a.to_string()).collect())
        .unwrap_or(split_artists(
            &[tag.artist().unwrap_or_default().to_owned()].to_vec(),
            &cfg.artist_split_exceptions,
        ))
        .into_iter()
        .map(|a| a.to_string())
        .collect();

    dbg!(tag.duration());

    let meta = AudioMetadata {
        name: tag.title().unwrap_or_default().to_string(),
        number: tag.track().unwrap_or(25565),
        duration: tag.duration().unwrap_or_else(|| {
            mp3_duration::from_path(path)
                .unwrap_or(Duration::ZERO)
                .as_secs() as u32
        }),
        album: tag.album().unwrap_or_default().to_string(),
        album_artist: tag.album_artist().unwrap_or_default().to_string(),
        album_sort: sort_string(tag.album()),
        artists,
        genre,
        picture: tag
            .pictures()
            .map(|p| Picture {
                picture_type: p.picture_type.to_string(),
                bytes: p.data.clone(),
            })
            .collect(),
        path: path.to_path_buf(),
        lossless: false,
        sample_rate: None,
        bits_per_sample: None,
        num_channels: None,
        year: tag.year(),
        disc: tag.disc(),
    };

    Ok(meta)
}

const ID3V1_GENRES: [&str; 192] = [
    "Blues",
    "Classic Rock",
    "Country",
    "Dance",
    "Disco",
    "Funk",
    "Grunge",
    "Hip-Hop",
    "Jazz",
    "Metal",
    "New Age",
    "Oldies",
    "Other",
    "Pop",
    "R&B",
    "Rap",
    "Reggae",
    "Rock",
    "Techno",
    "Industrial",
    "Alternative",
    "Ska",
    "Death Metal",
    "Pranks",
    "Soundtrack",
    "Euro-Techno",
    "Ambient",
    "Trip-Hop",
    "Vocal",
    "Jazz+Funk",
    "Fusion",
    "Trance",
    "Classical",
    "Instrumental",
    "Acid",
    "House",
    "Game",
    "Sound Clip",
    "Gospel",
    "Noise",
    "Alternative Rock",
    "Bass",
    "Soul",
    "Punk",
    "Space",
    "Meditative",
    "Instrumental Pop",
    "Instrumental Rock",
    "Ethnic",
    "Gothic",
    "Darkwave",
    "Techno-Industrial",
    "Electronic",
    "Pop-Folk",
    "Eurodance",
    "Dream",
    "Southern Rock",
    "Comedy",
    "Cult",
    "Gangsta",
    "Top 40",
    "Christian Rap",
    "Pop/Funk",
    "Jungle",
    "Native US",
    "Cabaret",
    "New Wave",
    "Psychadelic",
    "Rave",
    "Showtunes",
    "Trailer",
    "Lo-Fi",
    "Tribal",
    "Acid Punk",
    "Acid Jazz",
    "Polka",
    "Retro",
    "Musical",
    "Rock & Roll",
    "Hard Rock",
    "Folk",
    "Folk-Rock",
    "National Folk",
    "Swing",
    "Fast Fusion",
    "Bebop",
    "Latin",
    "Revival",
    "Celtic",
    "Bluegrass",
    "Avantgarde",
    "Gothic Rock",
    "Progressive Rock",
    "Psychedelic Rock",
    "Symphonic Rock",
    "Slow Rock",
    "Big Band",
    "Chorus",
    "Easy Listening",
    "Acoustic",
    "Humour",
    "Speech",
    "Chanson",
    "Opera",
    "Chamber Music",
    "Sonata",
    "Symphony",
    "Booty Bass",
    "Primus",
    "Porn Groove",
    "Satire",
    "Slow Jam",
    "Club",
    "Tango",
    "Samba",
    "Folklore",
    "Ballad",
    "Power Ballad",
    "Rhythmic Soul",
    "Freestyle",
    "Duet",
    "Punk Rock",
    "Drum Solo",
    "A capella",
    "Euro-House",
    "Dance Hall",
    "Goa",
    "Drum & Bass",
    "Club-House",
    "Hardcore",
    "Terror",
    "Indie",
    "BritPop",
    "Negerpunk",
    "Polsk Punk",
    "Beat",
    "Christian Gangsta Rap",
    "Heavy Metal",
    "Black Metal",
    "Crossover",
    "Contemporary Christian",
    "Christian Rock",
    "Merengue",
    "Salsa",
    "Trash Metal",
    "Anime",
    "Jpop",
    "Synthpop",
    "Christmas",
    "Art rock",
    "Baroque",
    "Bhangra",
    "Big beat",
    "Breakbeat",
    "Chillout",
    "Downtempo",
    "Dub",
    "EBM",
    "Eclectic",
    "Electro",
    "Electroclash",
    "Emo",
    "Experimental",
    "Garage",
    "Global",
    "IDM",
    "Illbient",
    "Industro-Goth",
    "Jam Band",
    "Krautrock",
    "Leftfield",
    "Lounge",
    "Math rock",
    "New romantic",
    "Nu-breakz",
    "Post-punk",
    "Post-rock",
    "Psytrance",
    "Shoegaze",
    "Space rock",
    "Trop rock",
    "World music",
    "Neoclassical",
    "Audiobook",
    "Audio theatre",
    "Neue Deutsche Welle",
    "Podcast",
    "Indie rock",
    "G-Funk",
    "Dubstep",
    "Garage rock",
    "Psybient",
];

fn match_id3v1_genres(input: &[&str]) -> Option<Vec<String>> {
    if let Some(index_str) = input.first() {
        if index_str.starts_with('(') && index_str.ends_with(')') {
            if let Ok(index) = index_str[1..index_str.len() - 1].parse::<usize>() {
                if index < ID3V1_GENRES.len() {
                    let genre = ID3V1_GENRES[index].to_string();
                    let name = input.get(1).unwrap_or(&"").to_string();
                    return Some(vec![genre, name]);
                }
            }
        }
    }

    None
}
