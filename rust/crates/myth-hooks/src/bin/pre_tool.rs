//! PreToolUse hook — The Gavel's runtime mouthpiece.
//!
//! Construction: `Gavel::init` loads rule YAML (~/.myth/bedrock|
//! foundation|surface-rules.yaml), opens state.db for the grid
//! override lookup, and owns an empty FatigueTracker.
//!
//! Dispatch: serialize the hook's `tool_input` object to a JSON
//! string, hand that + session_id + tool_name to `Gavel::judge`,
//! map the resulting `Verdict::enforcement` to a `HookResult`.
//!
//! If Gavel initialisation fails (missing rule files, corrupt
//! state.db, etc.) the bin returns `HookResult::Error` which
//! degrades to exit 0 — fail-safe for myth's observability loop,
//! not for the pre-execution security gate. Explicit Bedrock /
//! Foundation blocks are the only path to exit 2 here.

use std::process::ExitCode;

use anyhow::Context;
use mimalloc::MiMalloc;
use myth_common::Enforcement;
use myth_db::events::{HookEventType, Verdict as DbVerdict};
use myth_db::Database;
use myth_gavel::{Gavel, ToolInput};
use myth_hooks::core::session::parse_claude_session_id;
use myth_hooks::{run_hook, HookOutcome, HookPayload, HookResult, PartialHookEvent};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// Collapse Gavel's 7-level `Enforcement` onto the 3-state
/// `hook_events.verdict` column (allow / ask / deny). Advisory + Caution
/// both map to allow because Claude Code is still permitted to run the
/// tool; only Warn routes to ask and only Strike/Seal deny.
fn db_verdict_from(enforcement: Enforcement) -> DbVerdict {
    match enforcement {
        Enforcement::Dismiss
        | Enforcement::Note
        | Enforcement::Advisory
        | Enforcement::Caution => DbVerdict::Allow,
        // Remand is reserved for Milestone A; if it ever escapes the Gavel
        // demotion guard in v0.2 we record it as Ask, identical to Warn.
        Enforcement::Warn | Enforcement::Remand => DbVerdict::Ask,
        Enforcement::Strike | Enforcement::Seal => DbVerdict::Deny,
    }
}

fn main() -> ExitCode {
    run_hook("pre_tool", "myth-hook-pre-tool", |envelope| {
        let data = match &envelope.payload {
            HookPayload::PreToolUse(d) => d,
            _ => return Ok(HookResult::Allow.into()),
        };

        // Task 3.6 Step c — Connection sharing. Open state.db once
        // here, lend it to the Gavel via `init_with_db`, then reclaim
        // it via `into_db` so the runner's `persist_hook_event` uses
        // the same connection for the `hook_events` insert. The
        // alternative (Gavel opens its own db + runner opens another)
        // pushes pre_tool above 37 ms — Milestone C trigger territory.
        let db = Database::open(&myth_common::state_db_path())
            .context("opening state.db")?;
        let gavel = Gavel::init_with_db(db)?;
        let session_id = parse_claude_session_id(&envelope.common.session_id)?;

        // Gavel regexes run against a serialized JSON string so every
        // nested tool_input field (Bash command, Edit path, …) is in
        // scope at once. Failure to serialize is pathological; fall
        // back to `"{}"` so the pipeline continues.
        let serialized = serde_json::to_string(&data.tool_input)
            .unwrap_or_else(|_| "{}".to_string());

        let input = ToolInput {
            tool_name: data.tool_name.clone(),
            session_id,
            serialized,
        };

        let verdict = gavel.judge(&input);
        let json = verdict.to_hook_json();
        let db_verdict = db_verdict_from(verdict.enforcement);
        let lesson_id = verdict.lesson_id;

        // Reclaim the shared db before dropping the Gavel. In
        // production SqliteLessonStore always honours this; the
        // fallback path (fresh open by the runner) kicks in only if
        // a mock store is used, which is a test-only concern.
        let shared_db = gavel.into_db();

        let result = match verdict.enforcement {
            Enforcement::Dismiss | Enforcement::Note => HookResult::Allow,
            Enforcement::Advisory | Enforcement::Caution => HookResult::AllowWithContext(json),
            // Remand is reserved (Milestone A). The Gavel demotion guard
            // ensures we never reach this arm in v0.2; the explicit case
            // exists to keep the match exhaustive after the enum
            // expansion (see `experiment/remand-prototype/`).
            Enforcement::Warn | Enforcement::Remand => HookResult::Ask(json),
            Enforcement::Strike | Enforcement::Seal => HookResult::Block { output: json },
        };

        let event = PartialHookEvent {
            session_id,
            event_type: HookEventType::PreTool,
            tool_name: Some(data.tool_name.clone()),
            verdict: db_verdict,
            lesson_id,
        };
        let mut outcome = HookOutcome::from(result).with_event(event);
        if let Some(db) = shared_db {
            outcome = outcome.with_db(db);
        }
        Ok(outcome)
    })
}

#[cfg(test)]
mod tests {
    // Library-level Gavel verdict semantics are already covered in
    // myth-gavel's 29-test suite. This bin's glue logic is exercised
    // by the subprocess integration test in Task 3.6 and the
    // runtime-probe of Case A (pre-tool Strike → no PostToolUseFailure)
    // documented in the wave-3.4 commit body.
    #[test]
    fn compiles() {}
}
