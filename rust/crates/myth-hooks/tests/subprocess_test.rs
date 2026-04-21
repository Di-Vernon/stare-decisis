//! Task 3.6 — subprocess integration tests (items 1 + 6).
//!
//! Two axes, two test functions:
//!
//! - `test_hook_stdout_schema_per_bin`: one subprocess per bin /
//!   variant, driven by the real fixture envelopes, verifies the
//!   empirical stdout JSON shape + exit code for each bin. Covers the
//!   fact that with an empty rule set + fresh state.db the pre-tool
//!   path emits no stdout, and exercises both the Tier 0 silent path
//!   and the Tier 1 Variant B template path for post-tool-failure.
//!
//! - `test_concurrent_jsonl_append_no_race`: N concurrent
//!   `myth-hook-post-tool-failure` subprocesses on the Tier 1 path,
//!   verifies the three JSONL files (`lesson-state.jsonl`,
//!   `caselog.jsonl`, `metrics/reflector-shadow.jsonl`) each have
//!   exactly N well-formed lines with no byte interleaving and no
//!   duplicated / dropped records.
//!
//! Known coverage gap — Tier 0 concurrent write path: this test
//! covers only Tier 1 (no DB touch). The Tier 0 deterministic path
//! additionally writes to `state.db` via SQLite (WAL mode,
//! `busy_timeout=5000ms` — see `crates/myth-db/src/sqlite/pragmas.rs`).
//! SQLite's own writer serialisation is presumed correct by design
//! but is not exercised here. Revisit during Wave 6 / Wave 7
//! integration validation when end-to-end scenarios drive real
//! concurrent Tier 0 failures through `state.db::hook_events` and
//! `state.db::lessons`.
//!
//! Isolation: each case gets its own `TempDir` bound to the child's
//! HOME; XDG_STATE_HOME / XDG_CONFIG_HOME / XDG_DATA_HOME are cleared
//! so myth_state / myth_config fall through to HOME/.local/state and
//! HOME/.config respectively. CLAUDE_REVIEW_ACTIVE and MYTH_DISABLE
//! are removed so the runner doesn't short-circuit to Allow.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use tempfile::TempDir;

const PRE_TOOL: &str = env!("CARGO_BIN_EXE_myth-hook-pre-tool");
const POST_TOOL: &str = env!("CARGO_BIN_EXE_myth-hook-post-tool");
const POST_TOOL_FAILURE: &str = env!("CARGO_BIN_EXE_myth-hook-post-tool-failure");
const USER_PROMPT: &str = env!("CARGO_BIN_EXE_myth-hook-user-prompt");
const STOP: &str = env!("CARGO_BIN_EXE_myth-hook-stop");
const SESSION_START: &str = env!("CARGO_BIN_EXE_myth-hook-session-start");

struct SubprocOutput {
    code: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn run_bin(bin: &str, envelope: &str, home_dir: &Path) -> SubprocOutput {
    let mut child = Command::new(bin)
        .env("HOME", home_dir)
        .env_remove("XDG_STATE_HOME")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("XDG_DATA_HOME")
        .env_remove("CLAUDE_REVIEW_ACTIVE")
        .env_remove("MYTH_DISABLE")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn hook bin");

    {
        let mut stdin = child.stdin.take().expect("child stdin handle");
        stdin
            .write_all(envelope.as_bytes())
            .expect("write envelope to stdin");
    }

    let output = child.wait_with_output().expect("wait_with_output");
    SubprocOutput {
        code: output.status.code().unwrap_or(-1),
        stdout: output.stdout,
        stderr: output.stderr,
    }
}

fn prep_home(home: &Path) {
    std::fs::create_dir_all(home.join(".myth")).expect("create $HOME/.myth");
}

fn load_fixture(name: &str) -> String {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/envelopes")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read fixture {:?}: {}", path, e))
}

#[test]
fn test_hook_stdout_schema_per_bin() {
    // Each sub-case runs in its own TempDir so state (state.db rows,
    // JSONL writes, brief.md presence) can't cross-contaminate. On
    // failure the panic message names the sub-case; regressions in
    // any one bin stay diagnosable.

    // --- pre_tool: empty rules + fresh state.db → Dismiss → Allow.
    // Day-1 reality: `~/.myth/*-rules.yaml` do not exist in this
    // tempdir so RuleSet::load_all logs warnings and hands Gavel an
    // empty ruleset. No match → Dismiss → HookResult::Allow → no
    // stdout, exit 0.
    {
        let tmp = TempDir::new().unwrap();
        prep_home(tmp.path());
        let env = load_fixture("pre_tool_use.json");
        let out = run_bin(PRE_TOOL, &env, tmp.path());
        assert_eq!(
            out.code,
            0,
            "pre_tool exit: stderr={}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            out.stdout.is_empty(),
            "pre_tool with empty rules must emit no stdout; got: {}",
            String::from_utf8_lossy(&out.stdout)
        );
    }

    // --- post_tool: success event, one hook_events row, Allow, empty
    // stdout. The DB insert is observability-class; its failure
    // would warn-log but still return Allow. Here the path is clean.
    {
        let tmp = TempDir::new().unwrap();
        prep_home(tmp.path());
        let env = load_fixture("post_tool_use.json");
        let out = run_bin(POST_TOOL, &env, tmp.path());
        assert_eq!(out.code, 0, "post_tool exit");
        assert!(
            out.stdout.is_empty(),
            "post_tool stdout must be empty; got: {}",
            String::from_utf8_lossy(&out.stdout)
        );
    }

    // --- post_tool_failure Tier 0 hit: the fixture error contains
    // "No such file or directory" → FILE_NOT_FOUND_RE matches →
    // Tier 0 deterministic path → upsert lesson + record classification
    // → Allow, empty stdout (no Variant B template emitted).
    {
        let tmp = TempDir::new().unwrap();
        prep_home(tmp.path());
        let env = load_fixture("post_tool_use_failure.json");
        let out = run_bin(POST_TOOL_FAILURE, &env, tmp.path());
        assert_eq!(
            out.code,
            0,
            "Tier 0 exit: stderr={}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            out.stdout.is_empty(),
            "Tier 0 hit must not emit Variant B template; got: {}",
            String::from_utf8_lossy(&out.stdout)
        );
    }

    // --- post_tool_failure Tier 1 miss: error text matches no Tier 0
    // pattern (timeout / 429 / rate limit / file-not-found) so the
    // runner falls through to the Variant B template. Stdout must be
    // a JSON object carrying `continue: true`, the correct
    // hookEventName, and an additionalContext string that bears both
    // the "assessor" instruction and the reminder-id tag.
    {
        let tmp = TempDir::new().unwrap();
        prep_home(tmp.path());
        let env = serde_json::json!({
            "session_id": "11111111-0000-4000-8000-000000000001",
            "transcript_path": "/tmp/subproc-test.jsonl",
            "cwd": "/tmp/subproc-test",
            "permission_mode": "default",
            "hook_event_name": "PostToolUseFailure",
            "tool_name": "Bash",
            "tool_input": {"command": "python -c 'assert 2 == 3'"},
            "tool_use_id": "toolu_tier1_variant_b_stdout",
            "error": "Exit code 1\nAssertionError: unclassifiable for tier 0",
            "is_interrupt": false
        })
        .to_string();
        let out = run_bin(POST_TOOL_FAILURE, &env, tmp.path());
        assert_eq!(
            out.code,
            0,
            "Tier 1 exit: stderr={}",
            String::from_utf8_lossy(&out.stderr)
        );
        let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
            panic!(
                "Tier 1 stdout is not valid JSON: {}; bytes: {:?}",
                e,
                String::from_utf8_lossy(&out.stdout)
            )
        });
        assert_eq!(json["continue"], true, "Tier 1: continue must be true");
        assert_eq!(
            json["hookSpecificOutput"]["hookEventName"],
            "PostToolUseFailure",
            "Tier 1: hookEventName mismatch",
        );
        let ctx = json["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .expect("additionalContext must be a string");
        assert!(
            ctx.contains("assessor"),
            "Variant B template must mention the assessor subagent; got: {}",
            ctx
        );
        assert!(
            ctx.contains("assessor-trigger-id"),
            "Variant B template must include the reminder-id tag; got: {}",
            ctx
        );
    }

    // --- user_prompt: Day-1 read-only scan → Allow, empty stdout.
    // The bin only tracing::debug's the line count of
    // lesson-state.jsonl; no stdout, no state mutation.
    {
        let tmp = TempDir::new().unwrap();
        prep_home(tmp.path());
        let env = load_fixture("user_prompt_submit.json");
        let out = run_bin(USER_PROMPT, &env, tmp.path());
        assert_eq!(out.code, 0, "user_prompt exit");
        assert!(out.stdout.is_empty(), "user_prompt stdout must be empty");
    }

    // --- stop: Tier 2 hard-wired off (is_tier2_enabled == false on
    // Day-1) → Allow, empty stdout. The envelope's
    // stop_hook_active=false does not change this outcome.
    {
        let tmp = TempDir::new().unwrap();
        prep_home(tmp.path());
        let env = load_fixture("stop.json");
        let out = run_bin(STOP, &env, tmp.path());
        assert_eq!(out.code, 0, "stop exit");
        assert!(out.stdout.is_empty(), "stop stdout must be empty");
    }

    // --- session_start, no brief.md in HOME → read returns ENOENT,
    // bin falls back to Allow silently. Empty stdout, exit 0.
    {
        let tmp = TempDir::new().unwrap();
        prep_home(tmp.path());
        let env = load_fixture("session_start.json");
        let out = run_bin(SESSION_START, &env, tmp.path());
        assert_eq!(out.code, 0, "session_start no-brief exit");
        assert!(
            out.stdout.is_empty(),
            "session_start without brief.md must emit no stdout; got: {}",
            String::from_utf8_lossy(&out.stdout)
        );
    }

    // --- session_start, brief.md pre-populated → AllowWithContext,
    // stdout JSON carries the brief wrapped in <myth-brief> markers
    // inside hookSpecificOutput.additionalContext.
    {
        let tmp = TempDir::new().unwrap();
        prep_home(tmp.path());
        let brief_body = "# Active lessons\n\n- L-SAMPLE-0001: demo brief body for subprocess test.\n";
        std::fs::write(tmp.path().join(".myth/brief.md"), brief_body)
            .expect("write brief.md");
        let env = load_fixture("session_start.json");
        let out = run_bin(SESSION_START, &env, tmp.path());
        assert_eq!(
            out.code,
            0,
            "session_start with-brief exit: stderr={}",
            String::from_utf8_lossy(&out.stderr)
        );
        let json: serde_json::Value = serde_json::from_slice(&out.stdout)
            .unwrap_or_else(|e| panic!("session_start stdout not JSON: {}", e));
        assert_eq!(json["continue"], true);
        assert_eq!(
            json["hookSpecificOutput"]["hookEventName"],
            "SessionStart"
        );
        let ctx = json["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .expect("additionalContext must be a string");
        assert!(
            ctx.contains("<myth-brief>"),
            "context must be wrapped in <myth-brief> tag; got: {}",
            ctx
        );
        assert!(
            ctx.contains("L-SAMPLE-0001"),
            "context must carry the brief body; got: {}",
            ctx
        );
    }
}

#[test]
fn test_concurrent_jsonl_append_no_race() {
    // Spawn N concurrent post-tool-failure subprocesses on the Tier 1
    // path (error that misses every Tier 0 regex). Tier 1 touches
    // three JSONL files per invocation — lesson-state.jsonl,
    // caselog.jsonl, metrics/reflector-shadow.jsonl — and no SQLite,
    // so the test observes pure JSONL append contention.
    //
    // `JsonlWriter::append` opens the file per record, acquires an
    // exclusive fs2 advisory lock (`fcntl` flock on Linux), writes
    // the line, flushes, and releases the lock. The flock window
    // spans exactly one line per subprocess, so N concurrent bins
    // must produce N well-formed lines per file with no interleaving
    // and no duplicate or dropped records.

    const N: usize = 8;
    let tmp = TempDir::new().unwrap();
    prep_home(tmp.path());

    let envelopes: Vec<(String, String)> = (0..N)
        .map(|i| {
            let tool_use_id = format!("toolu_concurrent_{:04}", i);
            // Synthetic UUID v4 — version nibble 4 + variant bits 10,
            // otherwise zeroed out with the index salted at both ends.
            let session = format!(
                "{:08x}-0000-4000-8000-{:012x}",
                i as u32, i as u64
            );
            let env = serde_json::json!({
                "session_id": session,
                "transcript_path": "/tmp/concurrent-test.jsonl",
                "cwd": "/tmp/concurrent-test",
                "permission_mode": "default",
                "hook_event_name": "PostToolUseFailure",
                "tool_name": "Bash",
                "tool_input": {"command": format!("python -c 'assert {} == {}'", i, i + 1)},
                "tool_use_id": tool_use_id.clone(),
                "error": "Exit code 1\nAssertionError: concurrent-tier-1-case",
                "is_interrupt": false
            })
            .to_string();
            (tool_use_id, env)
        })
        .collect();

    // Launch every subprocess before waiting on any one so the
    // append windows overlap in wall clock time. Each child reads
    // stdin, classifies (Tier 0 miss), then performs three JSONL
    // appends — the lock contention window is the whole point of
    // the test.
    let mut children: Vec<std::process::Child> = Vec::with_capacity(N);
    for (_tuid, env) in &envelopes {
        let mut child = Command::new(POST_TOOL_FAILURE)
            .env("HOME", tmp.path())
            .env_remove("XDG_STATE_HOME")
            .env_remove("XDG_CONFIG_HOME")
            .env_remove("XDG_DATA_HOME")
            .env_remove("CLAUDE_REVIEW_ACTIVE")
            .env_remove("MYTH_DISABLE")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn post_tool_failure");
        {
            let mut stdin = child.stdin.take().expect("child stdin handle");
            stdin
                .write_all(env.as_bytes())
                .expect("write envelope to stdin");
        }
        children.push(child);
    }

    for child in children {
        let out = child.wait_with_output().expect("wait_with_output");
        assert_eq!(
            out.status.code(),
            Some(0),
            "concurrent child failed. stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let myth_home = tmp.path().join(".myth");
    let lesson_state = myth_home.join("lesson-state.jsonl");
    let caselog = myth_home.join("caselog.jsonl");
    let shadow = myth_home.join("metrics/reflector-shadow.jsonl");

    // Invariant 1: each file has exactly N lines, and each line is a
    // syntactically valid JSON object. Interleaved bytes from two
    // concurrent writes would land here as a parse error.
    for path in [&lesson_state, &caselog, &shadow] {
        let raw = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("read {:?}: {}", path, e));
        let lines: Vec<&str> = raw.lines().collect();
        assert_eq!(
            lines.len(),
            N,
            "{:?} should have {} lines, got {}",
            path,
            N,
            lines.len()
        );
        for line in &lines {
            let _v: serde_json::Value = serde_json::from_str(line).unwrap_or_else(|e| {
                panic!(
                    "interleaved / corrupt JSONL line in {:?}: {}; line: {}",
                    path, e, line
                )
            });
        }
    }

    // Invariant 2: lesson-state.jsonl carries each subprocess's
    // tool_use_id exactly once. This catches silent duplication or a
    // dropped record that wouldn't surface as a line-count mismatch
    // alone.
    let collected: Vec<String> = std::fs::read_to_string(&lesson_state)
        .unwrap()
        .lines()
        .map(|line| {
            let v: serde_json::Value = serde_json::from_str(line).unwrap();
            v["tool_use_id"].as_str().unwrap().to_string()
        })
        .collect();

    let expected_ids: std::collections::HashSet<String> =
        envelopes.iter().map(|(t, _)| t.clone()).collect();
    let got_ids: std::collections::HashSet<String> = collected.iter().cloned().collect();
    assert_eq!(
        got_ids, expected_ids,
        "tool_use_id set mismatch between input and lesson-state"
    );
    assert_eq!(
        got_ids.len(),
        N,
        "expected {} distinct tool_use_ids, got {}: {:?}",
        N,
        got_ids.len(),
        collected,
    );
}
