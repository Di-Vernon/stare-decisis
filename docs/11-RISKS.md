# myth — Risk Catalog

myth 개발·운영에서 **알고 있는 리스크**를 목록으로 박제한다. 각 리스크는 확률·영향·조기 경고·대응을 명시. 모르는 리스크를 다룰 수는 없지만, 아는 리스크는 **미리 준비**한다.

본 문서의 리스크는 **Day-0 시점 관점**. 각 Milestone 진입 시점에 재평가한다.

## 1. 리스크 카테고리

네 분류:
- **T**: 기술 리스크 (build, performance, bugs)
- **P**: 프로세스 리스크 (timeline, scope, external dependencies)
- **S**: 보안 리스크 (data, access, injection)
- **O**: 운영 리스크 (maintenance, upgrade, degradation)

각 리스크에 `ID: [카테고리]-N` 부여.

## 2. T-1: Rust 빌드 시간 폭주

**확률**: 중  
**영향**: 중 (개발 속도 저하)

**시나리오**: `cargo build --release` LTO fat + codegen-units=1로 **단일 코어에서 링크**. 11,000 LOC + 수백 개 의존성에서 10분 넘어가면 iteration 부담.

**조기 경고**:
- dev 빌드도 5분 초과 (증상)
- `cargo check` 2분 초과

**대응**:
- `profile.dev` 분리: LTO off, codegen-units = 16
- `profile.release-fast`: LTO thin (dev 빌드용 릴리스, 성능 약간 희생)
- mold linker 필수 (링크 시간 5배 감소)
- cargo workspace 분할 (이미 10개 crate로 분리됨 → 변경 빈번한 crate만 재컴파일)

**Day-1 조치**: `profile.release`는 그대로 두고, `profile.release-fast` 추가:

```toml
[profile.release-fast]
inherits = "release"
lto = "thin"
codegen-units = 4
```

개발 중에는 `cargo build --profile release-fast`. 최종 배포만 `cargo build --release`.

## 3. T-2: Hook P99 > 15ms (Milestone C 조기 트리거)

**확률**: 중  
**영향**: 중 (Milestone C 작업 필요 → 개발 bandwidth)

**시나리오**: WSL2에서 fork+exec가 예상보다 느림. 47개 정규식 DFA 컴파일이 콜드 경로에서 5ms 넘음. Binary-per-hook이 성능 예산을 못 맞춤.

**조기 경고**:
- `hook-latency.ndjson` P50 > 5ms
- 단일 outlier P99 > 30ms 주 1회 이상

**대응**:
- 사전 DFA 직렬화 (build.rs로 컴파일 타임에 빌드 → `include_bytes!`)
- mimalloc global allocator 확인
- regex-automata `Lazy<DFA>` 캐시 점검
- Milestone C 발동

**Day-1 조치**: 측정 + 대기. Milestone C 트리거 조건이 이미 명시.

## 4. T-3: multilingual-e5-small 다운로드 실패

**확률**: 낮음  
**영향**: 중 (Tier 2 identity 기능 불능)

**시나리오**: 방화벽·기업 프록시·오프라인 환경에서 HuggingFace 접근 차단. 첫 `myth-embed` 실행 시 모델 다운로드 실패.

**조기 경고**:
- 빌드/설치 단계에서 네트워크 오류

**대응**:
- `myth doctor`에 `--check-network` 포함, 초기 다운로드 가능 여부 확인
- 오프라인 설치 가이드: Jeffrey가 별도 머신에서 모델 미리 다운로드 → `~/.myth/embeddings/models/multilingual-e5-small/`에 수동 배치
- fastembed-rs 대체 경로 (직접 ONNX 파일 제공)

**Day-1 조치**: `scripts/install.sh`에 다운로드 사전 테스트:

```bash
# install.sh 중간에
myth-embed probe "initial model download test" || {
    echo "Failed to download embedding model."
    echo "If offline, manually place model at: ~/.myth/embeddings/models/multilingual-e5-small/"
    echo "See THIRD-PARTY.md §3 for details."
    exit 1
}
```

## 5. T-4: SQLite 동시 Write 경합

**확률**: 낮음  
**영향**: 중 (hook 실패 → tool 실행 계속되나 학습 누락)

**시나리오**: hook (빈번) + Observer 주간 실행 (가끔) + 사용자 명령 (`myth lesson appeal`) 이 동시에 DB write.

**조기 경고**:
- `hook-latency.ndjson`에 "SQLITE_BUSY" 관련 에러 JSON
- `caselog.jsonl`의 hook_event 누락

**대응**:
- WAL 모드 + `busy_timeout = 5000` (이미 적용)
- **Write 직렬화**: 모든 DB write를 단일 writer task로 (future: Milestone B 또는 C에서 daemon 내부에서 처리)
- Observer 실행 시간대 조정 (hook 활동 적은 시간)

**Day-1 조치**: WAL + busy_timeout 만으로 충분할 가능성. 실제 경합 발생 시 재평가.

## 6. T-5: Claude Code 버전 비호환

**확률**: 중  
**영향**: 높음 (핵심 기능 실패)

**시나리오**: Anthropic이 Claude Code를 업데이트하면서:
- Hook JSON schema 변경
- PostToolUseFailure 제거 또는 개편
- `.claude/settings.json` 포맷 변경
- tool_use_id, session_id 필드 rename

**조기 경고**:
- Claude Code 업데이트 후 `myth doctor` 실패
- hook에서 JSON 파싱 에러

**대응**:
- `myth-runtime::version::validate_compatible()` 철저 (2.1.27+ 체크)
- Hook 입력 파싱을 **관대하게**: 필수 필드 누락 시 graceful degradation
- Claude Code changelog 모니터링 (Anthropic discord, GitHub releases)
- v0.1 → v0.2 버전업으로 Claude Code 버전 변경 흡수

**Day-1 조치**: `ClaudeRuntime`에 version lock + warning. 2.1.27 미만 감지 시 "PostToolUseFailure may not be available" 경고.

## 7. T-6: vectors.bin 손상

**확률**: 낮음  
**영향**: 중 (Tier 2 identity 불능, Tier 1만으로 동작)

**시나리오**: 
- 시스템 크래시 중 atomic rename 실패
- WSL2 disk I/O 에러
- 잘못된 수동 편집

**조기 경고**:
- `myth-embed probe` 시 integrity_check 실패
- TUI에 "Vector store error" 표시

**대응**:
- 전용 integrity_check (magic, version, dim, count, generation)
- `myth doctor --fix-vectors`: 손상 감지 시 lessons 테이블에서 재임베딩
- 재임베딩 시간: 1000 lesson × 10ms = 10초. 감수 가능.

**Day-1 조치**: 손상 감지 + 재빌드 메커니즘 포함. `~/myth/docs/07-STATE.md` §복구 전략.

## 8. T-7: Variant B Template 준수율 저조

**확률**: 중  
**영향**: 중 (Tier 1 compliance < 70% → Tier 2/3 활성 필요)

**시나리오**: Claude가 Variant B의 `<instructions>` 블록을 꾸준히 따르지 않음. 실패 분석을 건너뛰고 바로 재시도하려 함.

**조기 경고**:
- `reflector-shadow.jsonl`의 Tier 1 compliance < 85%
- Day 21 분석에서 추세 하락

**대응**:
- Milestone A 활성: Tier 2 (Stop hook 재주장) 켜기
- 더 나아가 Tier 3 (외부 dispatch): `enable_tier3: true`
- 장기적으로 **Variant B 개선** — compliance 높은 표현 조합 연구
- Claude 모델 업데이트 (더 instruction-following 능력 높은 모델) 추적

**Day-1 조치**: Shadow mode로 데이터 수집만. 판단 보류 21일.

## 9. P-1: Jeffrey bandwidth 부족

**확률**: 높음 (단일 개발자)  
**영향**: 높음 (프로젝트 지연)

**시나리오**: Jeffrey가 다른 일로 바빠서 Day-1 이후 3주 관찰·Observer brief 리뷰를 놓침. Milestone A 판단 지연.

**조기 경고**: brief.md mtime > 14일 (Observer 실행 안 됨)

**대응**:
- `~/.myth/observer-cron` systemd timer 또는 cron 등록 (자동 실행)
- brief.md 갱신 알림 (메일, Slack, 등) — Day-1 미구현, Milestone 이후
- myth가 완전 자동화된 부분은 어차피 Jeffrey 개입 불필요

**Day-1 조치**: 자동 주간 실행 설정 안내를 `DEPLOYMENT.md`에 포함.

## 10. P-2: harness-orchestrator 쉘 재사용 호환성

**확률**: 낮음  
**영향**: 중 (orchestrator 기능 일부 깨짐)

**시나리오**: 기존 `lib/execute.sh` 등이 Jeffrey의 특정 환경 가정 (bash 5.x, gum 특정 버전). myth로 이식 시 다른 환경에서 실패.

**조기 경고**: Wave 4.2 테스트에서 shell subprocess 실패

**대응**:
- 쉘 스크립트 의존성 명시 (Dependencies 섹션 추가)
- 각 쉘 함수에 대한 Rust wrapper 테스트
- 장기적으로 쉘 → Rust 포팅

**Day-1 조치**: Jeffrey 환경에서 최초 통과 확인. WSL2 Ubuntu 24.04 + gum charmbracelet + tmux ≥ 3.x 전제.

## 11. P-3: Anthropic API 정책 변경

**확률**: 중  
**영향**: 높음 (Milestone A Tier 3 경로 차단)

**시나리오**: 
- `claude -p` subprocess 완전 차단 (2026-04 부분 정책 확장)
- Anthropic SDK API 호출에 일일 한도 부여
- Claude Max 구독 정책 변경 (API 분리 강제)

**조기 경고**:
- Anthropic 공식 블로그·changelog
- GitHub Issue #43333 (이미 알려진 과금 문제)

**대응**:
- `myth_py.assessor.dispatcher`가 API key 기반 (Max 구독 독립)
- Spend limit 10 USD/월 hard cap
- Tier 3 fallback: 로컬 LLM (Llama, 미래 대비)
- 또는 Tier 3 무효화 + Tier 1/2로만 운영

**Day-1 조치**: Day-1에 Tier 3 비활성이므로 직접 영향 없음. Milestone A 시점에 정책 재확인.

## 12. P-4: Claude Code 기능 폐기

**확률**: 낮음  
**영향**: 매우 높음 (myth 전체 불능)

**시나리오**: Anthropic이 Claude Code를 end-of-life. hook 시스템 전체 폐기.

**조기 경고**: Anthropic의 Claude Code 관련 공식 발표

**대응**:
- myth를 **다른 agent runtime에 이식** — myth-runtime이 추상화 레이어. Aider, Cline, Cursor 등으로 교체 가능.
- 단, hook 메커니즘이 각 runtime마다 다르므로 상당한 작업.
- 최악: myth 프로젝트 end-of-life.

**Day-1 조치**: 관찰만. 확률 낮음 (Anthropic의 Claude Code는 상업적 성공, 폐기 가능성 낮음).

## 13. S-1: `api_key` 파일 노출

**확률**: 낮음  
**영향**: 매우 높음 (API 비용 폭증, 계정 탈취)

**시나리오**: 
- 실수로 `~/.config/myth/api_key` 파일을 Git에 커밋
- 공용 컴퓨터에서 `cat ~/.config/myth/api_key` 
- 백업 파일이 다른 사용자에게 노출

**조기 경고**:
- Anthropic Console에서 이상한 사용 패턴
- Spend 갑자기 증가

**대응**:
- 파일 권한 0600 강제 (생성 시 `std::fs::set_permissions(..., Permissions::from_mode(0o600))`)
- `.gitignore`에 `~/.config/myth/api_key` 패턴 포함
- `myth doctor`가 `api_key` 권한 체크
- Spend limit $10 hard cap (Anthropic Console 설정)
- 노출 발견 시 즉시 API key revoke + 신규 발급

**Day-1 조치**: 파일 생성·체크 로직 `myth-cli::subcmd::key`에 구현.

## 14. S-2: 악의적 lesson 조작

**확률**: 낮음  
**영향**: 중 (특정 패턴이 dismiss로 강제됨)

**시나리오**: Jeffrey 컴퓨터 사용자가 아닌 제3자가 `~/.myth/state.db`를 직접 수정. lesson level을 하향 조정하거나 lesson을 무효화.

**조기 경고**:
- audit.jsonl의 hash chain 단절
- `myth audit verify` 실패

**대응**:
- Merkle audit chain (blake3)로 tamper-evident
- SQLite mode 0600 (다른 사용자 접근 차단)
- `myth audit verify` 주기 실행 (선택)

**Day-1 조치**: audit chain 구현. 수동 검증 명령 제공.

## 15. S-3: Hook에서 임의 코드 실행

**확률**: 매우 낮음  
**영향**: 매우 높음 (전체 시스템 침투)

**시나리오**: Hook 입력 JSON 파싱에서 deserialize gadget 공격. 또는 정규식 매칭 시 ReDoS로 CPU 100% 장시간.

**조기 경고**:
- Hook 응답 시간 급증
- CPU 사용률 이상

**대응**:
- `serde` safe deserialization (type-directed, no untrusted gadgets)
- `regex` crate DFA (polynomial time guarantee)
- 정규식 컴파일 시 `Regex::new`가 실패하면 rule 건너뛰기
- Timeout: Claude Code 자체가 hook에 5~10초 timeout 부여

**Day-1 조치**: 안전한 라이브러리만 사용. ReDoS 테스트 §Validation 10.3 참조.

## 16. O-1: Lesson 누적 과다

**확률**: 중 (장기)  
**영향**: 중 (DB 크기, 검색 속도)

**시나리오**: 1년 후 10,000+ lesson. SQLite 쿼리 속도 저하. vectors.bin 파일 크기 15 MB+. mmap 메모리 사용 증가.

**조기 경고**:
- `state.db` 크기 > 100 MB
- `lesson list` 명령 응답 > 1초

**대응**:
- Lapse로 오래된 lesson은 `archived` 상태 → 쿼리 제외
- 아주 오래된 (6개월+ idle) `archived` → `~/.myth/archive/` 디렉토리로 이동
- `SELECT ... WHERE status = 'active'` 인덱스 활용
- Milestone B 발동 (vector store 전환)

**Day-1 조치**: archived 상태 + 인덱스 설계. 자동 archive 로직 Observer에 포함.

## 17. O-2: 로그 파일 증가

**확률**: 높음  
**영향**: 낮음 (디스크 공간)

**시나리오**: `hook-latency.ndjson`이 rotation 없이 계속 증가. 1년 ~50 MB. `caselog.jsonl`이 500 MB.

**조기 경고**: `~/.myth/` 디렉토리 크기 > 1 GB

**대응**:
- logrotate 설정 (install.sh 시)
- `caselog.jsonl` 연 단위로 archive/caselog-2026.jsonl로 이동
- `audit.jsonl`은 rotation 하지 않음 (chain 보존)

**Day-1 조치**: logrotate `.conf` 템플릿 제공 + install.sh에서 사용자 안내.

## 18. O-3: 규칙 파일 편집 실수

**확률**: 중  
**영향**: 중 (myth 일시 중단)

**시나리오**: Jeffrey가 `~/.myth/bedrock-rules.yaml`을 직접 수정하다 YAML 파싱 오류. The Gavel fail-safe로 모든 tool 차단 → 작업 불가.

**조기 경고**: PreToolUse에서 "rules load failed" 에러

**대응**:
- 편집 전 백업 (`cp bedrock-rules.yaml bedrock-rules.yaml.bak`)
- `myth doctor --validate-rules`로 편집 후 검증
- 롤백: `cp bedrock-rules.yaml.bak bedrock-rules.yaml && myth doctor`
- 미래: `myth rules edit` subcommand로 validation 통합

**Day-1 조치**: `myth doctor`에 rules YAML 파싱 검증 포함.

## 19. O-4: Milestone 트리거 오판

**확률**: 낮음  
**영향**: 중 (premature migration)

**시나리오**: 짧은 스파이크로 Hook P99가 잠시 15ms 초과 → Milestone C 트리거 → 섣부른 Gavel daemon 개발.

**조기 경고**: Observer brief에서 migration readiness가 갑자기 triggered가 됨

**대응**:
- **AND 조건** 엄격 (2주 + 빌드 프로파일 + WSL2 green + PGO 시도 모두)
- Jeffrey가 migration 승인 권한 (자동 migration 없음)
- 트리거 조건 충족 후에도 수동 확인 (Observer가 "consider", "required" 만 권고)

**Day-1 조치**: Migration 조건에 4개 AND 엄격. Observer는 권고만.

## 20. 리스크 우선순위 요약

**Day-1 릴리스 전 필수 대응**:
- T-1 (빌드 시간): `profile.release-fast` 추가
- T-5 (Claude Code 버전): validate_compatible 구현
- S-1 (API key 노출): 권한·gitignore·spend limit

**Day-1 릴리스 후 모니터링**:
- T-2 (Hook P99)
- T-7 (Variant B 준수율)
- S-2 (audit chain 무결성)

**장기 관찰**:
- P-3 (Anthropic 정책)
- P-4 (Claude Code 폐기)
- O-1 (lesson 누적)

## 21. Risk Register 관리

이 문서는 **초기 리스크 지형도**. 실제 운영 중 새 리스크 발견 시 여기 추가:

```markdown
## NN. [T/P/S/O]-N: <title>
**확률**: ...
**영향**: ...
**조기 경고**: ...
**대응**: ...
**최초 식별**: YYYY-MM-DD
```

**매 Milestone 전환 시 리스크 재평가**. 해결된 리스크는 **삭제하지 않고** "Resolved" 표시 + 해결 방법 보존.

## 관련 문서

- `~/myth/docs/10-VALIDATION.md` — 검증으로 리스크 완화
- `~/myth/docs/12-DEPLOYMENT.md` — 운영 중 모니터링
- `~/myth/DECISIONS.md` — 리스크 기반 설계 결정 이력
