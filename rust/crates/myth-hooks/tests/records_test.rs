//! Unit tests for the JSONL record structs that post-tool-failure
//! writes. Integration (actual file writes against a real state.db
//! + JSONL files) lives in Task 3.6's subprocess runs.

use myth_hooks::records::{
    FailureRecord, LessonCreated, PendingReflection, RecurrenceIncrement, ShadowMetric,
};
use myth_hooks::PostToolUseFailureData;

fn sample_failure() -> PostToolUseFailureData {
    PostToolUseFailureData {
        tool_name: "Bash".into(),
        tool_input: serde_json::json!({"command": "cat /nonexistent"}),
        tool_use_id: "toolu_abc".into(),
        error: "Exit code 1\ncat: /x: No such file or directory".into(),
        is_interrupt: false,
    }
}

#[test]
fn failure_record_extracts_exit_code_from_error_line() {
    let data = sample_failure();
    let r = FailureRecord::from_envelope("sess-uuid", &data);
    assert_eq!(r.event, "post_tool_failure");
    assert_eq!(r.exit_code, Some(1));
    assert_eq!(r.tool_name, "Bash");
    assert!(!r.is_interrupt);
}

#[test]
fn failure_record_serialises_to_jsonl() {
    let data = sample_failure();
    let r = FailureRecord::from_envelope("sess-uuid", &data);
    let s = serde_json::to_string(&r).unwrap();
    // Key-level sanity: the JSON must include the H5 fields, not the
    // old tool_response shape.
    assert!(s.contains("\"event\":\"post_tool_failure\""));
    assert!(s.contains("\"error\""));
    assert!(s.contains("\"is_interrupt\":false"));
    assert!(!s.contains("\"tool_response\""));
}

#[test]
fn optional_fields_omit_when_none() {
    // skip_serializing_if = "Option::is_none" — verify the default
    // record (Tier 0/1 fields unset) doesn't pollute JSONL with null
    // columns.
    let data = sample_failure();
    let r = FailureRecord::from_envelope("sess-uuid", &data);
    let s = serde_json::to_string(&r).unwrap();
    assert!(!s.contains("classified_level"));
    assert!(!s.contains("lesson_id"));
    assert!(!s.contains("reminder_id"));
    // Optional fields that DO resolve stay in the JSONL (exit_code
    // was Some(1) via parse_exit_code).
    assert!(s.contains("\"exit_code\":1"));
}

#[test]
fn pending_reflection_shape() {
    let p = PendingReflection {
        ts: "2026-04-21T12:00:00Z".into(),
        event: "pending_reflection",
        reminder_id: "rid-uuid".into(),
        session_id: "sess-uuid",
        tool_name: "Bash",
        tool_use_id: "toolu_abc",
    };
    let s = serde_json::to_string(&p).unwrap();
    assert!(s.contains("\"event\":\"pending_reflection\""));
    assert!(s.contains("\"reminder_id\":\"rid-uuid\""));
    assert!(s.contains("\"tool_use_id\":\"toolu_abc\""));
}

#[test]
fn lesson_created_shape() {
    let l = LessonCreated {
        ts: "2026-04-21T12:00:00Z".into(),
        event: "lesson_created",
        lesson_id: "L-uuid".into(),
        session_id: "sess-uuid",
        level: 3,
        category: "correctness",
        rationale: "file_not_found",
    };
    let s = serde_json::to_string(&l).unwrap();
    assert!(s.contains("\"event\":\"lesson_created\""));
    assert!(s.contains("\"level\":3"));
    assert!(s.contains("\"rationale\":\"file_not_found\""));
}

#[test]
fn recurrence_increment_shape() {
    let r = RecurrenceIncrement {
        ts: "2026-04-21T12:00:00Z".into(),
        event: "recurrence_increment",
        lesson_id: "L-uuid".into(),
        session_id: "sess-uuid",
        new_count: 2.5,
    };
    let s = serde_json::to_string(&r).unwrap();
    assert!(s.contains("\"new_count\":2.5"));
}

#[test]
fn shadow_metric_tier0_shape() {
    let sh = ShadowMetric {
        ts: "2026-04-21T12:00:00Z".into(),
        tier_resolved: 0,
        variant: "B",
        reminder_id: None,
        rationale: Some("file_not_found"),
        tool_name: "Bash",
    };
    let s = serde_json::to_string(&sh).unwrap();
    assert!(s.contains("\"tier_resolved\":0"));
    assert!(!s.contains("reminder_id"));
    assert!(s.contains("\"rationale\":\"file_not_found\""));
}

#[test]
fn shadow_metric_tier1_shape() {
    let sh = ShadowMetric {
        ts: "2026-04-21T12:00:00Z".into(),
        tier_resolved: 1,
        variant: "B",
        reminder_id: Some("rid-uuid".into()),
        rationale: None,
        tool_name: "Bash",
    };
    let s = serde_json::to_string(&sh).unwrap();
    assert!(s.contains("\"tier_resolved\":1"));
    assert!(s.contains("\"reminder_id\":\"rid-uuid\""));
    assert!(!s.contains("rationale"));
}
