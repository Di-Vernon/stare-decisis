//! Active Tasks 패널 — orchestrator 실행 중 task 뷰.
//!
//! Day-1 외부 피딩용 단순 벡터. 실시간 wire-up은 Wave 5 `myth-cli` 또는
//! Wave 8 통합에서.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::theme::Theme;

#[derive(Debug, Clone)]
pub struct TaskRow {
    pub id: String,
    pub status: TaskStatus,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Running,
    Waiting,
    Done,
    Failed,
}

impl TaskStatus {
    pub fn label(self) -> &'static str {
        match self {
            TaskStatus::Running => "running",
            TaskStatus::Waiting => "waiting",
            TaskStatus::Done => "done",
            TaskStatus::Failed => "failed",
        }
    }
}

pub struct TasksPanel {
    rows: Vec<TaskRow>,
}

impl TasksPanel {
    pub fn new() -> Self {
        Self { rows: Vec::new() }
    }

    pub fn set_rows(&mut self, rows: Vec<TaskRow>) {
        self.rows = rows;
    }

    pub fn rows(&self) -> &[TaskRow] {
        &self.rows
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool, theme: &Theme) {
        let block = Block::default()
            .title(" Active Tasks ")
            .borders(Borders::ALL)
            .border_style(theme.border_style(focused));

        let items: Vec<ListItem> = self
            .rows
            .iter()
            .map(|r| {
                let color = match r.status {
                    TaskStatus::Running => theme.accent,
                    TaskStatus::Waiting => theme.dim,
                    TaskStatus::Done => theme.success,
                    TaskStatus::Failed => theme.error,
                };
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("[{}] ", r.id),
                        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{:<8}", r.status.label()),
                        Style::default().fg(color),
                    ),
                    Span::styled(r.description.clone(), Style::default().fg(theme.fg)),
                ]))
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}

impl Default for TasksPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_labels() {
        assert_eq!(TaskStatus::Running.label(), "running");
        assert_eq!(TaskStatus::Done.label(), "done");
        assert_eq!(TaskStatus::Failed.label(), "failed");
    }

    #[test]
    fn set_rows_replaces() {
        let mut p = TasksPanel::new();
        p.set_rows(vec![TaskRow {
            id: "T1".into(),
            status: TaskStatus::Running,
            description: "d".into(),
        }]);
        assert_eq!(p.rows().len(), 1);
    }
}
