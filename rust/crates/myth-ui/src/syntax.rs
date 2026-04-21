//! syntect 5 기반 구문 강조. Day-1은 ANSI 대신 `ratatui::Line`로 직접 변환.

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style as SynStyle, ThemeSet};
use syntect::parsing::SyntaxSet;

pub fn highlight_code(code: &str, lang_token: &str) -> Vec<Line<'static>> {
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ss
        .find_syntax_by_token(lang_token)
        .unwrap_or_else(|| ss.find_syntax_plain_text());
    let theme = &ts.themes["base16-ocean.dark"];
    let mut highlighter = HighlightLines::new(syntax, theme);

    let mut out = Vec::new();
    for line in syntect::util::LinesWithEndings::from(code) {
        let ranges = match highlighter.highlight_line(line, &ss) {
            Ok(r) => r,
            Err(_) => break,
        };
        let spans: Vec<Span<'static>> = ranges
            .into_iter()
            .map(|(style, text)| {
                Span::styled(
                    text.trim_end_matches('\n').to_string(),
                    convert_style(style),
                )
            })
            .collect();
        out.push(Line::from(spans));
    }
    out
}

fn convert_style(style: SynStyle) -> Style {
    Style::default().fg(Color::Rgb(
        style.foreground.r,
        style.foreground.g,
        style.foreground.b,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_rust() {
        let code = "fn main() {\n    println!(\"hi\");\n}";
        let lines = highlight_code(code, "rs");
        assert_eq!(lines.len(), 3);
        let first_text: String = lines[0].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(first_text.contains("fn"));
        assert!(first_text.contains("main"));
    }

    #[test]
    fn plain_text_fallback() {
        let lines = highlight_code("hello world", "totally-unknown-lang");
        assert_eq!(lines.len(), 1);
    }
}
