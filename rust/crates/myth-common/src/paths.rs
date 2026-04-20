//! XDG base directory resolvers and concrete myth file paths.

use std::path::PathBuf;

pub fn myth_home() -> PathBuf {
    dirs::home_dir()
        .expect("HOME must be set")
        .join(".myth")
}

pub fn myth_config() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .expect("HOME must be set")
                .join(".config")
        })
        .join("myth")
}

pub fn myth_state() -> PathBuf {
    dirs::state_dir()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .expect("HOME must be set")
                .join(".local")
                .join("state")
        })
        .join("myth")
}

pub fn myth_runtime() -> PathBuf {
    std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let uid = unsafe { libc::getuid() };
            PathBuf::from(format!("/tmp/myth-{}", uid))
        })
        .join("myth")
}

// Specific file paths (Constitution Part IX + ARCHITECTURE.md Contract 6).

pub fn bedrock_rules_path() -> PathBuf {
    myth_home().join("bedrock-rules.yaml")
}

pub fn foundation_rules_path() -> PathBuf {
    myth_home().join("foundation-rules.yaml")
}

pub fn surface_rules_path() -> PathBuf {
    myth_home().join("surface-rules.yaml")
}

pub fn grid_path() -> PathBuf {
    myth_home().join("grid.yaml")
}

pub fn state_db_path() -> PathBuf {
    myth_home().join("state.db")
}

pub fn vectors_bin_path() -> PathBuf {
    myth_home().join("vectors.bin")
}

pub fn caselog_path() -> PathBuf {
    myth_home().join("caselog.jsonl")
}

pub fn lesson_state_path() -> PathBuf {
    myth_home().join("lesson-state.jsonl")
}

pub fn audit_path() -> PathBuf {
    myth_home().join("audit.jsonl")
}

pub fn brief_path() -> PathBuf {
    myth_home().join("brief.md")
}

pub fn hook_latency_path() -> PathBuf {
    myth_state().join("hook-latency.ndjson")
}

pub fn embed_socket_path() -> PathBuf {
    myth_runtime().join("embed.sock")
}
