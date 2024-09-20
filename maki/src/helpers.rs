// use serde::{de, Deserialize, Deserializer};
// use std::{fmt, str::FromStr};

// / Serde deserialization decorator to map empty Strings to None
// / Stolen from https://github.com/tokio-rs/axum/blob/main/examples/query-params-with-empty-strings/src/main.rs
// pub fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
// where
//     D: Deserializer<'de>,
//     T: FromStr,
//     T::Err: fmt::Display,
// {
//     let opt = Option::<String>::deserialize(de)?;
//     match opt.as_deref() {
//         None | Some("") => Ok(None),
//         Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
//     }
// }

use regex::Regex;
use tracing::debug;
use unicode_normalization::UnicodeNormalization;

const COMMON_PREFIXES: [&str; 20] = [
    "the ", "a ", "an ", "la ", "le ", "les ", "el ", "los ", "las ", "l'", "das ", "der ", "die ",
    "een ", "de ", "den ", "det ", "het ", "ein ", "eine ",
];

pub fn sort_string(orig: Option<&str>) -> Option<String> {
    let orig = orig?;
    let mut sort_string = orig.to_owned();
    // Normalize characters
    sort_string = sort_string.nfkd().collect::<String>();
    // lowercase
    sort_string = sort_string.to_lowercase();

    for prefix in COMMON_PREFIXES {
        if sort_string.starts_with(prefix) {
            sort_string = sort_string.trim_start_matches(prefix).to_string() + ", " + prefix.trim();
        }
    }

    Some(sort_string)
}

pub fn split_artists(a: &Vec<String>, exceptions: &[String]) -> Vec<String> {
    debug!(target: "split-artists", "splitting artists: {:?}", a);
    let vec = a
        .iter()
        .flat_map(|artist| {
            // Check if the artist is in the exceptions list
            if exceptions.contains(artist) {
                return vec![artist.clone()];
            }
            // This regular expression matches on the following patterns:
            //
            // - ' feat.', ' feat ', ' feat.', ' feat ', ' ft.', ' ft ', ' ft.', ' ft ', etc.
            // - ' with', ' x', etc.
            //  It doesn't match on '&' and 'and' because those are often used in artist names.
            //  Note that there needs to be a space before and after the pattern to match.
            //  So, 'with' would not match but ' with' would. 'Quinn XCII' would not be matched :)
            //
            // The `(?: )` is a non-capturing group, which means that the
            // parentheses aren't included in the match. The `(?i)` makes the
            // match case-insensitive, so 'FEAT' is matched just like 'feat'.
            //
            // The point of this regex is to split artists by the above
            // patterns, so that "The Beatles feat. Billy Preston" is split
            // into two artists: "The Beatles" and "Billy Preston".
            Regex::new(r" (?i)(?:feat(?:\.|uring)?|ft(?:\.|)|with|x) ")
                .unwrap()
                .split(artist)
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>()
        })
        .collect::<Vec<String>>();
    debug!("split artists: {:?}", vec);
    vec
}
