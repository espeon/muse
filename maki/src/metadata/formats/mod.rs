pub mod flac;
pub mod mp3;

/// Convert seconds to a string in the format "hh:mm:ss"
/// If the duration is less than an hour, it will be in the format "mm:ss"
pub fn s2hms(secs: u32) -> String {
    let secs = time::Duration::seconds(secs as i64);
    let mut formatted_duration = format!(
        "{}:{:02}",
        &secs.whole_minutes(),
        secs.whole_seconds() - (secs.whole_minutes() * 60)
    );
    if secs.whole_hours() > 0 {
        formatted_duration = format!(
            "{}:{:02}:{:02}",
            secs.whole_hours(),
            &secs.whole_minutes(),
            secs.whole_seconds() - (secs.whole_minutes() * 60)
        )
    }

    formatted_duration
}
