//! Loads `grid_overrides` rows from `state.db` and merges them into the
//! in-memory map.

use std::collections::HashMap;

use anyhow::{anyhow, Context};
use myth_common::{Enforcement, Level, Recurrence};
use myth_db::Database;

pub fn apply_db_overrides(
    map: &mut HashMap<(Level, Recurrence), Enforcement>,
    db: &Database,
) -> anyhow::Result<()> {
    let mut stmt = db
        .conn
        .prepare("SELECT level, recurrence, enforcement FROM grid_overrides")
        .context("preparing grid_overrides SELECT")?;

    let rows = stmt.query_map([], |row| {
        let level_int: i64 = row.get(0)?;
        let recurrence_int: i64 = row.get(1)?;
        let enforcement: String = row.get(2)?;
        Ok((level_int, recurrence_int, enforcement))
    })?;

    for r in rows {
        let (level_int, recurrence_int, enforcement_str) = r.context("reading override row")?;
        let level = level_from_int(level_int as u8)?;
        let recurrence = recurrence_from_int(recurrence_int as u8)?;
        let enforcement = enforcement_from_str(&enforcement_str)?;
        map.insert((level, recurrence), enforcement);
    }

    Ok(())
}

fn level_from_int(n: u8) -> anyhow::Result<Level> {
    match n {
        1 => Ok(Level::Info),
        2 => Ok(Level::Low),
        3 => Ok(Level::Medium),
        4 => Ok(Level::High),
        5 => Ok(Level::Critical),
        _ => Err(anyhow!("invalid level: {}", n)),
    }
}

fn recurrence_from_int(n: u8) -> anyhow::Result<Recurrence> {
    match n {
        1 => Ok(Recurrence::I),
        2 => Ok(Recurrence::II),
        3 => Ok(Recurrence::III),
        4 => Ok(Recurrence::IV),
        5 => Ok(Recurrence::V),
        6 => Ok(Recurrence::VI),
        _ => Err(anyhow!("invalid recurrence: {}", n)),
    }
}

fn enforcement_from_str(s: &str) -> anyhow::Result<Enforcement> {
    match s {
        "dismiss" => Ok(Enforcement::Dismiss),
        "note" => Ok(Enforcement::Note),
        "advisory" => Ok(Enforcement::Advisory),
        "caution" => Ok(Enforcement::Caution),
        "warn" => Ok(Enforcement::Warn),
        "strike" => Ok(Enforcement::Strike),
        "seal" => Ok(Enforcement::Seal),
        _ => Err(anyhow!("unknown enforcement: {}", s)),
    }
}
