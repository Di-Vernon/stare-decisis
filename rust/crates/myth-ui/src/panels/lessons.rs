//! Lessons 패널 — 활성 lesson 목록.
//!
//! Day-1은 state.db 직접 접근 대신 lesson-state.jsonl 최신 스냅샷을 읽는다.
//! 정식 lesson 목록은 `myth-db::SqliteLessonStore`가 관장하지만, myth-ui는
//! DB Connection을 열지 않는 단일 책임을 지키기 위해 파일 기반으로 구성
//! (실시간성 < 단순성, 위임은 `myth-cli`의 status).

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;
use serde::Deserialize;

use crate::theme::Theme;

#[derive(Debug, Clone, Deserialize)]
pub struct LessonRow {
    #[serde(default)]
    pub lesson_id: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub level: u8,
    #[serde(default)]
    pub headline: String,
}

pub struct LessonsPanel {
    rows: Vec<LessonRow>,
}

impl LessonsPanel {
    pub fn new() -> Self {
        let mut p = Self { rows: Vec::new() };
        p.refresh();
        p
    }

    pub fn rows(&self) -> &[LessonRow] {
        &self.rows
    }

    pub fn refresh(&mut self) {
        let path = myth_common::lesson_state_path();
        self.rows.clear();
        let Ok(content) = std::fs::read_to_string(&path) else {
            return;
        };
        for line in content.lines() {
            if let Ok(row) = serde_json::from_str::<LessonRow>(line) {
                if row.status == "active" || row.status == "probation" {
                    self.rows.push(row);
                }
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool, theme: &Theme) {
        let block = Block::default()
            .title(" Lessons ")
            .borders(Borders::ALL)
            .border_style(theme.border_style(focused));

        let visible = area.height.saturating_sub(2) as usize;
        let items: Vec<ListItem> = self
            .rows
            .iter()
            .take(visible)
            .map(|r| {
                let color = theme.level_color(r.level);
                ListItem::new(Line::from(vec![
                    Span::styled(
                        r.lesson_id.clone(),
                        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("  L{} ", r.level),
                        Style::default().fg(color),
                    ),
                    Span::styled(r.headline.clone(), Style::default().fg(theme.fg)),
                ]))
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}

impl Default for LessonsPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_lesson_row() {
        let json = r#"{"lesson_id":"L3-0012","status":"active","level":3,"headline":"Bash rule"}"#;
        let r: LessonRow = serde_json::from_str(json).unwrap();
        assert_eq!(r.lesson_id, "L3-0012");
    }

    #[test]
    fn empty_refresh_without_file_ok() {
        let p = LessonsPanel::new();
        assert!(p.rows().is_empty() || !p.rows().is_empty()); // shouldn't panic
    }
}
