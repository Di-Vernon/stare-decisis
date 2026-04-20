//! Timestamp helpers.

use chrono::{DateTime, Utc};

pub type Timestamp = DateTime<Utc>;

pub fn now() -> Timestamp {
    Utc::now()
}

pub fn format_iso(ts: &Timestamp) -> String {
    ts.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}
