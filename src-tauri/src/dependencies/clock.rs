//! UTC time for the whole application.
//!
//! Every timestamp the app stores or compares is UTC. They are stored as `TEXT` in the
//! `YYYY-MM-DD HH:MM:SS` format, the same one SQLite's `datetime('now')` produces, so columns
//! written by SQL defaults and columns written from Rust compare and sort alike.
//!
//! Services never call `Utc::now()` directly. They take a [`Clock`], which tests replace with
//! `FixedClock` (test builds only) to control time.

use chrono::{DateTime, NaiveDateTime, Timelike, Utc};

/// Storage format of every timestamp column (UTC, second precision).
pub const UTC_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// Formats an instant for storage.
pub fn format_utc(instant: DateTime<Utc>) -> String {
    instant.format(UTC_FORMAT).to_string()
}

/// Parses a stored timestamp. Returns `None` when the text is not in [`UTC_FORMAT`].
pub fn parse_utc(text: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(text, UTC_FORMAT)
        .ok()
        .map(|naive| naive.and_utc())
}

/// Source of the current time.
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

/// The system clock, truncated to whole seconds to match the storage precision.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now().with_nanosecond(0).expect("0 is a valid nanosecond")
    }
}

/// A clock that only moves when told to. For tests.
#[cfg(test)]
pub struct FixedClock(std::sync::Mutex<DateTime<Utc>>);

#[cfg(test)]
impl FixedClock {
    pub fn at(text: &str) -> Self {
        Self(std::sync::Mutex::new(parse_utc(text).expect("valid UTC timestamp")))
    }

    pub fn set(&self, text: &str) {
        *self.0.lock().unwrap() = parse_utc(text).expect("valid UTC timestamp");
    }
}

#[cfg(test)]
impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        *self.0.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_and_parse_round_trip() {
        let instant = parse_utc("2026-10-12 14:30:05").unwrap();
        assert_eq!(format_utc(instant), "2026-10-12 14:30:05");
    }

    #[test]
    fn rejects_other_formats() {
        assert!(parse_utc("2026-10-12T14:30:05Z").is_none());
        assert!(parse_utc("not a date").is_none());
    }

    #[test]
    fn fixed_clock_moves_only_when_set() {
        let clock = FixedClock::at("2026-01-01 00:00:00");
        assert_eq!(format_utc(clock.now()), "2026-01-01 00:00:00");
        clock.set("2026-01-02 12:00:00");
        assert_eq!(format_utc(clock.now()), "2026-01-02 12:00:00");
    }
}
