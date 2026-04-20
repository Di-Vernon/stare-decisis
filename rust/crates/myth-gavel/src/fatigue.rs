//! Per-session enforcement caps per Constitution Article 15.
//!
//! Caps are hard-coded for Day-1 (advisory 2, caution 3, warn 2) and
//! are not environment-variable overridable. If later policy demands
//! tuning, introduce config plumbing explicitly — don't silently
//! accept an env-var knob (Article 15 amendment procedure applies).

use std::collections::HashMap;

use myth_common::{Enforcement, SessionId};

pub const ADVISORY_CAP: u32 = 2;
pub const CAUTION_CAP: u32 = 3;
pub const WARN_CAP: u32 = 2;

#[derive(Debug, Default)]
pub struct SessionFatigue {
    pub advisory: u32,
    pub caution: u32,
    pub warn: u32,
}

pub struct FatigueTracker {
    session_counts: HashMap<SessionId, SessionFatigue>,
}

impl FatigueTracker {
    pub fn new() -> Self {
        Self {
            session_counts: HashMap::new(),
        }
    }

    /// Register an enforcement intent for this session and return the
    /// enforcement that should actually fire. When a cap is reached,
    /// the intensity steps down one level (Warn → Caution, Caution →
    /// Note, Advisory → Note). Strike and Seal never downgrade.
    pub fn register(&mut self, session_id: SessionId, enforcement: Enforcement) -> Enforcement {
        let fatigue = self.session_counts.entry(session_id).or_default();
        match enforcement {
            Enforcement::Advisory => {
                if fatigue.advisory >= ADVISORY_CAP {
                    return Enforcement::Note;
                }
                fatigue.advisory += 1;
                Enforcement::Advisory
            }
            Enforcement::Caution => {
                if fatigue.caution >= CAUTION_CAP {
                    return Enforcement::Note;
                }
                fatigue.caution += 1;
                Enforcement::Caution
            }
            Enforcement::Warn => {
                if fatigue.warn >= WARN_CAP {
                    return Enforcement::Caution;
                }
                fatigue.warn += 1;
                Enforcement::Warn
            }
            other => other,
        }
    }

    pub fn snapshot(&self, session_id: SessionId) -> Option<&SessionFatigue> {
        self.session_counts.get(&session_id)
    }
}

impl Default for FatigueTracker {
    fn default() -> Self {
        Self::new()
    }
}
