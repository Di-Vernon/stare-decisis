//! `run_hook` — the shared `main()` boilerplate for every hook bin.
//!
//! A bin's `main.rs` becomes three lines:
//!
//! ```ignore
//! fn main() -> std::process::ExitCode {
//!     myth_hooks::run_hook("pre_tool", "myth-hook-pre-tool", |envelope| {
//!         // … produce a HookResult …
//!         Ok(myth_hooks::HookResult::Allow.into())
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
//!    `anyhow::Result<HookOutcome>`, so `?` works naturally inside the
//!    per-bin logic — the ergonomic loss of `FnOnce() -> HookOutcome`
//!    (no `?` early-return) is avoided.
//! 3. Any hook-global invariant (e.g. `CLAUDE_REVIEW_ACTIVE` bypass,
//!    `MYTH_DISABLE` escape hatch) can be added in one place and
//!    automatically cover every bin.
//! 4. `HookOutcome` carries an optional `PartialHookEvent` so the
//!    runner can stamp the canonical latency (same value it writes
//!    into `hook-latency.ndjson`) onto the `state.db::hook_events`
//!    row. Bins no longer measure time themselves and never drift
//!    out of sync with the NDJSON path (Task 3.6 Step c wire-through).

use std::process::ExitCode;

use myth_db::events::{self, HookEvent};
use myth_db::Database;
use uuid::Uuid;

use crate::core::input::{read_envelope_from_stdin, HookEnvelope};
use crate::core::latency::record_ignore_err;
use crate::core::output::{HookOutcome, HookResult, PartialHookEvent};

pub fn run_hook<F>(event_name: &'static str, bin_name: &'static str, f: F) -> ExitCode
where
    F: FnOnce(HookEnvelope) -> anyhow::Result<HookOutcome>,
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

    let outcome = f(envelope).unwrap_or_else(|e| HookResult::Error(format!("{:#}", e)).into());

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    record_ignore_err(event_name, elapsed_ms, outcome.result.label());

    // hook_events DB append — observability path. A DB failure must
    // degrade silently (tracing::warn + swallow); correctness of the
    // hook result is already locked in by this point.
    if let Some(event) = outcome.event {
        if let Err(e) = persist_hook_event(event, elapsed_ms, outcome.db) {
            tracing::warn!(error = %e, "hook_events DB append failed");
        }
    }

    outcome.result.emit()
}

/// Stamp `latency_ms` onto the partial event and insert it. When the
/// bin already owns a `Database` (pre_tool via `Gavel::into_db`,
/// post_tool_failure Tier 0 via `SqliteLessonStore::into_db`) it is
/// passed in via `shared_db` and reused here — avoids the second
/// `Database::open` whose cost (~30 ms WAL/PRAGMA/migration) pushes
/// Tier 0 above the 50 ms ARCHITECTURE §4 line 264 budget. For bins
/// that don't hold a db (post_tool, post_tool_failure Tier 1 path)
/// this falls back to a fresh open — one open total, the wire-through
/// baseline we accept.
fn persist_hook_event(
    partial: PartialHookEvent,
    latency_ms: f64,
    shared_db: Option<Database>,
) -> anyhow::Result<()> {
    let db = match shared_db {
        Some(db) => db,
        None => Database::open(&myth_common::state_db_path())?,
    };
    let event = HookEvent {
        id: Uuid::new_v4(),
        session_id: partial.session_id,
        event_type: partial.event_type,
        tool_name: partial.tool_name,
        ts: myth_common::now(),
        latency_ms,
        verdict: partial.verdict,
        lesson_id: partial.lesson_id,
    };
    events::insert(&db, &event)
}
