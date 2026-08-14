use super::parse_duration;
use std::time::Duration;

#[test]
fn parses_supported_watch_durations() {
    assert_eq!(parse_duration("500ms"), Ok(Duration::from_millis(500)));
    assert_eq!(parse_duration("1.5s"), Ok(Duration::from_millis(1_500)));
    assert_eq!(parse_duration("2s"), Ok(Duration::from_secs(2)));
    assert_eq!(parse_duration("1m"), Ok(Duration::from_secs(60)));
}

#[test]
fn rejects_too_fast_watch_durations() {
    assert!(parse_duration("99ms").is_err());
}

#[test]
fn rejects_watch_durations_that_do_not_fit_in_duration() {
    let too_large = format!("{}m", "9".repeat(307));
    assert!(parse_duration(&too_large).is_err());
}
