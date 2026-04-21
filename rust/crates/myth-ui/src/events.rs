//! 이벤트 수집 — tick + keypress. 파일 watch는 Wave 8 확장.
//!
//! WSL2에서 inotify가 Windows 파일시스템 쪽에서 흔들리는 사례가 있어
//! Day-1은 tick 기반 주기 refresh로 시작한다 (200ms tick).

use std::time::Duration;

use crossterm::event::{self, KeyEvent};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

#[derive(Debug, Clone)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Quit,
}

pub struct EventStream {
    rx: mpsc::Receiver<Event>,
    _tick_handle: JoinHandle<()>,
    _key_handle: JoinHandle<()>,
}

impl EventStream {
    pub fn spawn(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel(128);

        let tx_tick = tx.clone();
        let tick_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tick_rate);
            loop {
                interval.tick().await;
                if tx_tick.send(Event::Tick).await.is_err() {
                    break;
                }
            }
        });

        let tx_key = tx.clone();
        let key_handle_blocking = tokio::task::spawn_blocking(move || loop {
            if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                if let Ok(event::Event::Key(k)) = event::read() {
                    if tx_key.blocking_send(Event::Key(k)).is_err() {
                        break;
                    }
                }
            }
        });
        // Wrap blocking handle in async JoinHandle for uniform field type.
        let key_handle = tokio::spawn(async move {
            let _ = key_handle_blocking.await;
        });

        Self {
            rx,
            _tick_handle: tick_handle,
            _key_handle: key_handle,
        }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn event_enum_construct() {
        // Just ensure variants are constructable.
        let _ = Event::Tick;
        let _ = Event::Quit;
    }
}
