//! Lesson struct, LessonStore trait, SQLite-backed implementation.

use anyhow::Context;
use myth_common::{Category, Level, LessonId, Timestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::sqlite::Database;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LessonStatus {
    Active,
    Lapsed,
    Archived,
    Superseded,
}

impl LessonStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Lapsed => "lapsed",
            Self::Archived => "archived",
            Self::Superseded => "superseded",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Lesson {
    pub id: LessonId,
    pub identity_hash_tier1: [u8; 20],
    pub level: Level,
    pub category: Category,
    pub recurrence_count: f64,
    pub missed_hook_count: u32,
    pub first_seen: Timestamp,
    pub last_seen: Timestamp,
    pub lapse_score: f64,
    pub appeals: u32,
    pub status: LessonStatus,
    pub description: String,
    pub rationale: String,
    pub meta_json: Option<String>,
}

/// `LessonStore` has no auto-trait bound because `rusqlite::Connection`
/// is neither `Sync` (internal `StatementCache` uses `RefCell`) nor,
/// transitively through `&Database`, `Send`. myth currently runs every
/// hook in its own short-lived process, so a single-threaded store
/// satisfies every call site. When the Milestone C daemon model
/// introduces shared in-process state, wrap the connection in a `Mutex`
/// at that point and tighten the bound (`Send`, possibly `Sync`).
pub trait LessonStore {
    fn insert(&self, lesson: &Lesson) -> anyhow::Result<LessonId>;
    fn get(&self, id: LessonId) -> anyhow::Result<Option<Lesson>>;
    fn find_by_identity(&self, hash: &[u8; 20]) -> anyhow::Result<Option<Lesson>>;
    fn update(&self, lesson: &Lesson) -> anyhow::Result<()>;
    fn increment_recurrence(&self, id: LessonId) -> anyhow::Result<f64>;
    fn list_active(&self) -> anyhow::Result<Vec<Lesson>>;
    fn list_lapsed(&self) -> anyhow::Result<Vec<Lesson>>;
    fn mark_status(&self, id: LessonId, status: LessonStatus) -> anyhow::Result<()>;

    /// Consuming escape hatch for implementations that own a
    /// single-writer `Database`. Default returns `None` so mocks and
    /// non-SQL stores stay unaffected. `SqliteLessonStore` overrides
    /// this to surrender its underlying connection, enabling Task 3.6
    /// Step c Connection sharing: Gavel can hand the same db that
    /// powered `find_by_identity` back to the hook runner for the
    /// `hook_events` insert, avoiding a second `Database::open` whose
    /// cost (~30 ms WAL+PRAGMA+migration) would push post_tool_failure
    /// Tier 0 above the 50 ms ARCHITECTURE §4 line 264 budget.
    ///
    /// The `self: Box<Self>` receiver is object-safe; trait objects
    /// call through the vtable.
    fn into_boxed_db(self: Box<Self>) -> Option<Database> {
        None
    }
}

pub struct SqliteLessonStore {
    db: Database,
}

impl SqliteLessonStore {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Access the underlying database for operations outside
    /// `LessonStore` (e.g. `events::insert`, `appeal::file_appeal`).
    pub fn db(&self) -> &Database {
        &self.db
    }

    /// Consume the store and return the inner `Database`. Task 3.6
    /// Step c's hook_events wire-through uses this on the
    /// post-tool-failure Tier 0 path so the same `Database::open` that
    /// powered the lesson upsert can be reused for the `hook_events`
    /// insert. Symmetric with `new(db)`.
    pub fn into_db(self) -> Database {
        self.db
    }
}

fn category_sql(c: Category) -> &'static str {
    match c {
        Category::Security => "security",
        Category::Correctness => "correctness",
        Category::Process => "process",
        Category::DataSafety => "data_safety",
        Category::Temporal => "temporal",
    }
}

impl LessonStore for SqliteLessonStore {
    fn insert(&self, lesson: &Lesson) -> anyhow::Result<LessonId> {
        self.db
            .conn
            .execute(
                "INSERT INTO lessons (
                    id, identity_hash_tier1, level, category,
                    recurrence_count, missed_hook_count,
                    first_seen, last_seen, lapse_score, appeals,
                    status, description, rationale, meta_json
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                rusqlite::params![
                    lesson.id.as_bytes().as_slice(),
                    lesson.identity_hash_tier1.as_slice(),
                    lesson.level as u8,
                    category_sql(lesson.category),
                    lesson.recurrence_count,
                    lesson.missed_hook_count,
                    lesson.first_seen.timestamp(),
                    lesson.last_seen.timestamp(),
                    lesson.lapse_score,
                    lesson.appeals,
                    lesson.status.as_str(),
                    &lesson.description,
                    &lesson.rationale,
                    &lesson.meta_json,
                ],
            )
            .context("inserting lesson")?;
        Ok(lesson.id)
    }

    fn get(&self, id: LessonId) -> anyhow::Result<Option<Lesson>> {
        let result = self.db.conn.query_row(
            "SELECT id, identity_hash_tier1, level, category,
                    recurrence_count, missed_hook_count,
                    first_seen, last_seen, lapse_score, appeals,
                    status, description, rationale, meta_json
             FROM lessons WHERE id = ?1",
            rusqlite::params![id.as_bytes().as_slice()],
            row_to_lesson,
        );
        match result {
            Ok(l) => Ok(Some(l)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::Error::from(e).context("get lesson")),
        }
    }

    fn find_by_identity(&self, hash: &[u8; 20]) -> anyhow::Result<Option<Lesson>> {
        let result = self.db.conn.query_row(
            "SELECT id, identity_hash_tier1, level, category,
                    recurrence_count, missed_hook_count,
                    first_seen, last_seen, lapse_score, appeals,
                    status, description, rationale, meta_json
             FROM lessons WHERE identity_hash_tier1 = ?1 LIMIT 1",
            rusqlite::params![hash.as_slice()],
            row_to_lesson,
        );
        match result {
            Ok(l) => Ok(Some(l)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::Error::from(e).context("find_by_identity")),
        }
    }

    fn update(&self, lesson: &Lesson) -> anyhow::Result<()> {
        self.db
            .conn
            .execute(
                "UPDATE lessons SET
                    level = ?2, category = ?3,
                    recurrence_count = ?4, missed_hook_count = ?5,
                    first_seen = ?6, last_seen = ?7,
                    lapse_score = ?8, appeals = ?9,
                    status = ?10, description = ?11,
                    rationale = ?12, meta_json = ?13
                 WHERE id = ?1",
                rusqlite::params![
                    lesson.id.as_bytes().as_slice(),
                    lesson.level as u8,
                    category_sql(lesson.category),
                    lesson.recurrence_count,
                    lesson.missed_hook_count,
                    lesson.first_seen.timestamp(),
                    lesson.last_seen.timestamp(),
                    lesson.lapse_score,
                    lesson.appeals,
                    lesson.status.as_str(),
                    &lesson.description,
                    &lesson.rationale,
                    &lesson.meta_json,
                ],
            )
            .context("updating lesson")?;
        Ok(())
    }

    fn increment_recurrence(&self, id: LessonId) -> anyhow::Result<f64> {
        self.db
            .conn
            .execute(
                "UPDATE lessons SET recurrence_count = recurrence_count + 1 WHERE id = ?1",
                rusqlite::params![id.as_bytes().as_slice()],
            )
            .context("incrementing recurrence")?;
        let new_count: f64 = self
            .db
            .conn
            .query_row(
                "SELECT recurrence_count FROM lessons WHERE id = ?1",
                rusqlite::params![id.as_bytes().as_slice()],
                |r| r.get(0),
            )
            .context("reading new recurrence count")?;
        Ok(new_count)
    }

    fn list_active(&self) -> anyhow::Result<Vec<Lesson>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, identity_hash_tier1, level, category,
                    recurrence_count, missed_hook_count,
                    first_seen, last_seen, lapse_score, appeals,
                    status, description, rationale, meta_json
             FROM lessons WHERE status = 'active' ORDER BY last_seen DESC",
        )?;
        let rows = stmt.query_map([], row_to_lesson)?;
        rows.map(|r| r.map_err(anyhow::Error::from)).collect()
    }

    fn list_lapsed(&self) -> anyhow::Result<Vec<Lesson>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, identity_hash_tier1, level, category,
                    recurrence_count, missed_hook_count,
                    first_seen, last_seen, lapse_score, appeals,
                    status, description, rationale, meta_json
             FROM lessons WHERE status = 'lapsed' ORDER BY last_seen DESC",
        )?;
        let rows = stmt.query_map([], row_to_lesson)?;
        rows.map(|r| r.map_err(anyhow::Error::from)).collect()
    }

    fn mark_status(&self, id: LessonId, status: LessonStatus) -> anyhow::Result<()> {
        self.db
            .conn
            .execute(
                "UPDATE lessons SET status = ?1 WHERE id = ?2",
                rusqlite::params![status.as_str(), id.as_bytes().as_slice()],
            )
            .context("marking status")?;
        Ok(())
    }

    fn into_boxed_db(self: Box<Self>) -> Option<Database> {
        Some((*self).into_db())
    }
}

fn row_to_lesson(row: &rusqlite::Row) -> rusqlite::Result<Lesson> {
    use rusqlite::types::Type;
    use rusqlite::Error::InvalidColumnType;

    let id_bytes: Vec<u8> = row.get(0)?;
    let id_arr: [u8; 16] = id_bytes
        .as_slice()
        .try_into()
        .map_err(|_| InvalidColumnType(0, "id length".into(), Type::Blob))?;

    let ih_bytes: Vec<u8> = row.get(1)?;
    let ih: [u8; 20] = ih_bytes
        .as_slice()
        .try_into()
        .map_err(|_| InvalidColumnType(1, "identity_hash_tier1 length".into(), Type::Blob))?;

    let level_int: i64 = row.get(2)?;
    let level = match level_int {
        1 => Level::Info,
        2 => Level::Low,
        3 => Level::Medium,
        4 => Level::High,
        5 => Level::Critical,
        n => {
            return Err(InvalidColumnType(
                2,
                format!("invalid level: {}", n),
                Type::Integer,
            ))
        }
    };

    let cat_str: String = row.get(3)?;
    let category = match cat_str.as_str() {
        "security" => Category::Security,
        "correctness" => Category::Correctness,
        "process" => Category::Process,
        "data_safety" => Category::DataSafety,
        "temporal" => Category::Temporal,
        other => {
            return Err(InvalidColumnType(
                3,
                format!("invalid category: {}", other),
                Type::Text,
            ))
        }
    };

    let recurrence_count: f64 = row.get(4)?;
    let missed_hook_count: u32 = row.get::<_, i64>(5)? as u32;

    let first_seen_secs: i64 = row.get(6)?;
    let first_seen = chrono::DateTime::<chrono::Utc>::from_timestamp(first_seen_secs, 0)
        .ok_or_else(|| {
            InvalidColumnType(6, "invalid first_seen timestamp".into(), Type::Integer)
        })?;

    let last_seen_secs: i64 = row.get(7)?;
    let last_seen = chrono::DateTime::<chrono::Utc>::from_timestamp(last_seen_secs, 0)
        .ok_or_else(|| {
            InvalidColumnType(7, "invalid last_seen timestamp".into(), Type::Integer)
        })?;

    let lapse_score: f64 = row.get(8)?;
    let appeals: u32 = row.get::<_, i64>(9)? as u32;

    let status_str: String = row.get(10)?;
    let status = match status_str.as_str() {
        "active" => LessonStatus::Active,
        "lapsed" => LessonStatus::Lapsed,
        "archived" => LessonStatus::Archived,
        "superseded" => LessonStatus::Superseded,
        other => {
            return Err(InvalidColumnType(
                10,
                format!("invalid status: {}", other),
                Type::Text,
            ))
        }
    };

    let description: String = row.get(11)?;
    let rationale: String = row.get(12)?;
    let meta_json: Option<String> = row.get(13)?;

    Ok(Lesson {
        id: LessonId(Uuid::from_bytes(id_arr)),
        identity_hash_tier1: ih,
        level,
        category,
        recurrence_count,
        missed_hook_count,
        first_seen,
        last_seen,
        lapse_score,
        appeals,
        status,
        description,
        rationale,
        meta_json,
    })
}
