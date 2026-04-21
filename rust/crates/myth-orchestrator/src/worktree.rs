//! git worktree 관리. 실행은 `git` 바이너리 위임.
//!
//! merge_to_main은 fast-forward만 시도. ort 전략 등 복잡 머지는 Wave 8
//! (통합 검증) 때 평가.

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct Worktree {
    pub path: PathBuf,
    pub branch: String,
    pub base: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeResult {
    FastForward,
    Conflict(String),
    Noop,
}

pub async fn create(base_repo: &Path, task_id: &str) -> Result<Worktree> {
    let worktree_path = myth_common::myth_home()
        .join("worktrees")
        .join(task_id);
    let branch = format!("myth/task-{task_id}");

    if worktree_path.exists() {
        return Err(anyhow!(
            "worktree path already exists: {:?}",
            worktree_path
        ));
    }

    let output = Command::new("git")
        .current_dir(base_repo)
        .args(["worktree", "add", "-b", &branch])
        .arg(&worktree_path)
        .output()
        .await?;

    if !output.status.success() {
        return Err(anyhow!(
            "git worktree add failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(Worktree {
        path: worktree_path,
        branch,
        base: base_repo.to_path_buf(),
    })
}

pub async fn remove(wt: &Worktree) -> Result<()> {
    let status = Command::new("git")
        .current_dir(&wt.base)
        .args(["worktree", "remove", "--force"])
        .arg(&wt.path)
        .status()
        .await?;
    if !status.success() {
        return Err(anyhow!("git worktree remove failed"));
    }
    Ok(())
}

/// Fast-forward merge from worktree branch → main_branch. 실패 시 Conflict.
pub async fn merge_to_main(wt: &Worktree, main_branch: &str) -> Result<MergeResult> {
    let co = Command::new("git")
        .current_dir(&wt.base)
        .args(["checkout", main_branch])
        .output()
        .await?;
    if !co.status.success() {
        return Err(anyhow!(
            "git checkout {} failed: {}",
            main_branch,
            String::from_utf8_lossy(&co.stderr)
        ));
    }

    let merge = Command::new("git")
        .current_dir(&wt.base)
        .args(["merge", "--ff-only", &wt.branch])
        .output()
        .await?;

    if merge.status.success() {
        let stdout = String::from_utf8_lossy(&merge.stdout);
        if stdout.contains("Already up to date") || stdout.contains("Already up-to-date") {
            Ok(MergeResult::Noop)
        } else {
            Ok(MergeResult::FastForward)
        }
    } else {
        Ok(MergeResult::Conflict(
            String::from_utf8_lossy(&merge.stderr).into_owned(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worktree_struct_fields() {
        let w = Worktree {
            path: PathBuf::from("/tmp/a"),
            branch: "myth/task-x".into(),
            base: PathBuf::from("/tmp/b"),
        };
        assert_eq!(w.branch, "myth/task-x");
    }

    #[test]
    fn merge_result_enum() {
        assert_eq!(MergeResult::FastForward, MergeResult::FastForward);
        assert_ne!(MergeResult::FastForward, MergeResult::Noop);
        if let MergeResult::Conflict(s) = MergeResult::Conflict("x".into()) {
            assert_eq!(s, "x");
        }
    }
}
