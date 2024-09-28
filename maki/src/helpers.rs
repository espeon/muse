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

use base64::Engine;
use regex::Regex;
use ring::hmac;
use tracing::{debug, info};
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

#[derive(Debug)]
pub struct HmacMessage {
    pub st: u64,
    pub exp: u64,
    pub acl: String,
    pub uid: String,
}

impl HmacMessage {
    // ?tk=st=1370627194~exp=1370627409~acl=*~hmac=6a10b3f602ffde88c02cd1b89665bfdfdc0fc84c3cf7049752931ad7
    pub fn new(st: u64, exp: u64, acl: String, uid: String) -> Self {
        HmacMessage { st, exp, acl, uid }
    }
    pub fn decode(message: &str) -> Option<Self> {
        let parts: Vec<&str> = message.split('~').collect();
        let st = parts[0].replace("st=", "").parse::<u64>().ok()?;
        let exp = parts[1].replace("exp=", "").parse::<u64>().ok()?;
        let acl = parts[2].replace("acl=", "").to_string();
        let uid = parts[3].replace("uid=", "").to_string();
        Some(HmacMessage::new(st, exp, acl, uid))
    }
    pub fn sign(&self, key: &[u8]) -> String {
        let sig = create_hmac(key, self);
        let encoded_sig = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(sig);
        info!(
            "signed hmac (key:{:?}, message:{:?}, signature:{:?})",
            key,
            self.encode(),
            encoded_sig
        );
        format!("{}~hmac={}", self.encode(), encoded_sig)
    }
    pub fn encode(&self) -> String {
        format!(
            "st={}~exp={}~acl={}~uid={}",
            self.st, self.exp, self.acl, self.uid
        )
    }
}

// hmac decode and encode
pub fn create_hmac(key: &[u8], message: &HmacMessage) -> Vec<u8> {
    let key = hmac::Key::new(hmac::HMAC_SHA384, key);
    let signature = hmac::sign(&key, message.encode().as_bytes());
    info!(
        "created hmac (key:{:?}, message:{:?}, signature:{:?})",
        key,
        message.encode().as_bytes(),
        signature
    );
    signature.as_ref().to_vec()
}

pub fn verify_hmac(key: &[u8], message: &[u8], signature: &[u8]) -> bool {
    let key = hmac::Key::new(hmac::HMAC_SHA384, key);
    // take down key message sig
    info!(
        "verifying hmac (key:{:?}, message:{:?}, signature:{:?})",
        key, message, signature
    );
    hmac::verify(&key, message, signature).is_ok()
}
