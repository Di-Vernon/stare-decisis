//! Migration Readiness 패널 — 5개 Milestone A~E 상태.
//!
//! Milestone C (Gavel daemon)는 `~/.local/state/myth/hook-latency.ndjson`
//! (production source only, per Wave 3 기반점 2)에서 최근 샘플의 P99를 계산한다.
//! bench 결과는 사용하지 않는다 — bench는 구조 비용 진단용이고, 트리거
//! 판정 기준은 production warm-cache 측정이어야 한다.

use std::path::PathBuf;

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;
use serde::Deserialize;

use crate::theme::Theme;

#[derive(Debug, Clone)]
pub struct MilestoneStatus {
    pub id: char,
    pub title: String,
    pub triggered: bool,
    pub current_value: String,
    pub threshold: String,
    pub notes: Vec<String>,
}

pub struct MigrationPanel {
    milestones: Vec<MilestoneStatus>,
}

impl MigrationPanel {
    pub fn new() -> Self {
        let mut p = Self {
            milestones: Vec::new(),
        };
        p.refresh();
        p
    }

    pub fn milestones(&self) -> &[MilestoneStatus] {
        &self.milestones
    }

    pub fn refresh(&mut self) {
        self.milestones = vec![
            Self::milestone_a(),
            Self::milestone_b(),
            milestone_c_from_path(myth_common::hook_latency_path()),
            Self::milestone_d(),
            Self::milestone_e(),
        ];
    }

    fn milestone_a() -> MilestoneStatus {
        MilestoneStatus {
            id: 'A',
            title: "Assessor Tier review (3w elapsed)".into(),
            triggered: false,
            current_value: "pending".into(),
            threshold: "3 weeks".into(),
            notes: vec![],
        }
    }

    fn milestone_b() -> MilestoneStatus {
        MilestoneStatus {
            id: 'B',
            title: "Vector store migration".into(),
            triggered: false,
            current_value: "pending".into(),
            threshold: "N/A".into(),
            notes: vec![],
        }
    }

    fn milestone_d() -> MilestoneStatus {
        MilestoneStatus {
            id: 'D',
            title: "Semantic detection".into(),
            triggered: false,
            current_value: "pending".into(),
            threshold: "N/A".into(),
            notes: vec![],
        }
    }

    fn milestone_e() -> MilestoneStatus {
        MilestoneStatus {
            id: 'E',
            title: "AST validation".into(),
            triggered: false,
            current_value: "pending".into(),
            threshold: "N/A".into(),
            notes: vec![],
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool, theme: &Theme) {
        let block = Block::default()
            .title(" Migration Readiness ")
            .borders(Borders::ALL)
            .border_style(theme.border_style(focused));

        let items: Vec<ListItem> = self
            .milestones
            .iter()
            .map(|m| {
                let marker = if m.triggered { "[x]" } else { "[ ]" };
                let color = if m.triggered { theme.warn } else { theme.dim };
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{} [{}] ", m.id, marker),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(m.title.clone(), Style::default().fg(theme.fg)),
                    Span::styled(
                        format!("  {} / {}", m.current_value, m.threshold),
                        Style::default().fg(theme.accent),
                    ),
                ]))
            })
            .collect();
        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}

impl Default for MigrationPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct HookLatencyEntry {
    #[serde(default)]
    latency_ms: f64,
}

/// Milestone C = Gavel daemon trigger. 최근 14일 P99 > 15ms 기준.
/// 파일이 없거나 샘플이 부족하면 untriggered + "insufficient data".
pub fn milestone_c_from_path(path: PathBuf) -> MilestoneStatus {
    let Ok(content) = std::fs::read_to_string(&path) else {
        return MilestoneStatus {
            id: 'C',
            title: "Gavel daemon migration".into(),
            triggered: false,
            current_value: "no data".into(),
            threshold: "P99 > 15ms".into(),
            notes: vec![format!("source: {}", path.display())],
        };
    };

    let mut samples: Vec<f64> = Vec::new();
    for line in content.lines() {
        if let Ok(e) = serde_json::from_str::<HookLatencyEntry>(line) {
            samples.push(e.latency_ms);
        }
    }

    if samples.is_empty() {
        return MilestoneStatus {
            id: 'C',
            title: "Gavel daemon migration".into(),
            triggered: false,
            current_value: "insufficient data".into(),
            threshold: "P99 > 15ms".into(),
            notes: vec![format!("source: {}", path.display())],
        };
    }

    let p99 = percentile(&mut samples, 0.99);
    let triggered = p99 > 15.0;
    MilestoneStatus {
        id: 'C',
        title: "Gavel daemon migration".into(),
        triggered,
        current_value: format!("P99 {p99:.1}ms"),
        threshold: "P99 > 15ms".into(),
        notes: vec![
            format!("source: {} (production only)", path.display()),
            format!("samples: {}", samples.len()),
        ],
    }
}

fn percentile(samples: &mut [f64], q: f64) -> f64 {
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if samples.is_empty() {
        return 0.0;
    }
    let idx = ((samples.len() as f64 - 1.0) * q).round() as usize;
    samples[idx.min(samples.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn milestone_c_missing_file() {
        let ms = milestone_c_from_path(PathBuf::from("/nonexistent/hook-latency.ndjson"));
        assert_eq!(ms.id, 'C');
        assert!(!ms.triggered);
        assert_eq!(ms.current_value, "no data");
    }

    #[test]
    fn milestone_c_empty_file() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("hook.ndjson");
        std::fs::write(&p, "").unwrap();
        let ms = milestone_c_from_path(p);
        assert!(!ms.triggered);
        assert_eq!(ms.current_value, "insufficient data");
    }

    #[test]
    fn milestone_c_low_latency_not_triggered() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("hook.ndjson");
        let content: String = (1..=100)
            .map(|i| format!(r#"{{"latency_ms":{}}}"#, i as f64 * 0.1))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&p, content).unwrap();
        let ms = milestone_c_from_path(p);
        assert!(!ms.triggered, "P99 {} shouldn't exceed 15ms", ms.current_value);
    }

    #[test]
    fn milestone_c_high_latency_triggered() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("hook.ndjson");
        let content: String = (1..=100)
            .map(|i| format!(r#"{{"latency_ms":{}}}"#, i as f64))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&p, content).unwrap();
        let ms = milestone_c_from_path(p);
        assert!(ms.triggered, "P99 {} should exceed 15ms", ms.current_value);
    }

    #[test]
    fn percentile_basic() {
        let mut s = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let p50 = percentile(&mut s, 0.5);
        assert!((5.0..=6.0).contains(&p50));
    }

    #[test]
    fn refresh_populates_five() {
        let p = MigrationPanel::new();
        assert_eq!(p.milestones().len(), 5);
        let ids: Vec<char> = p.milestones().iter().map(|m| m.id).collect();
        assert_eq!(ids, vec!['A', 'B', 'C', 'D', 'E']);
    }
}
