//! `myth key set/show/clear` — Milestone A 지연 stub.

use anyhow::Result;
use std::process::ExitCode;

use crate::args::KeyArgs;

pub async fn run(args: KeyArgs) -> Result<ExitCode> {
    let _ = args.action;
    println!("Extra Usage / API key management is not yet enabled.");
    println!("  Milestone A triggers Anthropic API key configuration once Assessor Tier review");
    println!("  has accumulated 3 weeks of data.");
    Ok(ExitCode::SUCCESS)
}
