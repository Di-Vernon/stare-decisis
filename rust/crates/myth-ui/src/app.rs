//! App 상태 머신 — 패널 인스턴스 + 포커스 + 종료 플래그.

use crossterm::event::{KeyCode, KeyEvent};

use crate::events::Event;
use crate::panels::{
    BriefPanel, CaselogPanel, LessonsPanel, MigrationPanel, PanelId, TasksPanel,
};
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Help,
}

pub struct App {
    pub theme: Theme,
    pub mode: AppMode,
    pub should_quit: bool,
    pub focused: PanelId,
    pub session_short: String,

    pub caselog: CaselogPanel,
    pub tasks: TasksPanel,
    pub lessons: LessonsPanel,
    pub brief: BriefPanel,
    pub migration: MigrationPanel,
}

impl App {
    pub fn new(session_short: impl Into<String>) -> Self {
        Self {
            theme: Theme::default(),
            mode: AppMode::Normal,
            should_quit: false,
            focused: PanelId::Caselog,
            session_short: session_short.into(),
            caselog: CaselogPanel::new(),
            tasks: TasksPanel::new(),
            lessons: LessonsPanel::new(),
            brief: BriefPanel::new(),
            migration: MigrationPanel::new(),
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Tick => self.on_tick(),
            Event::Key(k) => self.on_key(k),
            Event::Quit => self.should_quit = true,
        }
    }

    fn on_tick(&mut self) {
        self.caselog.refresh();
        self.lessons.refresh();
        self.brief.refresh();
        self.migration.refresh();
    }

    fn on_key(&mut self, key: KeyEvent) {
        if self.mode == AppMode::Help {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?')) {
                self.mode = AppMode::Normal;
            }
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('?') | KeyCode::Char('h') => self.mode = AppMode::Help,
            KeyCode::Tab => self.cycle_focus_forward(),
            KeyCode::BackTab => self.cycle_focus_backward(),
            KeyCode::Char('r') => self.on_tick(),
            KeyCode::Char('j') | KeyCode::Down => self.scroll_focused_down(),
            KeyCode::Char('k') | KeyCode::Up => self.scroll_focused_up(),
            _ => {}
        }
    }

    pub fn cycle_focus_forward(&mut self) {
        self.focused = match self.focused {
            PanelId::Caselog => PanelId::Tasks,
            PanelId::Tasks => PanelId::Lessons,
            PanelId::Lessons => PanelId::Brief,
            PanelId::Brief => PanelId::Migration,
            PanelId::Migration => PanelId::Caselog,
        };
    }

    pub fn cycle_focus_backward(&mut self) {
        self.focused = match self.focused {
            PanelId::Caselog => PanelId::Migration,
            PanelId::Tasks => PanelId::Caselog,
            PanelId::Lessons => PanelId::Tasks,
            PanelId::Brief => PanelId::Lessons,
            PanelId::Migration => PanelId::Brief,
        };
    }

    fn scroll_focused_down(&mut self) {
        match self.focused {
            PanelId::Caselog => self.caselog.scroll_down(),
            PanelId::Brief => self.brief.scroll_down(),
            _ => {}
        }
    }

    fn scroll_focused_up(&mut self) {
        match self.focused {
            PanelId::Caselog => self.caselog.scroll_up(),
            PanelId::Brief => self.brief.scroll_up(),
            _ => {}
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new("new")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn q_quits() {
        let mut app = App::default();
        app.on_key(key(KeyCode::Char('q')));
        assert!(app.should_quit);
    }

    #[test]
    fn esc_quits_normal() {
        let mut app = App::default();
        app.on_key(key(KeyCode::Esc));
        assert!(app.should_quit);
    }

    #[test]
    fn question_toggles_help() {
        let mut app = App::default();
        app.on_key(key(KeyCode::Char('?')));
        assert_eq!(app.mode, AppMode::Help);
        app.on_key(key(KeyCode::Char('q'))); // in help, q exits help, not quit
        assert_eq!(app.mode, AppMode::Normal);
        assert!(!app.should_quit);
    }

    #[test]
    fn tab_cycles_focus() {
        let mut app = App::default();
        assert_eq!(app.focused, PanelId::Caselog);
        app.on_key(key(KeyCode::Tab));
        assert_eq!(app.focused, PanelId::Tasks);
        app.on_key(key(KeyCode::Tab));
        assert_eq!(app.focused, PanelId::Lessons);
        for _ in 0..3 {
            app.on_key(key(KeyCode::Tab));
        }
        assert_eq!(app.focused, PanelId::Caselog);
    }

    #[test]
    fn backtab_reverses() {
        let mut app = App::default();
        app.on_key(key(KeyCode::BackTab));
        assert_eq!(app.focused, PanelId::Migration);
    }
}
