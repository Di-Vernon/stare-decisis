//! `myth-cli` — user-facing entrypoint.
//!
//! `main.rs`에서 parse → dispatch. 상위 레이어 공개 타입은 args 모듈.

pub mod args;
pub mod output;
pub mod subcmd;

pub use args::{Command, MythArgs, OutputFormat};
