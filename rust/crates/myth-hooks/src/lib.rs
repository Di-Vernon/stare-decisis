//! myth-hooks — Claude Code hook bridge (Layer 3).
//!
//! This crate's library surface is a set of shared helpers in `core`
//! for parsing Claude Code's stdin JSON envelopes, producing the
//! return JSON, recording per-invocation latency, and orchestrating
//! the tiny fixed boilerplate every hook binary shares. The actual
//! executables (myth-hook-pre-tool, etc.) land in
//! `src/bin/*` from Task 3.2 onwards.

pub mod core;
pub mod records;
pub mod remand;
pub mod templates;
pub mod tier0;
pub mod tier3_dispatch;

pub use core::input::{
    parse_envelope, read_envelope_from_stdin, HookCommon, HookEnvelope, HookEventName,
    HookPayload, PostToolUseData, PostToolUseFailureData, PostToolUseResponse,
    PreToolUseData, SessionStartData, StopData, UserPromptSubmitData,
};
pub use core::latency::{record as record_latency, record_ignore_err as record_latency_quiet};
pub use core::output::{HookOutcome, HookResult, PartialHookEvent};
pub use core::runner::run_hook;
pub use core::session::parse_claude_session_id;
pub use tier0::{classify as classify_tier0, DeterministicClassification};
pub use tier3_dispatch::{maybe_tier3_dispatch, tier3_gate_active};
