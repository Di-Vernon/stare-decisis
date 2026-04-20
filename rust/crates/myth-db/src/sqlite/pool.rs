//! Connection opening. A thin wrapper today; a pool abstraction can
//! replace this when read-heavy concurrent access becomes a bottleneck.

use std::path::Path;

use anyhow::Context;
use rusqlite::Connection;

pub fn open(path: &Path) -> anyhow::Result<Connection> {
    Connection::open(path).with_context(|| format!("failed to open sqlite at {:?}", path))
}
