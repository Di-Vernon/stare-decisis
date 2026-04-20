//! Vector store abstraction + Day-1 in-memory implementation.
//!
//! Consumers deal with the `VectorStore` trait; `InMemoryStore` is the
//! Day-1 backend. `sqlite_vec` and `usearch` are stubs that activate
//! at Milestone B per `DECISIONS.md` Decision 1.

pub mod in_memory;
pub mod sqlite_vec;
pub mod usearch;

pub use in_memory::InMemoryStore;

use myth_common::LessonId;

pub const EMBEDDING_DIM: usize = 384;
pub type Embedding = [f32; EMBEDDING_DIM];

pub trait VectorStore {
    fn upsert(&self, id: LessonId, vec: &Embedding) -> anyhow::Result<()>;
    fn knn(&self, query: &Embedding, k: usize) -> anyhow::Result<Vec<(LessonId, f32)>>;
    fn delete(&self, id: LessonId) -> anyhow::Result<()>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn integrity_check(&self) -> anyhow::Result<IntegrityReport>;
}

#[derive(Debug, Clone)]
pub struct IntegrityReport {
    pub total_vectors: usize,
    pub index_consistent: bool,
    pub generation_match: bool,
    pub norm_anomalies: usize,
}

impl IntegrityReport {
    pub fn is_healthy(&self) -> bool {
        self.index_consistent && self.generation_match && self.norm_anomalies == 0
    }
}
