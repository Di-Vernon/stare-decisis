//! UserPromptSubmit hook — Day-1 read-only scan of
//! `~/.myth/lesson-state.jsonl`.
//!
//! Full compliance checking (pending_reflection matching against
//! transcript tool_uses) is deferred; see 392c8c9
//! (docs(crates): user-prompt Day-1 simplification).

use std::process::ExitCode;

use mimalloc::MiMalloc;
use myth_db::JsonlWriter;
use myth_hooks::{run_hook, HookPayload, HookResult};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> ExitCode {
    run_hook(
        "user_prompt_submit",
        "myth-hook-user-prompt",
        |envelope| {
            if !matches!(envelope.payload, HookPayload::UserPromptSubmit(_)) {
                return Ok(HookResult::Allow);
            }

            let path = myth_common::lesson_state_path();
            if path.exists() {
                let writer = JsonlWriter::new(&path);
                let count = writer.count_lines().unwrap_or(0);
                tracing::debug!(
                    lesson_state_lines = count,
                    "user-prompt scan (Day-1: read-only)"
                );
            }

            Ok(HookResult::Allow)
        },
    )
}

#[cfg(test)]
mod tests {
    // Day-1 bin has no internal helpers to unit-test. Real
    // compliance-check logic (transcript parse + tool_use matching)
    // lands with Task 3.5's pending_reflection writer. The
    // subprocess round-trip is covered by Task 3.6 integration.
    #[test]
    fn compiles() {}
}
