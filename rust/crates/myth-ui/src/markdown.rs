//! pulldown-cmark 0.10 → ratatui `Line` 변환.
//!
//! Day-1 범위: 헤딩/단락/줄바꿈/인라인 코드. 목록/인용/테이블은 텍스트만
//! 추출해서 기본 스타일로 표시 (Wave 8에서 확장).

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::theme::Theme;

pub fn render_markdown(markdown: &str, theme: &Theme) -> Vec<Line<'static>> {
    let parser = Parser::new(markdown);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current: Vec<Span<'static>> = Vec::new();

    let mut heading_level: Option<u8> = None;
    let mut in_code = false;
    let mut emph = false;
    let mut strong = false;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    heading_level = Some(heading_level_to_u8(level));
                }
                Tag::Emphasis => emph = true,
                Tag::Strong => strong = true,
                Tag::CodeBlock(_) => in_code = true,
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_)
                | TagEnd::Paragraph
                | TagEnd::Item
                | TagEnd::BlockQuote => {
                    if !current.is_empty() {
                        lines.push(Line::from(std::mem::take(&mut current)));
                    }
                    heading_level = None;
                }
                TagEnd::CodeBlock => {
                    if !current.is_empty() {
                        lines.push(Line::from(std::mem::take(&mut current)));
                    }
                    in_code = false;
                }
                TagEnd::Emphasis => emph = false,
                TagEnd::Strong => strong = false,
                _ => {}
            },
            Event::Text(text) => {
                let style = text_style(theme, heading_level, in_code, emph, strong);
                current.push(Span::styled(text.to_string(), style));
            }
            Event::Code(text) => {
                let style = Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::ITALIC);
                current.push(Span::styled(format!("`{text}`"), style));
            }
            Event::SoftBreak => {
                current.push(Span::raw(" "));
            }
            Event::HardBreak => {
                lines.push(Line::from(std::mem::take(&mut current)));
            }
            Event::Rule => {
                if !current.is_empty() {
                    lines.push(Line::from(std::mem::take(&mut current)));
                }
                lines.push(Line::from(Span::styled(
                    "────────────".to_string(),
                    Style::default().fg(theme.dim),
                )));
            }
            _ => {}
        }
    }

    if !current.is_empty() {
        lines.push(Line::from(current));
    }
    lines
}

fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn text_style(
    theme: &Theme,
    heading_level: Option<u8>,
    in_code: bool,
    emph: bool,
    strong: bool,
) -> Style {
    if let Some(level) = heading_level {
        return theme.heading_style(level);
    }
    let mut style = Style::default().fg(theme.fg);
    if in_code {
        style = style.fg(theme.accent);
    }
    if strong {
        style = style.add_modifier(Modifier::BOLD);
    }
    if emph {
        style = style.add_modifier(Modifier::ITALIC);
    }
    style
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_heading_bold() {
        let theme = Theme::default();
        let lines = render_markdown("# Title\n\nParagraph text.", &theme);
        assert!(!lines.is_empty());
        // First line should contain "Title" span styled as heading
        let first = &lines[0];
        let found = first
            .spans
            .iter()
            .any(|s| s.content.contains("Title") && s.style.add_modifier.contains(Modifier::BOLD));
        assert!(found, "heading not bold: {first:?}");
    }

    #[test]
    fn renders_paragraph_plain() {
        let theme = Theme::default();
        let lines = render_markdown("Just a paragraph.", &theme);
        let text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect();
        assert!(text.contains("Just a paragraph"));
    }

    #[test]
    fn renders_inline_code_backticked() {
        let theme = Theme::default();
        let lines = render_markdown("Use `cargo test`.", &theme);
        let text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect();
        assert!(text.contains("`cargo test`"));
    }

    #[test]
    fn empty_input_empty_output() {
        let theme = Theme::default();
        let lines = render_markdown("", &theme);
        assert!(lines.is_empty());
    }
}
