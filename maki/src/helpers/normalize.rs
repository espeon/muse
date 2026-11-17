//! Normalized keys for multilingual (Japanese, Chinese, romanised) text search.

use unicode_normalization::UnicodeNormalization;

/// Returns true for the Latin combining diacritical marks covered by the
/// haystack/range used below. Kana voicing marks (U+3099/U+309A) live outside
/// this range and are deliberately left alone.
fn is_latin_diacritic(c: char) -> bool {
    ('\u{0300}'..='\u{036F}').contains(&c)
}

/// Compute a normalized search key for artist, album and song names.
///
/// The pipeline order is load-bearing:
/// 1. NFKC normalization (full-width to half-width, compatibility forms).
/// 2. Remove Latin combining diacritical marks in U+0300..=U+036F ONLY. The
///    NFD/NFC around the filter are internal to this step: they expose marks
///    hidden in precomposed characters (e.g. `é`) and then recompose kana so
///    voicing marks (U+3099/U+309A) survive. Other marks are never removed.
/// 3. Lowercase.
/// 4. Fold katakana U+30A1..=U+30F6 to hiragana by subtracting 0x60. Han,
///    half-width katakana and the prolonged sound mark (ー) are not mapped.
/// 5. Collapse runs of whitespace to one ASCII space and trim.
pub fn search_key(input: &str) -> String {
    let normalized: String = input
        .nfkc()
        .nfd()
        .filter(|&c| !is_latin_diacritic(c))
        .nfc()
        .collect();
    let lowercased = normalized.to_lowercase();
    let folded: String = lowercased
        .chars()
        .map(|c| match c as u32 {
            u @ 0x30A1..=0x30F6 => char::from_u32(u - 0x60).unwrap_or(c),
            _ => c,
        })
        .collect();

    folded.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Like [`search_key`], but escapes SQL `LIKE` metacharacters (`%`, `_`, `\`)
/// by prefixing each with a backslash so user input cannot widen a query.
/// The SQL query must use `ESCAPE '\'` for the backslashes to be interpreted.
pub fn search_query(input: &str) -> String {
    let key = search_key(input);
    let mut escaped = String::with_capacity(key.len() + 8);
    for c in key.chars() {
        if c == '%' || c == '_' || c == '\\' {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::{search_key, search_query};

    #[test]
    fn full_width_romanisation_matches_half_width() {
        assert_eq!(search_key("ＢＡＤモード"), search_key("BADモード"));
        assert_eq!(search_key("ＢＡＤモード"), "badもーど");
    }

    #[test]
    fn katakana_and_hiragana_match() {
        assert_eq!(search_key("バド"), search_key("ばど"));
        assert_eq!(search_key("バド"), "ばど");
    }

    #[test]
    fn katakana_voicing_mark_composes_and_survives() {
        // ハ + U+3099 composes to バ, then folds to ば (not は).
        assert_eq!(search_key("\u{30CF}\u{3099}"), "ば");
        assert_eq!(search_key("\u{30CF}\u{3099}ド"), search_key("バド"));
        assert!(!search_key("バド").contains('\u{3099}'));
    }

    #[test]
    fn latin_diacritics_are_stripped() {
        assert_eq!(search_key("Café"), "cafe");
        assert_eq!(search_key("Cafe\u{0301}"), "cafe");
        assert_eq!(search_key("NAÏVE"), "naive");
    }

    #[test]
    fn han_is_preserved_without_conflating_scripts() {
        assert!(search_key("周杰倫").contains("周杰倫"));
        assert_eq!(search_key("周杰倫"), "周杰倫");
        assert_ne!(search_key("蓄音"), search_key("贮音"));
    }

    #[test]
    fn whitespace_is_collapsed_and_trimmed() {
        assert_eq!(search_key("  Bad\tモード\n  \u{3000} mix  "), "bad もーど mix");
        assert_eq!(search_key("   "), "");
    }

    #[test]
    fn search_query_escapes_like_metacharacters() {
        assert_eq!(search_query("50%_x"), r"50\%\_x");
        assert_eq!(search_query(r"a\b"), r"a\\b");
        assert_eq!(search_query("Ａ_b%"), r"a\_b\%");
    }
}
