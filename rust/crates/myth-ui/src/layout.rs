//! 레이아웃 계산 + 한 프레임 렌더링.
//!
//! 레이아웃:
//! ```text
//! ┌─ header ──────────────────────────────┐
//! ├─ left (33%) ─┬─ right (67%) ──────────┤
//! │ Caselog      │ Brief / Migration      │
//! │ Tasks        │                        │
//! │ Lessons      │                        │
//! └──────────────┴────────────────────────┘
//! ```

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use crate::panels::PanelId;
use crate::theme::Theme;

pub struct Areas {
    pub header: Rect,
    pub caselog: Rect,
    pub tasks: Rect,
    pub lessons: Rect,
    pub right_top: Rect,
    pub right_bottom: Rect,
}

pub fn compute_areas(frame_area: Rect) -> Areas {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(frame_area);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(67)])
        .split(outer[1]);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ])
        .split(body[0]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(body[1]);

    Areas {
        header: outer[0],
        caselog: left[0],
        tasks: left[1],
        lessons: left[2],
        right_top: right[0],
        right_bottom: right[1],
    }
}

pub fn render(app: &App, frame: &mut Frame) {
    let theme = &app.theme;
    let areas = compute_areas(frame.size());

    render_header(app, frame, areas.header, theme);

    app.caselog
        .render(frame, areas.caselog, app.focused == PanelId::Caselog, theme);
    app.tasks
        .render(frame, areas.tasks, app.focused == PanelId::Tasks, theme);
    app.lessons
        .render(frame, areas.lessons, app.focused == PanelId::Lessons, theme);
    app.brief
        .render(frame, areas.right_top, app.focused == PanelId::Brief, theme);
    app.migration.render(
        frame,
        areas.right_bottom,
        app.focused == PanelId::Migration,
        theme,
    );
}

fn render_header(app: &App, frame: &mut Frame, area: Rect, theme: &Theme) {
    let active_tasks = app
        .tasks
        .rows()
        .iter()
        .filter(|r| r.status.label() == "running")
        .count();

    let line = Line::from(vec![
        Span::styled(
            " myth ",
            Style::default()
                .fg(theme.heading_h1)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("│ Session: {} ", app.session_short),
            Style::default().fg(theme.fg),
        ),
        Span::styled(
            format!("│ Active: {active_tasks} "),
            Style::default().fg(theme.accent),
        ),
        Span::styled(
            "│ [Tab] panel  [q] quit  [?] help",
            Style::default().fg(theme.dim),
        ),
    ]);

    let para = Paragraph::new(line);
    frame.render_widget(para, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_areas_splits_frame() {
        let frame = Rect::new(0, 0, 120, 40);
        let a = compute_areas(frame);
        assert_eq!(a.header.height, 1);
        assert!(a.caselog.width > 0);
        assert!(a.right_top.width > a.caselog.width);
    }

    #[test]
    fn compute_areas_tight_frame() {
        let frame = Rect::new(0, 0, 40, 10);
        let a = compute_areas(frame);
        assert_eq!(a.header.height, 1);
        assert!(a.caselog.height + a.tasks.height + a.lessons.height <= 9);
    }
}
