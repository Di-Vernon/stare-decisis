//! The Gavel itself — combines rules, grid and fatigue into a single
//! `judge()` call.

use std::sync::Mutex;

use anyhow::Context;
use myth_common::{Enforcement, Recurrence};
use myth_db::Database;

use crate::fatigue::FatigueTracker;
use crate::grid::Grid;
use crate::rules::{RuleMatch, RuleSet};
use crate::tool_input::ToolInput;
use crate::verdict::Verdict;

pub struct Gavel {
    rules: RuleSet,
    grid: Grid,
    fatigue: Mutex<FatigueTracker>,
    // NOTE(Task 2.3): a `LessonStore` handle will be added when
    // myth-identity lands so Foundation/Surface matches can resolve
    // actual recurrence. Until then, every Grid-path match is treated
    // as Recurrence::I.
}

impl Gavel {
    /// Production entry: load rules, open state.db, build grid from
    /// overrides, start an empty fatigue tracker.
    pub fn init() -> anyhow::Result<Self> {
        let rules = RuleSet::load_all().context("loading rule sets")?;
        let db = Database::open(&myth_common::state_db_path())
            .context("opening state.db")?;
        let grid = Grid::load(&db).context("loading grid")?;
        Ok(Self {
            rules,
            grid,
            fatigue: Mutex::new(FatigueTracker::new()),
        })
    }

    pub fn from_parts(rules: RuleSet, grid: Grid) -> Self {
        Self {
            rules,
            grid,
            fatigue: Mutex::new(FatigueTracker::new()),
        }
    }

    pub fn judge(&self, input: &ToolInput) -> Verdict {
        // 1. Bedrock → always Seal.
        if let Some(m) = self.rules.bedrock.match_any(&input.serialized) {
            return Verdict::seal(m);
        }

        // 2. Foundation → Grid + fatigue.
        if let Some(m) = self.rules.foundation.match_any(&input.serialized) {
            return self.grid_path(input, m);
        }

        // 3. Surface → same Grid path.
        if let Some(m) = self.rules.surface.match_any(&input.serialized) {
            return self.grid_path(input, m);
        }

        Verdict::allow()
    }

    fn grid_path(&self, input: &ToolInput, m: RuleMatch) -> Verdict {
        // Day-1: treat every Foundation/Surface match as first occurrence.
        // Task 2.3 (myth-identity) will wire in actual recurrence lookup.
        let raw_enforcement = self.grid.lookup(m.level, Recurrence::I);
        let final_enforcement = {
            let mut fatigue = self.fatigue.lock().expect("fatigue mutex poisoned");
            fatigue.register(input.session_id, raw_enforcement)
        };

        if matches!(
            final_enforcement,
            Enforcement::Dismiss | Enforcement::Note
        ) {
            Verdict {
                enforcement: final_enforcement,
                rule_match: Some(m),
                lesson_id: None,
                rationale: String::new(),
            }
        } else {
            Verdict::with_enforcement(final_enforcement, m, None)
        }
    }
}
