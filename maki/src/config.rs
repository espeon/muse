use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HlsProfile {
    pub name: String,
    /// ffmpeg encoder name: "aac" or "flac"
    pub codec: String,
    /// Bitrate in bps (None for lossless)
    pub bitrate: Option<u32>,
}

fn default_hls_profiles() -> Vec<HlsProfile> {
    vec![
        HlsProfile { name: "ultralow".into(), codec: "aac".into(),  bitrate: Some(32_000) },
        HlsProfile { name: "low".into(),      codec: "aac".into(),  bitrate: Some(128_000) },
        HlsProfile { name: "medium".into(),   codec: "aac".into(),  bitrate: Some(256_000) },
        HlsProfile { name: "high".into(),     codec: "aac".into(),  bitrate: Some(320_000) },
        HlsProfile { name: "lossless".into(), codec: "flac".into(), bitrate: None },
    ]
}

fn default_hls_max_cache_bytes() -> u64 {
    5 * 1024 * 1024 * 1024 // 5 GB
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub artist_split_exceptions: Vec<String>,
    #[serde(default = "default_hls_profiles")]
    pub hls_profiles: Vec<HlsProfile>,
    #[serde(default = "default_hls_max_cache_bytes")]
    pub hls_max_cache_bytes: u64,
}

fn create_default_config(path: &str) -> Config {
    let default_config = Config {
        artist_split_exceptions: vec![
            "ラッツ&スター".to_string(),
        ],
        hls_profiles: default_hls_profiles(),
        hls_max_cache_bytes: default_hls_max_cache_bytes(),
    };

    let config_json =
        serde_json::to_string_pretty(&default_config).expect("Failed to serialize default config");

    fs::write(path, config_json).expect("Failed to write default config to file");

    default_config
}

pub fn load_or_create_config(path: &str) -> Config {
    if Path::new(path).exists() {
        let config_file = fs::read_to_string(path).expect("Unable to read config file");
        serde_json::from_str(&config_file).expect("JSON was not well-formatted")
    } else {
        create_default_config(path)
    }
}
