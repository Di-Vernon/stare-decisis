//! Forward-only SQLite migrations driven by `PRAGMA user_version`.
//!
//! `ARCHITECTURE.md` Contract 5: we only add tables/columns; never
//! remove or rename. A bumped user_version means "apply the next
//! migration in `MIGRATIONS` and advance the version inside that SQL".

use anyhow::Context;
use rusqlite::Connection;

const MIGRATION_001: &str = include_str!("../../migrations/001_initial.sql");

/// Ordered list of (target_version, sql). Each SQL body is expected to
/// set `PRAGMA user_version = N;` where N matches the tuple's first
/// element.
const MIGRATIONS: &[(u32, &str)] = &[(1, MIGRATION_001)];

pub fn apply(conn: &Connection) -> anyhow::Result<()> {
    let current: u32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .context("querying user_version")?;

    for (target, sql) in MIGRATIONS {
        if *target > current {
            conn.execute_batch(sql)
                .with_context(|| format!("applying migration to v{}", target))?;
            tracing::info!(target_version = target, "applied migration");
        }
    }

    Ok(())
}
