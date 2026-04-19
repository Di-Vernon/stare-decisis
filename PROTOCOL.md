# myth-embed Wire Protocol

**Version**: 1  
**Status**: Stable (frozen for myth v1)  
**Scope**: Unix socket communication between myth-embed daemon and its clients

---

## 1. 개요

이 문서는 `myth-embed` 데몬과 클라이언트 간의 **유선 프로토콜(wire protocol)**을 공식 정의한다. 프로토콜 v1은 myth v1 전체 생애주기 동안 **변경되지 않는다**. 클라이언트와 데몬이 같은 바이너리(`myth-embed`)에서 컴파일되므로 버전 호환이 자동이지만, 외부 도구가 프로토콜을 구현할 경우 이 스펙만으로 충분해야 한다.

**주요 특성**:
- 단일 방향 request/response (polling 없음, 스트리밍 없음)
- 연결당 하나의 request
- 바이너리 프로토콜 (bincode)
- length-prefixed framing
- UTF-8 텍스트는 페이로드 내부에만

## 2. Transport

### 2.1 Socket

- **Family**: `AF_UNIX`
- **Type**: `SOCK_STREAM`
- **Path**: `$XDG_RUNTIME_DIR/myth/embed.sock`  
  (fallback: `/tmp/myth-$UID/embed.sock` if `XDG_RUNTIME_DIR` unset)
- **Permission**: 0600 (owner only)
- **Directory permission**: 0700

### 2.2 Connection lifecycle

1. 클라이언트: `connect()` → 연결 수립
2. 클라이언트: Request 전송 (§4)
3. 데몬: Response 전송 (§4)
4. 양측: `close()`

**연결 재사용 없음**. 각 request는 새 연결.

### 2.3 Timeouts

- 클라이언트 read timeout: **5초** (기본값)
- 데몬 read timeout: **1초** (클라가 request 안 보내면 끊음)
- Connection timeout: **2초** (connect 시도)

대형 텍스트(수십 KB) 임베딩 시 5초 내 완료 목표. 이를 초과하면 클라이언트가 에러 반환.

## 3. Framing

각 메시지는 **length-prefixed**:

```
┌───────────────┬─────────────────────────┐
│ length: u32   │ bincode payload (N bytes)│
│ (little-end)  │                          │
└───────────────┴─────────────────────────┘
    4 bytes         N bytes (N ≤ 1,000,000)
```

### 3.1 Length field

- **Type**: `u32`
- **Endianness**: Little-endian
- **Max**: 1,000,000 (1 MB). 초과 시 즉시 연결 종료, 에러 로그.
- **Min**: 1 (빈 페이로드 금지)

### 3.2 Payload

`bincode` v1 (serde_json 아님). Rust `bincode` crate 1.x 포맷.

```rust
// 예: Request 직렬화
let request = Request { /* ... */ };
let payload: Vec<u8> = bincode::serialize(&request)?;
let len = u32::try_from(payload.len())?;
// wire: len.to_le_bytes() + payload
```

## 4. Message types

프로토콜은 **Request/Response** 쌍으로 구성. 각 메시지는 version prefix와 request id를 포함한다.

### 4.1 Request

```rust
pub struct Request {
    pub version: u8,     // = 1
    pub id: [u8; 16],    // UUID v4 bytes
    pub op: Op,
}

pub enum Op {
    Embed { text: String },
    Ping,
    Shutdown,
}
```

### 4.2 Response

```rust
pub struct Response {
    pub version: u8,     // = 1
    pub id: [u8; 16],    // echo back from request
    pub result: OpResult,
}

pub enum OpResult {
    Embedded { 
        vector: Vec<f32>,   // length = 384
    },
    Pong { 
        uptime_secs: u64,
        request_count: u64,
        rss_bytes: u64,
        model_name: String,
    },
    ShuttingDown,
    Error { 
        code: ErrorCode,
        message: String,
    },
}

#[repr(u16)]
pub enum ErrorCode {
    Ok = 0,
    InvalidVersion = 1,
    InvalidOperation = 2,
    TextTooLong = 3,
    ModelNotLoaded = 4,
    InternalError = 5,
}
```

### 4.3 Version handshake

- 클라이언트가 `version != 1`인 Request 보내면 → 데몬이 `Error { code: InvalidVersion, message: "unsupported version: N" }` 반환 후 연결 종료
- 데몬이 `version != 1`인 Response 보내면 → 클라이언트 에러 반환
- v2 도입 시: 새 프로토콜은 별도 소켓 경로 (`embed-v2.sock`) 사용. 두 버전 병존 가능.

### 4.4 ID 필드

UUID v4. 클라이언트가 생성, 데몬이 그대로 echo. 로깅·디버깅 용도. 데몬은 `id`를 재사용·검증하지 않음.

## 5. 각 Op 상세

### 5.1 `Embed`

**요청**:
```rust
Op::Embed { text: String }
```

- `text`: UTF-8 인코딩. 길이는 문자 수가 아닌 **바이트 수 기준 1 MB 미만**.
- 앞뒤 공백은 허용 (데몬이 내부에서 trim하지 않음, 클라이언트가 알아서).
- 빈 문자열 `""` 허용 → 제로 벡터 반환 (의미 있는 경우 있음).

**응답 (성공)**:
```rust
OpResult::Embedded { vector: Vec<f32> /* len=384 */ }
```

- `vector`: 정확히 384개 f32. 
- 각 float는 IEEE 754 single precision LE.
- 벡터는 **normalized** (L2 norm ≈ 1.0). 클라이언트는 추가 정규화 불필요.
- multilingual-e5-small 모델은 자동으로 query prefix (`query: `)를 추가 — 클라이언트는 원시 텍스트만 넘기면 됨.

**응답 (실패)**:
```rust
OpResult::Error { 
    code: ErrorCode::TextTooLong | ModelNotLoaded | InternalError,
    message: "...",
}
```

### 5.2 `Ping`

**요청**:
```rust
Op::Ping
```

**응답**:
```rust
OpResult::Pong {
    uptime_secs: u64,
    request_count: u64,      // Embed 요청만 카운트 (Ping 제외)
    rss_bytes: u64,          // 현재 RSS
    model_name: String,      // "multilingual-e5-small"
}
```

`myth embed status` 명령이 내부적으로 사용.

### 5.3 `Shutdown`

**요청**:
```rust
Op::Shutdown
```

**응답 (즉시)**:
```rust
OpResult::ShuttingDown
```

데몬은 응답을 보낸 후 **100ms 대기 → exit**. 진행 중인 다른 요청은 정상 완료 후 종료.

`myth embed stop` 명령이 사용. 사용자가 만든 Request로 누구나 종료 가능 (보안 경계는 socket permission 0600에 의존).

## 6. Error handling

### 6.1 에러 카테고리

| Code | 이름 | 의미 | 클라이언트 기대 동작 |
|---|---|---|---|
| 0 | Ok | 에러 아님 (응답 필드 기본값) | - |
| 1 | InvalidVersion | Request version ≠ 1 | 업그레이드 필요 |
| 2 | InvalidOperation | 알 수 없는 Op (bincode 파싱 실패) | 버그 리포트 |
| 3 | TextTooLong | text 바이트 수 > 1MB | 텍스트 분할 후 재시도 |
| 4 | ModelNotLoaded | 모델 로드 실패 | daemon log 확인 |
| 5 | InternalError | 기타 | daemon log 확인 |

### 6.2 연결 종료 없이 에러 전달

대부분의 에러는 `OpResult::Error`로 응답한 후 **정상 연결 종료**. 연결 강제 종료는 다음 경우만:
- Request 프레임 파싱 자체 실패 (길이 헤더 이상, bincode 구조 불일치)
- 1 MB 초과 payload
- 클라이언트 timeout

### 6.3 Idempotency

모든 Op는 idempotent:
- `Embed` 같은 텍스트 여러 번 → 같은 벡터 (모델 결정론적)
- `Ping` 여러 번 → 안전
- `Shutdown` 여러 번 → 첫 호출 후 데몬 없으므로 ECONNREFUSED

클라이언트가 timeout 후 재시도해도 문제 없음.

## 7. 로그 포맷

데몬 로그: `~/.local/state/myth/embed-daemon.log` (JSON Lines).

각 엔트리:
```json
{
  "ts": "2026-04-19T14:23:45.123Z",
  "level": "info|debug|warn|error",
  "msg": "human-readable message",
  "request_id": "uuid (optional)",
  "op": "Embed|Ping|Shutdown (optional)",
  "latency_ms": 12.3,
  "text_len": 234
}
```

## 8. 구현 참조

### 8.1 Rust (공식 클라이언트)

`~/myth/rust/crates/myth-embed/src/client.rs`:

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

pub async fn embed(text: &str) -> Result<[f32; 384]> {
    let socket = myth_common::embed_socket_path();
    let mut stream = UnixStream::connect(&socket).await?;
    
    let req = Request {
        version: 1,
        id: Uuid::new_v4().into_bytes(),
        op: Op::Embed { text: text.to_string() },
    };
    
    let payload = bincode::serialize(&req)?;
    stream.write_u32_le(payload.len() as u32).await?;
    stream.write_all(&payload).await?;
    stream.flush().await?;
    
    let len = stream.read_u32_le().await? as usize;
    if len > 1_000_000 {
        return Err(anyhow!("response too large"));
    }
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    
    let resp: Response = bincode::deserialize(&buf)?;
    match resp.result {
        OpResult::Embedded { vector } => {
            vector.try_into().map_err(|_| anyhow!("wrong dim"))
        }
        OpResult::Error { code, message } => {
            Err(anyhow!("embed error {}: {}", code as u16, message))
        }
        _ => Err(anyhow!("unexpected response")),
    }
}
```

### 8.2 Python

별도 bincode 라이브러리가 필요. 간단한 대안은 `myth-embed probe <text>` subprocess 호출 → 출력 파싱.

```python
import socket
import struct

def embed_via_subprocess(text: str) -> list[float]:
    """Fallback path for Python — slower but simpler."""
    import subprocess
    result = subprocess.run(
        ["myth-embed", "probe", text],
        capture_output=True, text=True, check=True,
    )
    # probe 출력 포맷 파싱 (개별적인 debug 지향 format)
    # 정확한 벡터 복원은 어려우므로, Python은 대부분 주기적 Observer 작업에서만 사용
    ...
```

Python이 정확한 벡터를 필요로 하는 경우는 **Milestone B 이후 Observer의 벡터 재계산** 정도. 그 시점엔 Python용 bincode bridge 구현.

### 8.3 CLI tool (디버깅)

```bash
# hex dump로 프로토콜 확인
myth-embed probe "hello world" --verbose
# Request sent:  [bincode hex dump]
# Response received: [bincode hex dump]
# Vector (first 5): [0.023, -0.041, ...]
```

## 9. 프로토콜 v2 (미래)

v1은 myth v1의 모든 요구에 충분. 만약 v2가 필요해지는 시나리오:

- **배치 요청**: `BatchEmbed { texts: Vec<String> }` — N회 왕복 → 1회
- **스트리밍**: 긴 텍스트를 스트림 기반 임베딩
- **다른 모델**: 모델 선택 필드 추가 (코드 임베딩 대응, Milestone D 관련)

v2 도입 시 원칙:
- 별도 소켓 경로 (v1 병존)
- 버전 필드 필수 유지 (v1 클라가 v2 데몬에 연결 → InvalidVersion 명확)
- v1 deprecate 전 최소 6개월 병존

## 10. 보안 고려사항

### 10.1 접근 통제

- 소켓 파일 권한 0600 → 같은 사용자만 접근
- 디렉토리 권한 0700 → 외부 프로세스 발견 자체 차단
- `XDG_RUNTIME_DIR`는 systemd-user가 tmpfs로 제공 → 재부팅 시 자동 정리

### 10.2 프로토콜 레벨 공격 표면

- **DoS**: 1 MB 제한으로 메모리 고갈 방지. concurrent request 수 제한 없음 (Tokio 동시성에 의존).
- **Injection**: `Embed.text`는 그냥 모델 입력으로 전달. Shell interpolation 없음.
- **Info leak**: Ping 응답의 `rss_bytes`, `uptime_secs`는 같은 사용자에게만 노출 (소켓 권한).

### 10.3 신뢰 경계

myth-embed 데몬은 **사용자의 다른 myth 프로세스와 같은 신뢰 수준**. 외부 네트워크 노출 없음. 프로토콜 검증 부담은 낮음.

## 11. 변경 이력

| 날짜 | 버전 | 변경 |
|---|---|---|
| 2026-04-19 | 1.0 | 초기 스펙. bincode v1, length prefix, 3개 Op. |

---

**이 문서는 계약서다.** 미래에 수정된다면:
1. 버전 증가 (v2, v3...)
2. 기존 v1 클라이언트 동작 보존
3. 변경 이력에 기록

v1 자체의 구조는 **절대 변경되지 않는다**.
