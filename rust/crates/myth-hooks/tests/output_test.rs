//! HookResult → label / is_blocking semantics. Actual stdout/ExitCode
//! round-trip is covered in Task 3.6's subprocess integration tests.

use myth_hooks::HookResult;
use serde_json::json;

#[test]
fn allow_is_non_blocking() {
    let r = HookResult::Allow;
    assert_eq!(r.label(), "allow");
    assert!(!r.is_blocking());
}

#[test]
fn allow_with_context_is_non_blocking() {
    let r = HookResult::AllowWithContext(json!({
        "continue": true,
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "additionalContext": "demo"
        }
    }));
    assert_eq!(r.label(), "allow_with_context");
    assert!(!r.is_blocking());
}

#[test]
fn ask_is_non_blocking() {
    let r = HookResult::Ask(json!({"continue": true}));
    assert_eq!(r.label(), "ask");
    assert!(!r.is_blocking());
}

#[test]
fn block_is_blocking() {
    let r = HookResult::Block {
        output: json!({"continue": false, "stopReason": "demo"}),
    };
    assert_eq!(r.label(), "block");
    assert!(r.is_blocking());
}

#[test]
fn error_is_not_blocking() {
    // Intentional: an internal myth hook error degrades to exit 0 so
    // a myth bug cannot cascade into a blocked Claude Code session.
    let r = HookResult::Error("demo".into());
    assert_eq!(r.label(), "error");
    assert!(!r.is_blocking());
}
