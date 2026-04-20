//! Thin writer around the `hook_events` table.

use anyhow::Context;
use myth_common::{LessonId, SessionId, Timestamp};
use uuid::Uuid;

use crate::sqlite::Database;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookEventType {
    SessionStart,
    UserPrompt,
    PreTool,
    PostTool,
    PostToolFailure,
    Stop,
}

impl HookEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SessionStart => "session_start",
            Self::UserPrompt => "user_prompt",
            Self::PreTool => "pre_tool",
            Self::PostTool => "post_tool",
            Self::PostToolFailure => "post_tool_failure",
            Self::Stop => "stop",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Allow,
    Deny,
    Ask,
}

impl Verdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
            Self::Ask => "ask",
        }
    }
}

#[derive(Debug, Clone)]
pub struct HookEvent {
    pub id: Uuid,
    pub session_id: SessionId,
    pub event_type: HookEventType,
    pub tool_name: Option<String>,
    pub ts: Timestamp,
    pub latency_ms: f64,
    pub verdict: Verdict,
    pub lesson_id: Option<LessonId>,
}

pub fn insert(db: &Database, event: &HookEvent) -> anyhow::Result<()> {
    let lesson_bytes: Option<Vec<u8>> = event.lesson_id.map(|l| l.as_bytes().to_vec());

    db.conn
        .execute(
            "INSERT INTO hook_events (
                id, session_id, event_type, tool_name,
                ts, latency_ms, verdict, lesson_id
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                event.id.as_bytes().as_slice(),
                event.session_id.as_bytes().as_slice(),
                event.event_type.as_str(),
                &event.tool_name,
                event.ts.timestamp_millis(),
                event.latency_ms,
                event.verdict.as_str(),
                lesson_bytes,
            ],
        )
        .context("inserting hook_event")?;
    Ok(())
}
