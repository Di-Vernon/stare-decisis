//! End-to-end file-based tests for `myth link` / `myth unlink`.
//!
//! Exercises the public I/O wrappers (`run_link` / `run_unlink`) against
//! a tempdir project so we catch serialisation, atomic-write, and
//! backup-file behaviour the pure-fn unit tests in `link.rs` can't see.

use myth_cli::subcmd::link::{run_link, run_unlink};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn read_json(path: &Path) -> Value {
    let s = fs::read_to_string(path).unwrap();
    serde_json::from_str(&s).unwrap()
}

fn setup_project(dir: &Path, settings_name: &str, initial: Value) -> PathBuf {
    let claude_dir = dir.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();
    let file = claude_dir.join(settings_name);
    fs::write(&file, serde_json::to_string_pretty(&initial).unwrap()).unwrap();
    file
}

#[test]
fn test_link_creates_all_six_hooks() {
    let tmp = tempdir().unwrap();
    let file = setup_project(tmp.path(), "settings.json", json!({}));

    run_link(tmp.path()).unwrap();

    let result = read_json(&file);
    let hooks = result.get("hooks").and_then(|v| v.as_object()).unwrap();
    assert_eq!(hooks.len(), 6);

    let expected = [
        ("PreToolUse", "myth-hook-pre-tool"),
        ("PostToolUse", "myth-hook-post-tool"),
        ("PostToolUseFailure", "myth-hook-post-tool-failure"),
        ("SessionStart", "myth-hook-session-start"),
        ("UserPromptSubmit", "myth-hook-user-prompt"),
        ("Stop", "myth-hook-stop"),
    ];
    for (event, binary) in expected {
        let arr = hooks
            .get(event)
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| panic!("missing event {event}"));
        assert_eq!(arr.len(), 1, "{event} should have exactly one entry");
        let cmd = arr[0]
            .pointer("/hooks/0/command")
            .and_then(|c| c.as_str())
            .unwrap_or_else(|| panic!("missing command under {event}"));
        assert_eq!(cmd, binary, "{event}: expected {binary}, got {cmd}");
    }
}

#[test]
fn test_link_preserves_existing_entries() {
    let tmp = tempdir().unwrap();
    let initial = json!({
        "permissions": { "allow": ["Bash(ls)"] },
        "hooks": {
            "PreToolUse": [
                { "hooks": [{ "type": "command", "command": "third-party-linter" }] }
            ],
            "PostToolUse": [
                { "matcher": "Write|Edit", "hooks": [{ "type": "command", "command": "formatter.sh" }] }
            ]
        }
    });
    let file = setup_project(tmp.path(), "settings.json", initial);

    run_link(tmp.path()).unwrap();

    let result = read_json(&file);

    // Top-level non-hooks key preserved.
    assert_eq!(
        result.pointer("/permissions/allow/0").and_then(|v| v.as_str()),
        Some("Bash(ls)")
    );

    // PreToolUse: original third-party entry + myth entry (append, not replace).
    let pre = result
        .pointer("/hooks/PreToolUse")
        .and_then(|v| v.as_array())
        .unwrap();
    assert_eq!(pre.len(), 2, "PreToolUse should have 2 entries");
    assert_eq!(
        pre[0].pointer("/hooks/0/command").and_then(|v| v.as_str()),
        Some("third-party-linter"),
        "third-party entry must remain first"
    );
    assert_eq!(
        pre[1].pointer("/hooks/0/command").and_then(|v| v.as_str()),
        Some("myth-hook-pre-tool"),
        "myth entry must be appended last"
    );

    // PostToolUse: matcher entry preserved verbatim + myth appended.
    let post = result
        .pointer("/hooks/PostToolUse")
        .and_then(|v| v.as_array())
        .unwrap();
    assert_eq!(post.len(), 2);
    assert_eq!(
        post[0].pointer("/matcher").and_then(|v| v.as_str()),
        Some("Write|Edit")
    );
    assert_eq!(
        post[0].pointer("/hooks/0/command").and_then(|v| v.as_str()),
        Some("formatter.sh")
    );
}

#[test]
fn test_link_is_idempotent() {
    let tmp = tempdir().unwrap();
    let file = setup_project(tmp.path(), "settings.json", json!({}));

    run_link(tmp.path()).unwrap();
    let first = read_json(&file);
    run_link(tmp.path()).unwrap();
    let second = read_json(&file);

    assert_eq!(first, second, "second link should not mutate JSON content");

    // No duplicates: each event still has exactly one entry.
    for event in [
        "PreToolUse",
        "PostToolUse",
        "PostToolUseFailure",
        "SessionStart",
        "UserPromptSubmit",
        "Stop",
    ] {
        let arr = second
            .pointer(&format!("/hooks/{event}"))
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| panic!("missing {event}"));
        assert_eq!(arr.len(), 1, "{event} duplicated on re-link");
    }
}

#[test]
fn test_unlink_removes_only_myth_entries() {
    let tmp = tempdir().unwrap();
    let file = setup_project(tmp.path(), "settings.json", json!({}));

    run_link(tmp.path()).unwrap();
    run_unlink(tmp.path()).unwrap();

    let result = read_json(&file);
    // `hooks` object may still exist but contains no myth commands.
    if let Some(hooks) = result.get("hooks").and_then(|v| v.as_object()) {
        for (_, arr_val) in hooks {
            let arr = arr_val.as_array().unwrap();
            for entry in arr {
                if let Some(inner) = entry.get("hooks").and_then(|v| v.as_array()) {
                    for h in inner {
                        let cmd = h.get("command").and_then(|c| c.as_str()).unwrap_or("");
                        assert!(
                            !cmd.starts_with("myth-hook-"),
                            "myth entry survived unlink: {cmd}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_unlink_preserves_non_myth_tool_entries() {
    let tmp = tempdir().unwrap();
    let initial = json!({
        "permissions": { "allow": [] },
        "hooks": {
            "PreToolUse": [
                { "hooks": [{ "type": "command", "command": "third-party-linter" }] }
            ],
            "Stop": [
                { "hooks": [{ "type": "command", "command": "review-loop.sh" }] }
            ]
        }
    });
    let file = setup_project(tmp.path(), "settings.json", initial);

    run_link(tmp.path()).unwrap();
    run_unlink(tmp.path()).unwrap();

    let result = read_json(&file);

    // Third-party entries survive the round trip.
    let pre = result
        .pointer("/hooks/PreToolUse")
        .and_then(|v| v.as_array())
        .unwrap();
    assert_eq!(pre.len(), 1);
    assert_eq!(
        pre[0].pointer("/hooks/0/command").and_then(|v| v.as_str()),
        Some("third-party-linter")
    );

    let stop = result
        .pointer("/hooks/Stop")
        .and_then(|v| v.as_array())
        .unwrap();
    assert_eq!(stop.len(), 1);
    assert_eq!(
        stop[0].pointer("/hooks/0/command").and_then(|v| v.as_str()),
        Some("review-loop.sh")
    );

    // Top-level non-hooks key preserved.
    assert!(result.get("permissions").is_some());

    // Events that only held myth entries are gone; nothing extra introduced.
    let hooks = result.get("hooks").and_then(|v| v.as_object()).unwrap();
    assert!(hooks.get("PostToolUse").is_none());
    assert!(hooks.get("PostToolUseFailure").is_none());
    assert!(hooks.get("SessionStart").is_none());
    assert!(hooks.get("UserPromptSubmit").is_none());
}

#[test]
fn test_link_prefers_settings_local_json() {
    let tmp = tempdir().unwrap();
    setup_project(tmp.path(), "settings.json", json!({ "from": "main" }));
    let local = setup_project(
        tmp.path(),
        "settings.local.json",
        json!({ "from": "local" }),
    );

    run_link(tmp.path()).unwrap();

    let local_result = read_json(&local);
    assert!(
        local_result.get("hooks").is_some(),
        "local settings should have been modified"
    );
    assert_eq!(
        local_result.get("from").and_then(|v| v.as_str()),
        Some("local"),
        "local scalar field preserved"
    );

    // main settings.json untouched.
    let main_result = read_json(&tmp.path().join(".claude/settings.json"));
    assert!(
        main_result.get("hooks").is_none(),
        "main settings must not be modified when local exists"
    );
    assert_eq!(
        main_result.get("from").and_then(|v| v.as_str()),
        Some("main")
    );
}

#[test]
fn test_link_creates_backup() {
    let tmp = tempdir().unwrap();
    let file = setup_project(
        tmp.path(),
        "settings.json",
        json!({ "sentinel": "original" }),
    );

    run_link(tmp.path()).unwrap();

    let claude_dir = tmp.path().join(".claude");
    let backups: Vec<_> = fs::read_dir(&claude_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|n| n.contains(".pre-myth-"))
                .unwrap_or(false)
        })
        .collect();
    assert_eq!(backups.len(), 1, "exactly one backup should be created");

    // Backup contents == original pre-link file.
    let backup_content: Value =
        serde_json::from_str(&fs::read_to_string(backups[0].path()).unwrap()).unwrap();
    assert_eq!(
        backup_content.get("sentinel").and_then(|v| v.as_str()),
        Some("original")
    );
    assert!(backup_content.get("hooks").is_none());

    // Confirm live file has myth hooks now.
    let live = read_json(&file);
    assert!(live.get("hooks").is_some());
}

#[test]
fn test_link_fails_without_claude_dir() {
    let tmp = tempdir().unwrap();
    // No `.claude/` subdirectory created.
    let err = run_link(tmp.path()).unwrap_err();
    let msg = format!("{err:#}");
    assert!(
        msg.contains("myth init") || msg.contains(".claude"),
        "error should guide user: {msg}"
    );
}
