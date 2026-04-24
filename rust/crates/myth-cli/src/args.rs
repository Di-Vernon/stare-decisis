//! clap parser — 13 서브커맨드 (Day-1 11 + Milestone 지연 2).

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "myth",
    version,
    about = "Local AI agent orchestrator wrapping Claude Code"
)]
pub struct MythArgs {
    #[command(subcommand)]
    pub command: Command,

    /// Output format
    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,

    /// Verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Yaml,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Scaffold myth into the current project
    Init(InitArgs),
    /// Install myth binaries to ~/.local/bin
    Install(InstallArgs),
    /// Uninstall myth from ~/.local/bin
    Uninstall(UninstallArgs),
    /// Run Claude Code under myth supervision
    Run(RunArgs),
    /// Show short status summary
    Status(StatusArgs),
    /// Launch TUI dashboard
    Watch(WatchArgs),
    /// Health check (claude binary, WSL2, migration readiness)
    Doctor(DoctorArgs),
    /// Manage lessons (list, show, appeal, retrial, split, merge)
    Lesson(LessonArgs),
    /// Observer operations (run weekly analysis via Python)
    Observer(ObserverArgs),
    /// The Gavel daemon management (Milestone C — stub)
    Gavel(GavelArgs),
    /// myth-embed daemon management
    Embed(EmbedArgs),
    /// View CONSTITUTION.md in $PAGER
    Constitution(ConstitutionArgs),
    /// Manage Anthropic API key (Milestone A — stub)
    Key(KeyArgs),
    /// Link myth hooks into a Claude Code project's .claude/settings.json
    Link(LinkArgs),
    /// Remove myth hooks from a Claude Code project's .claude/settings.json
    Unlink(UnlinkArgs),
}

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Project path (default: current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,
    /// Overwrite existing .claude/ files
    #[arg(long)]
    pub force: bool,
}

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// Override ~/.local/bin
    #[arg(long)]
    pub prefix: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct UninstallArgs {
    /// Override ~/.local/bin
    #[arg(long)]
    pub prefix: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Plan JSON file for parallel execution
    #[arg(long)]
    pub plan: Option<PathBuf>,
    /// Max concurrent tasks
    #[arg(long, default_value_t = 3)]
    pub max_concurrent: usize,
    /// Pass-through args for claude binary
    #[arg(trailing_var_arg = true)]
    pub claude_args: Vec<String>,
}

#[derive(Args, Debug)]
pub struct StatusArgs {}

#[derive(Args, Debug)]
pub struct WatchArgs {}

#[derive(Args, Debug)]
pub struct DoctorArgs {
    /// Additionally check release build profile + PGO
    #[arg(long)]
    pub perf_check: bool,
    /// Additionally check WSL2 environment specifics
    #[arg(long)]
    pub wsl_check: bool,
    /// Additionally report migration readiness (A-E)
    #[arg(long)]
    pub migration: bool,
}

#[derive(Args, Debug)]
pub struct LessonArgs {
    #[command(subcommand)]
    pub action: LessonAction,
}

#[derive(Subcommand, Debug)]
pub enum LessonAction {
    /// List lessons
    List {
        #[arg(long)]
        level: Option<u8>,
        /// active / lapsed / archived
        #[arg(long)]
        status: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Show detail of one lesson
    Show { id: String },
    /// Request re-evaluation
    Appeal {
        id: String,
        #[arg(long)]
        reason: String,
    },
    /// Full retrial (Level 4-5 only)
    Retrial {
        id: String,
        #[arg(long)]
        reason: String,
    },
    /// Split a lesson into two (Wave 8 — stub)
    Split {
        id: String,
        #[arg(long)]
        reason: String,
    },
    /// Merge two lessons (Wave 8 — stub)
    Merge {
        id1: String,
        id2: String,
        #[arg(long)]
        reason: String,
    },
}

#[derive(Args, Debug)]
pub struct ObserverArgs {
    #[command(subcommand)]
    pub action: ObserverAction,
}

#[derive(Subcommand, Debug)]
pub enum ObserverAction {
    /// Run weekly analysis (requires Python runtime — Wave 6)
    Run {
        #[arg(long)]
        dry: bool,
    },
}

#[derive(Args, Debug)]
pub struct GavelArgs {
    #[command(subcommand)]
    pub action: GavelAction,
}

#[derive(Subcommand, Debug)]
pub enum GavelAction {
    Status,
    Stop,
}

#[derive(Args, Debug)]
pub struct EmbedArgs {
    #[command(subcommand)]
    pub action: EmbedAction,
}

#[derive(Subcommand, Debug)]
pub enum EmbedAction {
    Status,
    Stop,
    Probe { text: String },
}

#[derive(Args, Debug)]
pub struct ConstitutionArgs {}

#[derive(Args, Debug)]
pub struct KeyArgs {
    #[command(subcommand)]
    pub action: KeyAction,
}

#[derive(Subcommand, Debug)]
pub enum KeyAction {
    Set {
        #[arg(long)]
        from_stdin: bool,
    },
    Show,
    Clear,
}

#[derive(Args, Debug)]
pub struct LinkArgs {
    /// Project path (default: current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

#[derive(Args, Debug)]
pub struct UnlinkArgs {
    /// Project path (default: current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn clap_definition_valid() {
        MythArgs::command().debug_assert();
    }

    #[test]
    fn parses_init() {
        let args = MythArgs::try_parse_from(["myth", "init", "."]).unwrap();
        assert!(matches!(args.command, Command::Init(_)));
    }

    #[test]
    fn parses_run_with_plan() {
        let args = MythArgs::try_parse_from(["myth", "run", "--plan", "plan.json"]).unwrap();
        match args.command {
            Command::Run(r) => {
                assert_eq!(r.plan.as_deref().unwrap().to_str().unwrap(), "plan.json");
                assert_eq!(r.max_concurrent, 3);
            }
            _ => panic!("expected Run"),
        }
    }

    #[test]
    fn parses_lesson_list() {
        let args =
            MythArgs::try_parse_from(["myth", "lesson", "list", "--limit", "5"]).unwrap();
        match args.command {
            Command::Lesson(l) => match l.action {
                LessonAction::List { limit, .. } => assert_eq!(limit, 5),
                _ => panic!("expected List"),
            },
            _ => panic!("expected Lesson"),
        }
    }

    #[test]
    fn parses_doctor_flags() {
        let args = MythArgs::try_parse_from([
            "myth", "doctor", "--perf-check", "--wsl-check", "--migration",
        ])
        .unwrap();
        match args.command {
            Command::Doctor(d) => {
                assert!(d.perf_check && d.wsl_check && d.migration);
            }
            _ => panic!("expected Doctor"),
        }
    }

    #[test]
    fn global_format_json() {
        let args =
            MythArgs::try_parse_from(["myth", "--format", "json", "status"]).unwrap();
        assert!(matches!(args.format, OutputFormat::Json));
    }
}
