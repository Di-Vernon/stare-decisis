//! Tier 3 — LLM judge (disabled until Milestone A).
//!
//! Decision 3 routes Tier 3 through a Python subprocess that calls
//! Anthropic's Haiku / Sonnet API. The actual dispatch lives in the
//! Python layer (`myth_py.assessor.dispatcher`) and is wired up once
//! Milestone A's Tier 1 compliance data shows we need it.
//!
//! Until then: `judge()` returns `Ok(false)` (never auto-merge) and
//! logs the disabled state once per process.

use std::sync::atomic::{AtomicBool, Ordering};

static LOG_EMITTED: AtomicBool = AtomicBool::new(false);

pub fn is_enabled() -> bool {
    std::env::var_os("MYTH_TIER3_ENABLED").is_some()
}

pub fn judge(_candidate_a: &str, _candidate_b: &str) -> anyhow::Result<bool> {
    if !is_enabled() {
        if !LOG_EMITTED.swap(true, Ordering::Relaxed) {
            tracing::info!("tier3 disabled until Milestone A — judge returning false");
        }
        return Ok(false);
    }
    // Future Milestone A path: spawn myth_py.assessor.dispatcher
    // subprocess, pipe both candidates in, parse structured response.
    anyhow::bail!("tier3 Milestone A dispatch path not yet implemented")
}
