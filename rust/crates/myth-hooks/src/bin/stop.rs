//! Stop hook — Day-1 no-op. Tier 2 reinforcement path activates at
//! Milestone A once shadow-mode data supports it; until then the bin
//! returns Allow immediately after the standard runner plumbing.

use std::process::ExitCode;

use mimalloc::MiMalloc;
use myth_hooks::{run_hook, HookPayload, HookResult};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn is_tier2_enabled() -> bool {
    // Day-1: Tier 2 reinforcement is OFF. Future form reads
    // `~/.config/myth/config.yaml`'s `assessor.tier_2_enabled`.
    false
}

fn main() -> ExitCode {
    run_hook("stop", "myth-hook-stop", |envelope| {
        if !matches!(envelope.payload, HookPayload::Stop(_)) {
            return Ok(HookResult::Allow);
        }

        // stop_hook_active indicates Claude Code already blocked once
        // this turn — second trigger must be a no-op to avoid loops.
        if envelope.common.stop_hook_active {
            tracing::debug!("stop_hook_active set — short-circuiting");
            return Ok(HookResult::Allow);
        }

        if !is_tier2_enabled() {
            return Ok(HookResult::Allow);
        }

        // Milestone A target path — reinforce a missed assessor
        // trigger with Variant B. Not implemented until then.
        Ok(HookResult::Allow)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier2_off_on_day1() {
        assert!(!is_tier2_enabled());
    }
}
