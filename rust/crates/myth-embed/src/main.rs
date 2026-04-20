//! myth-embed binary — unified client/daemon entry.
//!
//! Mode selection:
//!   --daemon            → enter the long-running daemon loop
//!   status              → print daemon stats (Ping)
//!   stop                → ask the daemon to exit (Shutdown)
//!   probe <text>        → embed <text> and print the vector summary
//!   (no/unknown args)   → print usage
//!
//! The binary runs on a single-thread tokio runtime. No heavy model
//! work on the main task — fastembed blocking calls go through
//! `spawn_blocking`.

use std::process::ExitCode;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    myth_common::logging::init_logging("myth-embed");

    let args: Vec<String> = std::env::args().collect();

    let result = match args.get(1).map(|s| s.as_str()) {
        Some("--daemon") => myth_embed::daemon::run().await,
        Some("status") => myth_embed::cli::status().await,
        Some("stop") => myth_embed::cli::stop().await,
        Some("probe") => myth_embed::cli::probe(&args[2..]).await,
        _ => {
            print_usage();
            Ok(ExitCode::SUCCESS)
        }
    };

    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("myth-embed: {:#}", e);
            ExitCode::from(1)
        }
    }
}

fn print_usage() {
    eprintln!("usage: myth-embed <command>");
    eprintln!();
    eprintln!("commands:");
    eprintln!("  --daemon            run as daemon (auto-spawned by clients)");
    eprintln!("  status              show daemon status");
    eprintln!("  stop                shut the daemon down");
    eprintln!("  probe <text>        embed <text> and print vector summary");
}
