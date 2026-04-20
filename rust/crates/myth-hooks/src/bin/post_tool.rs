//! PostToolUse hook — record a success event in `state.db::hook_events`
//! and return Allow. Failure is exclusive to PostToolUseFailure, so
//! seeing this bin fire means the tool succeeded.

use std::process::ExitCode;

use anyhow::Context;
use mimalloc::MiMalloc;
use myth_db::events::{self, HookEvent, HookEventType, Verdict};
use myth_db::Database;
use myth_hooks::core::session::parse_claude_session_id;
use myth_hooks::{run_hook, HookPayload, HookResult};
use uuid::Uuid;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> ExitCode {
    run_hook("post_tool", "myth-hook-post-tool", |envelope| {
        let data = match &envelope.payload {
            HookPayload::PostToolUse(d) => d,
            _ => return Ok(HookResult::Allow),
        };

        // Observability-class write. A DB failure must not cascade
        // into a blocked tool result — record a warning and continue.
        if let Err(e) = record_event(&envelope.common.session_id, &data.tool_name) {
            tracing::warn!(error = %e, "hook_events append failed");
        }

        Ok(HookResult::Allow)
    })
}

fn record_event(session_id_str: &str, tool_name: &str) -> anyhow::Result<()> {
    let db = Database::open(&myth_common::state_db_path())
        .context("opening state.db")?;
    let session_id =
        parse_claude_session_id(session_id_str).context("parsing session_id")?;

    let event = HookEvent {
        id: Uuid::new_v4(),
        session_id,
        event_type: HookEventType::PostTool,
        tool_name: Some(tool_name.to_string()),
        ts: myth_common::now(),
        // Day-1 note: per-hook latency lives in hook-latency.ndjson
        // (written by the runner). The DB column stays at 0.0 here
        // until Task 3.6 wires the runner's elapsed measurement
        // through to the DB insert.
        latency_ms: 0.0,
        verdict: Verdict::Allow,
        lesson_id: None,
    };
    events::insert(&db, &event)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    // DB-backed assertions live in tests/post_tool_test.rs. Compile-
    // time unit test only.
    #[test]
    fn compiles() {}
}
