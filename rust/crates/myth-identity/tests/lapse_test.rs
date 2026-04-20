use chrono::{Duration, Utc};
use myth_common::{Category, Level, LessonId};
use myth_db::{Lesson, LessonStatus};
use myth_identity::{compute_lapse_score, lapse_threshold, should_lapse};

fn make_lesson(level: Level, idle_days: i64, missed_hooks: u32) -> Lesson {
    let now = Utc::now();
    Lesson {
        id: LessonId::new(),
        identity_hash_tier1: [1u8; 20],
        level,
        category: Category::Correctness,
        recurrence_count: 1.0,
        missed_hook_count: missed_hooks,
        first_seen: now - Duration::days(idle_days),
        last_seen: now - Duration::days(idle_days),
        lapse_score: 0.0,
        appeals: 0,
        status: LessonStatus::Active,
        description: "test lesson".into(),
        rationale: "test".into(),
        meta_json: None,
    }
}

#[test]
fn lapse_threshold_values_match_constitution() {
    assert_eq!(lapse_threshold(Level::Info), Some(50.0));
    assert_eq!(lapse_threshold(Level::Low), Some(50.0));
    assert_eq!(lapse_threshold(Level::Medium), Some(200.0));
    assert_eq!(lapse_threshold(Level::High), Some(200.0));
    assert_eq!(lapse_threshold(Level::Critical), None);
}

#[test]
fn lapse_score_is_hooks_plus_ten_days() {
    let lesson = make_lesson(Level::Low, 3, 5);
    let now = Utc::now();
    let score = compute_lapse_score(&lesson, now);
    // idle_days may vary by ±1 due to timing; bounds suffice.
    assert!((30.0..=40.0).contains(&score), "got {}", score);
}

#[test]
fn should_lapse_triggers_for_level1_past_threshold() {
    // Level 1 threshold is 50. 6 idle days × 10 + 0 missed = 60 → lapse.
    let lesson = make_lesson(Level::Info, 6, 0);
    assert!(should_lapse(&lesson, Utc::now()));
}

#[test]
fn should_lapse_silent_for_level1_under_threshold() {
    // 3 × 10 + 5 = 35 < 50.
    let lesson = make_lesson(Level::Info, 3, 5);
    assert!(!should_lapse(&lesson, Utc::now()));
}

#[test]
fn should_lapse_triggers_for_level3_past_two_hundred() {
    // 21 days idle = 210 score ≥ 200.
    let lesson = make_lesson(Level::Medium, 21, 0);
    assert!(should_lapse(&lesson, Utc::now()));
}

#[test]
fn critical_never_lapses() {
    // Even 365 days + heavy missed hooks must not lapse Level 5.
    let lesson = make_lesson(Level::Critical, 365, 1000);
    assert!(!should_lapse(&lesson, Utc::now()));
}

#[test]
fn negative_idle_clamps_to_zero() {
    // Edge case: lesson.last_seen in the future (clock skew).
    let now = Utc::now();
    let mut lesson = make_lesson(Level::Low, 0, 0);
    lesson.last_seen = now + Duration::days(1);
    let score = compute_lapse_score(&lesson, now);
    assert!(score >= 0.0);
}
