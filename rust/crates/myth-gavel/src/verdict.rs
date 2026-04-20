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
}

impl Verdict {
    pub fn allow() -> Self {
        Self {
            enforcement: Enforcement::Dismiss,
            rule_match: None,
            lesson_id: None,
            rationale: String::new(),
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
            Enforcement::Warn => json!({
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
