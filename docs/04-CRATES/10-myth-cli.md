# `myth-cli` — 주 CLI 진입점

## 역할

사용자가 직접 호출하는 **주 CLI 바이너리**. 단일 `myth` 커맨드에 모든 서브커맨드를 dispatch한다. 다른 모든 crate를 orchestrate하는 **최상위 레이어**.

**의존**: 모든 crate (Layer 0~4). myth-cli가 myth 전체의 사용자 진입점이다.
**의존받음**: 없음 (최종 실행체).

## Cargo.toml

```toml
[package]
name = "myth-cli"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[dependencies]
myth-common = { path = "../myth-common" }
myth-db = { path = "../myth-db" }
myth-gavel = { path = "../myth-gavel" }
myth-identity = { path = "../myth-identity" }
myth-orchestrator = { path = "../myth-orchestrator" }
myth-runtime = { path = "../myth-runtime" }
myth-ui = { path = "../myth-ui" }

clap = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
serde_yaml = { workspace = true }
tokio = { workspace = true, features = ["rt-multi-thread", "macros", "fs", "process"] }
mimalloc = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }

[[bin]]
name = "myth"
path = "src/main.rs"
```

## 모듈 구조

```
crates/myth-cli/
└── src/
    ├── main.rs                 # clap parse + dispatch
    ├── args.rs                 # clap Args 정의
    ├── subcmd/
    │   ├── mod.rs
    │   ├── init.rs             # myth init (프로젝트 스캐폴딩)
    │   ├── install.rs          # myth install / uninstall
    │   ├── run.rs              # myth run (Claude Code 호출)
    │   ├── status.rs           # myth status (짧은 요약)
    │   ├── watch.rs            # myth watch (TUI)
    │   ├── doctor.rs           # myth doctor (health check)
    │   ├── lesson.rs           # myth lesson list/show/appeal/retrial/split/merge
    │   ├── observer.rs         # myth observer run
    │   ├── gavel.rs            # myth gavel status/stop (Milestone C 이후)
    │   ├── embed.rs            # myth embed status/stop/probe
    │   ├── constitution.rs     # myth constitution view
    │   └── key.rs              # myth key set/show/clear (Milestone A 이후)
    └── output.rs               # 출력 포맷 helper (text/json 선택)
```

## `main.rs` — 진입점

```rust
use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use clap::Parser;

#[tokio::main]
async fn main() -> std::process::ExitCode {
    myth_common::logging::init_logging("myth");
    
    let args = args::MythArgs::parse();
    
    let result = match args.command {
        Command::Init(a) => subcmd::init::run(a).await,
        Command::Install(a) => subcmd::install::run(a).await,
        Command::Uninstall(a) => subcmd::install::uninstall(a).await,
        Command::Run(a) => subcmd::run::run(a).await,
        Command::Status(a) => subcmd::status::run(a).await,
        Command::Watch(a) => subcmd::watch::run(a).await,
        Command::Doctor(a) => subcmd::doctor::run(a).await,
        Command::Lesson(a) => subcmd::lesson::run(a).await,
        Command::Observer(a) => subcmd::observer::run(a).await,
        Command::Gavel(a) => subcmd::gavel::run(a).await,
        Command::Embed(a) => subcmd::embed::run(a).await,
        Command::Constitution(a) => subcmd::constitution::run(a).await,
        Command::Key(a) => subcmd::key::run(a).await,
    };
    
    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {:#}", e);
            std::process::ExitCode::from(1)
        }
    }
}
```

## `args.rs` — clap 정의

```rust
use clap::{Parser, Subcommand, Args};

#[derive(Parser)]
#[command(name = "myth", version, about = "Local AI agent orchestrator wrapping Claude Code")]
pub struct MythArgs {
    #[command(subcommand)]
    pub command: Command,
    
    /// Output format
    #[arg(long, global = true, value_enum, default_value = "text")]
    pub format: OutputFormat,
    
    /// Verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
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
    
    /// Health check (build profile, WSL2, migration readiness)
    Doctor(DoctorArgs),
    
    /// Manage lessons (list, show, appeal, retrial, split, merge)
    Lesson(LessonArgs),
    
    /// Observer operations (run weekly analysis)
    Observer(ObserverArgs),
    
    /// The Gavel daemon management (post-Milestone C)
    Gavel(GavelArgs),
    
    /// myth-embed daemon management
    Embed(EmbedArgs),
    
    /// View CONSTITUTION.md
    Constitution(ConstitutionArgs),
    
    /// Manage Anthropic API key (post-Milestone A)
    Key(KeyArgs),
}

#[derive(Args)]
pub struct InitArgs {
    /// Project path (default: current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,
    
    /// Overwrite existing .claude/ files
    #[arg(long)]
    pub force: bool,
}

#[derive(Args)]
pub struct RunArgs {
    /// Plan JSON file for parallel execution
    #[arg(long)]
    pub plan: Option<PathBuf>,
    
    /// Max concurrent tasks
    #[arg(long, default_value = "3")]
    pub max_concurrent: usize,
    
    /// Pass-through args for claude binary
    #[arg(trailing_var_arg = true)]
    pub claude_args: Vec<String>,
}

#[derive(Args)]
pub struct LessonArgs {
    #[command(subcommand)]
    pub action: LessonAction,
}

#[derive(Subcommand)]
pub enum LessonAction {
    /// List lessons
    List {
        #[arg(long)] level: Option<u8>,
        #[arg(long)] status: Option<String>,  // active/lapsed/archived
        #[arg(long, default_value = "20")] limit: usize,
    },
    /// Show detail of one lesson
    Show {
        id: String,  // LessonId prefix (e.g., "L3-0012")
    },
    /// Request re-evaluation
    Appeal {
        id: String,
        #[arg(long)] reason: String,
    },
    /// Full retrial with different model (Level 4-5 only)
    Retrial {
        id: String,
        #[arg(long)] reason: String,
    },
    /// Split a lesson into two
    Split {
        id: String,
        #[arg(long)] reason: String,
    },
    /// Merge two lessons into one
    Merge {
        id1: String,
        id2: String,
        #[arg(long)] reason: String,
    },
}

// GavelArgs, EmbedArgs, ObserverArgs, DoctorArgs, KeyArgs 등 유사한 구조
```

## 서브커맨드 동작

### `myth init` — 프로젝트 스캐폴딩

```rust
pub async fn run(args: InitArgs) -> Result<ExitCode> {
    let project = args.path.canonicalize()?;
    
    // Git repo 확인
    if !project.join(".git").exists() {
        eprintln!("warning: {} is not a git repository", project.display());
    }
    
    let claude_dir = project.join(".claude");
    
    if claude_dir.exists() && !args.force {
        return Err(anyhow!("{} already exists. Use --force to overwrite.", claude_dir.display()));
    }
    
    std::fs::create_dir_all(&claude_dir)?;
    std::fs::create_dir_all(claude_dir.join("agents"))?;
    
    // 1. settings.json 생성
    let settings = build_settings_json()?;
    std::fs::write(claude_dir.join("settings.json"), settings)?;
    
    // 2. agents/*.md 심볼릭 링크 (또는 복사)
    symlink_or_copy(
        &myth_home().join("templates/.claude/agents/assessor.md"),
        &claude_dir.join("agents/assessor.md"),
    )?;
    symlink_or_copy(
        &myth_home().join("templates/.claude/agents/observer.md"),
        &claude_dir.join("agents/observer.md"),
    )?;
    
    // 3. CLAUDE.md 템플릿 (선택)
    if !project.join("CLAUDE.md").exists() {
        let template = std::fs::read_to_string(
            myth_home().join("templates/CLAUDE.md.template")
        )?;
        std::fs::write(project.join("CLAUDE.md"), template)?;
    }
    
    println!("myth initialized in {}", project.display());
    println!("  .claude/settings.json");
    println!("  .claude/agents/assessor.md");
    println!("  .claude/agents/observer.md");
    println!("");
    println!("Next: run `myth run` to start a supervised Claude Code session.");
    
    Ok(ExitCode::SUCCESS)
}
```

### `myth install` — 바이너리 설치

```rust
pub async fn run(args: InstallArgs) -> Result<ExitCode> {
    let bin_dir = dirs::home_dir().unwrap().join(".local/bin");
    std::fs::create_dir_all(&bin_dir)?;
    
    let myth_rust_target = myth_home().join("rust/target/release");
    
    // 8개 바이너리 심볼릭 링크 (또는 복사)
    let binaries = [
        "myth",
        "myth-hook-pre-tool",
        "myth-hook-post-tool",
        "myth-hook-post-tool-failure",
        "myth-hook-user-prompt",
        "myth-hook-stop",
        "myth-hook-session-start",
        "myth-embed",
    ];
    
    for bin in binaries {
        let src = myth_rust_target.join(bin);
        let dst = bin_dir.join(bin);
        
        if !src.exists() {
            return Err(anyhow!(
                "binary {} not found. Run `cargo build --release` in ~/myth/rust first.",
                bin
            ));
        }
        
        if dst.exists() { std::fs::remove_file(&dst)?; }
        symlink_or_copy(&src, &dst)?;
        println!("installed: {}", dst.display());
    }
    
    // Python shim scripts (myth-assessor, myth-observer)
    write_python_shim(&bin_dir, "myth-assessor", "myth_py.assessor.cli")?;
    write_python_shim(&bin_dir, "myth-observer", "myth_py.observer.cli")?;
    
    // PATH 확인
    if !std::env::var("PATH").unwrap_or_default().contains(".local/bin") {
        eprintln!("");
        eprintln!("warning: ~/.local/bin is not in your PATH");
        eprintln!("Add to ~/.bashrc: export PATH=\"$HOME/.local/bin:$PATH\"");
    }
    
    // 초기 데이터 구조 생성
    init_myth_home()?;
    
    Ok(ExitCode::SUCCESS)
}

fn init_myth_home() -> Result<()> {
    let home = myth_home();
    std::fs::create_dir_all(&home)?;
    std::fs::create_dir_all(home.join("metrics"))?;
    std::fs::create_dir_all(home.join("archive"))?;
    std::fs::create_dir_all(myth_common::myth_state())?;
    
    // 기본 rule·grid 파일 (이미 있으면 건드리지 않음)
    if !home.join("bedrock-rules.yaml").exists() {
        let default = include_str!("../../../../templates/bedrock-rules.yaml");
        std::fs::write(home.join("bedrock-rules.yaml"), default)?;
    }
    if !home.join("foundation-rules.yaml").exists() {
        let default = include_str!("../../../../templates/foundation-rules.yaml");
        std::fs::write(home.join("foundation-rules.yaml"), default)?;
    }
    if !home.join("surface-rules.yaml").exists() {
        std::fs::write(home.join("surface-rules.yaml"), "rules: []\n")?;
    }
    if !home.join("grid.yaml").exists() {
        let default = include_str!("../../../../templates/grid.yaml");
        std::fs::write(home.join("grid.yaml"), default)?;
    }
    
    // SQLite 초기화 (Database::open이 마이그레이션 적용)
    let _ = Database::open(&myth_common::state_db_path())?;
    
    Ok(())
}
```

### `myth run` — Claude Code 실행

```rust
pub async fn run(args: RunArgs) -> Result<ExitCode> {
    if let Some(plan_path) = args.plan {
        // 병렬 실행 모드
        let orch = Orchestrator::new(OrchestratorConfig {
            max_concurrent: args.max_concurrent,
            ..Default::default()
        });
        let report = orch.execute_plan(&plan_path).await?;
        println!("{}", report.to_summary());
        Ok(if report.failed == 0 { ExitCode::SUCCESS } else { ExitCode::from(1) })
    } else {
        // 인터랙티브 모드
        let worktree = std::env::current_dir()?;
        let runtime = ClaudeRuntime::new(&worktree)?;
        runtime.spawn_interactive().await
    }
}
```

### `myth status` — 간단 요약

```rust
pub async fn run(args: StatusArgs) -> Result<ExitCode> {
    let db = Database::open(&myth_common::state_db_path())?;
    let store = SqliteLessonStore::new(db);
    
    let active = store.list_active()?.len();
    let lapsed = store.list_lapsed()?.len();
    let archived_count = store.count_archived()?;
    
    let embed_status = EmbedClient::new().ping().await.ok();
    
    println!("myth status");
    println!("  Lessons: {} active, {} lapsed, {} archived", active, lapsed, archived_count);
    match embed_status {
        Some(info) => println!("  myth-embed: running (uptime {}s, {} requests)",
                               info.uptime_secs, info.request_count),
        None => println!("  myth-embed: not running"),
    }
    
    // Hook latency 7-day P99
    let p99 = compute_hook_p99_last_7d()?;
    println!("  Hook P99 (7d): {:.1}ms", p99);
    
    // Brief 마지막 갱신
    if let Ok(meta) = std::fs::metadata(myth_common::brief_path()) {
        let mtime = meta.modified()?;
        let age_days = (SystemTime::now().duration_since(mtime)?).as_secs() / 86400;
        println!("  Brief updated: {} days ago", age_days);
    }
    
    Ok(ExitCode::SUCCESS)
}
```

### `myth watch` — TUI

```rust
pub async fn run(args: WatchArgs) -> Result<ExitCode> {
    myth_ui::run_dashboard().await?;
    Ok(ExitCode::SUCCESS)
}
```

### `myth doctor` — 헬스 체크

```rust
pub async fn run(args: DoctorArgs) -> Result<ExitCode> {
    let mut checks = Vec::new();
    
    // 기본 체크
    checks.push(check_claude_binary());
    checks.push(check_myth_home());
    checks.push(check_rules_files());
    checks.push(check_sqlite_integrity());
    checks.push(check_embed_daemon());
    checks.push(check_hook_registration());
    
    if args.perf_check {
        checks.push(check_build_profile());
        checks.push(check_pgo_applied());
    }
    if args.wsl_check {
        checks.push(check_wsl2_environment());
    }
    if args.migration {
        checks.extend(check_all_milestones()?);
    }
    
    let mut failed = 0;
    for check in &checks {
        match check {
            CheckResult::Pass(msg) => println!("  ✓ {}", msg),
            CheckResult::Warn(msg) => println!("  ⚠ {}", msg),
            CheckResult::Fail(msg) => {
                println!("  ✗ {}", msg);
                failed += 1;
            }
        }
    }
    
    if failed > 0 {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}
```

### `myth lesson` — lesson 관리

```rust
pub async fn run(args: LessonArgs) -> Result<ExitCode> {
    match args.action {
        LessonAction::List { level, status, limit } => list_lessons(level, status, limit).await,
        LessonAction::Show { id } => show_lesson(&id).await,
        LessonAction::Appeal { id, reason } => appeal_lesson(&id, &reason).await,
        LessonAction::Retrial { id, reason } => retrial_lesson(&id, &reason).await,
        LessonAction::Split { id, reason } => split_lesson(&id, &reason).await,
        LessonAction::Merge { id1, id2, reason } => merge_lessons(&id1, &id2, &reason).await,
    }
}

async fn appeal_lesson(id: &str, reason: &str) -> Result<ExitCode> {
    let lesson_id = resolve_lesson_id(id)?;
    let db = Database::open(&myth_common::state_db_path())?;
    let store = SqliteLessonStore::new(db.clone());
    
    let lesson = store.get(lesson_id)?
        .ok_or_else(|| anyhow!("lesson {} not found", id))?;
    
    // Level별 appeal 제약 확인
    let allowed = match lesson.level {
        Level::Critical => lesson.appeals < 5,  // 실제로는 공동 서명자 필요 (Day-1 미구현, 5 soft limit)
        Level::High => lesson.appeals < 3,
        Level::Medium => lesson.appeals < 2,
        Level::Low | Level::Info => lesson.appeals < 1,
    };
    
    if !allowed {
        return Err(anyhow!("appeal limit exceeded for level {:?}", lesson.level));
    }
    
    // Bedrock Seal은 불가
    if is_bedrock_match(&lesson)? {
        return Err(anyhow!("Bedrock Rule matches cannot be appealed (Seal)"));
    }
    
    // appeal_history 기록
    let appeal = AppealRecord {
        lesson_id,
        appeal_type: "appeal".into(),
        ts: now(),
        result: "pending".into(),
        rationale: Some(reason.to_string()),
    };
    store.record_appeal(&appeal)?;
    
    // lesson.appeals 증가
    let mut updated = lesson.clone();
    updated.appeals += 1;
    store.update(&updated)?;
    
    println!("Appeal recorded for lesson {}", id);
    println!("This will be reviewed in the next Observer run.");
    Ok(ExitCode::SUCCESS)
}

async fn list_lessons(level: Option<u8>, status: Option<String>, limit: usize) -> Result<ExitCode> {
    let db = Database::open(&myth_common::state_db_path())?;
    let store = SqliteLessonStore::new(db);
    
    let all = store.query(LessonQuery { level, status, limit })?;
    
    for l in all {
        println!("{:8} L{} {:10} {:20} (rec {}, {})",
                 l.id.short(),
                 l.level as u8,
                 l.level.label(),
                 truncate(&l.rationale, 20),
                 Recurrence::from_count(l.recurrence_count) as u8,
                 format_category(&l.category));
    }
    
    Ok(ExitCode::SUCCESS)
}
```

### `myth observer run`

```rust
pub async fn run(args: ObserverArgs) -> Result<ExitCode> {
    match args.action {
        ObserverAction::Run { dry } => {
            println!("Running Observer weekly analysis...");
            
            // Python subprocess 호출
            let output = tokio::process::Command::new("python3")
                .args(["-m", "myth_py.observer.cli", "run"])
                .arg(if dry { "--dry" } else { "" })
                .env("MYTH_SESSION_ID", SessionId::new().to_string())
                .output().await?;
            
            if !output.status.success() {
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                return Ok(ExitCode::from(1));
            }
            
            println!("{}", String::from_utf8_lossy(&output.stdout));
            Ok(ExitCode::SUCCESS)
        }
    }
}
```

### `myth gavel status/stop` (Milestone C 이후)

```rust
pub async fn run(args: GavelArgs) -> Result<ExitCode> {
    match args.action {
        GavelAction::Status => {
            let client = GavelDaemonClient::new();  // Milestone C 이후 구현
            match client.ping().await {
                Ok(info) => {
                    println!("The Gavel daemon");
                    println!("  PID:      {}", info.pid);
                    println!("  Uptime:   {}s", info.uptime_secs);
                    println!("  Requests: {}", info.request_count);
                    println!("  P99:      {:.1}ms", info.p99_ms);
                    Ok(ExitCode::SUCCESS)
                }
                Err(_) => {
                    println!("The Gavel daemon is not running (binary-per-hook mode)");
                    Ok(ExitCode::SUCCESS)
                }
            }
        }
        GavelAction::Stop => {
            // Milestone C 이후만 의미 있음
            if !is_milestone_c_reached()? {
                println!("No Gavel daemon to stop (still in binary-per-hook mode)");
                return Ok(ExitCode::SUCCESS);
            }
            // shutdown 요청
            Ok(ExitCode::SUCCESS)
        }
    }
}
```

### `myth embed status/stop/probe`

```rust
pub async fn run(args: EmbedArgs) -> Result<ExitCode> {
    // myth-embed 바이너리에 직접 위임 (이미 구현됨)
    let exe = std::env::current_exe()?
        .parent().unwrap()
        .join("myth-embed");
    
    let mut cmd = tokio::process::Command::new(&exe);
    match args.action {
        EmbedAction::Status => cmd.arg("status"),
        EmbedAction::Stop => cmd.arg("stop"),
        EmbedAction::Probe { text } => cmd.args(["probe", &text]),
    };
    
    let status = cmd.status().await?;
    Ok(ExitCode::from(status.code().unwrap_or(1) as u8))
}
```

### `myth constitution [view]`

```rust
pub async fn run(args: ConstitutionArgs) -> Result<ExitCode> {
    let path = myth_home().join("CONSTITUTION.md");
    
    if !path.exists() {
        return Err(anyhow!("CONSTITUTION.md not found at {}", path.display()));
    }
    
    // less로 열기 (pager)
    let pager = std::env::var("PAGER").unwrap_or_else(|_| "less".to_string());
    let status = tokio::process::Command::new(pager)
        .arg(&path)
        .status().await?;
    
    Ok(ExitCode::from(status.code().unwrap_or(0) as u8))
}
```

### `myth key` (Milestone A 이후)

```rust
pub async fn run(args: KeyArgs) -> Result<ExitCode> {
    match args.action {
        KeyAction::Set { from_stdin } => {
            let key = if from_stdin {
                let mut s = String::new();
                std::io::stdin().read_line(&mut s)?;
                s.trim().to_string()
            } else {
                rpassword::prompt_password("Enter Anthropic API key: ")?
            };
            
            if !key.starts_with("sk-ant-") {
                return Err(anyhow!("invalid key format"));
            }
            
            let path = myth_common::myth_config().join("api_key");
            std::fs::create_dir_all(path.parent().unwrap())?;
            std::fs::write(&path, format!("{}\n", key))?;
            std::fs::set_permissions(&path, Permissions::from_mode(0o600))?;
            
            println!("API key saved to {}", path.display());
            println!("Enable Tier 3 in ~/.config/myth/config.yaml: assessor.tier_3_enabled: true");
            Ok(ExitCode::SUCCESS)
        }
        KeyAction::Show => {
            let path = myth_common::myth_config().join("api_key");
            if !path.exists() {
                println!("No API key configured");
                return Ok(ExitCode::from(1));
            }
            let key = std::fs::read_to_string(&path)?;
            let preview = format!("{}...{}", &key[..12], &key[key.len()-4..]);
            println!("API key: {}", preview);
            Ok(ExitCode::SUCCESS)
        }
        KeyAction::Clear => {
            let path = myth_common::myth_config().join("api_key");
            if path.exists() {
                std::fs::remove_file(&path)?;
                println!("API key cleared");
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}
```

## `output.rs` — 출력 포맷

```rust
pub fn format_output<T: Serialize>(
    data: &T,
    format: OutputFormat,
) -> Result<String> {
    match format {
        OutputFormat::Text => {
            // 각 타입별 Display 구현 사용
            Ok(format!("{}", data))  // 실제로는 Display trait
        }
        OutputFormat::Json => Ok(serde_json::to_string_pretty(data)?),
        OutputFormat::Yaml => Ok(serde_yaml::to_string(data)?),
    }
}
```

`--format json` 사용 시:
```bash
myth status --format json
# → {"active_lessons": 23, "lapsed": 2, ...}
```

## 전체 서브커맨드 요약표

| 명령 | 목적 | Milestone |
|---|---|---|
| `myth init [path]` | 프로젝트 스캐폴딩 | Day-1 |
| `myth install` | 바이너리 설치 | Day-1 |
| `myth uninstall` | 제거 | Day-1 |
| `myth run [--plan ... ]` | Claude Code 실행 (인터랙티브/병렬) | Day-1 |
| `myth status` | 간단 요약 | Day-1 |
| `myth watch` | TUI 대시보드 | Day-1 |
| `myth doctor [--perf-check --wsl-check --migration]` | 헬스 체크 | Day-1 |
| `myth lesson list/show/appeal/retrial/split/merge` | lesson 관리 | Day-1 |
| `myth observer run [--dry]` | 주간 분석 수동 실행 | Day-1 |
| `myth gavel status/stop` | The Gavel daemon 관리 | Milestone C |
| `myth embed status/stop/probe` | embed daemon 관리 | Day-1 |
| `myth constitution` | CONSTITUTION 열람 | Day-1 |
| `myth key set/show/clear` | API key 관리 | Milestone A |

## 성능

CLI 콜드 스타트:
- 경량 명령 (`status`, `lesson list`): ~20-50ms
- `watch` (TUI 초기화): ~100ms
- `run` (Claude Code spawn): Claude Code 시작 시간에 의존 (수 초)

mimalloc + LTO fat로 Cargo release 프로파일 적용.

## 테스트

```
tests/
├── cli_parse_test.rs          # clap 파싱 엣지 케이스
├── init_test.rs               # 스캐폴딩 결과물 검증
├── lesson_commands_test.rs    # DB fixture로 lesson 명령 검증
├── doctor_test.rs             # health check 동작
└── integration/
    └── end_to_end.rs           # install → init → run 시나리오
```

## 관련 결정

- Decision 8 (문서 분할): 이 crate가 "사용자가 실제로 쓰는" 진입점
- 카테고리 8 (사용자 행동): appeal/retrial/split/merge 모두 이 CLI에서 실행
- ARCHITECTURE §7 (파일 레이아웃): install.sh의 Rust 재구현이 `myth install` 서브커맨드
- 네이밍 전체: 모든 서브커맨드가 확정 용어를 반영
