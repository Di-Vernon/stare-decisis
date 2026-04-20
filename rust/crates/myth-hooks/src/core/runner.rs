//! `run_hook` — the shared `main()` boilerplate for every hook bin.
//!
//! A bin's `main.rs` becomes three lines:
//!
//! ```ignore
//! fn main() -> std::process::ExitCode {
//!     myth_hooks::run_hook("pre_tool", "myth-hook-pre-tool", |envelope| {
//!         // … produce a HookResult …
//!         Ok(myth_hooks::HookResult::Allow)
//!     })
//! }
//! ```
//!
//! Why we chose the helper over per-bin duplication:
//!
//! 1. Five boilerplate lines (logging init, stdin read, timer, latency
//!    append, ExitCode map) would be copied six times with subtle
//!    drift risk — the latency record in particular is easy to
//!    forget and only shows up as a silent observability gap.
//! 2. The helper's signature takes a closure returning
//!    `anyhow::Result<HookResult>`, so `?` works naturally inside the
//!    per-bin logic — the ergonomic loss of `FnOnce() -> HookResult`
//!    (no `?` early-return) is avoided.
//! 3. Any hook-global invariant (e.g. `CLAUDE_REVIEW_ACTIVE` bypass,
//!    `MYTH_DISABLE` escape hatch) can be added in one place and
//!    automatically cover every bin.

use std::process::ExitCode;

use crate::core::input::{read_envelope_from_stdin, HookEnvelope};
use crate::core::latency::record_ignore_err;
use crate::core::output::HookResult;

pub fn run_hook<F>(event_name: &'static str, bin_name: &'static str, f: F) -> ExitCode
where
    F: FnOnce(HookEnvelope) -> anyhow::Result<HookResult>,
{
    let start = std::time::Instant::now();
    myth_common::logging::init_logging(bin_name);

    // Global escape hatches (per ARCHITECTURE.md Contract 4).
    if std::env::var_os("MYTH_DISABLE").is_some() {
        tracing::debug!("MYTH_DISABLE set — short-circuiting to Allow");
        return HookResult::Allow.emit();
    }

    let envelope = match read_envelope_from_stdin() {
        Ok(e) => e,
        Err(err) => {
            return HookResult::Error(format!("envelope parse: {:#}", err)).emit();
        }
    };

    // Prevent recursive myth invocations from re-triggering the full
    // hook pipeline when a myth process itself calls Claude.
    if std::env::var_os("CLAUDE_REVIEW_ACTIVE").is_some() {
        tracing::debug!("CLAUDE_REVIEW_ACTIVE set — short-circuiting to Allow");
        return HookResult::Allow.emit();
    }

    let result = f(envelope).unwrap_or_else(|e| HookResult::Error(format!("{:#}", e)));

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    record_ignore_err(event_name, elapsed_ms, result.label());

    result.emit()
}
