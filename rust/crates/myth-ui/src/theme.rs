//! Tokyo Night 기반 색상 팔레트. 패널 별 fg/bg + 강조/경고색.

use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub fg: Color,
    pub bg: Color,
    pub dim: Color,
    pub accent: Color,
    pub focused_border: Color,
    pub unfocused_border: Color,
    pub success: Color,
    pub warn: Color,
    pub error: Color,
    pub heading_h1: Color,
    pub heading_h2: Color,
    pub heading_h3: Color,
}

impl Theme {
    pub fn tokyo_night() -> Self {
        Self {
            fg: Color::Rgb(0xc0, 0xca, 0xf5),
            bg: Color::Reset,
            dim: Color::Rgb(0x56, 0x5f, 0x89),
            accent: Color::Rgb(0x7a, 0xa2, 0xf7),
            focused_border: Color::Rgb(0xbb, 0x9a, 0xf7),
            unfocused_border: Color::Rgb(0x41, 0x48, 0x68),
            success: Color::Rgb(0x9e, 0xce, 0x6a),
            warn: Color::Rgb(0xe0, 0xaf, 0x68),
            error: Color::Rgb(0xf7, 0x76, 0x8e),
            heading_h1: Color::Rgb(0x7d, 0xcf, 0xff),
            heading_h2: Color::Rgb(0xe0, 0xaf, 0x68),
            heading_h3: Color::Rgb(0xbb, 0x9a, 0xf7),
        }
    }

    pub fn border_style(&self, focused: bool) -> Style {
        Style::default().fg(if focused {
            self.focused_border
        } else {
            self.unfocused_border
        })
    }

    pub fn heading_style(&self, level: u8) -> Style {
        let color = match level {
            1 => self.heading_h1,
            2 => self.heading_h2,
            _ => self.heading_h3,
        };
        Style::default().fg(color).add_modifier(Modifier::BOLD)
    }

    /// Category level → color. L4/L5 = critical = error; L3 = warn; L1/L2 = accent/dim.
    pub fn level_color(&self, level: u8) -> Color {
        match level {
            5 => self.error,
            4 => self.error,
            3 => self.warn,
            2 => self.accent,
            _ => self.dim,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::tokyo_night()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_tokyo_night() {
        let t = Theme::default();
        assert_eq!(t.fg, Color::Rgb(0xc0, 0xca, 0xf5));
    }

    #[test]
    fn border_style_focus_diff() {
        let t = Theme::default();
        let f = t.border_style(true);
        let u = t.border_style(false);
        assert_ne!(f.fg, u.fg);
    }

    #[test]
    fn heading_style_colors() {
        let t = Theme::default();
        assert_eq!(t.heading_style(1).fg.unwrap(), t.heading_h1);
        assert_eq!(t.heading_style(2).fg.unwrap(), t.heading_h2);
        assert_eq!(t.heading_style(5).fg.unwrap(), t.heading_h3);
    }

    #[test]
    fn level_color_mapping() {
        let t = Theme::default();
        assert_eq!(t.level_color(5), t.error);
        assert_eq!(t.level_color(3), t.warn);
        assert_eq!(t.level_color(1), t.dim);
    }
}
