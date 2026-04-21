//! `myth lesson` — list/show/appeal/retrial + split/merge stub.
//!
//! 드리프트 4 대응 (옵션 P):
//!   - list: list_active + list_lapsed 조합 + client-side filter
//!   - show/appeal/retrial: 실 API (file_appeal 등)
//!   - split/merge: Milestone 지연 stub

use anyhow::{anyhow, Result};
use myth_common::Level;
use myth_db::appeal::{file_appeal, AppealType};
use myth_db::{Database, Lesson, LessonStore, SqliteLessonStore};
use std::process::ExitCode;

use crate::args::{LessonAction, LessonArgs, OutputFormat};

pub async fn run(args: LessonArgs, _format: OutputFormat) -> Result<ExitCode> {
    match args.action {
        LessonAction::List { level, status, limit } => list(level, status, limit).await,
        LessonAction::Show { id } => show(&id).await,
        LessonAction::Appeal { id, reason } => appeal(&id, &reason).await,
        LessonAction::Retrial { id, reason } => retrial(&id, &reason).await,
        LessonAction::Split { .. } | LessonAction::Merge { .. } => split_merge_stub(),
    }
}

fn split_merge_stub() -> Result<ExitCode> {
    eprintln!(
        "split/merge: not yet implemented, planned for Wave 8 integration \
         (requires LessonStore DB-level support)"
    );
    Ok(ExitCode::SUCCESS)
}

async fn list(
    level_filter: Option<u8>,
    status_filter: Option<String>,
    limit: usize,
) -> Result<ExitCode> {
    let db = Database::open(&myth_common::state_db_path())?;
    let store = SqliteLessonStore::new(db);

    let lessons: Vec<Lesson> = match status_filter.as_deref() {
        Some("archived") => {
            eprintln!(
                "warning: --status archived not yet supported (count-only path pending Wave 8)"
            );
            return Ok(ExitCode::SUCCESS);
        }
        Some("lapsed") => store.list_lapsed()?,
        Some("active") | None => store.list_active()?,
        Some(other) => {
            return Err(anyhow!("unknown status {other}. Use: active|lapsed|archived"));
        }
    };

    let filtered: Vec<&Lesson> = lessons
        .iter()
        .filter(|l| match level_filter {
            Some(lv) => l.level as u8 == lv,
            None => true,
        })
        .take(limit)
        .collect();

    if filtered.is_empty() {
        println!("(no lessons match)");
        return Ok(ExitCode::SUCCESS);
    }

    for l in filtered {
        println!(
            "{:8}  L{}  {:<10}  {}  (rec {:.1}, appeals {})",
            l.id.short(),
            l.level as u8,
            level_label(l.level),
            truncate(&l.rationale, 40),
            l.recurrence_count,
            l.appeals
        );
    }
    Ok(ExitCode::SUCCESS)
}

async fn show(id_prefix: &str) -> Result<ExitCode> {
    let db = Database::open(&myth_common::state_db_path())?;
    let store = SqliteLessonStore::new(db);
    let lesson = find_by_prefix(&store, id_prefix)?;

    println!("Lesson {}", lesson.id);
    println!("  Level:       {} ({})", lesson.level as u8, level_label(lesson.level));
    println!("  Category:    {:?}", lesson.category);
    println!("  Status:      {}", lesson.status.as_str());
    println!("  Recurrence:  {:.1}", lesson.recurrence_count);
    println!("  Appeals:     {}", lesson.appeals);
    println!("  First seen:  {}", lesson.first_seen);
    println!("  Last seen:   {}", lesson.last_seen);
    println!("  Description: {}", lesson.description);
    println!("  Rationale:   {}", lesson.rationale);
    Ok(ExitCode::SUCCESS)
}

async fn appeal(id_prefix: &str, reason: &str) -> Result<ExitCode> {
    let db = Database::open(&myth_common::state_db_path())?;
    let store = SqliteLessonStore::new(db);
    let lesson = find_by_prefix(&store, id_prefix)?;

    let max = match lesson.level {
        Level::Critical => 5,
        Level::High => 3,
        Level::Medium => 2,
        Level::Low | Level::Info => 1,
    };
    if lesson.appeals >= max {
        return Err(anyhow!(
            "appeal limit exceeded for level {:?} ({}/{})",
            lesson.level,
            lesson.appeals,
            max
        ));
    }

    file_appeal(store.db(), lesson.id, AppealType::Appeal, Some(reason))?;

    let mut updated = lesson.clone();
    updated.appeals += 1;
    store.update(&updated)?;
    println!("appeal recorded for lesson {}", lesson.id.short());
    Ok(ExitCode::SUCCESS)
}

async fn retrial(id_prefix: &str, reason: &str) -> Result<ExitCode> {
    let db = Database::open(&myth_common::state_db_path())?;
    let store = SqliteLessonStore::new(db);
    let lesson = find_by_prefix(&store, id_prefix)?;

    if !matches!(lesson.level, Level::High | Level::Critical) {
        return Err(anyhow!(
            "retrial only allowed for Level 4-5 (this lesson is level {})",
            lesson.level as u8
        ));
    }
    file_appeal(store.db(), lesson.id, AppealType::Retrial, Some(reason))?;
    println!("retrial filed for lesson {}", lesson.id.short());
    Ok(ExitCode::SUCCESS)
}

fn find_by_prefix(store: &SqliteLessonStore, prefix: &str) -> Result<Lesson> {
    // 전체 active + lapsed 중에서 id.short() prefix 매치
    let mut candidates: Vec<Lesson> = store.list_active()?;
    candidates.extend(store.list_lapsed()?);

    let matches: Vec<Lesson> = candidates
        .into_iter()
        .filter(|l| l.id.short().starts_with(prefix) || l.id.to_string().starts_with(prefix))
        .collect();

    match matches.len() {
        0 => Err(anyhow!("no lesson matches id prefix {prefix}")),
        1 => Ok(matches.into_iter().next().unwrap()),
        n => Err(anyhow!(
            "ambiguous id prefix {prefix} — {n} matches, please provide more characters"
        )),
    }
}

fn level_label(l: Level) -> &'static str {
    match l {
        Level::Info => "INFO",
        Level::Low => "LOW",
        Level::Medium => "MEDIUM",
        Level::High => "HIGH",
        Level::Critical => "CRITICAL",
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n.saturating_sub(1)).collect::<String>() + "…"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_labels() {
        assert_eq!(level_label(Level::High), "HIGH");
        assert_eq!(level_label(Level::Critical), "CRITICAL");
    }

    #[test]
    fn truncate_long() {
        let out = truncate("abcdefghij", 5);
        assert_eq!(out.chars().count(), 5);
    }

    #[tokio::test]
    async fn split_merge_stub_ok() {
        split_merge_stub().unwrap();
    }
}
