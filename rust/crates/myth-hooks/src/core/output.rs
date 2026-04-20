//! Return value mapping: `HookResult` → stdout JSON + ExitCode.
//!
//! ExitCode semantics (per ARCHITECTURE.md Contract 3):
//!   0  — allow (continue normally)
//!   2  — block (tool execution halted, stderr fed back to Claude)
//!   any other — non-blocking internal error
//!
//! myth deliberately maps internal hook errors to exit 0 so a bug in a
//! myth hook never cascades into a blocked Claude Code session. Only
//! an explicit Strike/Seal verdict produces exit 2.

use std::process::ExitCode;

use serde_json::Value;

#[derive(Debug, Clone)]
pub enum HookResult {
    /// Plain allow — no stdout, exit 0.
    Allow,
    /// Allow with `additionalContext` injected into Claude's context.
    AllowWithContext(Value),
    /// Ask the user (permissionDecision: "ask") — stdout JSON, exit 0.
    Ask(Value),
    /// Block tool execution — stdout JSON, exit 2.
    Block {
        output: Value,
    },
    /// Internal hook error — stderr logged, exit 0 (degrade, never
    /// cascade a myth bug into Claude's session).
    Error(String),
}

impl HookResult {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::AllowWithContext(_) => "allow_with_context",
            Self::Ask(_) => "ask",
            Self::Block { .. } => "block",
            Self::Error(_) => "error",
        }
    }

    pub fn is_blocking(&self) -> bool {
        matches!(self, Self::Block { .. })
    }

    /// Emit the hook output per contract and return the ExitCode.
    pub fn emit(self) -> ExitCode {
        match self {
            HookResult::Allow => ExitCode::SUCCESS,
            HookResult::AllowWithContext(v) | HookResult::Ask(v) => {
                print_json(&v);
                ExitCode::SUCCESS
            }
            HookResult::Block { output } => {
                print_json(&output);
                ExitCode::from(2)
            }
            HookResult::Error(msg) => {
                eprintln!("myth hook error: {}", msg);
                ExitCode::SUCCESS
            }
        }
    }
}

fn print_json(v: &Value) {
    // to_string on a valid Value cannot fail in practice; fall back
    // to a bare `{}` if serialisation ever does (never allow an error
    // printing to surface as a blocked tool).
    match serde_json::to_string(v) {
        Ok(s) => println!("{}", s),
        Err(e) => {
            eprintln!("myth hook: failed to serialise output: {}", e);
            println!("{{}}");
        }
    }
}
