//! `myth install / uninstall` — `~/.local/bin` 심볼릭 링크.
//!
//! 드리프트 2 대응: templates/ 전체를 `~/.myth/templates/`로 복사.
//! 드리프트 3 대응: 빌드 경로 fallback 3단계.

use anyhow::{anyhow, Context, Result};
use myth_db::Database;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::args::{InstallArgs, UninstallArgs};

const BINARIES: &[&str] = &[
    "myth",
    "myth-hook-pre-tool",
    "myth-hook-post-tool",
    "myth-hook-post-tool-failure",
    "myth-hook-user-prompt",
    "myth-hook-stop",
    "myth-hook-session-start",
    "myth-embed",
];

pub async fn run(args: InstallArgs) -> Result<ExitCode> {
    let bin_dir = resolve_bin_dir(&args.prefix)?;
    std::fs::create_dir_all(&bin_dir)?;

    let rust_target = locate_rust_target()?;
    let repo_root = rust_target
        .ancestors()
        .nth(2)
        .ok_or_else(|| anyhow!("cannot derive repo root from {}", rust_target.display()))?
        .to_path_buf();

    for bin in BINARIES {
        let src = rust_target.join(bin);
        let dst = bin_dir.join(bin);

        if !src.exists() {
            eprintln!(
                "warning: binary {} not found at {}, skipping. Run `cargo build --release` in {}/rust first.",
                bin,
                src.display(),
                repo_root.display()
            );
            continue;
        }
        if dst.exists() || dst.is_symlink() {
            std::fs::remove_file(&dst).ok();
        }
        symlink_or_copy(&src, &dst)?;
        println!("installed: {}", dst.display());
    }

    write_python_shim(&bin_dir, "myth-assessor", "myth_py.assessor.cli")?;
    write_python_shim(&bin_dir, "myth-observer", "myth_py.observer.cli")?;

    copy_templates(&repo_root)?;
    init_myth_home()?;
    install_python_package(&repo_root);

    if !std::env::var("PATH")
        .unwrap_or_default()
        .contains(".local/bin")
    {
        eprintln!();
        eprintln!("warning: ~/.local/bin is not in your PATH");
        eprintln!("Add to ~/.bashrc: export PATH=\"$HOME/.local/bin:$PATH\"");
    }

    Ok(ExitCode::SUCCESS)
}

pub async fn uninstall(args: UninstallArgs) -> Result<ExitCode> {
    let bin_dir = resolve_bin_dir(&args.prefix)?;
    for bin in BINARIES
        .iter()
        .copied()
        .chain(["myth-assessor", "myth-observer"])
    {
        let dst = bin_dir.join(bin);
        if dst.exists() || dst.is_symlink() {
            std::fs::remove_file(&dst)
                .with_context(|| format!("removing {}", dst.display()))?;
            println!("removed: {}", dst.display());
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn resolve_bin_dir(prefix: &Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = prefix {
        return Ok(p.clone());
    }
    let home = dirs::home_dir().ok_or_else(|| anyhow!("HOME not set"))?;
    Ok(home.join(".local/bin"))
}

/// 빌드 경로 fallback 3단계 (드리프트 3):
///   1. `std::env::current_exe()?.parent()` — 그 parent에 myth-embed 등 형제 바이너리가 있으면 target dir로 판정
///   2. `MYTH_REPO_ROOT` env — `{root}/rust/target/release`
///   3. hardcoded `~/myth/rust/target/release/`
pub fn locate_rust_target() -> Result<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            if parent.join("myth-embed").exists()
                || parent.join("myth-hook-pre-tool").exists()
            {
                return Ok(parent.to_path_buf());
            }
        }
    }
    if let Ok(root) = std::env::var("MYTH_REPO_ROOT") {
        let p = PathBuf::from(root).join("rust/target/release");
        if p.exists() {
            return Ok(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join("myth/rust/target/release");
        if p.exists() {
            return Ok(p);
        }
    }
    Err(anyhow!(
        "binary source not found. Set MYTH_REPO_ROOT or run from repo."
    ))
}

fn symlink_or_copy(src: &Path, dst: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(src, dst)
            .with_context(|| format!("symlink {} -> {}", dst.display(), src.display()))?;
    }
    #[cfg(not(unix))]
    {
        std::fs::copy(src, dst)
            .with_context(|| format!("copy {} -> {}", src.display(), dst.display()))?;
    }
    Ok(())
}

fn write_python_shim(bin_dir: &Path, name: &str, module: &str) -> Result<()> {
    let path = bin_dir.join(name);
    let body = format!(
        "#!/usr/bin/env bash\n\
         exec python3 -m {module} \"$@\"\n"
    );
    std::fs::write(&path, body)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
    }
    println!("installed: {}", path.display());
    Ok(())
}

/// Repo의 templates/ 디렉토리를 `~/.myth/templates/`로 복사. `myth init`이
/// 소스로 사용한다.
fn copy_templates(repo_root: &Path) -> Result<()> {
    let src = repo_root.join("templates");
    if !src.exists() {
        eprintln!(
            "warning: {} not found, skipping templates copy",
            src.display()
        );
        return Ok(());
    }
    let dst = myth_common::myth_home().join("templates");
    copy_dir_recursive(&src, &dst)?;
    println!("templates copied: {}", dst.display());
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &target)?;
        } else {
            std::fs::copy(&path, &target)?;
        }
    }
    Ok(())
}

/// Wave 8 Task 8.2 — best-effort auto-install of the Python package.
///
/// Prefers `uv pip` (fast, reproducible) over bare `pip` / `pip3`.
/// On any failure (tool missing, install error, missing python dir)
/// prints a warning with the exact manual command and returns
/// without propagating — the Rust-side install still succeeds so
/// users don't lose the symlinked binaries over a Python hiccup.
fn install_python_package(repo_root: &Path) {
    let python_dir = repo_root.join("python");
    if !python_dir.exists() {
        eprintln!(
            "warning: python package dir {} not found, skipping Python install",
            python_dir.display()
        );
        return;
    }

    let (installer, args): (&str, Vec<&str>) = if which_exists("uv") {
        ("uv", vec!["pip", "install", "-e", ".[dev]"])
    } else if which_exists("pip3") {
        ("pip3", vec!["install", "-e", ".[dev]"])
    } else if which_exists("pip") {
        ("pip", vec!["install", "-e", ".[dev]"])
    } else {
        eprintln!();
        eprintln!("warning: neither `uv` nor `pip`/`pip3` found on PATH");
        eprintln!("  To install myth-py manually:");
        eprintln!(
            "    cd {} && uv pip install -e \".[dev]\"",
            python_dir.display()
        );
        return;
    };

    println!("installing Python package with {}...", installer);
    let status = std::process::Command::new(installer)
        .args(&args)
        .current_dir(&python_dir)
        .status();

    match status {
        Ok(st) if st.success() => {
            println!("Python package installed: myth_py (editable, with dev extras)");
        }
        Ok(st) => {
            eprintln!();
            eprintln!(
                "warning: {} exited with status {} — Python install failed",
                installer,
                st.code().unwrap_or(-1)
            );
            eprintln!(
                "  Retry manually: cd {} && {} {}",
                python_dir.display(),
                installer,
                args.join(" ")
            );
        }
        Err(e) => {
            eprintln!();
            eprintln!("warning: failed to spawn {}: {}", installer, e);
            eprintln!(
                "  Install manually: cd {} && {} {}",
                python_dir.display(),
                installer,
                args.join(" ")
            );
        }
    }
}

/// Cheap `which`-equivalent. Returns true iff `name` resolves on
/// PATH. Uses `sh -c command -v` so it works across shells and
/// avoids pulling in a dependency.
fn which_exists(name: &str) -> bool {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {} > /dev/null 2>&1", name))
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn init_myth_home() -> Result<()> {
    let home = myth_common::myth_home();
    std::fs::create_dir_all(&home)?;
    std::fs::create_dir_all(home.join("metrics"))?;
    std::fs::create_dir_all(home.join("archive"))?;
    std::fs::create_dir_all(myth_common::myth_state())?;

    // 기본 rule/grid 파일 (이미 있으면 건드리지 않음) — include_str! (컴파일 타임 보장)
    if !home.join("bedrock-rules.yaml").exists() {
        std::fs::write(
            home.join("bedrock-rules.yaml"),
            include_str!("../../../../../templates/bedrock-rules.yaml"),
        )?;
    }
    if !home.join("foundation-rules.yaml").exists() {
        std::fs::write(
            home.join("foundation-rules.yaml"),
            include_str!("../../../../../templates/foundation-rules.yaml"),
        )?;
    }
    if !home.join("surface-rules.yaml").exists() {
        std::fs::write(home.join("surface-rules.yaml"), "rules: []\n")?;
    }
    if !home.join("grid.yaml").exists() {
        std::fs::write(
            home.join("grid.yaml"),
            include_str!("../../../../../templates/grid.yaml"),
        )?;
    }

    // SQLite 초기화 (Database::open이 migration 적용)
    let _ = Database::open(&myth_common::state_db_path())
        .context("initializing state.db")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locate_rust_target_env_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        let fake_target = tmp.path().join("rust/target/release");
        std::fs::create_dir_all(&fake_target).unwrap();
        std::env::set_var("MYTH_REPO_ROOT", tmp.path());
        // current_exe 바이너리 옆에 myth-embed는 없는 상태에서 env fallback 시도
        // 다만 이 테스트는 test 바이너리의 current_exe.parent()가 fake_target과 다를 때만 의미
        let out = locate_rust_target();
        std::env::remove_var("MYTH_REPO_ROOT");
        // 적어도 에러 없이 하나는 선택되어야
        assert!(out.is_ok());
    }

    #[test]
    fn binaries_count() {
        assert_eq!(BINARIES.len(), 8);
    }

    #[test]
    fn which_exists_resolves_sh() {
        // `sh` exists on every POSIX system myth targets.
        assert!(which_exists("sh"));
    }

    #[test]
    fn which_exists_rejects_bogus() {
        assert!(!which_exists("definitely_not_a_real_binary_xyzzy_9999"));
    }
}
