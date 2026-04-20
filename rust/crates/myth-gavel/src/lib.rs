//! myth-gavel — The Gavel, pre-execution judgement.
//!
//! Library only. The hook binary `myth-hook-pre-tool` (Wave 3) is the
//! actual runtime entry; this crate provides the `Gavel` type and its
//! judgement logic.
//!
//! Fail-safe rule: if YAML parsing or regex compilation fails, the
//! whole `RuleSet::load_all` returns an error — `Gavel::init` then
//! fails, and callers are expected to treat that as deny-by-default.
//! If a file simply doesn't exist yet, the crate logs a warning and
//! uses an empty rule set (Day-1 reality before Wave 7 fixtures).

pub mod fatigue;
pub mod grid;
pub mod judge;
pub mod rules;
pub mod tool_input;
pub mod verdict;

pub use fatigue::FatigueTracker;
pub use grid::Grid;
pub use judge::Gavel;
pub use rules::{CompiledRule, CompiledRules, Likelihood, RuleMatch, RuleSet};
pub use tool_input::ToolInput;
pub use verdict::Verdict;
