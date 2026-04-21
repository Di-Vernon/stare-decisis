//! PostToolUse hook — build a `PartialHookEvent` for the success and
//! hand it to the runner, which stamps the canonical latency and
//! appends the row to `state.db::hook_events`. Failure is exclusive
//! to PostToolUseFailure, so seeing this bin fire means the tool
//! succeeded and the DB verdict is always `allow`.
//!
//! Task 3.6 Step c note: the previous inline `record_event` helper
//! wrote `latency_ms: 0.0` as a hard-coded placeholder. That wire-
//! through now lives in `run_hook`, using the same `start.elapsed()`
//! value that's already appended to `hook-latency.ndjson` — the two
//! observability streams can no longer drift.

use std::process::ExitCode;

use mimalloc::MiMalloc;
use myth_db::events::{HookEventType, Verdict as DbVerdict};
use myth_hooks::core::session::parse_claude_session_id;
use myth_hooks::{run_hook, HookOutcome, HookPayload, HookResult, PartialHookEvent};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> ExitCode {
    run_hook("post_tool", "myth-hook-post-tool", |envelope| {
        let data = match &envelope.payload {
            HookPayload::PostToolUse(d) => d,
            _ => return Ok(HookResult::Allow.into()),
        };

        let session_id = parse_claude_session_id(&envelope.common.session_id)?;
        let event = PartialHookEvent {
            session_id,
            event_type: HookEventType::PostTool,
            tool_name: Some(data.tool_name.clone()),
            verdict: DbVerdict::Allow,
            lesson_id: None,
        };
        Ok(HookOutcome::from(HookResult::Allow).with_event(event))
    })
}

#[cfg(test)]
mod tests {
    // DB-backed assertions live in tests/post_tool_test.rs. Compile-
    // time unit test only.
    #[test]
    fn compiles() {}
}
