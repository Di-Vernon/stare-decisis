//! `myth link` / `myth unlink` — project-level hook wiring.
//!
//! Non-destructive wrapper around `.claude/settings(.local).json`. Adds or
//! removes myth hook entries without touching unrelated third-party entries.
//!
//! Supersedes `scripts/install_myth_to_project.sh` (deprecated in v0.1.1).

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::args::{LinkArgs, UnlinkArgs};

/// Canonical event → binary mapping. Output order mirrors this list.
const MYTH_HOOKS: &[(&str, &str)] = &[
    ("PreToolUse", "myth-hook-pre-tool"),
    ("PostToolUse", "myth-hook-post-tool"),
    ("PostToolUseFailure", "myth-hook-post-tool-failure"),
    ("SessionStart", "myth-hook-session-start"),
    ("UserPromptSubmit", "myth-hook-user-prompt"),
    ("Stop", "myth-hook-stop"),
];

const MYTH_PREFIX: &str = "myth-hook-";

#[derive(Debug, PartialEq, Eq)]
pub enum LinkStatus {
    Added,
    Unchanged,
}

#[derive(Debug, PartialEq, Eq)]
pub enum UnlinkStatus {
    Removed,
    NotPresent,
}

#[derive(Debug)]
pub struct LinkReport {
    pub entries: Vec<(String, LinkStatus)>,
}

#[derive(Debug)]
pub struct UnlinkReport {
    pub entries: Vec<(String, UnlinkStatus)>,
}

// ── pure functions (test-friendly core) ──────────────────────────

/// Add myth hook entries to `settings`. Idempotent, non-destructive.
/// Non-object input is replaced with an empty object.
pub fn link_hooks(settings: Value) -> (Value, LinkReport) {
    let mut settings = ensure_object(settings);
    let obj = settings.as_object_mut().unwrap();

    let hooks = obj
        .entry("hooks".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    if !hooks.is_object() {
        *hooks = Value::Object(Map::new());
    }
    let hooks_map = hooks.as_object_mut().unwrap();

    let mut entries = Vec::with_capacity(MYTH_HOOKS.len());

    for (event, binary) in MYTH_HOOKS {
        let event_arr = hooks_map
            .entry((*event).to_string())
            .or_insert_with(|| Value::Array(Vec::new()));
        if !event_arr.is_array() {
            *event_arr = Value::Array(Vec::new());
        }
        let arr = event_arr.as_array_mut().unwrap();

        let already_present = arr.iter().any(|entry| entry_has_command(entry, binary));
        if already_present {
            entries.push(((*event).to_string(), LinkStatus::Unchanged));
        } else {
            arr.push(json!({
                "hooks": [
                    { "type": "command", "command": *binary }
                ]
            }));
            entries.push(((*event).to_string(), LinkStatus::Added));
        }
    }

    (settings, LinkReport { entries })
}

/// Remove myth hook entries from `settings`. Non-myth entries preserved.
/// If an event array becomes empty, its key is dropped so repeated
/// link/unlink cycles don't accumulate `{}` schema noise.
pub fn unlink_hooks(settings: Value) -> (Value, UnlinkReport) {
    let mut settings = settings;
    let mut entries = Vec::with_capacity(MYTH_HOOKS.len());

    let Some(obj) = settings.as_object_mut() else {
        for (event, _) in MYTH_HOOKS {
            entries.push(((*event).to_string(), UnlinkStatus::NotPresent));
        }
        return (settings, UnlinkReport { entries });
    };

    let Some(hooks) = obj.get_mut("hooks").and_then(|v| v.as_object_mut()) else {
        for (event, _) in MYTH_HOOKS {
            entries.push(((*event).to_string(), UnlinkStatus::NotPresent));
        }
        return (settings, UnlinkReport { entries });
    };

    for (event, _binary) in MYTH_HOOKS {
        let key = (*event).to_string();
        let removed = match hooks.get_mut(&key) {
            Some(Value::Array(arr)) => {
                let before = arr.len();
                arr.retain(|entry| !entry_contains_any_myth_command(entry));
                let after = arr.len();
                let changed = before != after;
                if arr.is_empty() {
                    hooks.remove(&key);
                }
                changed
            }
            _ => false,
        };
        entries.push((
            key,
            if removed {
                UnlinkStatus::Removed
            } else {
                UnlinkStatus::NotPresent
            },
        ));
    }

    (settings, UnlinkReport { entries })
}

fn entry_has_command(entry: &Value, binary: &str) -> bool {
    entry
        .get("hooks")
        .and_then(|h| h.as_array())
        .map(|arr| {
            arr.iter()
                .any(|h| h.get("command").and_then(|c| c.as_str()) == Some(binary))
        })
        .unwrap_or(false)
}

fn entry_contains_any_myth_command(entry: &Value) -> bool {
    entry
        .get("hooks")
        .and_then(|h| h.as_array())
        .map(|arr| {
            arr.iter().any(|h| {
                h.get("command")
                    .and_then(|c| c.as_str())
                    .map(|s| s.starts_with(MYTH_PREFIX))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

fn ensure_object(v: Value) -> Value {
    if v.is_object() {
        v
    } else {
        Value::Object(Map::new())
    }
}

// ── I/O wrappers (spec-public) ───────────────────────────────────

/// Wire myth hooks into `{path}/.claude/settings(.local).json`.
pub fn run_link(path: &Path) -> Result<()> {
    let project = path
        .canonicalize()
        .with_context(|| format!("resolving {}", path.display()))?;
    let claude_dir = project.join(".claude");
    if !claude_dir.exists() {
        return Err(anyhow!(
            "{} not found. Run `myth init` in the project first.",
            claude_dir.display()
        ));
    }

    let target = choose_target(&claude_dir)?;
    let existing = read_settings(&target)?;
    let (updated, report) = link_hooks(existing);

    backup_file(&target)?;
    atomic_write_json(&target, &updated)?;

    let rel = target
        .strip_prefix(&project)
        .unwrap_or(target.as_path())
        .display();
    println!("Linked myth hooks into {} ({})", project.display(), rel);
    for (event, status) in &report.entries {
        let tag = match status {
            LinkStatus::Added => "added",
            LinkStatus::Unchanged => "unchanged",
        };
        println!("  {:<24} [{}]", event, tag);
    }
    Ok(())
}

/// Remove myth hook entries from `{path}/.claude/settings(.local).json`.
pub fn run_unlink(path: &Path) -> Result<()> {
    let project = path
        .canonicalize()
        .with_context(|| format!("resolving {}", path.display()))?;
    let claude_dir = project.join(".claude");
    if !claude_dir.exists() {
        return Err(anyhow!(
            "{} not found. Nothing to unlink.",
            claude_dir.display()
        ));
    }

    let target = choose_target(&claude_dir)?;
    let existing = read_settings(&target)?;
    let (updated, report) = unlink_hooks(existing);

    backup_file(&target)?;
    atomic_write_json(&target, &updated)?;

    let rel = target
        .strip_prefix(&project)
        .unwrap_or(target.as_path())
        .display();
    println!("Unlinked myth hooks from {} ({})", project.display(), rel);
    for (event, status) in &report.entries {
        let tag = match status {
            UnlinkStatus::Removed => "removed",
            UnlinkStatus::NotPresent => "not present",
        };
        println!("  {:<24} [{}]", event, tag);
    }
    Ok(())
}

/// Pick `settings.local.json` over `settings.json`; fail if neither exists.
fn choose_target(claude_dir: &Path) -> Result<PathBuf> {
    let local = claude_dir.join("settings.local.json");
    let main = claude_dir.join("settings.json");
    if local.exists() {
        Ok(local)
    } else if main.exists() {
        Ok(main)
    } else {
        Err(anyhow!(
            "no settings.json or settings.local.json under {}. Run `myth init` first.",
            claude_dir.display()
        ))
    }
}

fn read_settings(target: &Path) -> Result<Value> {
    let body = std::fs::read_to_string(target)
        .with_context(|| format!("reading {}", target.display()))?;
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Ok(Value::Object(Map::new()));
    }
    let parsed: Value = serde_json::from_str(trimmed)
        .with_context(|| format!("parsing {} as JSON", target.display()))?;
    Ok(ensure_object(parsed))
}

fn backup_file(target: &Path) -> Result<()> {
    if !target.exists() {
        return Ok(());
    }
    let ts = Utc::now().format("%Y%m%d-%H%M%S");
    let file_name = target
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("invalid target path {}", target.display()))?;
    let backup = target.with_file_name(format!("{file_name}.pre-myth-{ts}"));
    std::fs::copy(target, &backup).with_context(|| {
        format!("backing up {} -> {}", target.display(), backup.display())
    })?;
    Ok(())
}

fn atomic_write_json(target: &Path, value: &Value) -> Result<()> {
    let body = serde_json::to_string_pretty(value)
        .with_context(|| format!("serializing JSON for {}", target.display()))?;
    let file_name = target
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("invalid target path {}", target.display()))?;
    let tmp = target.with_file_name(format!("{file_name}.tmp"));
    std::fs::write(&tmp, format!("{body}\n"))
        .with_context(|| format!("writing {}", tmp.display()))?;
    std::fs::rename(&tmp, target)
        .with_context(|| format!("renaming {} -> {}", tmp.display(), target.display()))?;
    Ok(())
}

// ── dispatcher-facing async wrappers (for main.rs) ──────────────

pub async fn run(args: LinkArgs) -> Result<ExitCode> {
    run_link(&args.path)?;
    Ok(ExitCode::SUCCESS)
}

pub async fn unlink(args: UnlinkArgs) -> Result<ExitCode> {
    run_unlink(&args.path)?;
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_hooks_adds_all_six_to_empty() {
        let (out, report) = link_hooks(json!({}));
        assert_eq!(report.entries.len(), 6);
        for (_, status) in &report.entries {
            assert_eq!(status, &LinkStatus::Added);
        }
        let hooks = out.get("hooks").and_then(|v| v.as_object()).unwrap();
        assert_eq!(hooks.len(), 6);
        for (event, binary) in MYTH_HOOKS {
            let arr = hooks.get(*event).and_then(|v| v.as_array()).unwrap();
            assert_eq!(arr.len(), 1);
            assert!(entry_has_command(&arr[0], binary));
        }
    }

    #[test]
    fn link_hooks_is_idempotent() {
        let (out1, _) = link_hooks(json!({}));
        let (out2, report2) = link_hooks(out1.clone());
        assert_eq!(out1, out2, "second link should not mutate");
        for (_, status) in &report2.entries {
            assert_eq!(status, &LinkStatus::Unchanged);
        }
    }

    #[test]
    fn link_hooks_preserves_third_party_entries() {
        let input = json!({
            "permissions": { "allow": ["Bash(ls)"] },
            "hooks": {
                "PreToolUse": [
                    { "hooks": [{ "type": "command", "command": "third-party" }] }
                ]
            }
        });
        let (out, _) = link_hooks(input);
        let pre = out
            .pointer("/hooks/PreToolUse")
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(pre.len(), 2);
        assert_eq!(
            pre[0].pointer("/hooks/0/command").and_then(|c| c.as_str()),
            Some("third-party")
        );
        assert_eq!(
            pre[1].pointer("/hooks/0/command").and_then(|c| c.as_str()),
            Some("myth-hook-pre-tool")
        );
        assert_eq!(
            out.pointer("/permissions/allow/0").and_then(|c| c.as_str()),
            Some("Bash(ls)")
        );
    }

    #[test]
    fn unlink_hooks_removes_only_myth() {
        let input = json!({
            "hooks": {
                "PreToolUse": [
                    { "hooks": [{ "type": "command", "command": "third-party" }] },
                    { "hooks": [{ "type": "command", "command": "myth-hook-pre-tool" }] }
                ]
            }
        });
        let (out, report) = unlink_hooks(input);
        let arr = out
            .pointer("/hooks/PreToolUse")
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(
            arr[0].pointer("/hooks/0/command").and_then(|c| c.as_str()),
            Some("third-party")
        );
        assert!(report
            .entries
            .iter()
            .any(|(e, s)| e == "PreToolUse" && matches!(s, UnlinkStatus::Removed)));
    }

    #[test]
    fn unlink_hooks_drops_empty_event_keys() {
        let (linked, _) = link_hooks(json!({}));
        let (out, _) = unlink_hooks(linked);
        let hooks = out.get("hooks").and_then(|v| v.as_object()).unwrap();
        assert!(hooks.is_empty(), "empty-after-unlink keys should be dropped");
    }

    #[test]
    fn ensure_object_replaces_non_object() {
        assert!(ensure_object(Value::Null).is_object());
        assert!(ensure_object(json!("string")).is_object());
        assert!(ensure_object(json!([1, 2])).is_object());
        assert!(ensure_object(json!({"k": 1})).is_object());
    }
}
