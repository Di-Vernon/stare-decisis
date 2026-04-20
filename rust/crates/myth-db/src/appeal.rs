//! Thin writer around the `appeal_history` table.

use anyhow::Context;
use myth_common::LessonId;

use crate::sqlite::Database;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppealType {
    Appeal,
    Retrial,
}

impl AppealType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Appeal => "appeal",
            Self::Retrial => "retrial",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppealResult {
    Pending,
    Granted,
    Denied,
    Withdrawn,
}

impl AppealResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Granted => "granted",
            Self::Denied => "denied",
            Self::Withdrawn => "withdrawn",
        }
    }
}

pub fn file_appeal(
    db: &Database,
    lesson_id: LessonId,
    appeal_type: AppealType,
    rationale: Option<&str>,
) -> anyhow::Result<i64> {
    let ts = myth_common::now().timestamp_millis();
    db.conn
        .execute(
            "INSERT INTO appeal_history (lesson_id, appeal_type, ts, result, rationale)
             VALUES (?1, ?2, ?3, 'pending', ?4)",
            rusqlite::params![
                lesson_id.as_bytes().as_slice(),
                appeal_type.as_str(),
                ts,
                rationale
            ],
        )
        .context("filing appeal")?;
    Ok(db.conn.last_insert_rowid())
}
