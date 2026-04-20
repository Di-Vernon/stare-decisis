//! SessionStart hook — inject `~/.myth/brief.md` as
//! `additionalContext` on every new Claude Code session.

use std::process::ExitCode;

use mimalloc::MiMalloc;
use myth_hooks::{run_hook, HookPayload, HookResult};
use serde_json::json;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const BRIEF_MAX_BYTES: usize = 10_000;
const TRUNCATE_SUFFIX: &str = "\n\n... (truncated, see ~/.myth/brief.md)";

/// Truncate `brief` to at most `max_bytes`, appending TRUNCATE_SUFFIX
/// when we cut. Backs up to the nearest UTF-8 boundary so the
/// resulting String is always valid UTF-8 even when the cut falls
/// mid-codepoint (e.g. Korean/Japanese content).
fn truncate_brief(brief: &str, max_bytes: usize) -> String {
    if brief.len() <= max_bytes {
        return brief.to_string();
    }
    let limit = max_bytes.saturating_sub(TRUNCATE_SUFFIX.len());
    let boundary = (0..=limit)
        .rev()
        .find(|&i| brief.is_char_boundary(i))
        .unwrap_or(0);
    let mut out = brief[..boundary].to_string();
    out.push_str(TRUNCATE_SUFFIX);
    out
}

fn main() -> ExitCode {
    run_hook("session_start", "myth-hook-session-start", |envelope| {
        if !matches!(envelope.payload, HookPayload::SessionStart(_)) {
            return Ok(HookResult::Allow);
        }

        let brief_path = myth_common::brief_path();
        // brief read failure is fire-and-forget — session opens with
        // a plain allow rather than failing the new-session event.
        let brief = match std::fs::read_to_string(&brief_path) {
            Ok(b) => b,
            Err(_) => return Ok(HookResult::Allow),
        };

        if brief.trim().is_empty() {
            return Ok(HookResult::Allow);
        }

        let injected = truncate_brief(&brief, BRIEF_MAX_BYTES);
        let context = format!("<myth-brief>\n{}\n</myth-brief>", injected);

        Ok(HookResult::AllowWithContext(json!({
            "continue": true,
            "hookSpecificOutput": {
                "hookEventName": "SessionStart",
                "additionalContext": context,
            }
        })))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_brief_passes_through_unchanged() {
        let s = "short brief content";
        assert_eq!(truncate_brief(s, 10_000), s);
    }

    #[test]
    fn long_ascii_brief_is_truncated_with_suffix() {
        let s = "x".repeat(15_000);
        let out = truncate_brief(&s, 10_000);
        assert!(out.ends_with(TRUNCATE_SUFFIX));
        assert!(out.len() <= 10_000);
    }

    #[test]
    fn truncate_does_not_panic_on_multibyte_boundary() {
        // "한글" is 6 bytes; repeating it fills multi-byte slots that
        // cross the naive byte cut. Rust Strings are always valid
        // UTF-8, so simply reaching the end of `truncate_brief` with
        // no panic already proves the boundary backoff is correct.
        let s = "한글".repeat(3_000);
        let out = truncate_brief(&s, 5_000);
        assert!(out.ends_with(TRUNCATE_SUFFIX));
        assert!(out.len() <= 5_000);
    }
}
