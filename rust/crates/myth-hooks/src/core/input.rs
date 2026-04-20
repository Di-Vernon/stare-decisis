//! stdin JSON envelope parsing.
//!
//! Field names and shapes match the Claude Code 2.1.114 runtime probe
//! captured at `/tmp/myth-hook-probe/` and preserved under
//! `tests/fixtures/envelopes/`. See `docs/06-HOOKS.md §Hook 입력 JSON
//! 스키마` for the authoritative spec.
//!
//! Design note on `deny_unknown_fields`: we **do not** enable it.
//! If Claude Code adds a new field in a minor release, myth should
//! continue to function rather than fail to parse. The tradeoff is
//! that silent field additions go unnoticed unless we re-probe;
//! periodic schema re-probing is a Wave 3+ operational task (tracked
//! in Task 3.6's future-work note) rather than a parser-level
//! invariant. Breakage in the fields we actually consume still
//! surfaces through the typed struct layout.

use std::io::Read;
use std::path::PathBuf;

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum HookEventName {
    SessionStart,
    UserPromptSubmit,
    PreToolUse,
    PostToolUse,
    PostToolUseFailure,
    Stop,
}

impl HookEventName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SessionStart => "session_start",
            Self::UserPromptSubmit => "user_prompt_submit",
            Self::PreToolUse => "pre_tool_use",
            Self::PostToolUse => "post_tool_use",
            Self::PostToolUseFailure => "post_tool_use_failure",
            Self::Stop => "stop",
        }
    }
}

/// Fields present on every hook envelope (with `#[serde(default)]`
/// for fields Claude Code omits on some events — e.g. `permission_mode`
/// is absent on SessionStart; `stop_hook_active` only appears on Stop).
#[derive(Debug, Clone, Deserialize)]
pub struct HookCommon {
    pub session_id: String,
    pub transcript_path: PathBuf,
    pub cwd: PathBuf,
    pub hook_event_name: HookEventName,
    #[serde(default)]
    pub stop_hook_active: bool,
    #[serde(default)]
    pub permission_mode: Option<String>,
}

/// Event-specific payload. Tagged by `hook_event_name` on the wire —
/// serde's internally-tagged enum picks the right variant per event.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "hook_event_name")]
pub enum HookPayload {
    SessionStart(SessionStartData),
    UserPromptSubmit(UserPromptSubmitData),
    PreToolUse(PreToolUseData),
    PostToolUse(PostToolUseData),
    PostToolUseFailure(PostToolUseFailureData),
    Stop(StopData),
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionStartData {
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserPromptSubmitData {
    pub prompt: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PreToolUseData {
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub tool_use_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostToolUseData {
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub tool_response: PostToolUseResponse,
    pub tool_use_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostToolUseResponse {
    #[serde(default)]
    pub stdout: String,
    #[serde(default)]
    pub stderr: String,
    #[serde(default)]
    pub interrupted: bool,
    #[serde(default, rename = "isImage")]
    pub is_image: bool,
    #[serde(default, rename = "noOutputExpected")]
    pub no_output_expected: bool,
}

/// **PostToolUseFailure**: the runtime field is `error` (multiline
/// string whose first line is typically `"Exit code N"`), **not** a
/// structured `tool_response`. See docs 823e749 (hook schema probe).
///
/// Serialize derive added in Task 3.5 because the Variant B template
/// embeds the payload as a compact JSON string in the
/// `additionalContext` hook output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostToolUseFailureData {
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub tool_use_id: String,
    pub error: String,
    #[serde(default)]
    pub is_interrupt: bool,
}

impl PostToolUseFailureData {
    /// Best-effort extract of the exit code from the first line of
    /// `error` when it matches `"Exit code N"`. Returns None if the
    /// pattern doesn't apply (non-Bash tools, interrupts, etc.).
    pub fn parse_exit_code(&self) -> Option<i32> {
        let first_line = self.error.lines().next()?;
        first_line
            .strip_prefix("Exit code ")
            .and_then(|s| s.parse::<i32>().ok())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct StopData {
    #[serde(default)]
    pub last_assistant_message: Option<String>,
}

/// Full parsed envelope — common fields plus the event-specific payload.
#[derive(Debug, Clone)]
pub struct HookEnvelope {
    pub common: HookCommon,
    pub payload: HookPayload,
}

/// Parse a complete hook envelope from JSON text. Double-deserialises:
/// once as `HookCommon` and once as `HookPayload` (internally tagged).
/// At ~1KB of JSON per event the cost is negligible.
pub fn parse_envelope(json: &str) -> anyhow::Result<HookEnvelope> {
    let common: HookCommon =
        serde_json::from_str(json).context("parsing HookCommon from envelope")?;
    let payload: HookPayload =
        serde_json::from_str(json).context("parsing HookPayload from envelope")?;
    Ok(HookEnvelope { common, payload })
}

/// Read the envelope from stdin (Claude Code pipes JSON there).
pub fn read_envelope_from_stdin() -> anyhow::Result<HookEnvelope> {
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .context("reading stdin")?;
    parse_envelope(&buf)
}
