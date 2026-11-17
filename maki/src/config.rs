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

fn default_enrichment_job_timeout_secs() -> u64 {
    900
}

fn default_album_match_min_score() -> f32 {
    0.78
}

fn default_album_match_candidate_floor() -> f32 {
    0.40
}

fn default_album_match_min_margin() -> f32 {
    0.05
}

fn default_max_prune_fraction() -> f64 {
    0.5
}

fn default_enrichment_enabled() -> bool {
    true
}

fn default_enrichment_concurrency() -> u32 {
    1
}

fn default_enrichment_max_attempts() -> u32 {
    5
}

fn default_enrichment_lease_secs() -> u64 {
    300
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
    /// Fetch bios, art and MBIDs after the scan rather than inside it.
    #[serde(default = "default_enrichment_enabled")]
    pub enrichment_enabled: bool,
    #[serde(default = "default_enrichment_concurrency")]
    pub enrichment_concurrency: u32,
    #[serde(default = "default_enrichment_max_attempts")]
    pub enrichment_max_attempts: u32,
    /// A `running` job older than this is reclaimed on startup after a crash.
    #[serde(default = "default_enrichment_lease_secs")]
    pub enrichment_lease_secs: u64,
    /// A handler that exceeds this is failed and retried instead of holding a slot forever.
    #[serde(default = "default_enrichment_job_timeout_secs")]
    pub enrichment_job_timeout_secs: u64,
    /// Refuse a stale prune that would remove more than this fraction of the library.
    #[serde(default = "default_max_prune_fraction")]
    pub max_prune_fraction: f64,
    #[serde(default = "default_album_match_min_score")]
    pub album_match_min_score: f32,
    #[serde(default = "default_album_match_candidate_floor")]
    pub album_match_candidate_floor: f32,
    #[serde(default = "default_album_match_min_margin")]
    pub album_match_min_margin: f32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            artist_split_exceptions: vec!["ラッツ&スター".to_string()],
            hls_profiles: default_hls_profiles(),
            hls_max_cache_bytes: default_hls_max_cache_bytes(),
            audio_analysis_enabled: false,
            audio_analysis_threads: default_audio_analysis_threads(),
            audio_analysis_command: default_audio_analysis_command(),
            mix_analysis_enabled: false,
            mix_analysis_url: None,
            mix_analysis_timeout_seconds: default_mix_analysis_timeout_seconds(),
            mix_analysis_max_pcm_bytes: default_mix_analysis_max_pcm_bytes(),
            enrichment_enabled: true,
            enrichment_concurrency: 1,
            enrichment_max_attempts: 5,
            enrichment_lease_secs: 300,
            enrichment_job_timeout_secs: 900,
            max_prune_fraction: 0.5,
            album_match_min_score: 0.78,
            album_match_candidate_floor: 0.40,
            album_match_min_margin: 0.05,
        }
    }
}

fn create_default_config(path: &str) -> Config {
    let default_config = Config::default();

    let config_json =
        serde_json::to_string_pretty(&default_config).expect("Failed to serialize default config");

    fs::write(path, config_json).expect("Failed to write default config to file");

    default_config
}

pub fn load_or_create_config(path: &str) -> Config {
    let cfg = if Path::new(path).exists() {
        let config_file = fs::read_to_string(path).expect("Unable to read config file");
        serde_json::from_str(&config_file).expect("JSON was not well-formatted")
    } else {
        create_default_config(path)
    };
    cfg.validate();
    cfg
}

impl Config {
    /// Reject values that would silently disable the safety fuses or spin the heartbeat loop.
    fn validate(&self) {
        assert!(
            (0.0..=1.0).contains(&self.max_prune_fraction) && self.max_prune_fraction > 0.0,
            "max_prune_fraction must be in (0, 1]"
        );
        assert!(self.enrichment_lease_secs >= 30, "enrichment_lease_secs must be >= 30");
        assert!(self.enrichment_concurrency >= 1, "enrichment_concurrency must be >= 1");
        assert!(
            (0.0..=1.0).contains(&self.album_match_min_score),
            "album_match_min_score must be in [0, 1]"
        );
        assert!(
            (0.0..=1.0).contains(&self.album_match_min_margin),
            "album_match_min_margin must be in [0, 1]"
        );
        assert!(
            self.album_match_candidate_floor <= self.album_match_min_score,
            "album_match_candidate_floor must be <= album_match_min_score"
        );
        assert!(
            self.enrichment_job_timeout_secs >= 60,
            "enrichment_job_timeout_secs must be >= 60"
        );
    }
}
