# myth — 설계 결정 이력

이 문서는 myth를 설계하는 과정에서 내려진 **주요 결정들의 기록**이다. 각 결정은 "언제·왜·무엇을·어떤 대안을 제치고" 선택했는지 남긴다. 미래의 사용자(및 Claude)가 "왜 이렇게 설계됐지?"를 물을 때 답을 찾는 원천이다.

결정은 번호순으로 누적된다. 결정이 뒤집히거나 재해석되면 원 결정을 **삭제하지 않고** 후속 결정을 추가해 이력을 보존한다.

---

## Decision 1 — 벡터 저장소

**날짜**: 2026-04-19 (설계 회의)  
**맥락**: myth-identity의 Tier 2 Identity (임베딩 유사도 매칭) 구현 방식을 결정해야 한다.

**문제**: 50,000개 규모의 384차원 벡터를 저장하고 KNN 검색하는 구조를 어떻게 할 것인가.

**선택지**:
- (A) 순수 Rust in-memory + mmap + `trait VectorStore` 추상화
- (B) sqlite-vec (원안)
- (C) trait 없이 하드코딩

**선택**: (A)

**근거**:
- sqlite-vec Rust crate가 0.1.6(2024-11)에 고착된 반면 본가는 0.1.9(2026-03)까지 갔다. Rust 소비자가 최신 버그 수정을 못 받는다.
- sqlite-vec README가 "pre-v1, breaking change 예상"을 명시.
- 2024-11~2026-03 약 1년의 유지보수 공백 이력.
- 공식 backup/upgrade 가이드 부재.
- 50,000 × 384-dim brute-force SIMD는 34ms. myth 규모에서 성능 충분.
- `trait VectorStore` 추상화로 미래 전환(sqlite-vec 또는 usearch)이 단일 파일 수준 변경.

**전환 조건(Milestone B)**: 레코드 20K 초과 AND P99 > 50ms 동시 충족 시.

**영향 범위**: `myth-identity` crate 구조, `~/.myth/vectors.bin` + state.db 사이드카 구조.

**참조**: Research #3 (sqlite-vec 성숙도).

---

## Decision 2 — 임베딩 모델

**날짜**: 2026-04-19  
**맥락**: Decision 1에서 벡터 저장소가 정해졌다. 이제 벡터를 **만드는** 모델이 필요하다.

**문제**: 영어와 한국어가 섞인 lesson/failure 텍스트를 어떻게 벡터화할 것인가.

**선택지**:
- (A) bge-small-en-v1.5 (원안, 영어 전용)
- (B) multilingual-e5-small (다국어)
- (C) bge-m3 (최고 품질, 대형)

**선택**: (B)

**근거**:
- bge-small-en-v1.5는 토크나이저가 영어 WordPiece 기반이라 한국어가 `[UNK]` 토큰으로 처리됨.
- 사용자 환경에서 한국어가 섞일 가능성이 있음 (Reflector 지시 위반, Bash string에 한글, 등).
- multilingual-e5-small은 **같은 384차원** — 저장 레이어 무변경.
- fastembed-rs 네이티브 지원 (`EmbeddingModel::MultilingualE5Small`).
- 크기 차이 32MB→116MB는 메모리 상주 시 실질 영향 없음.
- bge-m3(1024차원)는 저장 3배 증가 + 품질 차이가 myth 규모에 과잉.

**재검토 기록(2026-04-19)**: 사용자가 "에러 메시지는 영어"를 지적. 헌법 Part IX.3 (metadata 영어 고정)도 확인. 그럼에도 "116MB 메모리 아끼려고 한글 섞임 수동 방어 로직 짤 바에는 C로 간다"는 결론으로 (B) 유지.

**영향 범위**: fastembed-rs 모델 상수, myth-embed daemon 초기 로드.

**참조**: Research #3.

---

## Decision 3 — Assessor 호출 경로 (간소 Hybrid 2-Tier)

**날짜**: 2026-04-19  
**맥락**: 실패 시 Assessor(구 Reflector)를 어떻게 호출할지.

**문제**: Research #4가 제안한 4-Tier Hybrid를 Day-1에 전부 구현할지, 일부만 할지.

**선택지**:
- (A) Research 권고 완전 수용 — 4-Tier 풀셋 (Day-1 구현)
- (B) Ultraplan 원안 — Option B 단독
- (C) 간소 Hybrid — Tier 0+1만 Day-1, Tier 2/3는 실측 후 증축
- (D) 극단 보수 — Option A 단독

**선택**: (C)

**근거**:
- Research 권고의 **철학**(4-Tier fallback, shadow mode)은 수용하되 구현 부담을 Ultraplan 원안 수준으로.
- Tier 1(Option B, additionalContext) 준수율은 engineered template로 75~88% 예상.
- Tier 0(deterministic classify)가 실패의 20%를 LLM 없이 처리.
- 3주 실사용 후 Curator 리포트와 `reflector-shadow.jsonl` 실측 데이터 기반으로 Tier 2/3 증축 판단 (**Milestone A**).
- Rough Start 원칙 정합: "증거 없는 상태 구조 금지".

**Day-1 구현 범위**:
- PostToolUseFailure + UserPromptSubmit + Stop hook 등록 (3개 이벤트 미리 준비)
- Tier 0 deterministic classifier
- Tier 1 Option B with engineered Variant B 템플릿
- Shadow mode metrics: `~/.myth/metrics/reflector-shadow.jsonl`
- Tier 2/3 관련 코드는 **비활성 상태로 빌드 포함** (`enable_tier2/3: false` 플래그)

**증축 결정 변수(Milestone A)**:
- Tier 1 준수율 ≥85% → Tier 2/3 비활성 유지
- 70~85% → Tier 2 활성
- <70% → Tier 2+3 활성 (Decision 4 경로 발동)

**영향 범위**: `myth-hooks`, `myth_py.assessor`, Shadow metrics 인프라.

**참조**: Research #4.

---

## Decision 4 — Tier 3 증축 경로 (Anthropic SDK)

**날짜**: 2026-04-19  
**맥락**: Decision 3에서 Tier 3가 증축된다면 구체 경로가 필요하다.

**문제**: `claude -p` subprocess vs Anthropic Python SDK 직접 vs 로컬 LLM.

**선택지**:
- (α) `claude -p` subprocess (Ultraplan 원안)
- (β) Anthropic Python SDK 직접 호출
- (γ) 로컬 LLM (Llama 등)

**선택**: (β). **단 증축 시점 진입만 확정, Day-1 구현 불포함.**

**근거**:
- GitHub Issue #43333: `claude -p`의 OAuth Max 인증이 API로 잘못 과금 (2026-04-04 개시, 미해결). Boris Cherny가 "의도 아님" 인정했지만 패치 없음.
- 2026-04-04 Anthropic 정책: 서드파티 harness의 Max 사용 차단. `claude -p` subprocess는 회색지대.
- (γ)는 Claude 계열 품질 차이 + WSL2 로컬 추론 부담 + myth 철학(Option 4+ Hybrid Wrapper) 배치.
- (β)는 API 명시 과금이지만 예측 가능. Haiku 4.5 기준 월 $0.1~1.3 예상 (Tier 3 발동 빈도 10~27% 가정).

**Day-1 미구현 이유**: 
- Decision 3에서 Tier 3 비활성이 기본 상태.
- 3주 후 Milestone A에서 증축 필요가 결정돼야 API key 발급·관리 인프라가 의미 있음.
- 미리 구현하면 Rough Start 위반.

**증축 시 구현 예정**:
- `~/.config/myth/api_key` (mode 0600) 파일 저장
- `myth key set/show/clear` CLI
- `dispatcher.py` retry/backoff 로직
- Anthropic Console workspace spend limit $10 hard cap
- `~/.myth/logs/tier3-dispatch.jsonl` 관찰성 로그

**참조**: Research #4.

---

## Decision 5 — Bedrock Rule 정규식 47개 Day-1

**날짜**: 2026-04-19  
**맥락**: The Gavel(구 Trial Court)의 현재 정규식 coverage가 실증 5~15%로 평가됐다.

**문제**: Research #5의 47개 강화 패턴을 언제 도입할지.

**선택지**:
- (A) Day-1 full 47개
- (B) 단계적 확장 (Day-1 핵심 15, Phase 1.5 확장)
- (C) Ultraplan 원안 3개 유지
- (D) 보수적 10개 (anchored prefix only)

**선택**: (A)

**근거**:
- The Gavel은 **사후 학습 불가능한 유일 guardrail**. Assessor가 놓친 것은 lesson으로 학습하지만 The Gavel이 놓친 `rm -rf /`는 비가역.
- gitleaks(MIT) + detect-secrets(Apache-2.0) 커뮤니티 튜닝 결과물을 차용 → FP 거의 0.
- 각 rule에 positive 20 + negative 20 test fixture (총 560건). pre-commit hook으로 자동 검증.
- 이 결정과 동시에 **상위 원칙으로 Phase 개념 폐기**. Jeffrey의 "며칠 안에 100% 완성" 원칙과 정합.

**상위 원칙 — Phase 폐기**:
- Day-1에 **모든 기능 구현**
- 일부 기능은 **실측 데이터 기반 활성/전환**
- "Phase" 대신 **Milestone A~E** 체계 사용 (Decision 참조)

**라이선스 영향**: myth 라이선스를 `MIT OR Apache-2.0` 듀얼로 선언. `THIRD-PARTY.md`에 출처 귀속.

**rule 구조** (15개 정규식 그룹, 47개 패턴):
- `rm_rf_unsandboxed`: R1-A~G (flag permutation, find-delete, language APIs, git destructive, other verbs, docker/k8s, path sentinel)
- `production_secrets_commit`: R2-A~D (anchored prefixes, PEM+URL basic-auth, keyword+entropy, Korean/CJK)
- `auth_bypass_production`: R3-A~D (JWT/TLS, framework bypass, env bypass, decorator-absence diff-aware)

**영향 범위**: `~/.myth/bedrock-rules.yaml` 전면 작성, `myth-gavel` crate 정규식 컴파일 로직, `tests/fixtures/{positive,negative}/*.txt` 560건.

**참조**: Research #5.

---

## Decision 6 — myth-embed daemon (bincode)

**날짜**: 2026-04-19  
**맥락**: Decision 2에서 multilingual-e5-small이 정해졌다. 이 모델의 500ms~2초 콜드 로드를 어떻게 amortize할 것인가.

**선택지**:
- (A) Self-daemonizing Unix socket daemon (Research 권고)
- (B) systemd --user 서비스
- (C) 비동기 배치 (SessionEnd batch)
- (D) 매 hook마다 fastembed-rs 인프로세스

**선택**: (A)

**프로토콜 결정**: **bincode** (length-prefixed + `version: u8`).

**근거**:
- (D)는 500ms × 매 hook → 체감 불가능.
- (C)는 실시간 recurrence 탐지 불가 → 헌법 Article 6 (Certainty) 훼손.
- (B)는 WSL2 #10205 버그 + 사용자 수동 `systemctl enable` 필요.
- (A)는 emacsclient 25년 생존 패턴. 투명성 5요소로 Rough Start 원칙 정합.

**프로토콜 선택 근거 (JSON → bincode 수정)**:
- myth 환경은 Jeffrey + Claude 바이브코딩. "사람이 소켓 뜯을 일" 실제 없음.
- Claude는 bincode struct 정의 읽고 동일하게 이해.
- JSON 사람 친화의 가치 수혜자가 없음.
- bincode: 직렬화 5배 빠름, 부동소수 정밀도 보존, 페이로드 3배 작음.
- 관찰성은 **JSON Lines 로그** + **`myth embed probe` CLI**로 보완.

**투명성 5요소**:
- `myth embed status` (PID, uptime, RSS, 요청 수)
- `myth embed stop`
- 15분 유휴 자가 종료
- `~/.local/state/myth/embed-daemon.log` (JSON Lines)
- `--no-embed-daemon` 탈출구

**영향 범위**: 신규 `myth-embed` crate (~900 LOC), `PROTOCOL.md`, `myth embed` CLI subcommand.

**참조**: Research #2, Research #3.

---

## Decision 7 — The Gavel 실행 모델 + Milestone C 조건 박제

**날짜**: 2026-04-19  
**맥락**: Decision 6에서 embed daemon은 넣었다. The Gavel 자체도 데몬화해야 할지.

**선택지**:
- (A) binary-per-hook + 측정 인프라 + 전환 조건 박제 (Research 권고)
- (B) Day-1부터 daemon
- (C) 전환 조건 박제 안 함
- (D) 더 엄격한 조건 (P99 > 30ms 등)

**선택**: (A)

**근거**:
- starship 5년 실증: 더 빈번한 호출 패턴에서 binary-per-hook으로 생존.
- The Gavel은 Claude Code 사용의 **임계 경로**. 데몬 버그 나면 시스템 마비. 복잡도 보수적이어야 함.
- Research #2 빌드 프로파일로 binary-per-hook P99 5~10ms 달성 예상.
- 전환 조건 박제가 Master Principle("수렴은 우연이 아니라 법이다")의 직접 구현.

**Day-1 구현**:
- `myth-hook-pre-tool` Rust binary (glibc + mimalloc + LTO fat + panic abort)
- `~/.local/state/myth/hook-latency.ndjson` 자동 수집
- `myth doctor` / `myth doctor --perf-check` / `myth doctor --wsl-check`
- Observer 주간 브리프에 "Migration Readiness" 섹션 고정 포함
- PGO 빌드 스크립트 (`~/myth/scripts/pgo-build.sh`) 대기 상태
- v1 API 계약 6개 고정 (ARCHITECTURE.md 박제)

**Milestone C 전환 조건** (AND):
1. P99 hook latency > 15ms가 2주 연속 (hook-latency.ndjson 자동 집계)
2. Research #2 §4 빌드 프로파일 완전 적용 상태
3. WSL2 운영 체크리스트 그린 상태 (WSL2-SETUP.md)
4. PGO+BOLT 시도 후에도 개선 미흡

**비상 트리거**: 단일 P99 > 50ms 측정 시 즉시 검토 (2주 누적 불필요).

**전환 시 도입 방식**: Self-daemonizing (emacsclient 패턴), 같은 바이너리의 `--daemon` 플래그 모드 전환, Unix socket + bincode.

**영향 범위**: `myth-gavel` crate, `ARCHITECTURE.md` 전환 조건 섹션, `myth doctor` 서브커맨드, Observer 리포트 포맷.

**참조**: Research #2.

---

## Decision 8 — Ultraplan 재작성 범위 + 문서 분할 구조

**날짜**: 2026-04-19  
**맥락**: Decision 1~7이 Ultraplan 원안을 약 60% 수정.

**선택지 (재작성 범위)**:
- (A) 전면 재작성 (v2.0)
- (B) 증분 수정 (v1.1~v1.7)
- (C) 하이브리드 (일부 재작성 + 일부 섹션 교체)

**선택**: (A)

**근거**: 수정 규모가 60%. 특히 Part 7(Phase 폐기)과 Part 8(프롬프트 분할 폐기)은 구조 변경이라 증분 수정 거의 불가. v1은 `MYTH-ULTRAPLAN-v1.md`로 아카이브.

**선택지 (문서 분할)**:
- (1) 단일 파일 (기존)
- (2) α — Part 단위 12개 파일
- (3) β — 논리 섹션 5~6개 파일
- (4) γ — 세분화 25+개 파일 (crate별 개별)

**선택**: (4) γ — 세분화 25+개 파일

**근거**:
- Claude Desktop의 `create_file`/`view` 도구 한계(~16,000자). 단일 파일 3,000줄은 쓰기·읽기 부하.
- Jeffrey가 검토하는 것은 문제 아니지만 Claude 도구 한계 존중.
- crate별 독립 문서가 Claude Code에게 "이 crate만 구현" 지시 시 정확한 참조 제공.
- 평균 파일 150~300줄, 최대 400줄 (myth-gavel).

**최종 구조** (총 28개):
- `~/myth/` 최상위 7개: README, CONSTITUTION, ARCHITECTURE, PROTOCOL, WSL2-SETUP, THIRD-PARTY, DECISIONS
- `~/myth/docs/` 아래 21개: INDEX, OVERVIEW, CONCEPTS, DIRECTORY, CRATES/00+10, PYTHON, HOOKS, STATE, BUILD-SCOPE, CLAUDE-PROMPTS, VALIDATION, RISKS, DEPLOYMENT

**작업 방식**: Phase 단위 검토 (β 방식). Phase 1(기반 3개) → Jeffrey 검토 → Phase 2 → ... 총 6~8회 세션 응답.

**영향 범위**: `~/myth/` 디렉토리 전체 구조, 기존 `~/project/harness-orchestrator/MYTH-ULTRAPLAN.md` 아카이브.

---

## Decision 9 — Day-0 실행 시점

**상태**: 미확정 (Phase 1~6 문서 작업 완료 후 결정)

**맥락**: 모든 설계 문서가 완성되고, Jeffrey가 검토 통과한 뒤 myth 실제 구현을 Claude Code에게 지시하는 시점.

**전제 조건** (Day-0 실행 전 만족해야):
- [ ] 28개 문서 전부 완성
- [ ] CONSTITUTION v2.3 개정 (Lapse 관련)
- [ ] Jeffrey 전체 검토 통과
- [ ] `~/myth/` Git init 및 초기 commit

**Day-0 수행 주체**: Claude Code (myth 구현), Jeffrey가 `docs/09-CLAUDE-PROMPTS.md`를 Claude Code에 전달.

**결정 연기 이유**: 현 시점에서 구체 날짜·방식 확정이 Rough Start 위반. 전제 조건 달성 뒤에 결정.

---

## 메타: 네이밍 재작업 (카테고리 1~10)

**날짜**: 2026-04-19 (Decision 8 중간)

**맥락**: Decision 8 전면 재작성을 하며 사용자 가시 용어들을 일관되게 재명명.

**대상**: 사용자가 매뉴얼·CLI·리포트에서 접하는 용어만. 내부 함수명·경로는 제외.

**결정 요약**:

| # | 카테고리 | 변경 내용 |
|---|---|---|
| 1 | 세 판단 주체 | Trial Court→**The Gavel**, Reflector→**Assessor**, Curator→**Observer**. crate도 변경(myth-gavel 등). |
| 2 | 규칙 계층 | Tier 1/2/3 NEVER → **Bedrock/Foundation/Surface Rule**. NEVER 제거. |
| 3 | Level | **유지** (Level 1~5 + INFO/LOW/MEDIUM/HIGH/CRITICAL). 업계 호환. |
| 4 | Recurrence | **유지** (I~VI 로마자). Level과 시각 분리. |
| 5 | Enforcement | archive/passive-log/session-hint/jit-context/soft-block/hard-block/mandatory-min → **Dismiss/Note/Advisory/Caution/Warn/Strike/Seal**. 이모지 제거. |
| 6 | Lesson 계열 | Lesson·Identity 유지, Quiescence → **Lapse** (한글 "휴면"). |
| 7 | 문서 파일 | playbook.md → **brief.md**, failures.jsonl → **caselog.jsonl**. CONSTITUTION 유지. |
| 8 | 사용자 행동 | De novo appeal → **Retrial**. Appeal/Split/Merge 유지. |
| 9 | 관찰 지점 | α~ε → **Milestone A~E**. |
| 10 | myth-embed | **유지** (기술어 적절). |

**전체 변경 파일 수**: 기존 2,200줄 Ultraplan에서 네이밍 관련 교체 ~500곳 추정.

---

## 참조 문서

- Research #1: Claude Code 2.1.x runtime (21 hook events, PostToolUseFailure, 70 env vars, WSL2 issues)
- Research #2: Rust cold start on WSL2 (build profile, binary-per-hook vs daemon, Milestone C 근거)
- Research #3: sqlite-vec 성숙도 + 임베딩 모델 + 상주 데몬 필요성
- Research #4: Assessor(Reflector) 호출 경로 Hybrid 4-Tier, PostToolUseFailure 이벤트, issue #43333
- Research #5: Trial Court 정규식 한계 분석 + 47개 강화 rule + gitleaks 차용

원본 research 파일은 Jeffrey 로컬에 보존. 주요 결론은 이 문서의 각 Decision에 요약.
