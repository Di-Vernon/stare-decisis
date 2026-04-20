//! Envelope parsing against the real Claude Code 2.1.114 fixtures
//! captured at 2026-04-21 (see 823e749). These fixtures are the
//! authoritative schema contract — if Claude Code changes the shape
//! in a future release the re-probe produces fresh fixtures and
//! these tests update accordingly.

use std::path::Path;

use myth_hooks::{
    parse_envelope, HookEventName, HookPayload,
};

fn load_fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("envelopes")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read fixture {:?}: {}", path, e))
}

#[test]
fn parses_session_start_envelope() {
    let env = parse_envelope(&load_fixture("session_start.json")).unwrap();
    assert_eq!(env.common.hook_event_name, HookEventName::SessionStart);
    assert_eq!(env.common.cwd, Path::new("/tmp/myth-hook-probe"));
    // SessionStart carries no permission_mode in the runtime probe.
    assert!(env.common.permission_mode.is_none());
    assert!(!env.common.stop_hook_active);

    match env.payload {
        HookPayload::SessionStart(data) => assert_eq!(data.source, "startup"),
        other => panic!("wrong payload variant: {:?}", other),
    }
}

#[test]
fn parses_user_prompt_submit_envelope() {
    let env = parse_envelope(&load_fixture("user_prompt_submit.json")).unwrap();
    assert_eq!(env.common.hook_event_name, HookEventName::UserPromptSubmit);
    assert_eq!(
        env.common.permission_mode.as_deref(),
        Some("bypassPermissions")
    );

    match env.payload {
        HookPayload::UserPromptSubmit(data) => {
            assert!(data.prompt.contains("cat /nonexistent_file_for_probe_v2"));
        }
        other => panic!("wrong payload variant: {:?}", other),
    }
}

#[test]
fn parses_pre_tool_use_envelope() {
    let env = parse_envelope(&load_fixture("pre_tool_use.json")).unwrap();
    assert_eq!(env.common.hook_event_name, HookEventName::PreToolUse);
    match env.payload {
        HookPayload::PreToolUse(data) => {
            assert_eq!(data.tool_name, "Bash");
            assert!(data.tool_use_id.starts_with("toolu_"));
            assert_eq!(
                data.tool_input.get("command").and_then(|v| v.as_str()),
                Some("cat /nonexistent_file_for_probe_v2")
            );
        }
        other => panic!("wrong payload variant: {:?}", other),
    }
}

#[test]
fn parses_post_tool_use_envelope() {
    let env = parse_envelope(&load_fixture("post_tool_use.json")).unwrap();
    assert_eq!(env.common.hook_event_name, HookEventName::PostToolUse);
    match env.payload {
        HookPayload::PostToolUse(data) => {
            assert_eq!(data.tool_name, "Bash");
            assert_eq!(data.tool_response.stdout, "fixture-capture");
            assert_eq!(data.tool_response.stderr, "");
            assert!(!data.tool_response.interrupted);
            assert!(!data.tool_response.is_image);
            assert!(!data.tool_response.no_output_expected);
        }
        other => panic!("wrong payload variant: {:?}", other),
    }
}

#[test]
fn parses_post_tool_use_failure_envelope() {
    // This is the H5 schema that drifted most from the original
    // draft. `error` is a multi-line string; `tool_response` is gone.
    let env = parse_envelope(&load_fixture("post_tool_use_failure.json")).unwrap();
    assert_eq!(
        env.common.hook_event_name,
        HookEventName::PostToolUseFailure
    );
    match env.payload {
        HookPayload::PostToolUseFailure(data) => {
            assert_eq!(data.tool_name, "Bash");
            assert!(data.error.starts_with("Exit code 1"));
            assert!(data.error.contains("No such file or directory"));
            assert!(!data.is_interrupt);
            assert_eq!(data.parse_exit_code(), Some(1));
        }
        other => panic!("wrong payload variant: {:?}", other),
    }
}

#[test]
fn parses_stop_envelope() {
    let env = parse_envelope(&load_fixture("stop.json")).unwrap();
    assert_eq!(env.common.hook_event_name, HookEventName::Stop);
    // Stop is the only event carrying stop_hook_active in the
    // runtime; it's false on the first Stop of a turn.
    assert!(!env.common.stop_hook_active);

    match env.payload {
        HookPayload::Stop(data) => {
            let msg = data
                .last_assistant_message
                .as_deref()
                .expect("last_assistant_message present in fixture");
            assert!(msg.contains("cat"));
        }
        other => panic!("wrong payload variant: {:?}", other),
    }
}

#[test]
fn parse_exit_code_returns_none_for_non_bash_errors() {
    use myth_hooks::PostToolUseFailureData;

    let data = PostToolUseFailureData {
        tool_name: "Edit".into(),
        tool_input: serde_json::json!({"file": "/x"}),
        tool_use_id: "toolu_x".into(),
        error: "File has not been read yet".into(),
        is_interrupt: false,
    };
    assert_eq!(data.parse_exit_code(), None);
}

#[test]
fn parse_exit_code_handles_interrupted() {
    use myth_hooks::PostToolUseFailureData;

    let data = PostToolUseFailureData {
        tool_name: "Bash".into(),
        tool_input: serde_json::json!({"command": "sleep 60"}),
        tool_use_id: "toolu_y".into(),
        error: "Command was interrupted".into(),
        is_interrupt: true,
    };
    assert_eq!(data.parse_exit_code(), None);
    assert!(data.is_interrupt);
}

#[test]
fn rejects_malformed_envelope() {
    let bad = r#"{"hook_event_name": "PreToolUse"}"#; // missing required fields
    assert!(parse_envelope(bad).is_err());
}

#[test]
fn rejects_unknown_event_name() {
    let bad = r#"{
        "session_id": "00000000-0000-0000-0000-000000000000",
        "transcript_path": "/x",
        "cwd": "/x",
        "hook_event_name": "FutureEventNotInSpec"
    }"#;
    assert!(parse_envelope(bad).is_err());
}
