//! The Gavel itself — combines rules, grid and fatigue into a single
//! `judge()` call.

use std::sync::Mutex;

use anyhow::Context;
use myth_common::{Enforcement, Recurrence};
use myth_db::{Database, LessonStore, SqliteLessonStore};
use sha1::{Digest, Sha1};

use crate::fatigue::FatigueTracker;
use crate::grid::Grid;
use crate::rules::{RuleMatch, RuleSet};
use crate::tool_input::ToolInput;
use crate::verdict::Verdict;

pub struct Gavel {
    rules: RuleSet,
    grid: Grid,
    lesson_store: Box<dyn LessonStore>,
    fatigue: Mutex<FatigueTracker>,
}

impl Gavel {
    /// Production entry: load rules, open state.db, build grid from
    /// overrides, own the lesson store, start an empty fatigue tracker.
    pub fn init() -> anyhow::Result<Self> {
        let rules = RuleSet::load_all().context("loading rule sets")?;
        let db = Database::open(&myth_common::state_db_path())
            .context("opening state.db")?;
        let grid = Grid::load(&db).context("loading grid")?;
        let lesson_store: Box<dyn LessonStore> = Box::new(SqliteLessonStore::new(db));
        Ok(Self {
            rules,
            grid,
            lesson_store,
            fatigue: Mutex::new(FatigueTracker::new()),
        })
    }

    pub fn from_parts(
        rules: RuleSet,
        grid: Grid,
        lesson_store: Box<dyn LessonStore>,
    ) -> Self {
        Self {
            rules,
            grid,
            lesson_store,
            fatigue: Mutex::new(FatigueTracker::new()),
        }
    }

    /// Identity hash for a rule match. SHA1 of `rule_id || "|" ||
    /// matched_text`. Kept inside myth-gavel so this crate does not
    /// depend on myth-identity — full aggressive normalisation lives
    /// on the PostToolUseFailure path (Wave 3) via myth-identity.
    pub fn compute_identity(rule_id: &str, matched_text: &str) -> [u8; 20] {
        let mut h = Sha1::new();
        h.update(rule_id.as_bytes());
        h.update(b"|");
        h.update(matched_text.as_bytes());
        h.finalize().into()
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
        let hash = Self::compute_identity(&m.rule_id, &m.matched_text);
        let lesson_opt = self
            .lesson_store
            .find_by_identity(&hash)
            .ok()
            .flatten();
        let recurrence = lesson_opt
            .as_ref()
            .map(|l| Recurrence::from_count(l.recurrence_count))
            .unwrap_or(Recurrence::I);
        let lesson_id = lesson_opt.as_ref().map(|l| l.id);

        let raw_enforcement = self.grid.lookup(m.level, recurrence);
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
                lesson_id,
                rationale: String::new(),
            }
        } else {
            Verdict::with_enforcement(final_enforcement, m, lesson_id)
        }
    }
}
