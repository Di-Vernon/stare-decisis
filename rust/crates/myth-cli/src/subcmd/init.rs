//! `myth init [path]` — .claude/ 스캐폴딩.
//!
//! 드리프트 1 대응: templates/CLAUDE.md.template 없으면 skip + warning.

use anyhow::{anyhow, Context, Result};
use std::path::Path;
use std::process::ExitCode;

use crate::args::InitArgs;

pub async fn run(args: InitArgs) -> Result<ExitCode> {
    let project = args
        .path
        .canonicalize()
        .with_context(|| format!("resolving {:?}", args.path))?;

    if !project.join(".git").exists() {
        eprintln!("warning: {} is not a git repository", project.display());
    }

    let claude_dir = project.join(".claude");
    if claude_dir.exists() && !args.force {
        return Err(anyhow!(
            "{} already exists. Use --force to overwrite.",
            claude_dir.display()
        ));
    }

    std::fs::create_dir_all(claude_dir.join("agents"))?;

    // 1. settings.json — templates/.claude/settings.json.template이 있으면 복사.
    let templates = myth_common::myth_home().join("templates");
    let settings_src = templates.join(".claude/settings.json.template");
    let settings_dst = claude_dir.join("settings.json");
    if settings_src.exists() {
        std::fs::copy(&settings_src, &settings_dst)?;
    } else {
        // 최소 기본값
        std::fs::write(&settings_dst, default_settings_json())?;
    }

    // 2. agents/*.md
    copy_if_exists(
        &templates.join(".claude/agents/assessor.md"),
        &claude_dir.join("agents/assessor.md"),
    )?;
    copy_if_exists(
        &templates.join(".claude/agents/observer.md"),
        &claude_dir.join("agents/observer.md"),
    )?;

    // 3. CLAUDE.md — 템플릿 없으면 skip + warning (드리프트 1 처리)
    let claude_md_dst = project.join("CLAUDE.md");
    let claude_md_src = templates.join("CLAUDE.md.template");
    if !claude_md_dst.exists() {
        match std::fs::read_to_string(&claude_md_src) {
            Ok(body) => {
                std::fs::write(&claude_md_dst, body)?;
                println!("  created: {}", claude_md_dst.display());
            }
            Err(_) => {
                eprintln!(
                    "warning: CLAUDE.md.template not found at {}, skipping CLAUDE.md scaffold",
                    claude_md_src.display()
                );
            }
        }
    }

    println!("myth initialized in {}", project.display());
    println!("  {}", settings_dst.display());
    println!("  {}", claude_dir.join("agents/").display());
    println!();
    println!("Next: run `myth run` to start a supervised Claude Code session.");
    Ok(ExitCode::SUCCESS)
}

fn copy_if_exists(src: &Path, dst: &Path) -> Result<()> {
    if src.exists() {
        std::fs::copy(src, dst)?;
        println!("  created: {}", dst.display());
    } else {
        eprintln!("warning: template {} missing, skipping", src.display());
    }
    Ok(())
}

fn default_settings_json() -> &'static str {
    r#"{
  "$schema": "https://json.schemastore.org/claude-code-settings.json",
  "permissions": {
    "allow": []
  },
  "hooks": {
    "PreToolUse": [],
    "PostToolUse": [],
    "PostToolUseFailure": [],
    "UserPromptSubmit": [],
    "Stop": [],
    "SessionStart": []
  }
}
"#
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn init_scaffolds_claude_dir() {
        let tmp = tempdir().unwrap();
        let args = InitArgs {
            path: tmp.path().to_path_buf(),
            force: false,
        };
        let _code = run(args).await.unwrap();
        assert!(tmp.path().join(".claude/settings.json").exists());
        assert!(tmp.path().join(".claude/agents").exists());
    }

    #[tokio::test]
    async fn init_refuses_existing_without_force() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".claude")).unwrap();
        let args = InitArgs {
            path: tmp.path().to_path_buf(),
            force: false,
        };
        assert!(run(args).await.is_err());
    }

    #[tokio::test]
    async fn init_overwrites_with_force() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".claude")).unwrap();
        let args = InitArgs {
            path: tmp.path().to_path_buf(),
            force: true,
        };
        run(args).await.unwrap();
        assert!(tmp.path().join(".claude/settings.json").exists());
    }
}
