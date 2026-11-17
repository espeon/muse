use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::{env, path::PathBuf};

#[derive(Serialize)]
struct AnalysisOutput {
    version: u32,
    vector: Vec<f32>,
}

fn main() -> Result<()> {
    let path = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .context("usage: maki-analyzer <audio-file>")?;

    if env::args_os().nth(2).is_some() {
        bail!("usage: maki-analyzer <audio-file>");
    }

    let analysis = analyze(&path)?;
    let version = u16::from(analysis.features_version) as u32;
    let vector = analysis.as_vec();
    if vector.is_empty() || vector.iter().any(|value| !value.is_finite()) {
        bail!("bliss returned an invalid feature vector");
    }

    println!(
        "{}",
        serde_json::to_string(&AnalysisOutput { version, vector })?
    );
    Ok(())
}

fn analyze(path: &PathBuf) -> Result<bliss_audio::Analysis> {
    use bliss_audio::decoder::symphonia::SymphoniaDecoder;
    use bliss_audio::decoder::Decoder as _;
    use bliss_audio::Song;

    const SAMPLE_RATE: usize = 22_050;
    const WINDOW_SECONDS: usize = 90;

    let decoded = SymphoniaDecoder::decode(path)
        .with_context(|| format!("failed to decode {}", path.display()))?;
    let window_len = WINDOW_SECONDS * SAMPLE_RATE;
    let start = decoded.sample_array.len().saturating_sub(window_len) / 2;
    let end = (start + window_len).min(decoded.sample_array.len());

    Song::analyze(&decoded.sample_array[start..end])
        .with_context(|| format!("failed to analyze {}", path.display()))
}
