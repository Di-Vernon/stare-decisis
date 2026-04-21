//! Task 3.6 Step c — `hook_events.latency_ms` wire-through validation.
//!
//! Prior to Step c, `post_tool.rs` hard-coded `latency_ms: 0.0` when
//! inserting into `state.db::hook_events`, and neither `pre_tool` nor
//! `post_tool_failure` inserted into the table at all. Step c moves
//! the insert into `run_hook` so the canonical `start.elapsed()`
//! measurement (the same value written to `hook-latency.ndjson`) is
//! stamped onto the DB row. This test file exercises each of the
//! three wired bins as a real subprocess with an isolated `HOME`,
//! then opens the resulting `state.db` and asserts:
//!
//! 1. Exactly one `hook_events` row exists with the expected
//!    `event_type` and `tool_name`.
//! 2. `latency_ms > 0.0` (proves the 0.0 placeholder is gone).
//! 3. `latency_ms < 200.0` (sanity bound — debug builds and WSL2 I/O
//!    variability mean a tight upper bound is unsafe, but a row that
//!    took longer than 200 ms almost certainly indicates the
//!    measurement is pulling from the wrong clock).
//!
//! Together with the unchanged `test_hook_stdout_schema_per_bin`,
//! these prove that wire-through added DB observability **without**
//! altering user-visible stdout/exit behaviour.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use myth_db::Database;
use tempfile::TempDir;

const PRE_TOOL: &str = env!("CARGO_BIN_EXE_myth-hook-pre-tool");
const POST_TOOL: &str = env!("CARGO_BIN_EXE_myth-hook-post-tool");
const POST_TOOL_FAILURE: &str = env!("CARGO_BIN_EXE_myth-hook-post-tool-failure");

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

fn run_bin(bin: &str, envelope: &str, home_dir: &Path) -> (i32, Vec<u8>) {
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
    let out = child.wait_with_output().expect("wait_with_output");
    (out.status.code().unwrap_or(-1), out.stderr)
}

/// Open the tempdir's state.db and query the single expected
/// hook_events row. Panics (with diagnostic) if row count != 1 or any
/// expected column mismatches.
fn assert_single_event(
    home: &Path,
    expected_event_type: &str,
    expected_tool_name: &str,
    context: &str,
) -> f64 {
    let db_path = home.join(".myth/state.db");
    assert!(
        db_path.exists(),
        "[{}] state.db not created at {:?}",
        context,
        db_path
    );
    let db = Database::open(&db_path).expect("open state.db");

    let row_count: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM hook_events", [], |r| r.get(0))
        .expect("count hook_events");
    assert_eq!(
        row_count, 1,
        "[{}] expected exactly one hook_events row, got {}",
        context, row_count
    );

    let (event_type, tool_name, latency_ms, verdict): (String, String, f64, String) = db
        .conn
        .query_row(
            "SELECT event_type, tool_name, latency_ms, verdict \
             FROM hook_events LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .expect("read hook_events row");

    assert_eq!(
        event_type, expected_event_type,
        "[{}] event_type mismatch",
        context
    );
    assert_eq!(
        tool_name, expected_tool_name,
        "[{}] tool_name mismatch",
        context
    );
    assert!(
        latency_ms > 0.0,
        "[{}] latency_ms must be > 0 after Step c wire-through (got {})",
        context,
        latency_ms
    );
    assert!(
        latency_ms < 200.0,
        "[{}] latency_ms unreasonably high ({} ms); runner clock likely wrong",
        context,
        latency_ms
    );
    // verdict is just sanity-checked for presence — the bin-specific
    // expectation is asserted by the caller if it matters.
    assert!(
        matches!(verdict.as_str(), "allow" | "ask" | "deny"),
        "[{}] unknown verdict string: {:?}",
        context,
        verdict
    );
    latency_ms
}

#[test]
fn pre_tool_writes_hook_event_with_positive_latency() {
    let tmp = TempDir::new().unwrap();
    prep_home(tmp.path());
    let envelope = load_fixture("pre_tool_use.json");

    let (code, stderr) = run_bin(PRE_TOOL, &envelope, tmp.path());
    assert_eq!(
        code,
        0,
        "pre_tool exit; stderr: {}",
        String::from_utf8_lossy(&stderr)
    );

    let latency = assert_single_event(tmp.path(), "pre_tool", "Bash", "pre_tool");
    assert!(
        latency > 0.0,
        "pre_tool latency_ms must be > 0 (got {})",
        latency
    );
}

#[test]
fn post_tool_writes_hook_event_with_positive_latency() {
    let tmp = TempDir::new().unwrap();
    prep_home(tmp.path());
    let envelope = load_fixture("post_tool_use.json");

    let (code, stderr) = run_bin(POST_TOOL, &envelope, tmp.path());
    assert_eq!(
        code,
        0,
        "post_tool exit; stderr: {}",
        String::from_utf8_lossy(&stderr)
    );

    let latency = assert_single_event(tmp.path(), "post_tool", "Bash", "post_tool");
    assert!(
        latency > 0.0,
        "post_tool latency_ms must be > 0 (got {})",
        latency
    );
}

#[test]
fn post_tool_failure_tier0_writes_hook_event_with_positive_latency() {
    // The probe fixture's error contains "No such file or directory"
    // which matches FILE_NOT_FOUND_RE → Tier 0 deterministic path.
    let tmp = TempDir::new().unwrap();
    prep_home(tmp.path());
    let envelope = load_fixture("post_tool_use_failure.json");

    let (code, stderr) = run_bin(POST_TOOL_FAILURE, &envelope, tmp.path());
    assert_eq!(
        code,
        0,
        "post_tool_failure Tier 0 exit; stderr: {}",
        String::from_utf8_lossy(&stderr)
    );

    let latency =
        assert_single_event(tmp.path(), "post_tool_failure", "Bash", "tier0");
    assert!(
        latency > 0.0,
        "Tier 0 latency_ms must be > 0 (got {})",
        latency
    );
}

#[test]
fn post_tool_failure_tier1_writes_hook_event_with_positive_latency() {
    // AssertionError text dodges every Tier 0 regex → Tier 1 path.
    let tmp = TempDir::new().unwrap();
    prep_home(tmp.path());
    let envelope = serde_json::json!({
        "session_id": "22222222-0000-4000-8000-000000000002",
        "transcript_path": "/tmp/wire-tier1.jsonl",
        "cwd": "/tmp/wire-tier1",
        "permission_mode": "default",
        "hook_event_name": "PostToolUseFailure",
        "tool_name": "Bash",
        "tool_input": {"command": "python -c 'assert 2 == 3'"},
        "tool_use_id": "toolu_wire_tier1",
        "error": "Exit code 1\nAssertionError: unclassifiable",
        "is_interrupt": false
    })
    .to_string();

    let (code, stderr) = run_bin(POST_TOOL_FAILURE, &envelope, tmp.path());
    assert_eq!(
        code,
        0,
        "post_tool_failure Tier 1 exit; stderr: {}",
        String::from_utf8_lossy(&stderr)
    );

    let latency =
        assert_single_event(tmp.path(), "post_tool_failure", "Bash", "tier1");
    assert!(
        latency > 0.0,
        "Tier 1 latency_ms must be > 0 (got {})",
        latency
    );
}
