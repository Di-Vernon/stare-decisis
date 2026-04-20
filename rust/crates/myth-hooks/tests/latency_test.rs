//! Latency recording — including the fire-and-forget degrade path.

use myth_hooks::core::latency::{record_ignore_err_to, record_to};
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn record_to_writes_a_json_line() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("hook-latency.ndjson");
    record_to(&path, "pre_tool", 2.5, "allow").unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    assert_eq!(content.lines().count(), 1);
    assert!(content.contains("\"event\":\"pre_tool\""));
    assert!(content.contains("\"latency_ms\":2.5"));
    assert!(content.contains("\"result\":\"allow\""));
}

#[test]
fn record_to_appends_multiple_lines() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("hook-latency.ndjson");
    record_to(&path, "pre_tool", 1.0, "allow").unwrap();
    record_to(&path, "post_tool", 0.5, "allow").unwrap();
    record_to(&path, "pre_tool", 2.0, "block").unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    assert_eq!(content.lines().count(), 3);
}

#[test]
fn record_ignore_err_swallows_write_failure() {
    // An unreachable path must not propagate an error — hook
    // correctness does not depend on the observability path.
    let unreachable = PathBuf::from("/proc/cannot/write/here/latency.ndjson");

    // The call should simply return; any internal error becomes a
    // stderr warning rather than propagating.
    record_ignore_err_to(&unreachable, "pre_tool", 1.0, "allow");

    // And demonstrate that record_to itself would have erred, so we
    // know the degrade path isn't silently succeeding.
    let err = record_to(&unreachable, "pre_tool", 1.0, "allow");
    assert!(
        err.is_err(),
        "expected record_to on an unreachable path to surface an error"
    );
}

#[test]
fn record_to_creates_parent_directories() {
    let dir = tempdir().unwrap();
    // Target path with a non-existent parent dir — JsonlWriter
    // handles mkdir -p.
    let nested = dir.path().join("nested").join("dir").join("latency.ndjson");
    record_to(&nested, "session_start", 0.3, "allow").unwrap();
    assert!(nested.exists());
}
