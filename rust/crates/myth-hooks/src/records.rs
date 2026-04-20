//! Record structs that post-tool-failure writes to its three
//! observability JSONL files.
//!
//! Each file is append-only and independently consumable:
//!
//! - `caselog.jsonl` — every failure, Tier 0 hit or miss. Primary
//!   source of truth for Observer's weekly analysis.
//! - `lesson-state.jsonl` — lesson lifecycle events. On Tier 0 hit
//!   we emit `lesson_created` (or `recurrence_increment`); on Tier 0
//!   miss `pending_reflection` with a reminder_id that user-prompt /
//!   stop hooks will eventually resolve to compliant / missed.
//! - `metrics/reflector-shadow.jsonl` — one record per Tier 0/1
//!   decision, feeding Milestone A's Tier-1 compliance analysis.

use serde::Serialize;

use crate::core::input::PostToolUseFailureData;

/// caselog.jsonl schema. `classified_*` fields are populated when
/// Tier 0 resolves the failure; `reminder_id` is populated when Tier 1
/// defers to the assessor subagent.
#[derive(Debug, Serialize)]
pub struct FailureRecord<'a> {
    pub ts: String,
    pub session_id: &'a str,
    pub event: &'static str,
    pub tool_name: &'a str,
    pub tool_input: &'a serde_json::Value,
    pub tool_use_id: &'a str,
    pub error: &'a str,
    pub is_interrupt: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classified_level: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classified_category: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lesson_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reminder_id: Option<String>,
}

impl<'a> FailureRecord<'a> {
    /// Build a base record from an envelope. Callers overlay
    /// Tier 0 / Tier 1 specifics on top.
    pub fn from_envelope(session_id: &'a str, data: &'a PostToolUseFailureData) -> Self {
        Self {
            ts: myth_common::format_iso(&myth_common::now()),
            session_id,
            event: "post_tool_failure",
            tool_name: &data.tool_name,
            tool_input: &data.tool_input,
            tool_use_id: &data.tool_use_id,
            error: &data.error,
            is_interrupt: data.is_interrupt,
            exit_code: data.parse_exit_code(),
            classified_level: None,
            classified_category: None,
            rationale: None,
            lesson_id: None,
            reminder_id: None,
        }
    }
}

/// lesson-state.jsonl — pending_reflection (Tier 1 miss) event.
#[derive(Debug, Serialize)]
pub struct PendingReflection<'a> {
    pub ts: String,
    pub event: &'static str, // "pending_reflection"
    pub reminder_id: String,
    pub session_id: &'a str,
    pub tool_name: &'a str,
    pub tool_use_id: &'a str,
}

/// lesson-state.jsonl — lesson_created event (Tier 0 hit).
#[derive(Debug, Serialize)]
pub struct LessonCreated<'a> {
    pub ts: String,
    pub event: &'static str, // "lesson_created"
    pub lesson_id: String,
    pub session_id: &'a str,
    pub level: u8,
    pub category: &'static str,
    pub rationale: &'static str,
}

/// lesson-state.jsonl — recurrence_increment event (Tier 0 hit on
/// an existing lesson).
#[derive(Debug, Serialize)]
pub struct RecurrenceIncrement<'a> {
    pub ts: String,
    pub event: &'static str, // "recurrence_increment"
    pub lesson_id: String,
    pub session_id: &'a str,
    pub new_count: f64,
}

/// metrics/reflector-shadow.jsonl — one record per Tier 0/1 decision.
#[derive(Debug, Serialize)]
pub struct ShadowMetric<'a> {
    pub ts: String,
    /// 0 = Tier 0 resolved, 1 = Tier 1 (Variant B dispatched)
    pub tier_resolved: u8,
    pub variant: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reminder_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<&'static str>,
    pub tool_name: &'a str,
}
