//! Session id helpers.
//!
//! Claude Code's stdin JSON provides `session_id` as a string UUID.
//! myth currently uses that value directly as its own
//! `myth_common::SessionId` — no separate mapping on Day-1. A future
//! migration (distinct myth session vs Claude session) would land
//! here.

use anyhow::Context;
use myth_common::SessionId;
use uuid::Uuid;

pub fn parse_claude_session_id(s: &str) -> anyhow::Result<SessionId> {
    let uuid = Uuid::parse_str(s).with_context(|| format!("parsing session_id '{}'", s))?;
    Ok(SessionId(uuid))
}
