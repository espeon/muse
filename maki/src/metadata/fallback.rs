//! Metadata hints derived from file and folder names when tags are missing.
//!
//! All matching is done with plain string operations: no regex and no
//! transliteration. Non-ASCII names are preserved as written.

use std::path::Path;

/// Hints for a single track, derived from its file name and immediate parent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackHints {
    pub number: Option<u32>,
    pub title: String,
    pub disc: Option<u32>,
}

/// Hints for an album folder, derived from the folder name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderHints {
    pub artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<i32>,
}

/// Derive track hints from a path's file stem and immediate parent directory.
pub fn from_path(path: &Path) -> TrackHints {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();

    // Normalising is one character to one character, so character indices in
    // the normalised stem also index the original (verbatim) stem.
    let original: Vec<char> = stem.chars().collect();
    let normalized: Vec<char> = original.iter().map(|&c| normalize_char(c)).collect();

    let disc = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| disc_from_text(&n.to_string_lossy()));

    if let Some((number, start)) = split_leading_number(&normalized) {
        let title: String = original[start..].iter().collect();
        let title = title.trim();
        if !title.is_empty() {
            return TrackHints {
                number: Some(number),
                title: title.to_string(),
                disc,
            };
        }
    }

    TrackHints {
        number: None,
        title: stem,
        disc,
    }
}

/// Conservatively derive artist, album and year from an album folder name.
pub fn parse_folder_name(name: &str) -> FolderHints {
    let mut remaining = name.trim().to_string();
    let mut year = None;

    // A leading "[2022] Album" style year.
    if let Some((inner, rest)) = leading_group(&remaining) {
        if let Some(y) = parse_year(inner) {
            year = Some(y);
            remaining = rest.trim().to_string();
        }
    }

    // Strip trailing disc markers, years and format tags until none remain.
    loop {
        if let Some(prefix) = strip_disc_suffix(&remaining) {
            if !prefix.is_empty() {
                remaining = prefix;
                continue;
            }
        }
        if let Some((prefix, inner)) = trailing_group(&remaining) {
            let handled = match parse_year(inner) {
                Some(y) => {
                    if year.is_none() {
                        year = Some(y);
                    }
                    true
                }
                None => is_format_tag(inner),
            };
            if handled {
                remaining = prefix.to_string();
                continue;
            }
        }
        break;
    }

    let (artist, album) = match remaining.split_once(" - ") {
        Some((artist, album))
            if !artist.trim().is_empty()
                && !album.trim().is_empty()
                && parse_year(artist).is_none() =>
        {
            (Some(artist.trim().to_string()), album.trim().to_string())
        }
        _ => (None, remaining),
    };

    FolderHints {
        artist,
        album: Some(album),
        year,
    }
}

/// Map full-width digits and separators to ASCII for matching purposes only.
fn normalize_char(c: char) -> char {
    match c {
        '０'..='９' => char::from_digit(c as u32 - '０' as u32, 10).unwrap_or(c),
        '－' => '-',
        '．' => '.',
        '＿' => '_',
        '　' => ' ',
        '（' => '(',
        '）' => ')',
        '［' => '[',
        '］' => ']',
        _ => c,
    }
}

/// Return the leading track number and the character index where the title starts.
fn split_leading_number(chars: &[char]) -> Option<(u32, usize)> {
    let mut i = 0;
    while chars.get(i).map_or(false, |c| c.is_whitespace()) {
        i += 1;
    }

    let bracketed = matches!(chars.get(i).copied(), Some('[' | '('));
    if bracketed {
        i += 1;
    }

    let digits_start = i;
    while chars.get(i).map_or(false, |c| c.is_ascii_digit()) {
        i += 1;
    }
    if i == digits_start {
        return None;
    }
    let digits: String = chars[digits_start..i].iter().collect();
    let number = digits.parse::<u32>().ok()?;

    if bracketed {
        if !matches!(chars.get(i).copied(), Some(']' | ')')) {
            return None;
        }
        i += 1;
    }

    // A bare number needs a separator (whitespace alone counts). Consume any run of
    // whitespace/punctuation so "01 - BAD", "01. BAD" and "01_BAD" all start at "BAD".
    let mut saw_separator = false;
    while let Some(&c) = chars.get(i) {
        if c.is_whitespace() || matches!(c, '-' | '.' | '_') {
            saw_separator = true;
            i += 1;
        } else {
            break;
        }
    }
    if !saw_separator || i >= chars.len() {
        return None;
    }
    Some((number, i))
}

/// Parse the number in "disc 2", "cd01", "disk 3", etc.
fn disc_number(text: &str) -> Option<u32> {
    let lower = text.trim().to_ascii_lowercase();
    for keyword in &["disc", "cd", "disk"] {
        if let Some(rest) = lower.strip_prefix(keyword) {
            let rest = rest.trim_start();
            if !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()) {
                return rest.parse::<u32>().ok();
            }
        }
    }
    None
}

/// Find a disc number in a directory name, including "Album (Disc 2)".
fn disc_from_text(text: &str) -> Option<u32> {
    let mut current = text;
    loop {
        if let Some(n) = disc_number(current) {
            return Some(n);
        }
        match trailing_group(current) {
            Some((prefix, inner)) => {
                if let Some(n) = disc_number(inner) {
                    return Some(n);
                }
                current = prefix;
            }
            None => return None,
        }
    }
}

/// Strip a trailing "(Disc 2)" / "[CD1]" suffix, returning the remaining name.
fn strip_disc_suffix(name: &str) -> Option<String> {
    let (prefix, inner) = trailing_group(name)?;
    if !prefix.is_empty() && disc_number(inner).is_some() {
        Some(prefix.to_string())
    } else {
        None
    }
}

/// Recognise a disc subfolder: `(album name override, disc number)`. A folder named
/// `Disc 1` has no album name (the album is its parent); `Album (Disc 2)` has one.
pub fn disc_folder(name: &str) -> Option<(Option<String>, u32)> {
    let disc = disc_from_text(name)?;
    let album = strip_disc_suffix(name).filter(|s| !s.is_empty());
    Some((album, disc))
}

/// Split a trailing "(...)"/"[...]" group into (prefix, inner).
fn trailing_group(s: &str) -> Option<(&str, &str)> {
    let s = s.trim_end();
    let close = s.chars().last()?;
    if close != ')' && close != ']' {
        return None;
    }
    let before = &s[..s.len() - close.len_utf8()];
    let start = before.rfind('(').max(before.rfind('['))?;
    let open = before[start..].chars().next()?;
    Some((
        before[..start].trim_end(),
        &before[start + open.len_utf8()..],
    ))
}

/// Split a leading "(...)"/"[...]" group into (inner, rest).
fn leading_group(s: &str) -> Option<(&str, &str)> {
    let open = s.chars().next()?;
    let close = match open {
        '(' => ')',
        '[' => ']',
        _ => return None,
    };
    let content_start = open.len_utf8();
    let end = s[content_start..].find(close)? + content_start;
    Some((
        &s[content_start..end],
        s[end + close.len_utf8()..].trim_start(),
    ))
}

fn parse_year(s: &str) -> Option<i32> {
    let s = s.trim();
    if s.len() != 4 || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let year = s.parse::<i32>().ok()?;
    if (1000..=2999).contains(&year) {
        Some(year)
    } else {
        None
    }
}

fn is_format_tag(s: &str) -> bool {
    let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    matches!(
        compact.to_ascii_lowercase().as_str(),
        "flac" | "mp3" | "wav" | "aiff" | "320kbps" | "lossless"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_numbers_and_titles() {
        let cases: &[(&str, Option<u32>, &str)] = &[
            ("01 - BADモード.flac", Some(1), "BADモード"),
            ("01. BADモード.flac", Some(1), "BADモード"),
            ("01_BADモード.flac", Some(1), "BADモード"),
            ("01 BADモード.flac", Some(1), "BADモード"),
            ("[01] BADモード.flac", Some(1), "BADモード"),
            ("1 BADモード.flac", Some(1), "BADモード"),
            ("001 - 七里香.mp3", Some(1), "七里香"),
            ("12 - 周杰倫.flac", Some(12), "周杰倫"),
            ("０１ － ＢＡＤモード.flac", Some(1), "ＢＡＤモード"),
            ("０１．ＢＡＤモード.flac", Some(1), "ＢＡＤモード"),
            ("０１＿周杰倫.flac", Some(1), "周杰倫"),
            ("(02) 七里香.flac", Some(2), "七里香"),
        ];
        for &(path, number, title) in cases {
            let hints = from_path(Path::new(path));
            assert_eq!(hints.number, number, "number for {}", path);
            assert_eq!(hints.title, title, "title for {}", path);
        }
    }

    #[test]
    fn track_negative_cases_and_empty_title_fallback() {
        let cases: &[(&str, &str)] = &[
            ("BADモード1.flac", "BADモード1"),
            ("track01.mp3", "track01"),
            ("宇多田ヒカル - BADモード.flac", "宇多田ヒカル - BADモード"),
            ("01.flac", "01"),
            ("01 - .flac", "01 - "),
            ("01_.flac", "01_"),
            ("01BADモード.flac", "01BADモード"),
            ("1a - BADモード.flac", "1a - BADモード"),
        ];
        for &(path, title) in cases {
            let hints = from_path(Path::new(path));
            assert_eq!(hints.number, None, "number for {}", path);
            assert_eq!(hints.title, title, "title for {}", path);
        }
    }

    #[test]
    fn disc_comes_only_from_immediate_parent() {
        let cases: &[(&str, Option<u32>)] = &[
            ("/lib/Artist/Album/Disc 1/01 - track.flac", Some(1)),
            ("/lib/Artist/Album/disc01/01 - track.flac", Some(1)),
            ("/lib/Artist/Album/CD 2/01 - track.flac", Some(2)),
            ("/lib/Artist/Album/cd02/01 - track.flac", Some(2)),
            ("/lib/Artist/Album/disk03/01 - track.flac", Some(3)),
            ("/lib/Artist/Album/Disc 002/01 - track.flac", Some(2)),
            ("/lib/Artist/Album/BAD MODE (Disc 2)/01.flac", Some(2)),
            ("/lib/Artist/Album/七里香 [CD1]/01.flac", Some(1)),
            ("/lib/Artist/Album/Edition [FLAC] (Disc 3)/01.flac", Some(3)),
            ("/lib/Disc 1/Album/01 - track.flac", None),
            ("/lib/Album/Discography/01 - track.flac", None),
            ("/lib/Album/CD-ROM/01 - track.flac", None),
            ("/lib/Album/disc/01 - track.flac", None),
            ("/lib/Album/Disc 1 extra/01 - track.flac", None),
            ("/lib/Album/Disc 1 (2022)/01 - track.flac", Some(1)),
            ("01 - track.flac", None),
        ];
        for &(path, disc) in cases {
            assert_eq!(from_path(Path::new(path)).disc, disc, "disc for {}", path);
        }
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_components_do_not_panic() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        use std::path::PathBuf;

        let mut path = PathBuf::from(OsStr::from_bytes(b"/library/\xff/Album"));
        path.push("０１ － BADモード.flac");
        let hints = from_path(&path);
        assert_eq!(hints.number, Some(1));
        assert_eq!(hints.title, "BADモード");
        assert_eq!(hints.disc, None);

        let mut path = PathBuf::from(OsStr::from_bytes(b"/library/Disc \xff1"));
        path.push("01 - track.flac");
        assert_eq!(from_path(&path).disc, None);
    }

    #[test]
    fn folder_artist_album_and_year() {
        let cases: &[(&str, Option<&str>, Option<&str>, Option<i32>)] = &[
            (
                "宇多田ヒカル - BADモード",
                Some("宇多田ヒカル"),
                Some("BADモード"),
                None,
            ),
            ("周杰倫 - 七里香", Some("周杰倫"), Some("七里香"), None),
            ("Artist - Album", Some("Artist"), Some("Album"), None),
            (
                "Artist - Album (2022)",
                Some("Artist"),
                Some("Album"),
                Some(2022),
            ),
            (
                "Artist - Album [2022]",
                Some("Artist"),
                Some("Album"),
                Some(2022),
            ),
            ("[2022] Album", None, Some("Album"), Some(2022)),
            (
                "[2022] Artist - Album",
                Some("Artist"),
                Some("Album"),
                Some(2022),
            ),
            ("Album (2022)", None, Some("Album"), Some(2022)),
            ("Album [2022]", None, Some("Album"), Some(2022)),
            ("Album [FLAC] (2022)", None, Some("Album"), Some(2022)),
            ("2001 - Album", None, Some("2001 - Album"), None),
        ];
        for &(name, artist, album, year) in cases {
            let hints = parse_folder_name(name);
            assert_eq!(hints.artist.as_deref(), artist, "artist for {}", name);
            assert_eq!(hints.album.as_deref(), album, "album for {}", name);
            assert_eq!(hints.year, year, "year for {}", name);
        }
    }

    #[test]
    fn folder_format_tags_and_disc_suffixes() {
        let cases: &[(&str, Option<&str>, &str, Option<i32>)] = &[
            ("Album [FLAC]", None, "Album", None),
            ("Album [flac]", None, "Album", None),
            ("Album [MP3]", None, "Album", None),
            ("Album [WAV]", None, "Album", None),
            ("Album [AIFF]", None, "Album", None),
            ("Album [320kbps]", None, "Album", None),
            ("Album [320 kbps]", None, "Album", None),
            ("Album [Lossless]", None, "Album", None),
            ("BAD MODE (Disc 2)", None, "BAD MODE", None),
            ("BAD MODE (disc02)", None, "BAD MODE", None),
            ("七里香 [CD1]", None, "七里香", None),
            ("BAD MODE (Disc 2) [FLAC]", None, "BAD MODE", None),
            (
                "Artist - BAD MODE (Disc 2)",
                Some("Artist"),
                "BAD MODE",
                None,
            ),
            (
                "Artist - 七里香 [FLAC] (2022)",
                Some("Artist"),
                "七里香",
                Some(2022),
            ),
        ];
        for &(name, artist, album, year) in cases {
            let hints = parse_folder_name(name);
            assert_eq!(hints.artist.as_deref(), artist, "artist for {}", name);
            assert_eq!(hints.album.as_deref(), Some(album), "album for {}", name);
            assert_eq!(hints.year, year, "year for {}", name);
        }
    }

    #[test]
    fn folder_conservative_negative_cases() {
        let cases: &[&str] = &[
            "No Separator Here",
            "宇多田ヒカル",
            "artist-album",
            " - Album",
            "Artist -",
            "Album (202)",
            "Album (20222)",
            "Album (not a year)",
            "Album [OGG]",
            "Discography",
            "CD-ROM",
        ];
        for &name in cases {
            let hints = parse_folder_name(name);
            assert_eq!(hints.artist, None, "artist for {}", name);
            assert_eq!(hints.album.as_deref(), Some(name.trim()), "album for {}", name);
            assert_eq!(hints.year, None, "year for {}", name);
        }
    }
}
