use super::parse_duration;
use std::time::Duration;

#[test]
fn parses_supported_watch_durations() {
    assert_eq!(parse_duration("500ms"), Ok(Duration::from_millis(500)));
    assert_eq!(parse_duration("2s"), Ok(Duration::from_secs(2)));
    assert_eq!(parse_duration("1m"), Ok(Duration::from_secs(60)));
}

#[test]
fn rejects_too_fast_watch_durations() {
    assert!(parse_duration("99ms").is_err());
}
