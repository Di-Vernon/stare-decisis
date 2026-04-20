//! 3-Tier identity resolution. Sync-only — the embedding is passed in
//! by the caller so the matcher itself doesn't need a tokio runtime
//! or any direct coupling to the daemon.

use myth_db::{Lesson, LessonStore};

use crate::store::{Embedding, VectorStore};
use crate::{tier1, tier2, tier3};

#[derive(Debug, Clone)]
pub enum Resolution {
    /// Tier 1 exact match (SHA1 of the normalised text).
    Exact(Lesson),
    /// Tier 2 auto-merge — similarity ≥ 0.90.
    Similar { lesson: Lesson, similarity: f32 },
    /// Tier 3 confirmed an ambiguous (0.75..0.90) candidate.
    Confirmed { lesson: Lesson, similarity: f32 },
    /// No existing match; caller should create a new lesson.
    NoMatch {
        normalized_text: String,
        tier1_hash: [u8; 20],
    },
}

pub struct IdentityMatcher<L, V>
where
    L: LessonStore,
    V: VectorStore,
{
    lessons: L,
    vectors: V,
}

impl<L: LessonStore, V: VectorStore> IdentityMatcher<L, V> {
    pub fn new(lessons: L, vectors: V) -> Self {
        Self { lessons, vectors }
    }

    pub fn lessons(&self) -> &L {
        &self.lessons
    }

    pub fn vectors(&self) -> &V {
        &self.vectors
    }

    /// Fast path — tier 1 only. Used by the PreToolUse / Gavel path
    /// where Tier 2 is too expensive to run.
    pub fn tier1_lookup(&self, raw_text: &str) -> anyhow::Result<Option<Lesson>> {
        let normalized = tier1::normalize_aggressive(raw_text);
        let hash = tier1::tier1_hash(&normalized);
        self.lessons.find_by_identity(&hash)
    }

    /// Full resolution. If `embedding` is provided the matcher walks
    /// Tier 2 and, on borderline similarity, consults Tier 3 (disabled
    /// until Milestone A so it currently always says "no").
    pub fn resolve(
        &self,
        raw_text: &str,
        embedding: Option<&Embedding>,
    ) -> anyhow::Result<Resolution> {
        let normalized = tier1::normalize_aggressive(raw_text);
        let hash = tier1::tier1_hash(&normalized);

        if let Some(lesson) = self.lessons.find_by_identity(&hash)? {
            return Ok(Resolution::Exact(lesson));
        }

        if let Some(e) = embedding {
            let nearest = self.vectors.knn(e, 1)?;
            if let Some((candidate_id, distance)) = nearest.first().copied() {
                let similarity = tier2::similarity_from_distance(distance);
                match tier2::evaluate_candidate(similarity) {
                    tier2::Tier2Decision::AutoMerge => {
                        if let Some(lesson) = self.lessons.get(candidate_id)? {
                            return Ok(Resolution::Similar { lesson, similarity });
                        }
                    }
                    tier2::Tier2Decision::EscalateTier3 => {
                        if let Some(lesson) = self.lessons.get(candidate_id)? {
                            if tier3::judge(raw_text, &lesson.description)? {
                                return Ok(Resolution::Confirmed { lesson, similarity });
                            }
                        }
                    }
                    tier2::Tier2Decision::NewLesson => {}
                }
            }
        }

        Ok(Resolution::NoMatch {
            normalized_text: normalized,
            tier1_hash: hash,
        })
    }
}
