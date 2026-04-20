//! Unified myth error type.
//!
//! Typed variants cover myth-domain specific errors only. Foreign library
//! errors (rusqlite, serde_yaml, …) flow through `Other(#[from] anyhow::Error)`
//! so myth-common remains free of heavy dependencies — see the layer
//! boundary note in `docs/04-CRATES/01-myth-common.md`.

#[derive(thiserror::Error, Debug)]
pub enum MythError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("config not found at {path}")]
    ConfigMissing { path: String },

    #[error("rule file parse error in {file}: {message}")]
    RuleParse { file: String, message: String },

    #[error("hook timeout after {ms}ms")]
    HookTimeout { ms: u64 },

    #[error("daemon unavailable: {reason}")]
    DaemonUnavailable { reason: String },

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, MythError>;
