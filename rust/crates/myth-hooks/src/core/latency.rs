//! Per-hook latency recording to
//! `~/.local/state/myth/hook-latency.ndjson`.
//!
//! This is **observability**, not a correctness dependency. The
//! `record_ignore_err` wrapper deliberately swallows I/O failures
//! (disk full, permission drop, missing parent directory on a broken
//! install) with a stderr warning. A myth hook must never block
//! Claude Code because the latency file couldn't be appended to.
//!
//! Saltzer & Schroeder's fail-safe default (deny on failure) does not
//! apply here — this is an observational path, not a security gate.

use std::path::Path;

use serde::Serialize;

#[derive(Debug, Serialize)]
struct LatencyRecord {
    ts: String,
    event: String,
    latency_ms: f64,
    result: String,
}

/// Underlying record-to-path helper. Split out so tests can aim at a
/// tempdir-backed path without touching the real XDG state dir.
pub fn record_to(
    path: &Path,
    event: &str,
    latency_ms: f64,
    result: &str,
) -> anyhow::Result<()> {
    let rec = LatencyRecord {
        ts: myth_common::format_iso(&myth_common::now()),
        event: event.to_string(),
        latency_ms,
        result: result.to_string(),
    };
    myth_db::JsonlWriter::new(path).append(&rec)
}

pub fn record(event: &str, latency_ms: f64, result: &str) -> anyhow::Result<()> {
    record_to(&myth_common::hook_latency_path(), event, latency_ms, result)
}

/// Fire-and-forget — logs any append failure to stderr and returns.
/// Observability failures must never cascade into a blocked hook.
pub fn record_ignore_err(event: &str, latency_ms: f64, result: &str) {
    if let Err(e) = record(event, latency_ms, result) {
        eprintln!("myth: latency record failed: {:#}", e);
    }
}

/// Fire-and-forget variant aimed at a specific path (for tests).
pub fn record_ignore_err_to(path: &Path, event: &str, latency_ms: f64, result: &str) {
    if let Err(e) = record_to(path, event, latency_ms, result) {
        eprintln!("myth: latency record failed: {:#}", e);
    }
}
