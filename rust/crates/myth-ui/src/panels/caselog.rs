//! Caselog 패널 — JSONL tail 기반 최근 N 엔트리.

use std::collections::VecDeque;

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;
use serde::Deserialize;

use crate::theme::Theme;

#[derive(Debug, Clone, Deserialize)]
pub struct CaselogEntry {
    #[serde(default)]
    pub ts: String,
    #[serde(default)]
    pub tool: String,
    #[serde(default)]
    pub level: u8,
    #[serde(default)]
    pub summary: String,
}

pub struct CaselogPanel {
    entries: VecDeque<CaselogEntry>,
    scroll_offset: usize,
    max_entries: usize,
}

impl CaselogPanel {
    pub fn new() -> Self {
        let mut p = Self {
            entries: VecDeque::new(),
            scroll_offset: 0,
            max_entries: 100,
        };
        p.refresh();
        p
    }

    /// 베스트에포트 로드. 파일 없거나 파싱 실패 시 조용히 빈 상태 유지.
    pub fn refresh(&mut self) {
        let path = myth_common::caselog_path();
        self.entries.clear();
        let Ok(content) = std::fs::read_to_string(&path) else {
            return;
        };
        for line in content.lines().rev().take(self.max_entries) {
            if let Ok(entry) = serde_json::from_str::<CaselogEntry>(line) {
                self.entries.push_front(entry);
            }
        }
    }

    pub fn entries(&self) -> &VecDeque<CaselogEntry> {
        &self.entries
    }

    pub fn scroll_down(&mut self) {
        if self.scroll_offset + 1 < self.entries.len() {
            self.scroll_offset += 1;
        }
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool, theme: &Theme) {
        let block = Block::default()
            .title(" Caselog ")
            .borders(Borders::ALL)
            .border_style(theme.border_style(focused));

        let visible = area.height.saturating_sub(2) as usize;
        let items: Vec<ListItem> = self
            .entries
            .iter()
            .rev()
            .skip(self.scroll_offset)
            .take(visible)
            .map(|e| render_entry(e, theme))
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}

impl Default for CaselogPanel {
    fn default() -> Self {
        Self::new()
    }
}

fn render_entry(entry: &CaselogEntry, theme: &Theme) -> ListItem<'static> {
    let ts = entry.ts.chars().skip(11).take(5).collect::<String>();
    let level = entry.level;
    let color = theme.level_color(level);
    let summary = truncate(&entry.summary, 60);

    ListItem::new(Line::from(vec![
        Span::styled(ts + " ", Style::default().fg(theme.dim)),
        Span::styled(
            format!("L{level} "),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(entry.tool.clone(), Style::default().fg(theme.accent)),
        Span::styled(format!(" {summary}"), Style::default().fg(theme.fg)),
    ]))
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let prefix: String = s.chars().take(n.saturating_sub(1)).collect();
        format!("{prefix}…")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_short() {
        assert_eq!(truncate("hi", 10), "hi");
    }

    #[test]
    fn truncate_long() {
        let out = truncate("abcdefghij", 5);
        assert_eq!(out.chars().count(), 5);
        assert!(out.ends_with('…'));
    }

    #[test]
    fn parse_entry() {
        let json = r#"{"ts":"2026-04-21T13:45:00Z","tool":"Bash","level":3,"summary":"cmd failed"}"#;
        let e: CaselogEntry = serde_json::from_str(json).unwrap();
        assert_eq!(e.tool, "Bash");
        assert_eq!(e.level, 3);
    }

    #[test]
    fn scroll_clamps() {
        let mut p = CaselogPanel::new();
        p.entries.push_back(CaselogEntry {
            ts: String::new(),
            tool: String::new(),
            level: 0,
            summary: String::new(),
        });
        p.entries.push_back(CaselogEntry {
            ts: String::new(),
            tool: String::new(),
            level: 0,
            summary: String::new(),
        });
        p.scroll_down();
        assert_eq!(p.scroll_offset, 1);
        p.scroll_down();
        // clamped to len-1
        assert_eq!(p.scroll_offset, 1);
        p.scroll_up();
        p.scroll_up();
        assert_eq!(p.scroll_offset, 0);
    }
}
