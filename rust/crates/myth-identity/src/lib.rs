//! myth-identity — 3-Tier lesson identity matching.
//!
//! - Tier 1: aggressive text normalise + SHA1 (sub-millisecond, exact
//!   equivalence after canonicalisation).
//! - Tier 2: multilingual-e5-small embedding + vector KNN (through
//!   `myth-embed` daemon). Thresholds: sim ≥ 0.90 auto-merge,
//!   0.75 ≤ sim < 0.90 escalate to Tier 3.
//! - Tier 3: LLM judge via Python subprocess. Disabled until Milestone
//!   A (Constitution Article 18; Decision 3).
//!
//! Layer: same as myth-gavel (Layer 2). Does **not** depend on
//! myth-gavel. Depends on myth-embed only for `protocol::types`
//! (Cargo-level type sharing; actual traffic is Unix socket) — see
//! 6f42254 (docs independence scope) for the governance rationale.

pub mod lapse;
pub mod matcher;
pub mod store;
pub mod tier1;
pub mod tier2;
pub mod tier3;

pub use lapse::{compute_lapse_score, lapse_threshold, should_lapse};
pub use matcher::{IdentityMatcher, Resolution};
pub use store::{Embedding, InMemoryStore, IntegrityReport, VectorStore, EMBEDDING_DIM};
pub use tier1::{normalize_aggressive, tier1_hash};
pub use tier2::{evaluate_candidate, EmbedAdapter, Tier2Decision};
