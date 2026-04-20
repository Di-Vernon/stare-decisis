//! SQLite connection + migration + pragma orchestration.

pub mod migration;
pub mod pool;
pub mod pragmas;

use std::path::Path;

use anyhow::Context;
use rusqlite::Connection;

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let conn = pool::open(path).context("opening sqlite")?;
        pragmas::apply(&conn).context("applying pragmas")?;
        migration::apply(&conn).context("applying migrations")?;
        Ok(Self { conn })
    }
}
