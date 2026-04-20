//! Shared hook-binary helpers.
//!
//! Four concerns, one module:
//! - `input`   — parse the stdin JSON envelope Claude Code sends us.
//! - `output`  — produce the return JSON + map to an ExitCode.
//! - `latency` — record per-call latency to hook-latency.ndjson,
//!   fire-and-forget so hook correctness never depends on the
//!   observability path succeeding.
//! - `session` — small helpers around SessionId conversion.
//! - `runner`  — `run_hook(event, bin_name, |envelope| …)` wrapper
//!   so every bin has one-line `main`.

pub mod input;
pub mod latency;
pub mod output;
pub mod runner;
pub mod session;
