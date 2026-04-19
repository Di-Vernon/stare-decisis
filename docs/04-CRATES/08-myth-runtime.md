# `myth-runtime` — Claude Code Subprocess 관리

## 역할

Option 4+ **Hybrid Wrapper** 아키텍처의 핵심 계층. `claude` CLI 바이너리를 subprocess로 호출·관찰·격리한다. myth가 Claude Code를 "**감싸는**" 주체가 여기 있다.

**핵심 책임**:
1. `claude` 바이너리 찾기 + 버전 감지
2. 세션 ID 할당 + 환경 변수 주입
3. stdin/stdout/stderr 파이프 관리
4. Extra Usage fallback (Max quota 소진 시)
5. 세션 종료 감지 + graceful cleanup

**의존**: `myth-common`, `myth-db`.
**의존받음**: `myth-orchestrator`, `myth-cli`.

## Cargo.toml

```toml
[package]
name = "myth-runtime"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[dependencies]
myth-common = { path = "../myth-common" }
myth-db = { path = "../myth-db" }

serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true, features = ["process", "io-util", "sync", "time"] }
uuid = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }
which = "6"  # claude 바이너리 탐색
```

## 모듈 구조

```
crates/myth-runtime/
└── src/
    ├── lib.rs              # ClaudeRuntime 공개 API
    ├── discovery.rs        # claude 바이너리 위치·버전 감지
    ├── session.rs          # 세션 생명주기
    ├── env.rs              # 환경변수 주입
    ├── io.rs               # stdin/stdout/stderr 처리
    ├── fallback.rs         # Extra Usage 경로
    └── version.rs          # Claude Code 버전 호환성
```

## 공개 API — `ClaudeRuntime`

```rust
pub struct ClaudeRuntime {
    claude_path: PathBuf,
    version: ClaudeVersion,
    worktree: PathBuf,
    session_id: SessionId,
}

impl ClaudeRuntime {
    pub fn new(worktree: &Path) -> Result<Self> {
        let claude_path = discovery::find_claude()?;
        let version = discovery::detect_version(&claude_path)?;
        version.validate_compatible()?;
        
        Ok(Self {
            claude_path,
            version,
            worktree: worktree.to_path_buf(),
            session_id: SessionId::new(),
        })
    }
    
    /// 인터랙티브 세션 (사용자가 직접 입력)
    pub async fn spawn_interactive(&self) -> Result<ExitCode> {
        let mut cmd = self.base_command();
        let status = cmd.status().await?;
        Ok(ExitCode::from(status.code().unwrap_or(1) as u8))
    }
    
    /// 프로그래매틱 실행 (orchestrator가 호출)
    pub async fn execute_with_watchdog(
        &self,
        prompt: &str,
        hard_timeout: Duration,
        stale_threshold: Duration,
    ) -> TaskResult {
        let mut cmd = self.base_command();
        cmd.args(["-p", prompt])
            .args(["--max-turns", "10"])
            .arg("--no-session-persistence")
            .arg("--dangerously-skip-permissions");
        
        let mut child = cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        
        // 병렬 watchdog
        let watchdog = tokio::spawn(self.run_watchdog(
            child.id().unwrap(),
            hard_timeout,
            stale_threshold,
        ));
        
        let output = child.wait_with_output().await?;
        watchdog.abort();
        
        TaskResult::from_output(&output)
    }
    
    fn base_command(&self) -> tokio::process::Command {
        let mut cmd = tokio::process::Command::new(&self.claude_path);
        cmd.current_dir(&self.worktree);
        env::inject(&mut cmd, self.session_id);
        cmd
    }
}
```

## `discovery.rs` — claude 바이너리 탐색

```rust
pub fn find_claude() -> Result<PathBuf> {
    // 1. 환경변수 override
    if let Ok(path) = std::env::var("MYTH_CLAUDE_BIN") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }
    
    // 2. which
    if let Ok(p) = which::which("claude") {
        return Ok(p);
    }
    
    // 3. 표준 위치 후보
    let candidates = [
        dirs::home_dir().unwrap().join(".claude/local/claude"),
        PathBuf::from("/usr/local/bin/claude"),
        PathBuf::from("/opt/claude/bin/claude"),
    ];
    
    for p in &candidates {
        if p.exists() {
            return Ok(p.clone());
        }
    }
    
    Err(anyhow!("claude binary not found. Install Claude Code or set MYTH_CLAUDE_BIN"))
}

pub fn detect_version(claude_path: &Path) -> Result<ClaudeVersion> {
    let output = std::process::Command::new(claude_path)
        .arg("--version")
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    // "claude 2.1.109" 같은 형식 파싱
    ClaudeVersion::parse(&stdout)
}
```

## `version.rs` — 버전 호환성

```rust
#[derive(Debug, Clone)]
pub struct ClaudeVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl ClaudeVersion {
    pub fn parse(s: &str) -> Result<Self> {
        // regex 또는 수동 파싱
    }
    
    pub fn validate_compatible(&self) -> Result<()> {
        // myth v1은 Claude Code 2.1.x 기대
        if self.major != 2 || self.minor < 1 {
            return Err(anyhow!(
                "Claude Code version {} not supported by myth v1. \
                 Expected 2.1.x or later (with PostToolUseFailure support).",
                self
            ));
        }
        
        // PostToolUseFailure 이벤트는 2.1.27+
        if self.minor == 1 && self.patch < 27 {
            tracing::warn!(
                "Claude Code 2.1.{} < 2.1.27. PostToolUseFailure may not be available. \
                 myth will fall back to PostToolUse for all cases.",
                self.patch
            );
        }
        
        Ok(())
    }
}
```

## `env.rs` — 환경변수 주입

```rust
pub fn inject(cmd: &mut Command, session_id: SessionId) {
    // myth 고유
    cmd.env("MYTH_SESSION_ID", session_id.to_string());
    cmd.env("MYTH_ACTIVE", "1");
    
    // Claude Code가 사용하는 것들 (하위 호환)
    // (CLAUDE_PROJECT_DIR 등은 Claude Code가 알아서 설정)
    
    // Timeout 조정
    cmd.env("CLAUDE_STREAM_IDLE_TIMEOUT_MS", "120000");  // 2분
    
    // Prompt caching 활성
    cmd.env("ENABLE_PROMPT_CACHING_1H", "1");
    
    // myth-embed daemon 경로 알림 (Claude Code가 알 필요는 없지만 자식이 myth 바이너리 호출 시 필요)
    // Claude Code의 자식 프로세스(hooks)가 이 환경변수 상속
}
```

## `io.rs` — stdout/stderr 처리

```rust
pub struct OutputCapture {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub last_activity: Instant,
}

impl OutputCapture {
    pub async fn capture_from_child(
        child: &mut Child,
    ) -> Result<OutputCapture> {
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        
        // 각 스트림을 별도 task로 읽기
        let stdout_task = tokio::spawn(async move {
            let mut buf = Vec::new();
            let mut reader = BufReader::new(stdout);
            reader.read_to_end(&mut buf).await?;
            Ok::<Vec<u8>, std::io::Error>(buf)
        });
        
        let stderr_task = tokio::spawn(async move {
            let mut buf = Vec::new();
            let mut reader = BufReader::new(stderr);
            reader.read_to_end(&mut buf).await?;
            Ok::<Vec<u8>, std::io::Error>(buf)
        });
        
        let (stdout, stderr) = tokio::try_join!(stdout_task, stderr_task)?;
        
        Ok(OutputCapture {
            stdout: stdout?,
            stderr: stderr?,
            last_activity: Instant::now(),
        })
    }
}
```

## `fallback.rs` — Extra Usage 경로

Max quota 소진 시 Claude Code가 "extra usage"로 전환할지 묻는다. 이 흐름을 myth가 감지·기록:

```rust
pub fn detect_quota_exhausted(stderr: &str) -> bool {
    stderr.contains("rate limit")
        || stderr.contains("quota exceeded")
        || stderr.contains("upgrade to continue")
}

pub async fn handle_quota_exhausted(runtime: &ClaudeRuntime) -> QuotaAction {
    // ccusage로 현재 상태 조회
    let usage = ccusage_query().await;
    
    tracing::warn!(
        "Claude Code quota exhausted. Current usage: {:?}",
        usage
    );
    
    // myth 정책
    let config = load_config()?;
    match config.quota_policy {
        QuotaPolicy::Wait => QuotaAction::Wait { retry_after: usage.reset_in },
        QuotaPolicy::UseExtraUsage => QuotaAction::EnableExtraUsage,
        QuotaPolicy::Abort => QuotaAction::Abort,
    }
}

async fn ccusage_query() -> Option<UsageInfo> {
    // ccusage CLI 호출
    let output = tokio::process::Command::new("ccusage")
        .args(["blocks", "--json"])
        .output().await.ok()?;
    
    if !output.status.success() {
        return None;
    }
    
    serde_json::from_slice(&output.stdout).ok()
}
```

## `TaskResult` — 실행 결과

```rust
pub struct TaskResult {
    pub succeeded: bool,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
    pub failure_reason: Option<String>,
}

impl TaskResult {
    pub fn from_output(output: &Output) -> Self {
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        
        Self {
            succeeded: output.status.success(),
            exit_code: output.status.code().unwrap_or(-1),
            stdout,
            stderr,
            duration: Duration::ZERO,  // 외부에서 설정
            failure_reason: if output.status.success() {
                None
            } else {
                Some(detect_failure_reason(&stderr))
            },
        }
    }
    
    pub fn stderr_tail(&self, max_bytes: usize) -> String {
        if self.stderr.len() <= max_bytes {
            self.stderr.clone()
        } else {
            let start = self.stderr.len() - max_bytes;
            self.stderr[start..].to_string()
        }
    }
    
    pub fn crashed(reason: String) -> Self {
        Self {
            succeeded: false,
            exit_code: -1,
            stdout: String::new(),
            stderr: String::new(),
            duration: Duration::ZERO,
            failure_reason: Some(reason),
        }
    }
}
```

## 사용 패턴

### Pattern 1 — 인터랙티브 세션 (`myth run`)

```rust
// myth-cli의 myth run 구현
pub async fn run() -> Result<ExitCode> {
    let worktree = std::env::current_dir()?;
    let runtime = ClaudeRuntime::new(&worktree)?;
    
    // 세션 시작 로깅
    tracing::info!("starting claude session: {}", runtime.session_id);
    
    let exit = runtime.spawn_interactive().await?;
    
    // 세션 종료 로깅
    tracing::info!("claude session ended: {}", runtime.session_id);
    Ok(exit)
}
```

### Pattern 2 — 병렬 Task (orchestrator가 호출)

```rust
// myth-orchestrator::executor
let runtime = ClaudeRuntime::new(&worktree_path)?;
let result = runtime.execute_with_watchdog(
    &task.prompt,
    Duration::from_secs(660),  // 11분
    Duration::from_secs(120),  // 2분 stale
).await;
```

## 테스트

```
tests/
├── discovery_test.rs          # claude 바이너리 탐색
├── version_parse_test.rs      # 버전 파싱
├── env_inject_test.rs         # 환경변수 주입 검증
├── fallback_detect_test.rs    # quota 소진 감지
└── integration/
    └── mock_claude.rs         # 가짜 claude 바이너리로 통합 테스트
```

Mock `claude`:
```bash
#!/usr/bin/env bash
# tests/fixtures/mock-claude
case "$1" in
  --version) echo "claude 2.1.109" ;;
  -p) echo "mock response" ;;
  *) ;;
esac
```

## 성능

`ClaudeRuntime::new()`는 claude 바이너리 탐색·버전 감지 때문에 **~10-50ms**. 세션 시작당 1회.

인터랙티브 세션 시작 오버헤드는 무시 수준 (Claude Code 자체가 수초 걸림).

## 관련 결정

- ARCHITECTURE §1 (Option 4+ Hybrid Wrapper): 이 crate가 핵심 구현체
- Decision 4 (Tier 3 API key): Milestone A 이후 runtime은 무관, myth_py.assessor가 직접 SDK 호출
- 기존 harness-orchestrator의 `execute.sh` 접근 방식과 정합
