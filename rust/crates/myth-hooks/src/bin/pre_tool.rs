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

use mimalloc::MiMalloc;
use myth_common::Enforcement;
use myth_gavel::{Gavel, ToolInput};
use myth_hooks::core::session::parse_claude_session_id;
use myth_hooks::{run_hook, HookPayload, HookResult};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> ExitCode {
    run_hook("pre_tool", "myth-hook-pre-tool", |envelope| {
        let data = match &envelope.payload {
            HookPayload::PreToolUse(d) => d,
            _ => return Ok(HookResult::Allow),
        };

        let gavel = Gavel::init()?;
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

        let result = match verdict.enforcement {
            Enforcement::Dismiss | Enforcement::Note => HookResult::Allow,
            Enforcement::Advisory | Enforcement::Caution => HookResult::AllowWithContext(json),
            Enforcement::Warn => HookResult::Ask(json),
            Enforcement::Strike | Enforcement::Seal => HookResult::Block { output: json },
        };
        Ok(result)
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
