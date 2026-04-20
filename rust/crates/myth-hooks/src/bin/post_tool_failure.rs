//! PostToolUseFailure hook — Decision 3's Tier 0 + Tier 1 hybrid.
//!
//! Flow:
//!
//! 1. Parse the envelope; only act on `PostToolUseFailure`.
//! 2. Run `classify_tier0(error)`.
//! 3. On Tier 0 hit: upsert a lesson (SHA1 identity + aggressive
//!    normalise), emit `recurrence_increment` or `lesson_created`
//!    into lesson-state.jsonl, record the failure with classification
//!    in caselog.jsonl, write a `tier_resolved=0` shadow metric, and
//!    return `Allow`.
//! 4. On Tier 0 miss: generate a `reminder_id`, render the Variant B
//!    template, emit `pending_reflection` into lesson-state.jsonl,
//!    record failure+reminder in caselog.jsonl, write a
//!    `tier_resolved=1` shadow metric, return `AllowWithContext`
//!    with the template embedded in `additionalContext`.
//!
//! Every observability write is best-effort. A JSONL or DB failure
//! is logged at `warn` and swallowed so the hook still returns a
//! sane HookResult to the runner. Claude Code must never be blocked
//! because disk is full or a permission bit is off.

use std::path::PathBuf;
use std::process::ExitCode;

use mimalloc::MiMalloc;
use myth_common::{Category, LessonId, ReminderId};
use myth_db::{Database, JsonlWriter, Lesson, LessonStatus, LessonStore, SqliteLessonStore};
use myth_hooks::{
    classify_tier0, records, run_hook, templates, DeterministicClassification, HookPayload,
    HookResult, PostToolUseFailureData,
};
use myth_identity::{normalize_aggressive, tier1_hash};
use serde_json::json;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> ExitCode {
    run_hook(
        "post_tool_failure",
        "myth-hook-post-tool-failure",
        |envelope| {
            let data = match &envelope.payload {
                HookPayload::PostToolUseFailure(d) => d,
                _ => return Ok(HookResult::Allow),
            };
            let session_id_str = envelope.common.session_id.as_str();

            if let Some(c) = classify_tier0(&data.error) {
                // Tier 0 resolved. All observability writes are
                // fire-and-forget — never block on JSONL/DB failure.
                if let Err(e) = record_tier0(session_id_str, data, &c) {
                    tracing::warn!(error = %e, "Tier 0 record failed (observability-only)");
                }
                Ok(HookResult::Allow)
            } else {
                // Tier 0 miss → Tier 1 Variant B template.
                let reminder_id = ReminderId::new();
                let template = templates::variant_b::render(data, reminder_id);

                if let Err(e) = record_tier1(session_id_str, data, reminder_id) {
                    tracing::warn!(error = %e, "Tier 1 record failed (observability-only)");
                }

                Ok(HookResult::AllowWithContext(json!({
                    "continue": true,
                    "hookSpecificOutput": {
                        "hookEventName": "PostToolUseFailure",
                        "additionalContext": template,
                    }
                })))
            }
        },
    )
}

fn record_tier0(
    session_id_str: &str,
    data: &PostToolUseFailureData,
    c: &DeterministicClassification,
) -> anyhow::Result<LessonId> {
    let normalized = normalize_aggressive(&data.error);
    let hash = tier1_hash(&normalized);
    let ts = myth_common::format_iso(&myth_common::now());

    let db = Database::open(&myth_common::state_db_path())?;
    let store = SqliteLessonStore::new(db);

    let lesson_id = match store.find_by_identity(&hash)? {
        Some(existing) => {
            let new_count = store.increment_recurrence(existing.id)?;
            JsonlWriter::new(myth_common::lesson_state_path()).append(
                &records::RecurrenceIncrement {
                    ts: ts.clone(),
                    event: "recurrence_increment",
                    lesson_id: existing.id.to_string(),
                    session_id: session_id_str,
                    new_count,
                },
            )?;
            existing.id
        }
        None => {
            let now = myth_common::now();
            let description: String = normalized.chars().take(500).collect();
            let lesson = Lesson {
                id: LessonId::new(),
                identity_hash_tier1: hash,
                level: c.level,
                category: c.category,
                recurrence_count: 1.0,
                missed_hook_count: 0,
                first_seen: now,
                last_seen: now,
                lapse_score: 0.0,
                appeals: 0,
                status: LessonStatus::Active,
                description,
                rationale: c.rationale.to_string(),
                meta_json: None,
            };
            store.insert(&lesson)?;
            JsonlWriter::new(myth_common::lesson_state_path()).append(&records::LessonCreated {
                ts: ts.clone(),
                event: "lesson_created",
                lesson_id: lesson.id.to_string(),
                session_id: session_id_str,
                level: c.level as u8,
                category: category_sql_label(c.category),
                rationale: c.rationale,
            })?;
            lesson.id
        }
    };

    // caselog.jsonl — failure with classification + lesson id.
    let mut failure = records::FailureRecord::from_envelope(session_id_str, data);
    failure.classified_level = Some(c.level as u8);
    failure.classified_category = Some(category_sql_label(c.category));
    failure.rationale = Some(c.rationale);
    failure.lesson_id = Some(lesson_id.to_string());
    JsonlWriter::new(myth_common::caselog_path()).append(&failure)?;

    // reflector-shadow.jsonl — tier_resolved = 0.
    JsonlWriter::new(shadow_metrics_path()).append(&records::ShadowMetric {
        ts,
        tier_resolved: 0,
        variant: "B",
        reminder_id: None,
        rationale: Some(c.rationale),
        tool_name: &data.tool_name,
    })?;

    Ok(lesson_id)
}

fn record_tier1(
    session_id_str: &str,
    data: &PostToolUseFailureData,
    reminder_id: ReminderId,
) -> anyhow::Result<()> {
    let ts = myth_common::format_iso(&myth_common::now());
    let reminder_id_str = reminder_id.to_string();

    // caselog.jsonl — failure + reminder_id, no classification.
    let mut failure = records::FailureRecord::from_envelope(session_id_str, data);
    failure.reminder_id = Some(reminder_id_str.clone());
    JsonlWriter::new(myth_common::caselog_path()).append(&failure)?;

    // lesson-state.jsonl — pending_reflection.
    JsonlWriter::new(myth_common::lesson_state_path()).append(&records::PendingReflection {
        ts: ts.clone(),
        event: "pending_reflection",
        reminder_id: reminder_id_str.clone(),
        session_id: session_id_str,
        tool_name: &data.tool_name,
        tool_use_id: &data.tool_use_id,
    })?;

    // reflector-shadow.jsonl — tier_resolved = 1.
    JsonlWriter::new(shadow_metrics_path()).append(&records::ShadowMetric {
        ts,
        tier_resolved: 1,
        variant: "B",
        reminder_id: Some(reminder_id_str),
        rationale: None,
        tool_name: &data.tool_name,
    })?;

    Ok(())
}

fn shadow_metrics_path() -> PathBuf {
    myth_common::myth_home()
        .join("metrics")
        .join("reflector-shadow.jsonl")
}

fn category_sql_label(c: Category) -> &'static str {
    match c {
        Category::Security => "security",
        Category::Correctness => "correctness",
        Category::Process => "process",
        Category::DataSafety => "data_safety",
        Category::Temporal => "temporal",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use myth_common::{Category, Level};

    #[test]
    fn category_sql_labels_snake_case() {
        assert_eq!(category_sql_label(Category::Security), "security");
        assert_eq!(category_sql_label(Category::DataSafety), "data_safety");
        assert_eq!(category_sql_label(Category::Temporal), "temporal");
    }

    #[test]
    fn shadow_path_is_under_metrics_dir() {
        let p = shadow_metrics_path();
        assert!(p.to_string_lossy().contains("metrics"));
        assert!(p.ends_with("reflector-shadow.jsonl"));
    }

    // The DB-backed record_tier0 / record_tier1 functions read live
    // paths (state_db_path / caselog_path / …) via myth_common; they
    // are exercised via tests/post_tool_failure_test.rs against a
    // tempdir + XDG path override. Keep bin-internal unit tests
    // focused on pure helpers.
    #[allow(dead_code)]
    fn _level_copy_is_compact() {
        let _ = Level::Medium as u8;
    }
}
