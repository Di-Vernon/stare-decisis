//! myth-db — Layer 1 persistent storage.
//!
//! SQLite for queryable metadata (state.db), JSONL for append-only event
//! logs, and a tamper-evident Merkle audit chain (blake3).

pub mod appeal;
pub mod audit;
pub mod events;
pub mod jsonl;
pub mod lesson;
pub mod sqlite;

pub use audit::{AuditEntry, AuditEvent, AuditLog};
pub use jsonl::JsonlWriter;
pub use lesson::{Lesson, LessonStatus, LessonStore, SqliteLessonStore};
pub use sqlite::Database;
