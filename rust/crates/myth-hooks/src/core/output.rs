//! Return value mapping: `HookResult` → stdout JSON + ExitCode, plus
//! the `HookOutcome` / `PartialHookEvent` pair that lets bins ask the
//! runner to persist an observability row into `state.db::hook_events`
//! after the runner has measured the end-to-end latency.
//!
//! ExitCode semantics (per ARCHITECTURE.md Contract 3):
//!   0  — allow (continue normally)
//!   2  — block (tool execution halted, stderr fed back to Claude)
//!   any other — non-blocking internal error
//!
//! myth deliberately maps internal hook errors to exit 0 so a bug in a
//! myth hook never cascades into a blocked Claude Code session. Only
//! an explicit Strike/Seal verdict produces exit 2.

use std::process::ExitCode;

use myth_common::{LessonId, SessionId};
use myth_db::events::{HookEventType, Verdict};
use myth_db::Database;
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum HookResult {
    /// Plain allow — no stdout, exit 0.
    Allow,
    /// Allow with `additionalContext` injected into Claude's context.
    AllowWithContext(Value),
    /// Ask the user (permissionDecision: "ask") — stdout JSON, exit 0.
    Ask(Value),
    /// Block tool execution — stdout JSON, exit 2.
    Block {
        output: Value,
    },
    /// Internal hook error — stderr logged, exit 0 (degrade, never
    /// cascade a myth bug into Claude's session).
    Error(String),
}

impl HookResult {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::AllowWithContext(_) => "allow_with_context",
            Self::Ask(_) => "ask",
            Self::Block { .. } => "block",
            Self::Error(_) => "error",
        }
    }

    pub fn is_blocking(&self) -> bool {
        matches!(self, Self::Block { .. })
    }

    /// Emit the hook output per contract and return the ExitCode.
    pub fn emit(self) -> ExitCode {
        match self {
            HookResult::Allow => ExitCode::SUCCESS,
            HookResult::AllowWithContext(v) | HookResult::Ask(v) => {
                print_json(&v);
                ExitCode::SUCCESS
            }
            HookResult::Block { output } => {
                print_json(&output);
                ExitCode::from(2)
            }
            HookResult::Error(msg) => {
                eprintln!("myth hook error: {}", msg);
                ExitCode::SUCCESS
            }
        }
    }
}

fn print_json(v: &Value) {
    // to_string on a valid Value cannot fail in practice; fall back
    // to a bare `{}` if serialisation ever does (never allow an error
    // printing to surface as a blocked tool).
    match serde_json::to_string(v) {
        Ok(s) => println!("{}", s),
        Err(e) => {
            eprintln!("myth hook: failed to serialise output: {}", e);
            println!("{{}}");
        }
    }
}

/// The half of a `HookEvent` row a bin can fill in before knowing the
/// final latency. The runner receives this as part of `HookOutcome`,
/// stamps the measured `latency_ms` (the same value it writes into
/// `hook-latency.ndjson`), assigns a fresh UUID + timestamp, and
/// appends to `state.db::hook_events`.
///
/// Scope (Task 3.6 Step c, 해석 C per Jeffrey 승인 2026-04-21):
/// only `pre_tool`, `post_tool`, and `post_tool_failure` populate
/// this field. `session_start`, `user_prompt`, and `stop` return
/// `HookOutcome::from(HookResult::X)` with `event: None`, so they
/// never trigger the extra `Database::open` on the hot path.
/// ARCHITECTURE.md §4 line 264 엄격 예산 (50ms) 내 전 bin 수용을
///유지하는 것이 이 범위 제한의 근거.
#[derive(Debug, Clone)]
pub struct PartialHookEvent {
    pub session_id: SessionId,
    pub event_type: HookEventType,
    pub tool_name: Option<String>,
    pub verdict: Verdict,
    pub lesson_id: Option<LessonId>,
}

/// What a hook closure returns to `run_hook`: the user-visible result
/// (stdout JSON + exit code), plus two optional observability fields
/// the runner uses to persist a `hook_events` row after stamping the
/// canonical latency.
///
/// Chain `.with_event(partial)` to request a DB insert, and
/// `.with_db(db)` when the bin already owns a `Database`
/// (`pre_tool` via `Gavel::into_db`, `post_tool_failure` Tier 0 via
/// `SqliteLessonStore::into_db`) so the runner can reuse that
/// connection instead of opening a second one. Sharing avoids the
/// ~30 ms WAL/PRAGMA/migration cost per extra open — see
/// `docs/04-CRATES/05-myth-hooks.md` §Wire-through Connection 공유
/// 패턴.
// Debug is intentionally not derived: `myth_db::Database` (held in the
// optional `db` field for Connection sharing) doesn't implement Debug,
// and wrapping it just for diagnostic printing would be scope creep
// outside Task 3.6 Step c (original Jeffrey instruction: don't pull
// other crates into the wire-through work). If future debug needs
// arise, impl Debug manually and skip the `db` field.
pub struct HookOutcome {
    pub result: HookResult,
    pub event: Option<PartialHookEvent>,
    pub db: Option<Database>,
}

impl HookOutcome {
    pub fn with_event(mut self, event: PartialHookEvent) -> Self {
        self.event = Some(event);
        self
    }

    pub fn with_db(mut self, db: Database) -> Self {
        self.db = Some(db);
        self
    }
}

impl From<HookResult> for HookOutcome {
    fn from(result: HookResult) -> Self {
        Self {
            result,
            event: None,
            db: None,
        }
    }
}
