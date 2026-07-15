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

fn default_audio_analysis_threads() -> u32 {
    1
}

fn default_audio_analysis_enabled() -> bool {
    false
}

fn default_audio_analysis_command() -> Vec<String> {
    vec!["maki-analyzer".to_string()]
}

fn default_mix_analysis_timeout_seconds() -> u64 {
    300
}

fn default_mix_analysis_max_pcm_bytes() -> u64 {
    256 * 1024 * 1024
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub artist_split_exceptions: Vec<String>,
    #[serde(default = "default_hls_profiles")]
    pub hls_profiles: Vec<HlsProfile>,
    #[serde(default = "default_hls_max_cache_bytes")]
    pub hls_max_cache_bytes: u64,
    /// Runs after tag scanning; the worker owns the GPL audio-analysis stack.
    #[serde(default = "default_audio_analysis_enabled")]
    pub audio_analysis_enabled: bool,
    /// Maximum simultaneous analyzer worker processes.
    #[serde(default = "default_audio_analysis_threads")]
    pub audio_analysis_threads: u32,
    /// Executable and fixed arguments, without shell parsing. The song path is appended.
    #[serde(default = "default_audio_analysis_command")]
    pub audio_analysis_command: Vec<String>,
    /// Enables the optional Essentia-compatible mix-analysis sidecar.
    #[serde(default)]
    pub mix_analysis_enabled: bool,
    /// Internal sidecar base URL, for example http://mix-analysis:5030.
    #[serde(default)]
    pub mix_analysis_url: Option<String>,
    #[serde(default = "default_mix_analysis_timeout_seconds")]
    pub mix_analysis_timeout_seconds: u64,
    #[serde(default = "default_mix_analysis_max_pcm_bytes")]
    pub mix_analysis_max_pcm_bytes: u64,
}

fn create_default_config(path: &str) -> Config {
    let default_config = Config {
        artist_split_exceptions: vec![
            "ラッツ&スター".to_string(),
        ],
        hls_profiles: default_hls_profiles(),
        hls_max_cache_bytes: default_hls_max_cache_bytes(),
        audio_analysis_enabled: false,
        audio_analysis_threads: default_audio_analysis_threads(),
        audio_analysis_command: default_audio_analysis_command(),
        mix_analysis_enabled: false,
        mix_analysis_url: None,
        mix_analysis_timeout_seconds: default_mix_analysis_timeout_seconds(),
        mix_analysis_max_pcm_bytes: default_mix_analysis_max_pcm_bytes(),
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
