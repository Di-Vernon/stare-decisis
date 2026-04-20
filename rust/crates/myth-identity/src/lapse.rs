//! Lapse scoring — Constitution Article 13 (Desuetude → Lapse).
//!
//! Score formula (hook-count + wall-clock composite):
//!   lapse_score = missed_hook_count * 1.0 + idle_days * 10.0
//!
//! Thresholds by Level:
//!   Level 1 (Info) / 2 (Low)     → 50
//!   Level 3 (Medium) / 4 (High)  → 200
//!   Level 5 (Critical)           → None (exempt — Bedrock Rule
//!                                  territory; a Bedrock lesson never
//!                                  lapses)

use myth_common::{Level, Timestamp};
use myth_db::Lesson;

pub fn compute_lapse_score(lesson: &Lesson, now: Timestamp) -> f64 {
    let idle_days = (now - lesson.last_seen).num_days() as f64;
    let idle_days = idle_days.max(0.0);
    let missed_hooks = lesson.missed_hook_count as f64;
    missed_hooks * 1.0 + idle_days * 10.0
}

pub fn lapse_threshold(level: Level) -> Option<f64> {
    match level {
        Level::Info | Level::Low => Some(50.0),
        Level::Medium | Level::High => Some(200.0),
        Level::Critical => None,
    }
}

pub fn should_lapse(lesson: &Lesson, now: Timestamp) -> bool {
    match lapse_threshold(lesson.level) {
        Some(t) => compute_lapse_score(lesson, now) >= t,
        None => false,
    }
}
