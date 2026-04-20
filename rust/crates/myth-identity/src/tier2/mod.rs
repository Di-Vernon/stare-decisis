//! Tier 2 — embedding similarity via `myth-embed` + vector KNN.
//!
//! Decision thresholds (docs/04-CRATES/04-myth-identity.md §Tier 2):
//!   similarity >= 0.90   → auto-merge into the candidate lesson
//!   0.75 ≤ sim < 0.90    → escalate to Tier 3 (LLM judge)
//!   similarity < 0.75    → treat as new lesson

pub mod embed_client;

pub use embed_client::EmbedAdapter;

pub const AUTO_MERGE_THRESHOLD: f32 = 0.90;
pub const TIER3_ESCALATE_THRESHOLD: f32 = 0.75;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier2Decision {
    AutoMerge,
    EscalateTier3,
    NewLesson,
}

/// Map a cosine similarity in [0.0, 1.0] to a Tier 2 decision.
pub fn evaluate_candidate(similarity: f32) -> Tier2Decision {
    if similarity >= AUTO_MERGE_THRESHOLD {
        Tier2Decision::AutoMerge
    } else if similarity >= TIER3_ESCALATE_THRESHOLD {
        Tier2Decision::EscalateTier3
    } else {
        Tier2Decision::NewLesson
    }
}

/// Cosine distance → similarity helper used by the matcher. Input is
/// the `distance` returned by `VectorStore::knn` (1.0 - cosine for
/// normalised inputs). Clamped to `[0.0, 1.0]` because noisy
/// embeddings can occasionally produce slightly negative distances.
pub fn similarity_from_distance(distance: f32) -> f32 {
    (1.0 - distance).clamp(0.0, 1.0)
}
