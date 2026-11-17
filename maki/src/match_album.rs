//! Pure scoring for album-level MusicBrainz matching. No I/O here so the ranking can be tested
//! with synthetic tracklists.

use crate::metadata::musicbrainz::MbReleaseDetail;

#[derive(Debug, Clone)]
pub struct LocalTrack {
    pub disc: Option<u32>,
    pub position: i32,
    pub duration: u32,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct MatchInput {
    pub title: String,
    pub artist: String,
    pub tracks: Vec<LocalTrack>,
}

#[derive(Debug, Clone)]
pub struct AlbumScore {
    pub value: f32,
    pub median_duration_delta_ms: u32,
    pub track_count_delta: i32,
    /// How many paired tracks had usable durations on both sides. Automatic application needs
    /// enough duration evidence; a title match alone must not identify a release.
    pub comparable_tracks: usize,
}

/// Median absolute per-position duration difference, in milliseconds. The median (not the mean)
/// keeps one bonus track or one truncated file from dominating an otherwise exact match.
fn track_key(disc: Option<u32>, position: i32) -> (i32, i32) {
    (disc.unwrap_or(1) as i32, position)
}

/// Pair tracks by `(disc, position)` rather than ordinal: a missing track in the middle must not
/// shift every later comparison. Returns `(median_delta_ms, comparable_count)`.
fn median_duration_delta(local: &[LocalTrack], release: &MbReleaseDetail) -> (u32, usize) {
    let remote: std::collections::HashMap<(i32, i32), u32> = release
        .tracks
        .iter()
        .filter_map(|t| t.length.filter(|len| *len > 0).map(|len| ((t.disc, t.position), len)))
        .collect();
    let mut deltas: Vec<u32> = local
        .iter()
        .filter_map(|l| {
            if l.duration == 0 {
                return None;
            }
            remote
                .get(&track_key(l.disc, l.position))
                .map(|len| l.duration.abs_diff(*len))
        })
        .collect();
    let comparable = deltas.len();
    if deltas.is_empty() {
        return (0, 0);
    }
    deltas.sort_unstable();
    (deltas[deltas.len() / 2], comparable)
}

fn title_similarity(local: &str, remote: &str) -> f32 {
    let a = crate::helpers::normalize::search_key(local);
    let b = crate::helpers::normalize::search_key(remote);
    if a == b {
        return 1.0;
    }
    let words_a: Vec<&str> = a.split_whitespace().collect();
    let words_b: Vec<&str> = b.split_whitespace().collect();
    if words_a.is_empty() || words_b.is_empty() {
        return 0.0;
    }
    let shared = words_a
        .iter()
        .filter(|w| words_b.contains(w))
        .count() as f32;
    shared / words_a.len().max(words_b.len()) as f32
}

/// Score one candidate release. Zero means "not plausible" (track count too far apart).
pub fn score(input: &MatchInput, release: &MbReleaseDetail, mb_score: Option<f64>) -> AlbumScore {
    let delta = (input.tracks.len() as i32) - (release.tracks.len() as i32);
    if delta.abs() > 1 {
        return AlbumScore {
            value: 0.0,
            median_duration_delta_ms: u32::MAX,
            track_count_delta: delta,
            comparable_tracks: 0,
        };
    }

    let (median_ms, comparable) = median_duration_delta(&input.tracks, release);
    // With no comparable durations there is no duration evidence at all; do not award the
    // duration component just because no delta was computable.
    let duration_fit = if comparable == 0 {
        0.0
    } else {
        (-(median_ms as f32) / 15_000.0).exp()
    };
    let title = title_similarity(&input.title, &release.title);
    let remote_titles: std::collections::HashMap<(i32, i32), &str> = release
        .tracks
        .iter()
        .map(|t| ((t.disc, t.position), t.title.as_str()))
        .collect();
    let per_track_title = if input.tracks.is_empty() {
        0.0
    } else {
        input
            .tracks
            .iter()
            .filter_map(|l| {
                remote_titles
                    .get(&track_key(l.disc, l.position))
                    .map(|title| title_similarity(&l.title, title))
            })
            .sum::<f32>()
            / input.tracks.len() as f32
    };
    let mb = (mb_score.unwrap_or(50.0) / 100.0).clamp(0.0, 1.0) as f32;

    let count_penalty = if delta == 0 { 1.0 } else { 0.85 };
    let value = count_penalty
        * (0.55 * duration_fit + 0.20 * title + 0.10 * per_track_title + 0.15 * mb);
    let value = if value.is_finite() { value } else { 0.0 };
    AlbumScore {
        value,
        median_duration_delta_ms: median_ms,
        track_count_delta: delta,
        comparable_tracks: comparable,
    }
}

#[cfg(test)]
mod tests {
    use super::{score, LocalTrack, MatchInput};
    use crate::metadata::musicbrainz::{MbReleaseDetail, MbTrack};

    fn release(id: &str, title: &str, lengths: &[u32]) -> MbReleaseDetail {
        MbReleaseDetail {
            id: id.to_string(),
            title: title.to_string(),
            release_group: None,
            tracks: lengths
                .iter()
                .enumerate()
                .map(|(i, len)| MbTrack {
                    id: format!("reltrack-{i}"),
                    recording: Some(crate::metadata::musicbrainz::MbRecordingRef {
                        id: format!("rec-{i}"),
                    }),
                    disc: 1,
                    position: i as i32 + 1,
                    title: format!("track {}", i + 1),
                    length: Some(*len),
                })
                .collect(),
        }
    }

    fn input(title: &str, lengths: &[u32]) -> MatchInput {
        MatchInput {
            title: title.to_string(),
            artist: "some artist".to_string(),
            tracks: lengths
                .iter()
                .enumerate()
                .map(|(i, len)| LocalTrack {
                    disc: Some(1),
                    position: i as i32 + 1,
                    title: format!("track {}", i + 1),
                    duration: *len,
                })
                .collect(),
        }
    }

    #[test]
    fn exact_tracklist_scores_high() {
        let got = score(&input("BAD MODE", &[200_000, 210_000]), &release("r", "BAD MODE", &[200_000, 210_000]), Some(100.0));
        assert!(got.value > 0.95, "{:?}", got);
        assert_eq!(got.median_duration_delta_ms, 0);
    }

    #[test]
    fn wrong_track_count_is_rejected() {
        let got = score(&input("BAD MODE", &[200_000, 210_000, 220_000, 230_000]), &release("r", "BAD MODE", &[200_000, 210_000]), Some(100.0));
        assert_eq!(got.value, 0.0);
    }

    #[test]
    fn durations_dominate_title() {
        let exact = score(&input("OTHER TITLE", &[200_000, 210_000]), &release("a", "BAD MODE", &[200_000, 210_000]), Some(50.0));
        let off = score(&input("BAD MODE", &[50_000, 60_000]), &release("b", "BAD MODE", &[200_000, 210_000]), Some(100.0));
        assert!(exact.value > off.value, "exact {:?} vs off {:?}", exact, off);
    }

    #[test]
    fn one_bonus_track_is_penalised_but_not_rejected() {
        let got = score(&input("BAD MODE", &[200_000, 210_000, 220_000]), &release("r", "BAD MODE", &[200_000, 210_000]), Some(100.0));
        assert!(got.value > 0.0 && got.value < 0.95, "{:?}", got);
        assert_eq!(got.track_count_delta, 1);
    }
}

#[cfg(test)]
mod evidence_tests {
    use super::{score, LocalTrack, MatchInput};
    use crate::metadata::musicbrainz::{MbReleaseDetail, MbTrack};

    fn detail(lengths: Vec<Option<u32>>) -> MbReleaseDetail {
        MbReleaseDetail {
            id: "release".into(),
            title: "BAD MODE".into(),
            release_group: None,
            tracks: lengths
                .into_iter()
                .enumerate()
                .map(|(i, len)| MbTrack {
                    id: format!("rt-{i}"),
                    recording: None,
                    disc: 1,
                    position: i as i32 + 1,
                    title: "x".into(),
                    length: len,
                })
                .collect(),
        }
    }

    #[test]
    fn no_durations_is_not_a_perfect_match() {
        let input = MatchInput {
            title: "BAD MODE".into(),
            artist: "Utada".into(),
            tracks: vec![LocalTrack {
                disc: Some(1),
                position: 1,
                duration: 200_000,
                title: "x".into(),
            }],
        };
        let got = score(&input, &detail(vec![None]), Some(100.0));
        assert_eq!(got.comparable_tracks, 0);
        assert!(got.value < 0.78, "{:?}", got);
    }

    #[test]
    fn disc_and_position_are_sorted_independently() {
        let input = MatchInput {
            title: "BAD MODE".into(),
            artist: "Utada".into(),
            tracks: vec![
                LocalTrack { disc: Some(2), position: 1, duration: 50_000, title: "x".into() },
                LocalTrack { disc: Some(1), position: 1, duration: 200_000, title: "x".into() },
            ],
        };
        let got = score(&input, &detail(vec![Some(200_000), Some(50_000)]), Some(100.0));
        assert_eq!(got.median_duration_delta_ms, 0, "{:?}", got);
    }
}
