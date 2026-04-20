//! Sentencing matrix (Level × Recurrence → Enforcement) with DB-backed
//! overrides.

pub mod default;
pub mod overrides;

use std::collections::HashMap;

use anyhow::Context;
use myth_common::{Enforcement, Level, Recurrence};
use myth_db::Database;

pub struct Grid {
    matrix: [[Enforcement; 6]; 5],
    overrides: HashMap<(Level, Recurrence), Enforcement>,
}

impl Grid {
    pub fn new() -> Self {
        Self {
            matrix: default::DEFAULT_MATRIX,
            overrides: HashMap::new(),
        }
    }

    pub fn load(db: &Database) -> anyhow::Result<Self> {
        let mut grid = Self::new();
        overrides::apply_db_overrides(&mut grid.overrides, db)
            .context("loading grid overrides from DB")?;
        Ok(grid)
    }

    pub fn lookup(&self, level: Level, recurrence: Recurrence) -> Enforcement {
        if let Some(e) = self.overrides.get(&(level, recurrence)) {
            return *e;
        }
        self.matrix[level as usize - 1][recurrence as usize - 1]
    }

    /// In-memory override (tests and observer-suggested runtime edits).
    pub fn set_override(
        &mut self,
        level: Level,
        recurrence: Recurrence,
        enforcement: Enforcement,
    ) {
        self.overrides.insert((level, recurrence), enforcement);
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self::new()
    }
}
