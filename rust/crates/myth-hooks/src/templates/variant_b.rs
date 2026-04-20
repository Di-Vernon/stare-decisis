//! Variant B — "engineered" Tier 1 template (Decision 3).
//!
//! The template instructs Claude to dispatch the `assessor` subagent
//! on specific error classes. The runtime failure payload is embedded
//! as a compact JSON string so the subagent receives the exact
//! schema shipped by Claude Code 2.1.114 (H5 — `error` string +
//! `is_interrupt` bool; no `tool_response` object).
//!
//! Length management: the rendered template is intended to be pasted
//! into `hookSpecificOutput.additionalContext`. Claude Code imposes
//! no hard cap that we've observed, but long payloads burn tokens,
//! so we truncate the embedded error string before formatting.

use myth_common::ReminderId;

use crate::core::input::PostToolUseFailureData;

/// Maximum bytes of `error` string we embed in the template.
/// Multi-line tracebacks beyond this length get truncated. The
/// assessor still has enough context — the first few hundred bytes
/// of a Rust/Python traceback carry the error kind and the top frame.
pub const ERROR_MAX_BYTES: usize = 4_000;

const TRUNCATE_SUFFIX: &str = "… [truncated by myth]";

/// Render the Variant B prompt block for Tier 1 Assessor dispatch.
pub fn render(data: &PostToolUseFailureData, reminder_id: ReminderId) -> String {
    let truncated = truncate_error(&data.error);
    let payload = PayloadSnapshot {
        tool_name: &data.tool_name,
        tool_input: &data.tool_input,
        tool_use_id: &data.tool_use_id,
        error: &truncated,
        is_interrupt: data.is_interrupt,
    };
    let compact_json = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());

    format!(
        r#"<user-prompt-submit-hook>
<instructions>
The tool call in the previous turn failed with a non-recoverable error.
Per myth project policy, before any remediation attempt you must dispatch
the 'assessor' subagent to analyze this failure.
</instructions>

<decision_rule>
IF error_class IN {{transient_network, rate_limit_429}}: retry once; no assessor.
ELSE IF error_class IN {{schema_mismatch, permission_denied, logic_error, assertion_fail}}:
  your next action is a single Task tool call (no other output).
ELSE: ask the user.
</decision_rule>

<example>
Task(
  subagent_type="assessor",
  description="Analyze failure of {tool_name}",
  prompt="Failure payload: {compact_json}. Return the strict JSON per assessor contract. reminder_id: {rid}"
)
</example>

<rationale>
Skipping reflection on this failure class causes cascading errors.
The assessor returns a JSON verdict within one turn on Haiku 4.5.
</rationale>
<assessor-trigger-id>{rid}</assessor-trigger-id>
</user-prompt-submit-hook>"#,
        tool_name = data.tool_name,
        compact_json = compact_json,
        rid = reminder_id,
    )
}

/// Truncate the embedded error to ERROR_MAX_BYTES on a UTF-8 boundary,
/// appending TRUNCATE_SUFFIX when we cut.
pub fn truncate_error(error: &str) -> String {
    if error.len() <= ERROR_MAX_BYTES {
        return error.to_string();
    }
    let limit = ERROR_MAX_BYTES.saturating_sub(TRUNCATE_SUFFIX.len());
    let boundary = (0..=limit)
        .rev()
        .find(|&i| error.is_char_boundary(i))
        .unwrap_or(0);
    let mut out = error[..boundary].to_string();
    out.push_str(TRUNCATE_SUFFIX);
    out
}

/// Internal struct that mirrors the subset of
/// `PostToolUseFailureData` we embed in the prompt. Using a separate
/// struct lets us substitute the truncated `error` without mutating
/// the original envelope.
#[derive(serde::Serialize)]
struct PayloadSnapshot<'a> {
    tool_name: &'a str,
    tool_input: &'a serde_json::Value,
    tool_use_id: &'a str,
    error: &'a str,
    is_interrupt: bool,
}
