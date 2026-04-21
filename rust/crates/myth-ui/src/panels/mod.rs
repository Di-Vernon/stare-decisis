//! 대시보드 패널.
//!
//! 각 패널은 자체 상태 + render(frame, area, focused, theme) 메서드를 제공.
//! 포커스·스크롤 처리는 App이 관장.

pub mod brief;
pub mod caselog;
pub mod lessons;
pub mod migration;
pub mod tasks;

pub use brief::BriefPanel;
pub use caselog::{CaselogEntry, CaselogPanel};
pub use lessons::{LessonRow, LessonsPanel};
pub use migration::{MigrationPanel, MilestoneStatus};
pub use tasks::{TaskRow, TasksPanel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelId {
    Caselog,
    Tasks,
    Lessons,
    Brief,
    Migration,
}
