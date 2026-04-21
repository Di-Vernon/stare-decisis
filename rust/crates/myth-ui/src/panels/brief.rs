//! Brief 패널 — `~/.myth/brief.md` 마크다운 렌더.

use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::markdown::render_markdown;
use crate::theme::Theme;

pub struct BriefPanel {
    lines: Vec<Line<'static>>,
    scroll: u16,
}

impl BriefPanel {
    pub fn new() -> Self {
        let mut p = Self {
            lines: Vec::new(),
            scroll: 0,
        };
        p.refresh();
        p
    }

    pub fn refresh(&mut self) {
        let theme = Theme::default();
        let path = myth_common::brief_path();
        self.lines = match std::fs::read_to_string(&path) {
            Ok(md) => render_markdown(&md, &theme),
            Err(_) => vec![Line::from("(brief.md not found — run `myth brief` to generate)")],
        };
    }

    pub fn lines(&self) -> &[Line<'static>] {
        &self.lines
    }

    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool, theme: &Theme) {
        let block = Block::default()
            .title(" Brief ")
            .borders(Borders::ALL)
            .border_style(theme.border_style(focused));

        let para = Paragraph::new(self.lines.clone())
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));
        frame.render_widget(para, area);
    }
}

impl Default for BriefPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_fallback() {
        let p = BriefPanel::new();
        assert!(!p.lines().is_empty());
    }

    #[test]
    fn scroll_down_up() {
        let mut p = BriefPanel::new();
        p.scroll_down();
        assert_eq!(p.scroll, 1);
        p.scroll_up();
        assert_eq!(p.scroll, 0);
        p.scroll_up();
        assert_eq!(p.scroll, 0);
    }
}
