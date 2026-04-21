//! 세션 생명주기 — 현재 시작/종료 시각과 세션 ID만 보유.
//!
//! `myth-db`의 세션 테이블 기록은 myth-cli (Wave 5) 또는 myth-orchestrator
//! (Wave 4.2)가 책임진다. 이 crate는 Connection을 열지 않는다.

use myth_common::SessionId;
use std::time::Instant;

#[derive(Debug)]
pub struct Session {
    pub id: SessionId,
    pub started_at: Instant,
    ended_at: Option<Instant>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            id: SessionId::new(),
            started_at: Instant::now(),
            ended_at: None,
        }
    }

    pub fn from_id(id: SessionId) -> Self {
        Self {
            id,
            started_at: Instant::now(),
            ended_at: None,
        }
    }

    pub fn mark_ended(&mut self) {
        if self.ended_at.is_none() {
            self.ended_at = Some(Instant::now());
        }
    }

    pub fn is_active(&self) -> bool {
        self.ended_at.is_none()
    }

    pub fn elapsed(&self) -> std::time::Duration {
        match self.ended_at {
            Some(end) => end.duration_since(self.started_at),
            None => self.started_at.elapsed(),
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_session_active() {
        let s = Session::new();
        assert!(s.is_active());
    }

    #[test]
    fn mark_ended_is_idempotent() {
        let mut s = Session::new();
        s.mark_ended();
        let first_end = s.ended_at;
        std::thread::sleep(std::time::Duration::from_millis(5));
        s.mark_ended();
        assert_eq!(s.ended_at, first_end);
        assert!(!s.is_active());
    }

    #[test]
    fn from_id_preserves() {
        let id = SessionId::new();
        let s = Session::from_id(id);
        assert_eq!(s.id, id);
    }
}
