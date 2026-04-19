# myth — WSL2 환경 체크리스트

myth는 **WSL2 Ubuntu 24.04** 위에서 개발·운영된다. Research #2가 권고하는 Rust cold-start 최적화와 시스템 구성을 정리한다. 이 문서의 모든 항목은 **`myth doctor --wsl-check`**로 자동 검증된다.

## 0. 요약 — 꼭 해야 하는 것

Jeffrey가 myth 설치 전 확인해야 하는 최소 항목:

1. **WSL2** (WSL1 아님)
2. **Ubuntu 24.04** (22.04도 동작하나 24.04 권장)
3. **`.wslconfig`**에 cpu/memory 충분 배정
4. **mold linker** 설치
5. **systemd-user** 활성 (embed daemon 소켓 경로용)
6. **~/.local/bin이 PATH에 포함**

이 6개만 돼 있으면 myth는 작동한다. 아래는 성능·관측성 최적화.

## 1. WSL 버전

### 1.1 확인

```bash
wsl.exe --list --verbose
# 또는 WSL 안에서:
uname -a  # "WSL2" 문자열이 있어야 함
```

**WSL1**은 지원 안 함. 파일 시스템 성능 차이가 크고 systemd가 없다.

### 1.2 Ubuntu 버전

```bash
lsb_release -a
# Ubuntu 24.04 (권장)
# Ubuntu 22.04 (작동, 단 SQLite 3.45+가 없으면 빌드 옵션 조정 필요)
```

## 2. `.wslconfig`

Windows 쪽 `C:\Users\<Lime>\.wslconfig`:

```ini
[wsl2]
memory=16GB
processors=8
swap=4GB
localhostForwarding=true

# 성능 관련
kernelCommandLine=cgroup_enable=memory swapaccount=1
guiApplications=false
```

**최소 권장**:
- memory: 8GB (임베딩 모델 로드 150MB + 컴파일 여유)
- processors: 4 이상 (병렬 Cargo 빌드)

### 2.1 적용

```powershell
# Windows PowerShell
wsl --shutdown
# 30초 대기 후 WSL 재진입
```

## 3. 필수 패키지

```bash
# build essentials
sudo apt install -y build-essential pkg-config

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# mold linker (Rust 빌드 가속)
sudo apt install -y mold

# clang (mold와 함께 사용)
sudo apt install -y clang

# Python 3.11+
sudo apt install -y python3 python3-pip python3-venv

# SQLite (bundled이지만 로컬 도구용)
sudo apt install -y sqlite3

# tmux (병렬 실행)
sudo apt install -y tmux

# 기타 유틸
sudo apt install -y jq yq git curl

# 개발 편의
sudo apt install -y gum  # charmbracelet CLI UI
```

### 3.1 Node.js (Claude Code)

Claude Code는 Node.js 18+ 필요.

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
source ~/.bashrc
nvm install --lts
nvm use --lts
```

Claude Code 설치:
```bash
npm install -g @anthropic-ai/claude-code
claude --version  # 2.1.27 이상 권장 (PostToolUseFailure 지원)
```

## 4. systemd-user

myth-embed 데몬은 `$XDG_RUNTIME_DIR` (tmpfs)을 사용한다. 이는 systemd-user가 세션마다 자동 생성한다.

### 4.1 확인

```bash
systemctl --user status
# "Running" 상태여야 함

echo $XDG_RUNTIME_DIR
# /run/user/1000 같은 경로
# 빈 문자열이면 문제
```

### 4.2 없을 때

```bash
# WSL에서 systemd 활성
# /etc/wsl.conf:
[boot]
systemd=true
```

```bash
# 적용: WSL 재시작
wsl --shutdown
# 다시 진입
```

### 4.3 fallback

myth는 `XDG_RUNTIME_DIR`가 없으면 `/tmp/myth-$UID/`로 fallback. 작동은 하지만 재부팅 후 정리 의존이 달라짐.

## 5. Rust 빌드 최적화

### 5.1 `.cargo/config.toml`

`~/myth/rust/.cargo/config.toml` (이 파일은 `myth install` 시 생성):

```toml
[build]
rustflags = ["-C", "target-cpu=x86-64-v3"]

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = [
    "-C", "link-arg=-fuse-ld=mold",
    "-C", "target-cpu=x86-64-v3",
]
```

### 5.2 `Cargo.toml` 프로파일 (workspace)

`~/myth/rust/Cargo.toml`:

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = "symbols"
panic = "abort"
debug = 0
overflow-checks = false
incremental = false
```

빌드 시간 ~5~10분 (CPU 8코어 기준). `cargo build --release`.

### 5.3 `target-cpu=x86-64-v3`

x86-64-v3은 **AVX2 포함**. 대부분의 현대 CPU 지원. Jeffrey 환경(2020년 이후 Intel/AMD)은 거의 확실히 지원.

확인:
```bash
lscpu | grep -E "avx2|bmi2"
# avx2와 bmi2 모두 있으면 v3 OK
```

없으면 `x86-64-v2` 또는 `native`로 조정.

## 6. CPU 거버너 (선택)

WSL2는 Windows 전원 관리에 영향받는다. 배터리 모드에서 CPU throttle 발생 가능.

### 6.1 확인

```bash
cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor
# WSL2에서는 "schedutil" 또는 아예 디렉토리 없음
```

WSL2는 거버너 직접 제어가 제한적. Windows 쪽에서 **Balanced** 또는 **High Performance** 전원 계획 선택 권장.

## 7. 파일 시스템

### 7.1 WSL2 파일 시스템 주의

```bash
# OK: 네이티브 WSL 파일 시스템
~/myth/                     # /home/miirr/myth/ (ext4)

# 피하기: Windows 파일 시스템 마운트
/mnt/c/...                  # 10-100배 느림
```

**myth는 반드시 WSL 네이티브 fs에** 위치. `~/myth/`가 기본.

### 7.2 파일 권한

```bash
# ~/myth/ 는 일반 0755 (Git 저장소)
# ~/.config/myth/api_key 는 0600 (umask 0077 권장)

umask 0077  # 민감 파일 생성 기본값 안전화
```

bashrc에 추가:
```bash
# ~/.bashrc
umask 0077
```

### 7.3 inotify 제한

SQLite WAL, JSONL watch는 inotify 의존. WSL2 기본 제한이 낮음.

```bash
# 확인
sysctl fs.inotify.max_user_watches
# 8192 이하면 증가 권장

# /etc/sysctl.conf 추가
fs.inotify.max_user_watches = 524288
fs.inotify.max_user_instances = 1024

# 적용
sudo sysctl -p
```

## 8. 네트워크 (모델 다운로드)

myth-embed 첫 실행 시 HuggingFace에서 multilingual-e5-small ONNX (~116MB) 다운로드.

### 8.1 WSL2 DNS

```bash
# /etc/resolv.conf 가 있고 내용이 있으면 OK
cat /etc/resolv.conf
# nameserver ... 가 있어야 함
```

없으면 `/etc/wsl.conf`:
```ini
[network]
generateResolvConf = true
```

### 8.2 프록시

Jeffrey 환경이 기업 프록시 뒤라면 `~/.profile`:
```bash
export HTTP_PROXY=...
export HTTPS_PROXY=...
export NO_PROXY=localhost,127.0.0.1
```

## 9. PATH 설정

`~/.local/bin`을 PATH에 포함:

```bash
# ~/.bashrc
if [ -d "$HOME/.local/bin" ] ; then
    PATH="$HOME/.local/bin:$PATH"
fi
```

확인:
```bash
echo $PATH | tr ':' '\n' | grep '.local/bin'
```

## 10. 성능 검증

### 10.1 Cold start hook latency

```bash
# mold 적용 확인
readelf -p .comment ~/.local/bin/myth-hook-pre-tool | grep mold
# "mold ..." 문자열 있으면 OK

# latency 측정
hyperfine --warmup 3 'echo {} | myth-hook-pre-tool'
# P99 < 10ms 목표 (Milestone C 기준 15ms 이하)
```

### 10.2 목표 수치

| 항목 | 목표 | 측정 |
|---|---|---|
| `myth-hook-pre-tool` P50 | < 5ms | `myth doctor --perf-check` |
| `myth-hook-pre-tool` P99 | < 10ms | 동 |
| `myth-embed` cold spawn | < 2000ms | 첫 `myth embed probe "hello"` |
| `myth-embed` hot embed | < 20ms | 두 번째 probe |
| workspace `cargo build --release` | < 15min | `time cargo build --release` |

## 11. `myth doctor --wsl-check` 예상 출력

```
myth doctor --wsl-check

  ✓ WSL version: WSL2
  ✓ Ubuntu: 24.04
  ✓ .wslconfig memory: 16GB (>= 8GB required)
  ✓ .wslconfig processors: 8
  ✓ systemd-user: running
  ✓ XDG_RUNTIME_DIR: /run/user/1000
  ✓ mold linker: installed
  ✓ Rust: 1.82.0 (stable)
  ✓ Node.js: v20.10.0
  ✓ Claude Code: 2.1.109
  ✓ file system: ~/myth on ext4 (native WSL)
  ✓ umask: 0077
  ✓ ~/.local/bin in PATH
  ✓ inotify.max_user_watches: 524288
  
  WSL2 environment: green
```

## 12. 장애 조치

### 12.1 `myth-embed` spawn 실패

```
error: myth-embed cannot connect to $XDG_RUNTIME_DIR
```

- systemd-user 실행 중인지 확인
- `/etc/wsl.conf`에 `systemd=true` 있는지
- WSL 재시작

### 12.2 Hook P99 > 15ms

1. `cat /proc/cpuinfo | grep MHz` — CPU throttling 의심
2. Windows 전원 계획: High Performance
3. `.wslconfig` processors 증가
4. `mold` 설치 확인 (`readelf` 검증)

### 12.3 SQLite "database is locked"

- WAL 모드 확인: `sqlite3 ~/.myth/state.db "PRAGMA journal_mode;"` → `wal`
- busy_timeout 확인: `PRAGMA busy_timeout;` → 5000+
- 동시 write 프로세스 있는지: observer와 hook 동시 실행?

### 12.4 tmux 세션 관리 문제

```bash
# orphan 세션 정리
tmux kill-server
# 다음 myth run에서 재생성
```

### 12.5 Claude Code hook이 안 불림

`claude --debug` 로 세션 시작 → 어떤 hook이 로드됐는지 로그.

- `.claude/settings.json` 경로 맞는지
- 바이너리 실행 권한 (`chmod +x`)
- User scope (`~/.claude/settings.json`) 와 충돌 있는지

## 13. WSL2 특유 이슈 모음

Research #2에서 식별된 알려진 문제들:

### 13.1 Issue: clock drift

WSL2가 sleep에서 복귀 시 시계가 뒤처짐. `caselog.jsonl`의 timestamp가 엉킬 수 있음.

```bash
# Manual fix
sudo hwclock -s
```

주기적으로 자동:
```bash
# /etc/cron.hourly/ntp-sync
#!/bin/bash
sudo hwclock -s
```

### 13.2 Issue: file watcher overload

Rust `notify` crate + WSL2 조합에서 가끔 이벤트 누락. myth는 **polling + inotify 이중 전략** 사용하지만, 드물게 brief.md 변경이 TUI에 반영 지연.

해결: `myth watch` 상단 `r` 키로 수동 새로고침.

### 13.3 Issue: `ENOMEM` on large compiles

`.wslconfig` memory 부족 시 Cargo 빌드가 OOM. swap 늘리거나 memory 증가.

```bash
# 빌드 전 확인
free -h
```

## 14. 체크리스트 요약

설치 전 Jeffrey가 한 번에 확인:

- [ ] WSL2 (wsl --list -v)
- [ ] Ubuntu 24.04 (lsb_release)
- [ ] `.wslconfig` 존재 + memory ≥ 8GB
- [ ] systemd-user 실행 중
- [ ] `XDG_RUNTIME_DIR` 설정됨
- [ ] mold 설치됨
- [ ] clang 설치됨
- [ ] Rust stable 설치됨 (1.82+)
- [ ] Node.js 18+ 설치됨
- [ ] Claude Code 2.1.27+ 설치됨
- [ ] tmux 설치됨
- [ ] `~/.local/bin`이 PATH에
- [ ] umask 0077 설정
- [ ] inotify limits 증가
- [ ] `~/myth/`가 WSL 네이티브 fs에 위치
- [ ] `/mnt/c/` 아님

모두 OK면 Day-0 진입 가능.

## 15. 관련 문서

- `~/myth/docs/03-DIRECTORY.md` — 파일 레이아웃 전체
- `~/myth/docs/04-CRATES/06-myth-embed.md` — embed daemon 상세
- `~/myth/PROTOCOL.md` — myth-embed wire protocol
- Research #2 원본 (Jeffrey 로컬 보관)
