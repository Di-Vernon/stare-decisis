//! `myth gavel status/stop` — Milestone C 지연 stub.

use anyhow::Result;
use std::process::ExitCode;

use crate::args::{GavelAction, GavelArgs};

pub async fn run(args: GavelArgs) -> Result<ExitCode> {
    match args.action {
        GavelAction::Status => {
            println!("The Gavel daemon is not yet enabled (binary-per-hook mode active).");
            println!(
                "  Milestone C triggers daemon migration when hook P99 > 15ms for 14 days."
            );
            println!("  Run `myth status` for current hook latency.");
            Ok(ExitCode::SUCCESS)
        }
        GavelAction::Stop => {
            println!(
                "No Gavel daemon to stop (Milestone C not yet triggered, binary-per-hook)."
            );
            Ok(ExitCode::SUCCESS)
        }
    }
}
