//! `myth doctor` — health check 다항목.

use anyhow::Result;
use myth_db::Database;
use std::process::ExitCode;

use crate::args::DoctorArgs;

#[derive(Debug)]
pub enum CheckResult {
    Pass(String),
    Warn(String),
    Fail(String),
}

pub async fn run(args: DoctorArgs) -> Result<ExitCode> {
    let mut checks = vec![
        check_claude_binary(),
        check_myth_home(),
        check_rules_files(),
        check_sqlite_integrity(),
        check_embed_socket(),
    ];

    if args.perf_check {
        checks.push(check_build_profile());
    }
    if args.wsl_check {
        checks.push(check_wsl2_environment());
    }
    if args.migration {
        checks.extend(check_all_milestones());
    }

    let mut failed = 0;
    for check in &checks {
        match check {
            CheckResult::Pass(m) => println!("  [ok]   {m}"),
            CheckResult::Warn(m) => println!("  [warn] {m}"),
            CheckResult::Fail(m) => {
                println!("  [fail] {m}");
                failed += 1;
            }
        }
    }

    if failed > 0 {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn check_claude_binary() -> CheckResult {
    match myth_runtime::discovery::find_claude() {
        Ok(p) => CheckResult::Pass(format!("claude binary: {}", p.display())),
        Err(e) => CheckResult::Fail(format!("claude binary not found: {e}")),
    }
}

fn check_myth_home() -> CheckResult {
    let home = myth_common::myth_home();
    if home.exists() {
        CheckResult::Pass(format!("myth home: {}", home.display()))
    } else {
        CheckResult::Warn(format!(
            "myth home missing: {} (run `myth install`)",
            home.display()
        ))
    }
}

fn check_rules_files() -> CheckResult {
    let home = myth_common::myth_home();
    let missing: Vec<&str> = [
        ("bedrock-rules.yaml", home.join("bedrock-rules.yaml")),
        ("foundation-rules.yaml", home.join("foundation-rules.yaml")),
        ("grid.yaml", home.join("grid.yaml")),
    ]
    .iter()
    .filter(|(_, p)| !p.exists())
    .map(|(name, _)| *name)
    .collect();
    if missing.is_empty() {
        CheckResult::Pass("rules files present".into())
    } else {
        CheckResult::Warn(format!("missing rules: {}", missing.join(", ")))
    }
}

fn check_sqlite_integrity() -> CheckResult {
    match Database::open(&myth_common::state_db_path()) {
        Ok(_) => CheckResult::Pass("state.db open + migrations applied".into()),
        Err(e) => CheckResult::Fail(format!("state.db error: {e}")),
    }
}

fn check_embed_socket() -> CheckResult {
    use std::os::unix::net::UnixStream;
    let path = myth_common::embed_socket_path();
    if !path.exists() {
        return CheckResult::Warn(format!(
            "embed socket not present: {} (daemon will auto-spawn on first use)",
            path.display()
        ));
    }
    match UnixStream::connect(&path) {
        Ok(_) => CheckResult::Pass(format!("myth-embed socket reachable: {}", path.display())),
        Err(e) => CheckResult::Warn(format!("embed socket {} unreachable: {e}", path.display())),
    }
}

fn check_build_profile() -> CheckResult {
    // current_exe에 debug symbol 많으면 debug build 의심
    if let Ok(exe) = std::env::current_exe() {
        if let Ok(meta) = std::fs::metadata(&exe) {
            let size = meta.len();
            // release strip=symbols 기준 이진은 일반적으로 <100MB; debug는 훨씬 큼
            return if size < 200_000_000 {
                CheckResult::Pass(format!("myth binary size OK ({size} bytes)"))
            } else {
                CheckResult::Warn(format!(
                    "myth binary size {size} bytes — ensure `cargo build --release`"
                ))
            };
        }
    }
    CheckResult::Warn("cannot stat current_exe".into())
}

fn check_wsl2_environment() -> CheckResult {
    let version = std::fs::read_to_string("/proc/version").unwrap_or_default();
    if version.contains("WSL") || version.contains("Microsoft") {
        CheckResult::Pass("WSL2 environment detected".into())
    } else {
        CheckResult::Pass("native Linux (not WSL)".into())
    }
}

fn check_all_milestones() -> Vec<CheckResult> {
    use myth_ui::panels::migration::{milestone_c_from_path, MilestoneStatus};
    let c: MilestoneStatus = milestone_c_from_path(myth_common::hook_latency_path());
    let mk = |ms: MilestoneStatus| -> CheckResult {
        let line = format!(
            "Milestone {}: {} (current: {}, threshold: {})",
            ms.id, ms.title, ms.current_value, ms.threshold
        );
        if ms.triggered {
            CheckResult::Warn(format!("{line} — TRIGGERED"))
        } else {
            CheckResult::Pass(line)
        }
    };
    // Milestones A/B/D/E는 Day-1 데이터 없음 (pending)
    vec![
        CheckResult::Pass("Milestone A: pending (Assessor Tier review @ 3w)".into()),
        CheckResult::Pass("Milestone B: pending (Vector store migration)".into()),
        mk(c),
        CheckResult::Pass("Milestone D: pending (Semantic detection)".into()),
        CheckResult::Pass("Milestone E: pending (AST validation)".into()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_rules_files_variants() {
        // 파일 없으면 Warn 반환
        let _ = check_rules_files();
    }

    #[test]
    fn check_wsl_returns_something() {
        let r = check_wsl2_environment();
        assert!(matches!(r, CheckResult::Pass(_)));
    }

    #[test]
    fn check_all_milestones_returns_five() {
        let all = check_all_milestones();
        assert_eq!(all.len(), 5);
    }
}
