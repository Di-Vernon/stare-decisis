# `myth-embed` — 임베딩 데몬

## 역할

multilingual-e5-small 임베딩 모델을 **메모리에 상주**시키는 self-daemonizing 단일 바이너리. myth 전체에서 임베딩이 필요할 때 Unix socket으로 요청을 받는다.

**핵심 특성**:
- **단일 바이너리**: `myth-embed`. 클라이언트/데몬 모드가 같은 바이너리.
- **자동 관리**: ECONNREFUSED 감지 시 자동 spawn. 15분 유휴 시 자가 종료.
- **bincode 프로토콜**: 효율·정밀도 보존.
- **독립 crate**: 다른 crate와 IPC로만 소통. 약한 결합.

**의존**: `myth-common`만.
**의존받음**: 없음 (Unix socket 경유).

## Cargo.toml

```toml
[package]
name = "myth-embed"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[dependencies]
myth-common = { path = "../myth-common" }

serde = { workspace = true }
bincode = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
tokio = { workspace = true }
mimalloc = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }

# 임베딩
fastembed = { workspace = true }
# ort: not a direct dependency — fastembed brings it in transitively.
# See the "v0.1 Task 2.2 사전 API 확인" change box below.

# Unix socket
nix = { version = "0.28", features = ["fs"] }

[[bin]]
name = "myth-embed"
path = "src/main.rs"
```

> **v0.1 Task 2.2 사전 API 확인 중 발견** (Jeffrey 승인 2026-04-19)
>
> 초안은 `ort = "2.0-rc"`를 workspace 및 crate 의존성으로 명시했으나,
> Task 2.2 사전 조사에서 세 가지가 확인되었다:
>
> 1. **공개 API 사용 경로 0개** — 이 문서 전체에서 `ort::`, `use ort`,
>    `Session`, `Environment`, `ExecutionProvider` 직접 사용이 없다.
>    모든 임베딩 로직은 `fastembed::TextEmbedding` 레벨에서 종결.
> 2. **`"2.0-rc"`는 cargo semver syntax 오류** — `expected comma after
>    minor version number, found '-'`. 즉 resolve 이전 parse 단계에서
>    실패한다 (Wave 2.2 첫 줄에서 터졌을 문제; 이전까지 `ort`를 실제로
>    사용한 crate가 없어 cargo lazy-parse로 숨어 있었음).
> 3. **fastembed 5.13.2가 전이 의존으로 호환 `ort`를 자동 선택** — 우리가
>    workspace에서 직접 pin할 이유 없음.
>
> 결론: workspace·crate 양쪽에서 `ort` 직접 의존을 제거하고 fastembed가
> 가져오는 버전에 맡긴다. pre-release semver 지옥을 Day-1 단계에서 영구
> 제거. `ort` 공개 API를 직접 써야 할 경로가 미래에 등장하면 (예: custom
> ExecutionProvider) 그 시점에 exact-match 버전(`=2.0.0-rc.NN`)으로
> 다시 도입한다.

> **v0.1 Task 2.2 사전 API 확인 — fastembed rustls 전환** (Jeffrey 승인 2026-04-19)
>
> fastembed 5.13.2의 **기본 features**는 `hf-hub-native-tls`,
> `ort-download-binaries-native-tls`, `image-models` 세 가지다. 앞 두
> feature는 `openssl-sys`를 요구 → `libssl-dev` 시스템 패키지 필수 →
> `WSL2-SETUP.md` prerequisite 목록 증가 + `install.sh`가 `sudo apt`
> 경로 의존. myth의 "self-contained Rust build" 철학과 충돌.
>
> 해결: `default-features = false` + rustls 계열 features 3개만 명시:
>
> ```toml
> fastembed = { version = "5", default-features = false, features = [
>     "hf-hub",
>     "hf-hub-rustls-tls",
>     "ort-download-binaries-rustls-tls",
> ] }
> ```
>
> 이 조합의 효과:
> - 시스템 openssl 의존성 제거 → self-contained Rust 빌드
> - `WSL2-SETUP.md` prerequisite 목록 불변
> - Supply-chain audit 범위가 Rust crate만으로 축소
> - `install.sh`가 `sudo` 없이 동작
>
> `image-models` 제외 근거: `multilingual-e5-small`은 text-only 모델이라
> `image` crate 의존성이 불필요. 컴파일 시간·바이너리 크기·audit surface
> 전부 감소.
>
> 버전 범위 `"5"`는 유지. 실제 설치 버전은 `Cargo.lock`이 5.13.2로 고정.
> 업그레이드는 `cargo update -p fastembed`로 의도적으로만 수행.
>
> **⚠️ `ort-download-binaries` feature 함정 주의**
>
> fastembed 5.13.2의 `ort-download-binaries` feature는 이름이 중립적이지만
> 내부 정의가 `["ort-download-binaries-native-tls"]`로 **native-tls를
> 강제 활성화**한다. rustls 빌드를 원하면서 features에
> `ort-download-binaries`를 추가하면 `libssl-dev`가 다시 요구되어
> 의도와 반대 효과가 난다.
>
> 올바른 방식: **`ort-download-binaries-rustls-tls`만 사용**.
> 이 feature가 내부적으로 `ort/download-binaries` + `ort/tls-rustls`를
> 모두 활성화하므로 중간 단계 `ort-download-binaries` 추가는 불필요하고
> 해롭다. 위 3-feature 조합은 이 함정을 회피한 결과다.

> **v0.1 Task 2.2 사전 API 확인 — fastembed InitOptions builder 패턴** (Jeffrey 승인 2026-04-19)
>
> 초안의 `daemon/model.rs` 예시는 struct literal 형태였으나,
> fastembed 5.13.2의 실제 API는 builder 패턴이다:
>
> ```rust
> // 초안 (컴파일 안 됨):
> InitOptions {
>     model_name: EmbeddingModel::MultilingualE5Small,
>     cache_dir: ...,
>     ..Default::default()
> }
>
> // 실제 API:
> InitOptions::new(EmbeddingModel::MultilingualE5Small)
>     .with_cache_dir(...)
>     .with_show_download_progress(true)
> ```
>
> 또한 `InitOptions`는 `TextInitOptions`의 type alias로,
> fastembed가 text/image 임베딩 백엔드를 분리한 결과다. import 경로
> (`use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};`)는 그대로
> 유효하므로 호출 측 코드 외에 별도 조정은 없다.
>
> 아래 `daemon/model.rs` 코드 예시는 실제 API에 맞게 갱신된 상태다.

## 모듈 구조

```
crates/myth-embed/
└── src/
    ├── main.rs              # 클라이언트/데몬 모드 분기
    ├── protocol/
    │   ├── mod.rs
    │   ├── wire.rs          # bincode 직렬화, length prefix framing
    │   └── types.rs         # Request, Response, Op, OpResult
    ├── client.rs            # 클라이언트 모드 로직 (자동 spawn 포함)
    ├── daemon/
    │   ├── mod.rs          # 데몬 모드 entry
    │   ├── server.rs       # tokio UnixListener + 요청 처리
    │   ├── model.rs        # fastembed-rs 상주
    │   ├── idle.rs         # 15분 유휴 타이머
    │   └── stats.rs        # 요청 수, 레이턴시, RSS
    ├── spawn.rs             # self-fork-exec --daemon
    ├── lock.rs              # flock (동시 spawn race 방지)
    └── cli.rs               # status/stop/probe 서브커맨드
```

## 공개 프로토콜 (wire v1)

**bincode + length prefix 프레이밍**.

```rust
// src/protocol/types.rs

#[derive(Serialize, Deserialize)]
pub struct Request {
    pub version: u8,        // 1
    pub id: Uuid,
    pub op: Op,
}

#[derive(Serialize, Deserialize)]
pub enum Op {
    Embed { text: String },
    Ping,
    Shutdown,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub version: u8,        // 1
    pub id: Uuid,
    pub result: OpResult,
}

#[derive(Serialize, Deserialize)]
pub enum OpResult {
    Embedded { vector: Vec<f32> },  // 384 floats
    Pong { 
        uptime_secs: u64, 
        request_count: u64, 
        rss_bytes: u64 
    },
    ShuttingDown,
    Error { message: String },
}
```

### Framing

```
[u32 LE length] [bincode payload]
```

`length`는 payload 바이트 수. 최대 1MB로 제한 (임베딩 요청 크기 감안).

```rust
// src/protocol/wire.rs

pub async fn write_message<W: AsyncWrite + Unpin, T: Serialize>(
    w: &mut W, msg: &T
) -> Result<()> {
    let payload = bincode::serialize(msg)?;
    let len = u32::try_from(payload.len())?;
    w.write_u32_le(len).await?;
    w.write_all(&payload).await?;
    w.flush().await?;
    Ok(())
}

pub async fn read_message<R: AsyncRead + Unpin, T: DeserializeOwned>(
    r: &mut R
) -> Result<T> {
    let len = r.read_u32_le().await? as usize;
    if len > 1_000_000 {
        return Err(anyhow!("payload too large: {}", len));
    }
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf).await?;
    Ok(bincode::deserialize(&buf)?)
}
```

## `main()` 분기

```rust
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    myth_common::logging::init_logging("myth-embed");
    
    let args: Vec<String> = std::env::args().collect();
    
    match args.get(1).map(|s| s.as_str()) {
        Some("--daemon") => daemon::run().await,
        Some("status") => cli::status().await,
        Some("stop") => cli::stop().await,
        Some("probe") => cli::probe(&args[2..]).await,
        _ => client::run(&args[1..]).await,
    }
}
```

기본 호출 (인자 없거나 임의 인자): 클라이언트 모드.

## 클라이언트 모드

```rust
// src/client.rs

pub async fn run(args: &[String]) -> ExitCode {
    // stdin에서 bincode Request 읽음 (다른 myth 바이너리가 호출)
    let mut stdin = tokio::io::stdin();
    let request: Request = read_message(&mut stdin).await.unwrap();
    
    let response = query_daemon(request).await;
    
    let mut stdout = tokio::io::stdout();
    write_message(&mut stdout, &response).await.unwrap();
    
    ExitCode::SUCCESS
}

async fn query_daemon(req: Request) -> Response {
    let socket_path = myth_common::embed_socket_path();
    
    // 1차 시도
    match try_connect(&socket_path, &req).await {
        Ok(resp) => return resp,
        Err(e) => {
            tracing::debug!("first connect failed: {}", e);
        }
    }
    
    // ECONNREFUSED → self-spawn 시도
    if should_skip_autospawn() {
        return Response::error(&req.id, "daemon unavailable and --no-embed-daemon set");
    }
    
    match spawn::spawn_daemon().await {
        Ok(()) => {
            // 최대 30초 대기하며 재시도 (cold boot: 모델 다운로드 + ONNX
            // 로드에 실측 ~18.5초 걸렸음. hot path는 try_connect가
            // 첫 시도에 성공하므로 이 retry loop에 들어가지 않음.)
            for _ in 0..300 {
                tokio::time::sleep(Duration::from_millis(100)).await;
                if let Ok(resp) = try_connect(&socket_path, &req).await {
                    return resp;
                }
            }
            Response::error(&req.id, "daemon spawn timeout")
        }
        Err(e) => Response::error(&req.id, &format!("spawn failed: {}", e)),
    }
}

async fn try_connect(socket_path: &Path, req: &Request) -> Result<Response> {
    let mut stream = UnixStream::connect(socket_path).await?;
    write_message(&mut stream, req).await?;
    let resp: Response = read_message(&mut stream).await?;
    Ok(resp)
}

fn should_skip_autospawn() -> bool {
    std::env::var("MYTH_NO_EMBED_DAEMON").is_ok() 
        || std::env::args().any(|a| a == "--no-embed-daemon")
}
```

### 고수준 클라이언트 API (library)

다른 crate가 쓰는 간편 API:

```rust
// src/client.rs (pub API)

pub struct EmbedClient {
    socket_path: PathBuf,
}

impl EmbedClient {
    pub fn new() -> Self {
        Self { socket_path: myth_common::embed_socket_path() }
    }
    
    pub fn embed(&self, text: &str) -> Result<[f32; 384]> {
        // sync wrapper (다른 crate는 대부분 sync)
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        
        rt.block_on(async {
            let req = Request {
                version: 1,
                id: Uuid::new_v4(),
                op: Op::Embed { text: text.to_string() },
            };
            let resp = query_daemon(req).await;
            match resp.result {
                OpResult::Embedded { vector } => {
                    vector.try_into().map_err(|_| anyhow!("vector wrong dim"))
                }
                OpResult::Error { message } => Err(anyhow!("embed error: {}", message)),
                _ => Err(anyhow!("unexpected response")),
            }
        })
    }
}
```

`myth-identity`의 Tier 2 matcher가 사용.

## 데몬 모드 (`--daemon`)

```rust
// src/daemon/mod.rs

pub async fn run() -> ExitCode {
    // 1. flock 획득
    let lock = lock::acquire().await.expect("another daemon instance running");
    
    let socket_path = myth_common::embed_socket_path();
    
    // 2. 스테일 소켓 unlink
    let _ = std::fs::remove_file(&socket_path);
    
    // 3. 부모 디렉토리 생성 (0700)
    std::fs::create_dir_all(socket_path.parent().unwrap())?;
    
    // 4. 모델 로드 (500ms~2s)
    tracing::info!("loading multilingual-e5-small");
    let model = model::load().await?;
    tracing::info!("model loaded, listening on {:?}", socket_path);
    
    // 5. Unix socket bind
    let listener = UnixListener::bind(&socket_path)?;
    // mode 0600 설정
    std::fs::set_permissions(&socket_path, Permissions::from_mode(0o600))?;
    
    // 6. 상태·타이머 초기화
    let stats = Arc::new(Stats::new());
    let idle = Arc::new(IdleTracker::new(Duration::from_secs(15 * 60)));
    
    // 7. 이벤트 루프
    loop {
        tokio::select! {
            Ok((stream, _)) = listener.accept() => {
                let model = model.clone();
                let stats = stats.clone();
                let idle = idle.clone();
                tokio::spawn(async move {
                    handle_client(stream, model, stats, idle).await
                });
            }
            _ = idle.wait_for_timeout() => {
                tracing::info!("idle shutdown");
                break;
            }
        }
    }
    
    // 8. graceful shutdown
    drop(listener);
    let _ = std::fs::remove_file(&socket_path);
    drop(lock);
    
    ExitCode::SUCCESS
}

async fn handle_client(
    mut stream: UnixStream,
    model: Arc<Model>,
    stats: Arc<Stats>,
    idle: Arc<IdleTracker>,
) -> Result<()> {
    let req: Request = read_message(&mut stream).await?;
    idle.bump();
    stats.inc_request();
    
    let start = Instant::now();
    let result = match req.op {
        Op::Embed { text } => {
            match model.embed(&text).await {
                Ok(vector) => OpResult::Embedded { vector: vector.to_vec() },
                Err(e) => OpResult::Error { message: e.to_string() },
            }
        }
        Op::Ping => OpResult::Pong {
            uptime_secs: stats.uptime_secs(),
            request_count: stats.request_count(),
            rss_bytes: get_rss_bytes(),
        },
        Op::Shutdown => {
            tokio::spawn(async {
                tokio::time::sleep(Duration::from_millis(100)).await;
                std::process::exit(0);
            });
            OpResult::ShuttingDown
        }
    };
    
    let response = Response { version: 1, id: req.id, result };
    write_message(&mut stream, &response).await?;
    
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    tracing::debug!("request handled in {:.2}ms", elapsed_ms);
    Ok(())
}
```

### `daemon/model.rs`

```rust
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

pub struct Model {
    inner: Arc<TextEmbedding>,
}

impl Model {
    pub async fn load() -> Result<Arc<Self>> {
        // blocking task로 로드 (fastembed-rs는 sync).
        // NOTE: fastembed 5.13.2에서 `InitOptions`는 `TextInitOptions`의
        //       type alias이며 **builder 패턴**을 사용한다. 초안의
        //       `InitOptions { model_name, cache_dir, ..Default::default() }`
        //       struct literal 형태는 컴파일되지 않는다.
        let model = tokio::task::spawn_blocking(|| {
            TextEmbedding::try_new(
                InitOptions::new(EmbeddingModel::MultilingualE5Small)
                    .with_cache_dir(myth_common::myth_home().join("embeddings/models"))
                    .with_show_download_progress(true),
            )
        }).await??;

        Ok(Arc::new(Self { inner: Arc::new(model) }))
    }
    
    pub async fn embed(&self, text: &str) -> Result<[f32; 384]> {
        let text = text.to_string();
        let model = self.inner.clone();
        
        tokio::task::spawn_blocking(move || {
            let docs = vec![&text[..]];
            let embeddings = model.embed(docs, None)?;
            let vec: Vec<f32> = embeddings.into_iter().next().unwrap();
            vec.try_into().map_err(|_| anyhow!("wrong dim"))
        }).await?
    }
}
```

### `daemon/idle.rs`

```rust
pub struct IdleTracker {
    last_activity: Mutex<Instant>,
    timeout: Duration,
    notify: Notify,
}

impl IdleTracker {
    pub fn new(timeout: Duration) -> Self {
        Self {
            last_activity: Mutex::new(Instant::now()),
            timeout,
            notify: Notify::new(),
        }
    }
    
    pub fn bump(&self) {
        *self.last_activity.lock().unwrap() = Instant::now();
        self.notify.notify_waiters();
    }
    
    pub async fn wait_for_timeout(&self) {
        loop {
            let last = *self.last_activity.lock().unwrap();
            let elapsed = last.elapsed();
            
            if elapsed >= self.timeout {
                return;
            }
            
            let remaining = self.timeout - elapsed;
            tokio::select! {
                _ = tokio::time::sleep(remaining) => {}
                _ = self.notify.notified() => {}  // bump되면 다시 체크
            }
        }
    }
}
```

## `spawn.rs` — self-fork-exec

```rust
pub async fn spawn_daemon() -> Result<()> {
    let exe = std::env::current_exe()?;
    
    // flock으로 경쟁 확인
    let _ = lock::try_acquire()?;  // 실패 시 이미 다른 프로세스가 spawn 중
    
    let mut cmd = tokio::process::Command::new(&exe);
    cmd.arg("--daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(std::fs::File::create(
            myth_common::myth_state().join("embed-daemon.log")
        )?);
    
    // daemon은 부모에게서 detach
    unsafe {
        cmd.pre_exec(|| {
            nix::unistd::setsid().map_err(|e| std::io::Error::from(e))?;
            Ok(())
        });
    }
    
    cmd.spawn()?;
    // 부모는 즉시 반환. daemon은 독립 실행.
    Ok(())
}
```

## `cli.rs` — status/stop/probe

```rust
pub async fn status() -> ExitCode {
    let client = EmbedClient::new();
    let req = Request { version: 1, id: Uuid::new_v4(), op: Op::Ping };
    
    match client.query(req).await {
        Ok(Response { result: OpResult::Pong { uptime_secs, request_count, rss_bytes }, .. }) => {
            println!("myth-embed daemon");
            println!("  Socket:    {:?}", myth_common::embed_socket_path());
            println!("  Uptime:    {}s", uptime_secs);
            println!("  Requests:  {}", request_count);
            println!("  RSS:       {:.1} MB", rss_bytes as f64 / 1024.0 / 1024.0);
            ExitCode::SUCCESS
        }
        _ => {
            println!("myth-embed daemon is not running");
            ExitCode::from(1)
        }
    }
}

pub async fn stop() -> ExitCode {
    let client = EmbedClient::new();
    let req = Request { version: 1, id: Uuid::new_v4(), op: Op::Shutdown };
    match client.query(req).await {
        Ok(_) => {
            println!("myth-embed daemon stopping");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {}", e);
            ExitCode::from(1)
        }
    }
}

pub async fn probe(args: &[String]) -> ExitCode {
    let text = args.join(" ");
    if text.is_empty() {
        eprintln!("usage: myth-embed probe <text>");
        return ExitCode::from(2);
    }
    
    let client = EmbedClient::new();
    match client.embed(&text) {
        Ok(vector) => {
            println!("Text: {}", text);
            println!("Vector (first 5): {:?}", &vector[..5]);
            println!("Norm: {:.4}", vector_norm(&vector));
            println!("Dim: {}", vector.len());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {}", e);
            ExitCode::from(1)
        }
    }
}
```

## 성능·관찰성

**예산**:
- 첫 `Embed` 요청 (콜드 spawn 포함): 500~2000ms
- 이후 요청 (hot): 8~15ms (multilingual-e5-small 추론)
- `Ping`: <1ms
- Unix socket round-trip 오버헤드: ~100μs (bincode) + ~100μs (프로세스 간)

**로그** — `~/.local/state/myth/embed-daemon.log` (JSON Lines):

```
{"ts":"2026-04-19T14:23:45Z","level":"info","msg":"loading multilingual-e5-small"}
{"ts":"2026-04-19T14:23:47Z","level":"info","msg":"model loaded","took_ms":1823}
{"ts":"2026-04-19T14:23:48Z","level":"debug","msg":"request","op":"Embed","text_len":234,"latency_ms":12}
```

## 테스트

```
tests/
├── protocol_roundtrip.rs   # bincode 직렬화 왕복
├── daemon_lifecycle.rs     # spawn → serve → idle shutdown
├── concurrent_clients.rs   # N개 클라 동시 요청
├── spawn_race.rs           # 동시 spawn 경쟁 → flock 정상 작동
└── probe_cli.rs            # probe 명령 출력 형식
```

## 관련 결정

- Decision 2: multilingual-e5-small 모델
- Decision 6: self-daemonizing, bincode 프로토콜
- Decision 7: The Gavel은 별도 daemon (Milestone C), myth-embed와 독립
- ARCHITECTURE §5: embed daemon 아키텍처 상세

## 관련 문서

- `~/myth/PROTOCOL.md`: wire protocol 공식 스펙 (이 문서보다 상세)
