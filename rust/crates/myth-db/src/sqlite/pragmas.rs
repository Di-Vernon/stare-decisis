//! PRAGMA setup applied to every fresh connection.
//!
//! `journal_mode = WAL` is persistent (stored in the DB file), but
//! connection-local PRAGMAs (busy_timeout, foreign_keys, mmap_size) must
//! be re-applied whenever a new connection is opened.

use anyhow::Context;
use rusqlite::Connection;

pub fn apply(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA busy_timeout = 5000;
         PRAGMA foreign_keys = ON;
         PRAGMA mmap_size = 268435456;",
    )
    .context("execute_batch PRAGMAs")?;
    Ok(())
}
