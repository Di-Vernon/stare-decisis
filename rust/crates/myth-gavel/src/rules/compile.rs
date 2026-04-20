//! YAML → compiled regex rules. Used by bedrock/foundation/surface
//! loaders.

use std::path::Path;

use anyhow::{anyhow, Context};
use myth_common::Level;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Likelihood {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
pub struct CompiledRule {
    pub id: String,
    pub item: String,
    pub regex: regex::Regex,
    pub level: Level,
    pub likelihood: Likelihood,
    pub source: Option<String>,
}

pub struct CompiledRules {
    rules: Vec<CompiledRule>,
    set: regex::RegexSet,
}

#[derive(Debug, Clone)]
pub struct RuleMatch {
    pub rule_id: String,
    pub item: String,
    pub level: Level,
    pub matched_span: (usize, usize),
    pub matched_text: String,
}

// ---- YAML schema ------------------------------------------------------

#[derive(Deserialize)]
struct RuleFile {
    #[allow(dead_code)]
    version: u32,
    #[serde(default)]
    items: Vec<RuleItem>,
}

#[derive(Deserialize)]
struct RuleItem {
    id: String,
    #[serde(default)]
    #[allow(dead_code)]
    description: Option<String>,
    rules: Vec<RuleEntry>,
}

#[derive(Deserialize)]
struct RuleEntry {
    id: String,
    pattern: String,
    level: u8,
    #[serde(default = "default_likelihood")]
    likelihood: Likelihood,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    tests: Option<serde_yaml::Value>,
}

fn default_likelihood() -> Likelihood {
    Likelihood::Medium
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

// ---- CompiledRules API ------------------------------------------------

impl CompiledRules {
    pub fn empty() -> Self {
        Self {
            rules: Vec::new(),
            set: regex::RegexSet::empty(),
        }
    }

    pub fn load(path: &Path, category: &str) -> anyhow::Result<Self> {
        if !path.exists() {
            tracing::warn!(
                path = ?path,
                category,
                "rule file missing; using empty rule set"
            );
            return Ok(Self::empty());
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("reading {:?}", path))?;
        Self::from_yaml_str(&content, category, Some(path))
    }

    pub fn from_yaml_str(
        yaml: &str,
        category: &str,
        origin: Option<&Path>,
    ) -> anyhow::Result<Self> {
        let file: RuleFile = serde_yaml::from_str(yaml).with_context(|| match origin {
            Some(p) => format!("parsing YAML {:?}", p),
            None => format!("parsing YAML (inline, category={})", category),
        })?;

        let mut compiled = Vec::new();
        let mut patterns = Vec::new();

        for item in file.items {
            for rule in item.rules {
                let re = regex::Regex::new(&rule.pattern).with_context(|| {
                    format!("compiling regex for rule {}/{}", item.id, rule.id)
                })?;
                let level = level_from_int(rule.level).with_context(|| {
                    format!("parsing level for rule {}/{}", item.id, rule.id)
                })?;
                patterns.push(rule.pattern.clone());
                compiled.push(CompiledRule {
                    id: rule.id,
                    item: item.id.clone(),
                    regex: re,
                    level,
                    likelihood: rule.likelihood,
                    source: rule.source,
                });
            }
        }

        let set = regex::RegexSet::new(&patterns).context("building RegexSet")?;
        Ok(Self {
            rules: compiled,
            set,
        })
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub fn match_any(&self, text: &str) -> Option<RuleMatch> {
        let matches = self.set.matches(text);
        if !matches.matched_any() {
            return None;
        }
        let idx = matches.into_iter().next()?;
        let rule = &self.rules[idx];
        let m = rule.regex.find(text)?;
        Some(RuleMatch {
            rule_id: rule.id.clone(),
            item: rule.item.clone(),
            level: rule.level,
            matched_span: (m.start(), m.end()),
            matched_text: m.as_str().to_string(),
        })
    }
}
