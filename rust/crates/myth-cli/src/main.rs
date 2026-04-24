//! `myth` binary entrypoint.

use clap::Parser;
use mimalloc::MiMalloc;
use myth_cli::{args, subcmd, Command, MythArgs};
use std::process::ExitCode;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    myth_common::logging::init_logging("myth");

    let args_parsed = MythArgs::parse();
    let format = args_parsed.format;

    let result: anyhow::Result<ExitCode> = match args_parsed.command {
        Command::Init(a) => subcmd::init::run(a).await,
        Command::Install(a) => subcmd::install::run(a).await,
        Command::Uninstall(a) => subcmd::install::uninstall(a).await,
        Command::Run(a) => subcmd::run::run(a).await,
        Command::Status(a) => subcmd::status::run(a, format).await,
        Command::Watch(a) => subcmd::watch::run(a).await,
        Command::Doctor(a) => subcmd::doctor::run(a).await,
        Command::Lesson(a) => subcmd::lesson::run(a, format).await,
        Command::Observer(a) => subcmd::observer::run(a).await,
        Command::Gavel(a) => subcmd::gavel::run(a).await,
        Command::Embed(a) => subcmd::embed::run(a).await,
        Command::Constitution(a) => subcmd::constitution::run(a).await,
        Command::Key(a) => subcmd::key::run(a).await,
        Command::Link(a) => subcmd::link::run(a).await,
        Command::Unlink(a) => subcmd::link::unlink(a).await,
    };

    // 명시적으로 참조해 unused_imports 방지
    let _ = args::OutputFormat::Text;

    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::from(1)
        }
    }
}
