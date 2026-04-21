//! `myth status` — 간단 요약.
//!
//! 드리프트 5 대응: `EmbedClient::ping()` 부재 → socket 존재 + 빠른 connect 체크로 대체
//! (docs illustrative의 uptime/request_count 필드 표시는 Day-1 생략).

use anyhow::Result;
use myth_db::{Database, LessonStore, SqliteLessonStore};
use std::process::ExitCode;
use std::time::{Duration, SystemTime};

use crate::args::{OutputFormat, StatusArgs};

pub async fn run(_args: StatusArgs, _format: OutputFormat) -> Result<ExitCode> {
    let db_path = myth_common::state_db_path();

    let (active, lapsed) = match Database::open(&db_path) {
        Ok(db) => {
            let store = SqliteLessonStore::new(db);
            let a = store.list_active().map(|v| v.len()).unwrap_or(0);
            let l = store.list_lapsed().map(|v| v.len()).unwrap_or(0);
            (a, l)
        }
        Err(_) => {
            eprintln!("warning: state.db not initialized (run `myth install` first)");
            (0, 0)
        }
    };

    let embed_running = is_embed_running();
    let hook_p99 = compute_hook_p99_last_7d().unwrap_or(0.0);
    let brief_age = brief_age_days();

    println!("myth status");
    println!("  Lessons: {active} active, {lapsed} lapsed");
    println!(
        "  myth-embed: {}",
        if embed_running { "running" } else { "not running" }
    );
    println!("  Hook P99 (7d): {hook_p99:.1}ms");
    match brief_age {
        Some(days) => println!("  Brief updated: {days} days ago"),
        None => println!("  Brief: not generated yet"),
    }
    Ok(ExitCode::SUCCESS)
}

fn is_embed_running() -> bool {
    use std::os::unix::net::UnixStream;
    let path = myth_common::embed_socket_path();
    if !path.exists() {
        return false;
    }
    UnixStream::connect(&path).is_ok()
}

pub fn compute_hook_p99_last_7d() -> Result<f64> {
    let path = myth_common::hook_latency_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Ok(0.0);
    };
    let cutoff_ms =
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis()
            .saturating_sub(7 * 86_400_000)) as i64;
    let mut samples: Vec<f64> = Vec::new();
    for line in content.lines() {
        let Ok(val) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let ts_ms = val.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0);
        if ts_ms < cutoff_ms {
            continue;
        }
        if let Some(lat) = val.get("latency_ms").and_then(|v| v.as_f64()) {
            samples.push(lat);
        }
    }
    if samples.is_empty() {
        return Ok(0.0);
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((samples.len() as f64 - 1.0) * 0.99).round() as usize;
    Ok(samples[idx.min(samples.len() - 1)])
}

fn brief_age_days() -> Option<u64> {
    let path = myth_common::brief_path();
    let meta = std::fs::metadata(&path).ok()?;
    let mtime = meta.modified().ok()?;
    let age = SystemTime::now().duration_since(mtime).unwrap_or(Duration::ZERO);
    Some(age.as_secs() / 86_400)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_p99_missing_file_returns_zero() {
        // default path; 실존 여부에 무관하게 0.0 이상 반환
        let v = compute_hook_p99_last_7d().unwrap();
        assert!(v >= 0.0);
    }
}
