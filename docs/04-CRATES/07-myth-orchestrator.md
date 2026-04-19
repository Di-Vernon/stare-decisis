# `myth-orchestrator` — 병렬 실행 & Task 라이프사이클

## 역할

여러 Claude Code 세션·작업을 **병렬로 orchestration**하는 계층. Jeffrey의 기존 `~/project/harness-orchestrator/`에서 검증된 패턴을 **Rust 래퍼 + 기존 shell 재활용** 형태로 myth에 통합.

**핵심 책임**:
1. tmux 세션 생성·관리
2. git worktree 생성·정리
3. 최대 N개(기본 3~4) 동시 실행 제한
4. 실행 상태 관찰 (`myth status`)
5. 실패·중단 감지 → caselog 기록
6. Claude Squad 연동 (선택적)

**의존**: `myth-common`, `myth-db`, `myth-runtime`.
**의존받음**: `myth-cli` (`myth run --parallel` 등).

## Cargo.toml

```toml
[package]
name = "myth-orchestrator"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[dependencies]
myth-common = { path = "../myth-common" }
myth-db = { path = "../myth-db" }
myth-runtime = { path = "../myth-runtime" }

serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true, features = ["process", "fs", "sync"] }
uuid = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }
```

## 기존 harness-orchestrator 재사용 전략

Jeffrey의 `~/project/harness-orchestrator/lib/` 아래 셸 스크립트들:

```
lib/
├── execute.sh      # 병렬 실행 엔진
├── worktree.sh     # git worktree 관리
├── watchdog.sh     # 타임아웃·스테일 감지
├── state.sh        # 상태 파일 관리
├── ui.sh           # gum 기반 UI
├── plan.sh         # plan.json 파싱
└── report.sh       # 리포트 생성
```

**재사용 원칙**:
- **Rust에서 shell subprocess 호출**. 전체 재작성 없음.
- 기존 검증된 동작 그대로 유지 (v0.3 parallel 테스트 통과).
- Rust는 "상위 조율자" 역할 — 어떤 shell 스크립트를 언제·어떤 인자로 부를지.

**복사 vs 참조**:
- 복사: `~/myth/scripts/orchestrator/*.sh`로 복사 (myth가 자체 포함)
- 업데이트 흐름: 기존 harness-orchestrator에서 개선 → myth로 재동기화 (수동)

**Day-1에는 복사**. 장기적으로는 myth가 primary가 되고 harness-orchestrator는 deprecated.

## 모듈 구조

```
crates/myth-orchestrator/
└── src/
    ├── lib.rs                # Orchestrator API
    ├── tmux.rs                # tmux 세션 관리 (shell 호출 래퍼)
    ├── worktree.rs            # git worktree 관리
    ├── executor.rs            # 실제 Task 실행 조율
    ├── watchdog.rs            # 타임아웃·스테일 감지
    ├── plan.rs                # plan.json 파싱 (wave-based)
    ├── report.rs              # 실행 후 리포트
    ├── squad.rs               # Claude Squad 연동 (선택)
    └── scripts/               # 재사용 shell 스크립트 (복사본)
        ├── execute.sh
        ├── worktree.sh
        ├── watchdog.sh
        └── ...
```

## 공개 API — `Orchestrator`

```rust
pub struct Orchestrator {
    config: OrchestratorConfig,
    state_dir: PathBuf,
}

pub struct OrchestratorConfig {
    pub max_concurrent: usize,        // 기본 3
    pub task_timeout: Duration,        // 기본 11분
    pub stale_threshold: Duration,     // 기본 2분 (출력 무활동)
    pub worktree_base: PathBuf,        // 기본 ~/.myth/worktrees/
}

impl Orchestrator {
    pub async fn execute_plan(&self, plan_path: &Path) -> Result<ExecutionReport> {
        let plan = plan::load(plan_path)?;
        
        let mut report = ExecutionReport::new();
        
        for wave in plan.waves {
            let wave_result = self.execute_wave(&wave).await?;
            report.waves.push(wave_result);
            
            if wave_result.has_critical_failure() {
                tracing::warn!("critical failure in wave, stopping");
                break;
            }
        }
        
        Ok(report)
    }
    
    async fn execute_wave(&self, wave: &Wave) -> Result<WaveResult> {
        // max_concurrent 동시 실행 제한
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent));
        
        let mut tasks = Vec::new();
        for task in &wave.tasks {
            let sem = semaphore.clone();
            let task = task.clone();
            let orch = self.clone();
            tasks.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                orch.execute_task(&task).await
            }));
        }
        
        let results: Vec<TaskResult> = futures::future::join_all(tasks).await
            .into_iter()
            .map(|r| r.unwrap_or_else(|e| TaskResult::crashed(e.to_string())))
            .collect();
        
        Ok(WaveResult { wave_id: wave.id.clone(), tasks: results })
    }
    
    async fn execute_task(&self, task: &Task) -> TaskResult {
        // 1. worktree 생성
        let worktree = self.create_worktree(&task.id).await?;
        
        // 2. tmux session 생성
        let session = self.create_tmux_session(&task.id, &worktree).await?;
        
        // 3. Claude Code 호출
        let claude = ClaudeRuntime::new(&worktree);
        let task_result = claude.execute_with_watchdog(
            &task.prompt,
            self.config.task_timeout,
            self.config.stale_threshold,
        ).await;
        
        // 4. 실패 시 caselog 기록
        if !task_result.succeeded {
            self.record_failure(&task, &task_result).await?;
        }
        
        // 5. 정리 (tmux kill, worktree 병합 또는 제거)
        self.cleanup_task(&session, &worktree, &task_result).await?;
        
        task_result
    }
}
```

## `plan.rs` — Wave-based 실행 계획

```rust
#[derive(Deserialize)]
pub struct Plan {
    pub version: u32,
    pub title: String,
    pub waves: Vec<Wave>,
}

#[derive(Deserialize)]
pub struct Wave {
    pub id: String,
    pub description: String,
    pub tasks: Vec<Task>,
    pub max_concurrent: Option<usize>,  // override
}

#[derive(Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub prompt: String,               // Claude에게 전달할 프롬프트
    pub files_affected: Vec<String>,  // 이 task가 다룰 파일 (conflict 검사용)
    pub depends_on: Vec<String>,      // 이전 wave/task id
    pub timeout_seconds: Option<u64>,
}

pub fn load(path: &Path) -> Result<Plan> {
    let content = std::fs::read_to_string(path)?;
    let plan: Plan = serde_json::from_str(&content)?;
    validate_plan(&plan)?;
    Ok(plan)
}

fn validate_plan(plan: &Plan) -> Result<()> {
    // 같은 wave 안에서 files_affected 중복 금지
    for wave in &plan.waves {
        let mut seen_files = HashSet::new();
        for task in &wave.tasks {
            for file in &task.files_affected {
                if !seen_files.insert(file.clone()) {
                    return Err(anyhow!(
                        "wave {} has conflicting file {} in multiple tasks",
                        wave.id, file
                    ));
                }
            }
        }
    }
    Ok(())
}
```

`plan.json` 예시:

```json
{
  "version": 1,
  "title": "Implement Bedrock Rule loader + tests",
  "waves": [
    {
      "id": "W1-parallel-setup",
      "description": "병렬 가능한 초기 작업",
      "tasks": [
        {
          "id": "T1.1",
          "description": "myth-common 타입 정의",
          "prompt": "...",
          "files_affected": ["crates/myth-common/src/types.rs"],
          "depends_on": []
        },
        {
          "id": "T1.2",
          "description": "YAML 로더 스켈레톤",
          "prompt": "...",
          "files_affected": ["crates/myth-gavel/src/rules/mod.rs"],
          "depends_on": []
        }
      ]
    },
    {
      "id": "W2-integration",
      "description": "통합",
      "tasks": [
        {
          "id": "T2.1",
          "prompt": "...",
          "files_affected": ["crates/myth-gavel/src/judge.rs"],
          "depends_on": ["T1.1", "T1.2"]
        }
      ]
    }
  ]
}
```

## `tmux.rs` — tmux 세션 래퍼

```rust
pub struct TmuxSession {
    pub name: String,
    pub pid: Option<u32>,
}

pub async fn create_session(name: &str, cwd: &Path) -> Result<TmuxSession> {
    let output = tokio::process::Command::new("tmux")
        .args(["new-session", "-d", "-s", name, "-c"])
        .arg(cwd)
        .output().await?;
    
    if !output.status.success() {
        return Err(anyhow!("tmux new-session failed: {}", 
            String::from_utf8_lossy(&output.stderr)));
    }
    
    let pid = get_session_pid(name).await.ok();
    Ok(TmuxSession { name: name.to_string(), pid })
}

pub async fn send_keys(session: &str, cmd: &str) -> Result<()> {
    tokio::process::Command::new("tmux")
        .args(["send-keys", "-t", session])
        .arg(cmd)
        .arg("Enter")
        .status().await?;
    Ok(())
}

pub async fn capture_pane(session: &str) -> Result<String> {
    let output = tokio::process::Command::new("tmux")
        .args(["capture-pane", "-t", session, "-p"])
        .output().await?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub async fn kill_session(session: &str) -> Result<()> {
    tokio::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status().await?;
    Ok(())
}
```

## `worktree.rs` — git worktree 관리

```rust
pub struct Worktree {
    pub path: PathBuf,
    pub branch: String,
    pub base: PathBuf,  // 원본 repo 경로
}

pub async fn create(base_repo: &Path, task_id: &str) -> Result<Worktree> {
    let worktree_path = myth_common::myth_home()
        .join("worktrees")
        .join(task_id);
    
    let branch = format!("myth/task-{}", task_id);
    
    tokio::process::Command::new("git")
        .current_dir(base_repo)
        .args(["worktree", "add", "-b", &branch])
        .arg(&worktree_path)
        .status().await?;
    
    Ok(Worktree { 
        path: worktree_path, 
        branch, 
        base: base_repo.to_path_buf() 
    })
}

pub async fn remove(wt: &Worktree) -> Result<()> {
    tokio::process::Command::new("git")
        .current_dir(&wt.base)
        .args(["worktree", "remove", "--force"])
        .arg(&wt.path)
        .status().await?;
    Ok(())
}

pub async fn merge_to_main(wt: &Worktree, main_branch: &str) -> Result<MergeResult> {
    // fast-forward 시도 → 실패 시 ort merge
    // 성공/실패 결과 반환
}
```

## `watchdog.rs` — 타임아웃·스테일 감지

```rust
pub struct Watchdog {
    pub hard_timeout: Duration,
    pub stale_threshold: Duration,
}

impl Watchdog {
    pub async fn monitor(
        &self,
        session_name: &str,
        start: Instant,
    ) -> WatchdogResult {
        let mut last_output_len = 0usize;
        let mut last_change = Instant::now();
        
        let mut tick = tokio::time::interval(Duration::from_secs(5));
        
        loop {
            tick.tick().await;
            
            // hard timeout
            if start.elapsed() >= self.hard_timeout {
                return WatchdogResult::HardTimeout;
            }
            
            // stale check
            let output = tmux::capture_pane(session_name).await
                .unwrap_or_default();
            if output.len() > last_output_len {
                last_output_len = output.len();
                last_change = Instant::now();
            } else if last_change.elapsed() >= self.stale_threshold {
                return WatchdogResult::Stale;
            }
            
            // 완료 신호 감지 (DONE|{path} 패턴 또는 특정 종료 문자열)
            if output.contains("DONE|") || is_claude_exited(&output) {
                return WatchdogResult::Completed;
            }
        }
    }
}

pub enum WatchdogResult {
    Completed,
    HardTimeout,
    Stale,
}
```

## `squad.rs` — Claude Squad 연동

Claude Squad는 Jeffrey의 수동 모니터링 도구. myth-orchestrator는 **실행 엔진**이고 Squad는 **뷰어**. 서로 독립:

- myth-orchestrator: 실제 task 병렬 실행
- Claude Squad: 사람이 각 세션을 눈으로 따라감

연동 지점:
- myth-orchestrator가 tmux 세션 이름을 `myth-task-{id}` 규약으로
- Squad가 이 이름 규약으로 인식 가능 (현재 수동 설정)

```rust
pub fn squad_session_name(task_id: &str) -> String {
    format!("myth-task-{}", task_id)
}
```

## `report.rs` — 실행 후 리포트

```rust
pub struct ExecutionReport {
    pub started: Timestamp,
    pub ended: Option<Timestamp>,
    pub waves: Vec<WaveResult>,
    pub total_tasks: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub timed_out: usize,
}

impl ExecutionReport {
    pub fn to_summary(&self) -> String {
        format!(
            "Execution: {} waves, {}/{} succeeded\n\
             Elapsed:   {}\n\
             Failed:    {}\n\
             Timed out: {}\n\
             \n\
             {}",
            self.waves.len(),
            self.succeeded, self.total_tasks,
            format_duration(self.elapsed()),
            self.failed,
            self.timed_out,
            self.waves.iter().map(|w| w.summary()).collect::<Vec<_>>().join("\n"),
        )
    }
    
    pub fn to_markdown(&self) -> String {
        // brief.md에 포함될 형태
    }
}
```

## 기본 설정 값

```rust
impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 3,                              // 과거 검증
            task_timeout: Duration::from_secs(11 * 60),     // 11분
            stale_threshold: Duration::from_secs(2 * 60),   // 2분 무활동
            worktree_base: myth_common::myth_home().join("worktrees"),
        }
    }
}
```

## 사용 예 (CLI에서 호출)

```rust
// myth-cli의 `myth run plan.json --parallel` 구현
pub async fn run_parallel(plan_path: &Path) -> Result<()> {
    let orchestrator = Orchestrator::new(OrchestratorConfig::default());
    let report = orchestrator.execute_plan(plan_path).await?;
    
    println!("{}", report.to_summary());
    
    // brief.md에 추가
    let brief_path = myth_common::brief_path();
    let brief_append = format!("\n\n## Last Execution\n\n{}", report.to_markdown());
    std::fs::OpenOptions::new()
        .append(true).open(&brief_path)?
        .write_all(brief_append.as_bytes())?;
    
    Ok(())
}
```

## 실패 전파 → caselog

실행 중 task 실패 시 **PostToolUseFailure와 별개 경로**로 caselog 기록:

```rust
async fn record_failure(&self, task: &Task, result: &TaskResult) -> Result<()> {
    let record = OrchestratorFailure {
        ts: now(),
        source: "orchestrator",
        task_id: task.id.clone(),
        reason: result.failure_reason.clone(),
        stderr_excerpt: result.stderr_tail(1000),
        duration_ms: result.duration.as_millis() as u64,
    };
    
    JsonlWriter::new(myth_common::caselog_path()).append(&record)?;
    Ok(())
}
```

Assessor가 이것도 주간 분석 대상. Orchestrator 실패도 lesson으로 축적.

## 테스트

```
tests/
├── plan_load_test.rs           # plan.json 파싱·검증
├── tmux_wrapper_test.rs        # 실제 tmux 명령 (통합)
├── worktree_lifecycle_test.rs  # git worktree 생성→merge→remove
├── watchdog_stale_test.rs
└── parallel_execution_test.rs  # 3개 task 병렬, 격리 확인
```

실제 git, tmux 필요. CI에서는 skip 가능하게 feature flag.

## 관련 결정

- Decision 5 (Phase 폐기): orchestrator도 Day-1 완전 구현
- 카테고리 8 (Split/Merge): worktree 병합이 여기 구현
- 기존 `~/project/harness-orchestrator/` 재사용 원칙
