//! `plan.json` — wave-based 실행 계획.
//!
//! 한 wave 내에서는 병렬, wave 간에는 순차. 같은 wave의 task들은
//! `files_affected`가 중복될 수 없다 (merge conflict 예방).

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub version: u32,
    pub title: String,
    pub waves: Vec<Wave>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wave {
    pub id: String,
    #[serde(default)]
    pub description: String,
    pub tasks: Vec<Task>,
    pub max_concurrent: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    #[serde(default)]
    pub description: String,
    pub prompt: String,
    #[serde(default)]
    pub files_affected: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    pub timeout_seconds: Option<u64>,
}

pub fn load(path: &Path) -> Result<Plan> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("reading plan {:?}", path))?;
    let plan: Plan = serde_json::from_str(&content)
        .with_context(|| format!("parsing plan {:?}", path))?;
    validate_plan(&plan)?;
    Ok(plan)
}

pub fn validate_plan(plan: &Plan) -> Result<()> {
    if plan.version != 1 {
        return Err(anyhow!(
            "unsupported plan version {} (expected 1)",
            plan.version
        ));
    }
    if plan.waves.is_empty() {
        return Err(anyhow!("plan has no waves"));
    }

    let mut global_task_ids: HashSet<&str> = HashSet::new();
    for wave in &plan.waves {
        let mut wave_files: HashSet<&str> = HashSet::new();
        for task in &wave.tasks {
            if task.id.is_empty() {
                return Err(anyhow!("wave {} contains a task with empty id", wave.id));
            }
            if !global_task_ids.insert(&task.id) {
                return Err(anyhow!("duplicate task id {}", task.id));
            }
            for f in &task.files_affected {
                if !wave_files.insert(f) {
                    return Err(anyhow!(
                        "wave {} has conflicting file {} in multiple tasks",
                        wave.id,
                        f
                    ));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_task(id: &str, files: &[&str]) -> Task {
        Task {
            id: id.to_string(),
            description: String::new(),
            prompt: "p".into(),
            files_affected: files.iter().map(|s| s.to_string()).collect(),
            depends_on: vec![],
            timeout_seconds: None,
        }
    }

    fn mk_wave(id: &str, tasks: Vec<Task>) -> Wave {
        Wave {
            id: id.to_string(),
            description: String::new(),
            tasks,
            max_concurrent: None,
        }
    }

    #[test]
    fn valid_plan() {
        let plan = Plan {
            version: 1,
            title: "t".into(),
            waves: vec![mk_wave(
                "w1",
                vec![mk_task("t1", &["a.rs"]), mk_task("t2", &["b.rs"])],
            )],
        };
        validate_plan(&plan).unwrap();
    }

    #[test]
    fn rejects_version_mismatch() {
        let plan = Plan {
            version: 2,
            title: "t".into(),
            waves: vec![mk_wave("w1", vec![mk_task("t1", &[])])],
        };
        assert!(validate_plan(&plan).is_err());
    }

    #[test]
    fn rejects_empty_waves() {
        let plan = Plan {
            version: 1,
            title: "t".into(),
            waves: vec![],
        };
        assert!(validate_plan(&plan).is_err());
    }

    #[test]
    fn rejects_file_conflict_same_wave() {
        let plan = Plan {
            version: 1,
            title: "t".into(),
            waves: vec![mk_wave(
                "w1",
                vec![mk_task("t1", &["a.rs"]), mk_task("t2", &["a.rs"])],
            )],
        };
        let err = validate_plan(&plan).unwrap_err().to_string();
        assert!(err.contains("a.rs"), "err: {err}");
    }

    #[test]
    fn allows_same_file_across_waves() {
        let plan = Plan {
            version: 1,
            title: "t".into(),
            waves: vec![
                mk_wave("w1", vec![mk_task("t1", &["a.rs"])]),
                mk_wave("w2", vec![mk_task("t2", &["a.rs"])]),
            ],
        };
        validate_plan(&plan).unwrap();
    }

    #[test]
    fn rejects_duplicate_task_id() {
        let plan = Plan {
            version: 1,
            title: "t".into(),
            waves: vec![
                mk_wave("w1", vec![mk_task("t1", &["a.rs"])]),
                mk_wave("w2", vec![mk_task("t1", &["b.rs"])]),
            ],
        };
        assert!(validate_plan(&plan).is_err());
    }

    #[test]
    fn load_from_json() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("plan.json");
        let json = r#"{
            "version": 1,
            "title": "demo",
            "waves": [
                {
                    "id": "w1",
                    "description": "first",
                    "tasks": [
                        {
                            "id": "t1",
                            "prompt": "do it",
                            "files_affected": ["src/lib.rs"]
                        }
                    ]
                }
            ]
        }"#;
        std::fs::write(&p, json).unwrap();
        let plan = load(&p).unwrap();
        assert_eq!(plan.version, 1);
        assert_eq!(plan.waves.len(), 1);
        assert_eq!(plan.waves[0].tasks[0].id, "t1");
    }
}
