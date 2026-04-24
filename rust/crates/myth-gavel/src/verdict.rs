//! Verdict — the output of `Gavel::judge`. Knows how to serialise
//! itself into the Claude Code hook JSON schema.

use myth_common::{Enforcement, LessonId};
use serde_json::{json, Value};

use crate::rules::RuleMatch;

#[derive(Debug, Clone)]
pub struct Verdict {
    pub enforcement: Enforcement,
    pub rule_match: Option<RuleMatch>,
    pub lesson_id: Option<LessonId>,
    pub rationale: String,
    /// Subtleness score [0, 1] from Tier 3 assessor. `None` until
    /// Milestone A activation. See `experiment/remand-prototype/design/
    /// CONSTITUTION-v2.4-remand-draft.md` Part VII Section 2.3.
    pub subtleness_score: Option<f32>,
}

impl Verdict {
    pub fn allow() -> Self {
        Self {
            enforcement: Enforcement::Dismiss,
            rule_match: None,
            lesson_id: None,
            rationale: String::new(),
            subtleness_score: None,
        }
    }

    /// Bedrock match → always Seal. Bypasses Grid and fatigue caps.
    pub fn seal(rule_match: RuleMatch) -> Self {
        let rationale = format!(
            "Bedrock Rule {} matched: {} (level {})",
            rule_match.rule_id, rule_match.item, rule_match.level as u8
        );
        Self {
            enforcement: Enforcement::Seal,
            rule_match: Some(rule_match),
            lesson_id: None,
            rationale,
            subtleness_score: None,
        }
    }

    pub fn with_enforcement(
        enforcement: Enforcement,
        rule_match: RuleMatch,
        lesson_id: Option<LessonId>,
    ) -> Self {
        let rationale = format!(
            "{}: rule {} (item {}, level {})",
            enforcement.label_ko(),
            rule_match.rule_id,
            rule_match.item,
            rule_match.level as u8
        );
        Self {
            enforcement,
            rule_match: Some(rule_match),
            lesson_id,
            rationale,
            subtleness_score: None,
        }
    }

    pub fn is_blocking(&self) -> bool {
        self.enforcement.is_blocking()
    }

    pub fn to_hook_json(&self) -> Value {
        match self.enforcement {
            Enforcement::Dismiss | Enforcement::Note => json!({ "continue": true }),
            Enforcement::Advisory | Enforcement::Caution => json!({
                "continue": true,
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "allow",
                    "additionalContext": self.rationale,
                }
            }),
            // Remand variant is reserved for Milestone A activation. In
            // v0.2 it must never fire; if it does, demote to Warn-equivalent
            // hook output as a fail-safe (CONSTITUTION v2.4 draft VII.2.6).
            Enforcement::Warn | Enforcement::Remand => json!({
                "continue": true,
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "ask",
                    "additionalContext": self.rationale,
                }
            }),
            Enforcement::Strike | Enforcement::Seal => json!({
                "continue": false,
                "stopReason": self.rationale,
            }),
        }
    }
}
