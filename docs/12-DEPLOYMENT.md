# myth — Deployment & Operations

Day-1 릴리스부터 장기 운영까지의 **설치·배포·업그레이드·롤백** 절차. 단일 사용자(Jeffrey) 환경 전제. 팀·엔터프라이즈 배포는 범위 밖.

## 1. 배포 모델

myth는 **source-from-Git** 배포. 미리 컴파일된 바이너리 없음. 사용자가 직접 clone + build.

```
GitHub repo
    ↓ git clone
~/myth/ (source)
    ↓ scripts/install.sh
~/.local/bin/ (compiled binaries)
    ↓ myth install (CLI 내부에서 호출)
~/.myth/ (runtime data initialized)
```

이유:
- Rust 빌드가 사용자 환경(CPU, glibc)에 민감 → 각자 빌드가 가장 안전
- PGO/BOLT는 환경별 튜닝이 의미 있음
- 배포 인프라 부담 없음 (단일 사용자 전제)

## 2. Day-0 → Day-1 초기 설치

### 2.1 전제 조건 확인

```bash
# WSL2-SETUP.md 체크리스트 통과
# 특히
which mold clang cargo python3 claude
# 모두 찾아져야 함

claude --version
# 2.1.27 이상
```

### 2.2 Repository clone

```bash
cd ~
git clone https://github.com/Di-Vernon/myth.git
# 또는 로컬 작업 중이면 그냥 ~/myth/ 가 있으면 됨
```

### 2.3 빌드 및 설치

```bash
cd ~/myth
bash scripts/install.sh
```

`install.sh`가 하는 일:
1. `cd rust && cargo build --release` (약 8분)
2. `~/.local/bin/`에 8개 바이너리 symlink
3. Python shim 2개 작성
4. `pip install -e python/` (editable)
5. `~/.myth/` 초기 디렉토리 + 기본 rules/grid YAML 복사
6. SQLite DB 초기화 (마이그레이션 적용)
7. `myth doctor` 실행

### 2.4 첫 번째 프로젝트 등록

```bash
cd ~/project/my-existing-project
myth init
# .claude/settings.json 생성 (hooks 6개 등록)
# .claude/agents/assessor.md, observer.md symlink
```

### 2.5 초기 검증

```bash
myth doctor  # 모든 체크 green
myth embed probe "hello"  # 384-dim vector
myth status  # lesson 0, embed running
myth watch   # TUI 잠시 띄워서 확인 (q로 종료)
```

### 2.6 첫 번째 Claude 세션

```bash
cd ~/project/my-existing-project
myth run
# Claude Code가 실행됨. myth hook이 개입.
# 일부러 실패할 명령 실행해보기:
#   > cat /nonexistent/file
# → Variant B template 주입 확인
# → 다음 턴에 Task(assessor) 호출되는지 관찰
```

**세션 종료 후**:
```bash
myth lesson list  # 방금 실패에서 lesson 생성 확인
myth observer run --dry  # Observer 작동 확인
```

Day-1 설치 완료.

## 3. 일일 운영

### 3.1 평상시

Jeffrey는 `myth run`으로 Claude Code 시작. myth가 배경에서 hook 처리. 추가 신경 쓸 것 없음.

필요 시:
- `myth status` — 현재 상태 빠르게 확인
- `myth watch` — 세부 사항 TUI
- `myth lesson list` — 최근 lesson 조회

### 3.2 문제 발생 시

**Hook이 작동 안 함**:
```bash
myth doctor
# 실패 항목 확인 → ~/.claude/settings.json 경로·권한 체크
```

**The Gavel이 정상 명령을 차단**:
```bash
myth lesson appeal <lesson-id> --reason "legitimate use case"
# Observer 다음 주간 실행에서 재검토
```

**`myth-embed` 응답 없음**:
```bash
myth embed status
# not running 이면 다음 요청 시 자동 spawn
# 계속 실패하면:
ps aux | grep myth-embed  # 좀비 프로세스 확인
myth embed stop  # 있으면 종료
myth-embed probe "test"  # 재시도 → spawn
```

**디스크 사용량 이상**:
```bash
du -sh ~/.myth ~/.local/state/myth
# caselog.jsonl 비대화 확인
# logrotate 설정 점검
```

### 3.3 로그 로테이션

`/etc/logrotate.d/myth` (sudo 필요):

```
/home/miirr/.local/state/myth/hook-latency.ndjson
/home/miirr/.local/state/myth/embed-daemon.log
/home/miirr/.local/state/myth/gavel-daemon.log
/home/miirr/.local/state/myth/tier3-dispatch.jsonl
{
    weekly
    rotate 4
    compress
    missingok
    notifempty
    create 0600 miirr miirr
}
```

`caselog.jsonl`과 `lesson-state.jsonl`은 **rotate 하지 않음** — Observer가 전체 history 참조.

`audit.jsonl`도 rotate 안 함 (chain 보존).

## 4. 주간 운영

### 4.1 Observer 자동 실행

systemd user timer (권장):

`~/.config/systemd/user/myth-observer.timer`:

```ini
[Unit]
Description=myth Observer weekly analysis

[Timer]
OnCalendar=Mon 09:00
Persistent=true

[Install]
WantedBy=timers.target
```

`~/.config/systemd/user/myth-observer.service`:

```ini
[Unit]
Description=myth Observer service

[Service]
Type=oneshot
ExecStart=/home/miirr/.local/bin/myth observer run
StandardOutput=journal
StandardError=journal
```

활성:
```bash
systemctl --user enable --now myth-observer.timer
systemctl --user list-timers
```

대안 (cron):
```cron
# crontab -e
0 9 * * MON /home/miirr/.local/bin/myth observer run >> ~/.local/state/myth/observer.log 2>&1
```

### 4.2 수동 확인

매주 월요일 Jeffrey가:

```bash
# 1. brief 읽기
less ~/.myth/brief.md

# 2. Migration Readiness 확인
myth doctor --migration

# 3. 필요 시 Grid override 적용
myth lesson show <id>
# ... 판단
```

### 4.3 주간 체크리스트

- [ ] Observer 실행 성공 (`systemctl --user status myth-observer`)
- [ ] brief.md 생성됨 (mtime 최근 24h 이내)
- [ ] Tier 1 compliance rate 확인
- [ ] Migration Readiness 각 5개 상태 확인
- [ ] Lapse 전환 건수 확인
- [ ] 디스크 사용량 점검

## 5. 업그레이드

### 5.1 Patch 업그레이드 (v0.1.0 → v0.1.1)

버그 수정, 작은 기능. 호환성 유지.

```bash
cd ~/myth
git fetch
git checkout v0.1.1
bash scripts/install.sh
# 자동으로 rebuild + reinstall
```

### 5.2 Minor 업그레이드 (v0.1 → v0.2)

새 Milestone 활성 또는 큰 기능 추가. 새 crate 가능.

```bash
cd ~/myth
git fetch
git checkout v0.2.0

# 사전 체크
cat CHANGELOG.md  # 주요 변경 사항
myth doctor --backup  # state.db 백업 생성

# 업그레이드
bash scripts/install.sh

# 마이그레이션 (필요 시)
myth doctor --migrate
# SQLite user_version 업데이트 등
```

v0.1 → v0.2 예상 시나리오:
- Milestone A 활성 → `api_key` 관리 UI 추가, `config.yaml`에 `tier_2_enabled: true` 전환
- Milestone C 활성 → `myth-gavel` daemon 모드 추가, `myth gavel status` 명령 활성

### 5.3 Major 업그레이드 (v0.x → v1.0)

API 계약 고정. SemVer 엄격 시작.

```bash
# 대규모 검증 필요
myth doctor --backup
myth audit verify

# 전환
git checkout v1.0.0
bash scripts/install.sh

# v1.0은 v0.x 데이터 읽기 호환 (state.db forward-only migration)
# 이후로는 v1.x 시리즈에서 breaking change 없음
```

## 6. 롤백

### 6.1 즉시 롤백 (문제 발견)

```bash
cd ~/myth
git checkout <이전 tag>
bash scripts/install.sh
```

SQLite는 forward-only라 이전 버전이 새 스키마 읽으려 시도 → 에러 가능. 이 경우:

```bash
# 백업에서 복원
cp ~/.myth/backups/state-<timestamp>.db ~/.myth/state.db
```

### 6.2 긴급 비활성화 (myth 자체 문제)

myth가 오작동해도 Claude Code는 계속 써야 할 때:

```bash
# 1. hook 비활성 (프로젝트별)
cd ~/project/my-project
mv .claude/settings.json .claude/settings.json.bak
# → Claude Code는 hook 없이 동작

# 또는 2. myth 전역 비활성
export MYTH_DISABLE=1
# 이 세션에서 hook이 로드되지만 즉시 allow 반환
```

문제 해결 후 복구:
```bash
mv .claude/settings.json.bak .claude/settings.json
unset MYTH_DISABLE
```

### 6.3 완전 제거

```bash
bash ~/myth/scripts/uninstall.sh
# 하는 일:
# - ~/.local/bin/myth* 제거
# - Python pip uninstall myth_py
# - ~/.myth/ 유지 (데이터 보존, 사용자가 직접 지우면 됨)
# - ~/myth/ source 유지 (Git 저장소)
```

완전 삭제:
```bash
rm -rf ~/myth
rm -rf ~/.myth
rm -rf ~/.config/myth
rm -rf ~/.local/state/myth
# (~/.local/bin/myth* 는 uninstall.sh에서 이미 제거)
```

## 7. 백업

### 7.1 정기 백업

```bash
# ~/.myth/backups/ 안에 자동 백업 (주 1회 권장)
# ~/myth/scripts/backup.sh (Day-1 제공)

#!/usr/bin/env bash
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
BACKUP_DIR=~/.myth/backups
mkdir -p "$BACKUP_DIR"

# SQLite 온라인 백업
sqlite3 ~/.myth/state.db ".backup '$BACKUP_DIR/state-$TIMESTAMP.db'"

# JSONL 파일 스냅샷
cp ~/.myth/caselog.jsonl "$BACKUP_DIR/caselog-$TIMESTAMP.jsonl"
cp ~/.myth/lesson-state.jsonl "$BACKUP_DIR/lesson-state-$TIMESTAMP.jsonl"
cp ~/.myth/audit.jsonl "$BACKUP_DIR/audit-$TIMESTAMP.jsonl"

# 압축
tar czf "$BACKUP_DIR/myth-$TIMESTAMP.tar.gz" \
    -C ~/.myth \
    state.db caselog.jsonl lesson-state.jsonl audit.jsonl brief.md \
    bedrock-rules.yaml foundation-rules.yaml grid.yaml

# 30일 이상 된 백업 제거
find "$BACKUP_DIR" -name "myth-*.tar.gz" -mtime +30 -delete

echo "Backup: myth-$TIMESTAMP.tar.gz"
```

crontab:
```cron
0 3 * * SUN /home/miirr/myth/scripts/backup.sh
```

### 7.2 수동 백업 (업그레이드 전)

```bash
myth doctor --backup
# = sqlite3 .backup + JSONL copy
```

### 7.3 복원

```bash
# 전체 복원
cd ~/.myth
tar xzf ~/.myth/backups/myth-20260419-120000.tar.gz

# 또는 SQLite만
cp ~/.myth/backups/state-20260419-120000.db ~/.myth/state.db

myth doctor  # 정합성 검증
```

## 8. 모니터링

### 8.1 자동 (향후)

v0.2 이후 계획:
- brief.md 갱신 시 알림 (systemd notify, 또는 로컬 notification)
- Critical 지표 이상 시 경보

Day-1은 수동.

### 8.2 수동 주기

**매일**:
- `myth status` (건강성 빠른 확인)

**매주**:
- brief.md 통독
- Migration Readiness 상태
- Observer timer 실행 확인

**매월**:
- 디스크 사용량 (`du -sh ~/.myth ~/.local/state/myth`)
- 백업 존재 확인
- `myth audit verify` (무결성 체크)
- Grid override 적용 검토

**매 분기**:
- Risk Catalog (`11-RISKS.md`) 재평가
- 성능 수치 추세 (P99, compliance rate)

## 9. Milestone 전환 운영

각 Milestone 발동 시 **정해진 체크리스트**:

### 9.1 Milestone A 활성 절차

```bash
# 1. 전제 조건 확인
myth doctor --migration  # Milestone A: TRIGGERED 확인

# 2. 백업
myth doctor --backup

# 3. API key 발급 (Jeffrey가 Anthropic Console에서)
# https://console.anthropic.com/settings/keys

# 4. 설정
myth key set
# 프롬프트에서 API key 입력

# 5. Tier 2/3 활성 (편집 필요)
$EDITOR ~/.config/myth/config.yaml
# assessor:
#   tier_2_enabled: true
#   tier_3_enabled: true  # 선택

# 6. 검증
myth doctor --check-tier3
# API key 유효, spend limit 확인

# 7. 관찰
# 다음 실패 시 tier3-dispatch.jsonl 생성 확인
```

### 9.2 Milestone B 활성 절차

```bash
# 1. 전제 조건
myth doctor --migration  # Milestone B: TRIGGERED

# 2. 백업 (vector store 전환은 리스크 있음)
myth doctor --backup

# 3. 후보 선택 (sqlite-vec 또는 usearch)
# Day-1 시점 조건으로 Decision 1 재평가 필요

# 4. config 변경
$EDITOR ~/.config/myth/config.yaml
# vector_store: sqlite_vec  # 또는 usearch

# 5. migration 스크립트 실행
myth doctor --migrate-vectors
# In-memory store → 선택된 store로 데이터 이동
# 완료 시간: 1000 lesson당 수 분

# 6. 검증
myth embed probe "migration test"
myth lesson list | head  # 정상 조회 확인
```

### 9.3 Milestone C 활성 절차

```bash
# 1. 전제 조건
myth doctor --migration  # Milestone C: TRIGGERED (AND 4조건)

# 2. PGO 빌드 시도 (선택, Milestone C 발동 전제)
bash ~/myth/scripts/pgo-build.sh

# 3. Gavel daemon 코드 추가 (Claude Code 작업)
# - 설계: ARCHITECTURE.md §4 "Milestone C"
# - 구현: myth-gavel::daemon 모듈
# - Wave 컨셉: "Milestone C Wave"

# 4. config 활성
$EDITOR ~/.config/myth/config.yaml
# gavel:
#   daemon_enabled: true

# 5. 검증
myth gavel status  # daemon 정상 확인
# hook-latency.ndjson 모니터링 → P99 개선 확인
```

### 9.4 Milestone D, E

유사한 체크리스트. 발동 시점에 상세 절차를 DEPLOYMENT.md에 추가.

## 10. 보안 운영

### 10.1 API key 관리

- 파일 권한 0600 (install 시 자동)
- Git commit 금지 (`.gitignore` 포함)
- 분기마다 rotate 권장:
  ```bash
  # Anthropic Console에서 새 key 발급
  myth key set  # 새 key 입력
  # 이전 key는 Console에서 revoke
  ```

### 10.2 audit chain 검증

```bash
# 월간
myth audit verify
# 실패 시 tamper 의심 → 조사
```

### 10.3 비밀 유출 감지

myth 자체가 Bedrock Rule로 secret commit 차단. 추가로:

```bash
# 프로젝트마다 주기 실행
git log --all -p | gitleaks detect --stdin  # 과거 commit 검사
```

## 11. 장애 대응 시나리오

### 11.1 myth 설치 후 Claude Code가 시작 안 됨

```bash
# 1. myth disable
export MYTH_DISABLE=1
claude  # 정상 시작?

# 2. hook 경로 확인
cat .claude/settings.json | jq '.hooks | to_entries | .[] | .value.command'
# → 모두 절대 경로, 존재하는 바이너리인지

# 3. 각 바이너리 실행 권한
ls -l ~/.local/bin/myth-hook-*
# -rwxr-xr-x 이어야 함

# 4. 직접 실행 테스트
echo '{}' | ~/.local/bin/myth-hook-pre-tool
# 정상 JSON 반환?

# 원인별 대응:
# - 바이너리 없음 → `bash ~/myth/scripts/install.sh` 재실행
# - 권한 없음 → `chmod +x ~/.local/bin/myth-hook-*`
# - JSON 파싱 실패 → Claude Code 버전 호환 문제, `myth doctor`
```

### 11.2 hook latency 폭증

```bash
# 1. 즉시 확인
tail -n 100 ~/.local/state/myth/hook-latency.ndjson | \
    jq '.latency_ms' | sort -n | tail

# 2. outlier 원인
# - SQLite lock? → ps aux | grep myth
# - disk I/O 느림? → iostat
# - 다른 프로세스가 CPU 독점? → top

# 3. 완화
# - 임시로 MYTH_DISABLE=1
# - `myth observer run` 중이면 종료 후 재시작

# 4. 근본 원인
# → Milestone C 조건 확인
```

### 11.3 `~/.myth/` 디스크 가득

```bash
du -sh ~/.myth/*
# 가장 큰 파일 확인 (대개 caselog.jsonl)

# 오래된 caselog archive
mkdir -p ~/.myth/archive
mv ~/.myth/caselog.jsonl ~/.myth/archive/caselog-$(date +%Y%m%d).jsonl
touch ~/.myth/caselog.jsonl  # 새로 시작
```

**주의**: caselog archive 후 Observer가 과거 데이터 못 읽음. Observer 코드가 `archive/` 디렉토리도 스캔하도록 옵션 제공 (Day-1 미구현, 추후 기능).

## 12. 문서 업그레이드

myth 문서 (this repo의 `docs/`, `*.md`)는 코드와 함께 버전화.

- 새 기능 추가 → 관련 crate 문서 수정
- API 계약 변경 → `ARCHITECTURE.md` §2 반드시 업데이트
- 새 결정 → `DECISIONS.md` 신규 섹션
- Migration 발동 → `DEPLOYMENT.md` §9 해당 절차 추가

각 릴리스마다 `CHANGELOG.md` 업데이트.

## 13. End-of-Life

myth 프로젝트 종료 시 (가정):

**사용자 작업**:
1. 모든 프로젝트에서 `myth` hook 제거 (`.claude/settings.json` 복원)
2. `bash ~/myth/scripts/uninstall.sh`
3. `~/.myth/`, `~/.config/myth/` 백업 후 제거

**데이터 보존**:
- `caselog.jsonl` 은 **시간이 지나도 가치 있는 자산** (실패 사례 학습 기록)
- 아카이브 권장: `tar czf myth-final-archive.tar.gz ~/.myth/`

**Claude Code 복원**:
- hook 없이 원래 상태로 복귀
- 기능 손실: 실시간 차단, 자동 학습
- Claude Code 자체 동작은 영향 없음

## 14. 관련 문서

- `~/myth/WSL2-SETUP.md` — 사전 환경
- `~/myth/docs/08-BUILD-SCOPE.md` — Day-1 범위
- `~/myth/docs/10-VALIDATION.md` — 검증 기준
- `~/myth/docs/11-RISKS.md` — 리스크 관리
- `~/myth/CONSTITUTION.md` — 불변의 원칙
