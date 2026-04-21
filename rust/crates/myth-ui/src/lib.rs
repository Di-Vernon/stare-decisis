//! `myth-ui` — TUI dashboard (ratatui).
//!
//! 대시보드 진입점 `run_dashboard`는 `myth-cli` (Wave 5)에서 래핑되어
//! `myth status` / `myth watch` 로 노출된다. state.db는 열지 않으며,
//! 모든 관찰 데이터는 JSONL/markdown 파일 읽기로만 수집한다.

pub mod app;
pub mod events;
pub mod layout;
pub mod markdown;
pub mod panels;
pub mod syntax;
pub mod theme;

pub use app::{App, AppMode};
pub use events::{Event, EventStream};
pub use panels::PanelId;
pub use theme::Theme;

use anyhow::{Context, Result};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::Duration;

/// Entry point: 대시보드 실행. `session_short`는 헤더에 표시할 짧은 세션 ID.
pub async fn run_dashboard(session_short: impl Into<String>) -> Result<()> {
    enable_raw_mode().context("enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("create terminal")?;

    let mut app = App::new(session_short);
    let mut events = EventStream::spawn(Duration::from_millis(200));

    let result: Result<()> = (async {
        while !app.should_quit {
            terminal.draw(|f| layout::render(&app, f))?;
            if let Some(ev) = events.next().await {
                app.handle_event(ev);
            } else {
                break;
            }
        }
        Ok(())
    })
    .await;

    // 정리 — 결과와 무관하게 실행
    let _ = disable_raw_mode();
    let _ = execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    );
    let _ = terminal.show_cursor();

    result
}
