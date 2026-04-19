# myth — Third-Party Attributions

myth는 여러 오픈 소스 프로젝트의 코드·규칙·지식을 차용한다. 이 문서는 **모든 외부 차용의 출처·라이선스·사용 범위**를 박제한다. myth 자체는 `MIT OR Apache-2.0` 듀얼 라이선스다.

---

## 0. myth 자체 라이선스

```
MIT License OR Apache License 2.0
Copyright (c) 2026 Jeffrey (Di-Vernon)
```

사용자는 둘 중 원하는 라이선스를 선택해 사용 가능.

---

## 1. Bedrock Rule 정규식 — gitleaks

**프로젝트**: https://github.com/gitleaks/gitleaks  
**라이선스**: MIT  
**버전**: v8.x (2024~2025년)  
**사용 범위**: 40개 anchored secret detection rule

### 차용 내용

Decision 5에서 채택한 Bedrock Rule 47개 중 **Production secrets commit (R2-A, R2-B)**에 해당하는 anchored provider prefix 패턴:

- AWS access key prefix (`AKIA`, `ASIA`, `AGPA`, `AIDA`, ...)
- GitHub PAT (`ghp_`, `github_pat_`, `gho_`, ...)
- Slack token (`xoxb-`, `xoxp-`, `xoxa-`, ...)
- Stripe (`sk_live_`, `pk_live_`, `rk_live_`, ...)
- 기타 37개 provider

### 원본 파일

gitleaks의 `config/gitleaks.toml`에서 각 rule의 `regex` 필드를 차용. 정규식 자체는 단순 prefix 매칭이라 저작권 주장 대상이 아닐 수 있으나, **큐레이션 결과물의 권위**를 존중하여 MIT 출처 명기.

### 각 rule에 출처 표시

```yaml
# ~/.myth/bedrock-rules.yaml
rules:
  - id: R2-A
    item: production_secrets_commit
    pattern: 'AKIA[0-9A-Z]{16}'
    source: "gitleaks v8.x (MIT) - aws-access-key"
    level: 5
```

### MIT 라이선스 전문

```
MIT License

Copyright (c) 2019 Zachary Rice

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OF OTHER DEALINGS IN
THE SOFTWARE.
```

---

## 2. Keyword+Entropy 패턴 — detect-secrets

**프로젝트**: https://github.com/Yelp/detect-secrets  
**라이선스**: Apache License 2.0  
**사용 범위**: Bedrock Rule R2-C (keyword + entropy 기반 탐지)

### 차용 내용

Decision 5의 R2-C rule — 변수명이 `password`, `secret`, `api_key` 등을 포함하면서 값이 고엔트로피일 때 매칭:

```regex
(?i)(password|passwd|secret|api[_-]?key|token|auth)\s*[:=]\s*["']?([A-Za-z0-9+/=_\-]{20,})["']?
```

detect-secrets의 **KeywordDetector** 로직을 개념적으로 차용. 코드 직접 복사는 아니고 **접근 방식 참조**.

### Apache-2.0 라이선스 핵심 조건

- 사용·수정·배포 자유
- NOTICE 파일 제공 (이 문서가 그 역할)
- 변경 사항 명시 (해당 없음 — 개념만 참조)

### 필수 출처 명기

```yaml
# ~/.myth/bedrock-rules.yaml
rules:
  - id: R2-C
    item: production_secrets_commit
    pattern: '(?i)(password|passwd|secret|api[_-]?key|token|auth)\s*[:=]\s*["'']?([A-Za-z0-9+/=_\-]{20,})["'']?'
    source: "detect-secrets (Apache-2.0) - keyword+entropy approach"
    level: 4
```

---

## 3. 임베딩 모델 — multilingual-e5-small

**프로젝트**: intfloat/multilingual-e5-small (Microsoft Research)  
**라이선스**: MIT  
**버전**: 2024년 초 최신 weights  
**사용 범위**: myth-embed 데몬의 유일 모델

### 차용 내용

HuggingFace Hub에서 다운로드하는 ONNX int8 변환본. myth는 **모델을 수정하지 않고** 그대로 호출.

### 원본 출처

- 논문: "Multilingual E5 Text Embeddings: A Technical Report" (arXiv:2402.05672)
- HuggingFace: https://huggingface.co/intfloat/multilingual-e5-small
- ONNX 변환: Qdrant, fastembed-rs 등이 제공하는 공식 변환본

### 다운로드 정책

첫 `myth-embed` 실행 시 자동 다운로드. 사용자 홈의 캐시 디렉토리에 저장:

```
~/.myth/embeddings/models/multilingual-e5-small/
├── model.onnx              # ~116 MB
├── tokenizer.json
└── config.json
```

SHA-256 해시 검증. 다운로드 실패 시 에러 명확 안내.

---

## 4. Rust crate dependencies

myth는 많은 Rust crate에 의존한다. 모두 OSI 승인 라이선스 (MIT, Apache-2.0, BSD-3-Clause 또는 그 조합).

### 주요 의존성 (직접)

| Crate | 버전 | 라이선스 | 용도 |
|---|---|---|---|
| `rusqlite` | 0.31 | MIT | SQLite 접근 |
| `fastembed` | 5.x | Apache-2.0 | 임베딩 추론 |
| `ort` | 2.0-rc | MIT/Apache-2.0 | ONNX Runtime bindings |
| `tokio` | 1.x | MIT | async runtime |
| `bincode` | 1.x | MIT | 직렬화 |
| `ratatui` | 0.26 | MIT | TUI |
| `crossterm` | 0.27 | MIT | 터미널 |
| `syntect` | 5.x | MIT | 구문 강조 |
| `pulldown-cmark` | 0.10 | MIT | 마크다운 |
| `clap` | 4.x | MIT/Apache-2.0 | CLI 파싱 |
| `serde` | 1.x | MIT/Apache-2.0 | 직렬화 |
| `tracing` | 0.1 | MIT | 로깅 |
| `regex` | 1.x | MIT/Apache-2.0 | 정규식 |
| `regex-automata` | 0.4 | MIT/Apache-2.0 | 사전 DFA |
| `sha1` | 0.10 | MIT/Apache-2.0 | SHA1 해싱 |
| `blake3` | 1.x | CC0/Apache-2.0 | audit chain |
| `memmap2` | 0.9 | MIT/Apache-2.0 | mmap |
| `simsimd` | 4.x | Apache-2.0 | SIMD 벡터 연산 |
| `uuid` | 1.x | MIT/Apache-2.0 | UUID 생성 |
| `chrono` | 0.4 | MIT/Apache-2.0 | 시간 |
| `thiserror` | 1.x | MIT/Apache-2.0 | 에러 |
| `anyhow` | 1.x | MIT/Apache-2.0 | 에러 |
| `mimalloc` | 0.1 | MIT | allocator |
| `which` | 6.x | MIT | 바이너리 탐색 |
| `dirs` | 5.x | MIT/Apache-2.0 | XDG 경로 |
| `nix` | 0.28 | MIT | Unix API |
| `once_cell` | 1.x | MIT/Apache-2.0 | 지연 초기화 |

### 전이 의존성

`Cargo.lock`에 전체 목록. 수백 개의 indirect 의존성. 전부 OSI 승인 라이선스.

전체 라이선스 감사:
```bash
cargo install cargo-license
cd ~/myth/rust
cargo license --json > licenses.json
# 혹은 보고서
cargo license > LICENSES.txt
```

`myth install` 시 이 검사 자동 수행 권장. GPL/LGPL/AGPL 포함 시 경고.

---

## 5. Python dependencies

### 주요 의존성

| 패키지 | 버전 | 라이선스 | 용도 |
|---|---|---|---|
| `anthropic` | ^0.42 | MIT | Anthropic SDK (Milestone A 활성 시) |
| `pydantic` | ^2.7 | MIT | 데이터 검증 |
| `typer` | ^0.12 | MIT | CLI |
| `pyyaml` | ^6.0 | MIT | YAML 파싱 |
| `rich` | ^13.7 | MIT | 터미널 출력 |
| `python-dateutil` | ^2.9 | Apache-2.0/BSD-3 | 시간 파싱 |

모두 MIT, Apache-2.0, BSD-3 계열.

---

## 6. 연구 문헌 차용

myth의 설계 결정은 다음 문헌들의 아이디어를 차용한다. 학술 citation.

### 법학·거버넌스

- **Cesare Beccaria**, *Dei delitti e delle pene* (1764) — "처벌의 확실성"
- **Montesquieu**, *De l'esprit des lois* (1748) — 권력 분립
- **Ayres & Braithwaite**, *Responsive Regulation* (1992) — 규제 피라미드
- **US Sentencing Guidelines**, Amendment 821 (2023) — 양형 매트릭스

### AI 안전·평가

- Li et al., *Risk-Adaptive Evaluation of LLM Safety* (arXiv:2501.03444, 2026) — 5단계 Level 스케일
- Panickssery et al., *LLM Self-Preference in Judging* (2024) — 헤테로지니어스 앙상블
- ACE Team, *Adaptive Context Embedding* (arXiv:2510.04618, 2025) — identity hash 다층 구조
- Sentry, *Seer: Smart Issue Deduplication* (blog post, 2025-02) — 임베딩 기반 이슈 dedup

### 의료 분류

- Forrey et al., *NCC MERP Taxonomy Consistency Study* (2007) — 5-tier 인터레이터 신뢰도
- Snyder et al., *Medication Error Severity Classification* (2007)

### 기술 문서

- Claude Code 2.1.x official docs (Anthropic)
- fastembed-rs README + examples (Qdrant)
- regex-automata 논문 및 rust-lang/regex 레포
- Internet Draft RFC 8265, Unicode normalization

모든 인용은 **개념 차용**. 문장 복제 없음. 학술 fair use 범위.

---

## 7. 기존 Jeffrey 자산 재활용

myth는 Jeffrey의 기존 작업물 일부를 재사용한다.

### harness-orchestrator

**위치**: `~/project/harness-orchestrator/`  
**원작자**: Jeffrey  
**라이선스**: (개인 프로젝트, myth와 동일 듀얼 라이선스 적용)

**재사용 대상**:
- `lib/execute.sh` — 병렬 실행 엔진 로직
- `lib/worktree.sh` — git worktree 관리
- `lib/watchdog.sh` — 타임아웃 감지
- `lib/ui.sh` — gum 기반 UI 헬퍼
- `lib/state.sh` — 상태 파일 관리

이들은 `~/myth/rust/crates/myth-orchestrator/scripts/`로 복사·편입. Jeffrey 자신의 저작물이므로 라이선스 문제 없음.

### harness-template

**위치**: `~/project/harness-template/`  
**재사용 대상**:
- 5개 expert agents (assessor와 observer의 원형 아이디어)
- 4개 slash commands (일부가 myth CLI로 재탄생)

재작성 수준으로 변형되지만 아이디어 계보를 여기 기록.

### MYTH-ULTRAPLAN.md (v1)

**위치**: `~/project/harness-orchestrator/MYTH-ULTRAPLAN.md`  
**재사용**: 전체 설계 이력의 60%를 Decision 1~7에서 재작성. v1은 `MYTH-ULTRAPLAN-v1.md`로 아카이브.

### CONSTITUTION.md

**위치**: 기존 `~/project/harness-orchestrator/CONSTITUTION.md` v2.2  
**재사용**: myth 본체 `~/myth/CONSTITUTION.md`로 이전. v2.3으로 개정 (Lapse 관련 최소 수정).

---

## 8. 로고·아이콘

myth는 현재 로고 없음. 장기적으로 추가 시:
- 자체 디자인 또는
- 명확히 CC-BY 또는 public domain 소스

---

## 9. 감사 절차

### 9.1 사전 감사 (개발 단계)

`cargo license` 및 `pip-licenses` 주기 실행:

```bash
# Rust
cd ~/myth/rust
cargo license --json > /tmp/rust-licenses.json

# Python
cd ~/myth/python
pip install pip-licenses
pip-licenses --format=json > /tmp/python-licenses.json
```

**금지 라이선스** 확인 (myth는 copyleft 배척):
- GPL (v2, v3)
- LGPL (any version)
- AGPL
- Proprietary/commercial

### 9.2 릴리스 전 체크

```bash
~/myth/scripts/license-audit.sh
# 출력:
# Rust dependencies: 342 total
#   MIT: 201
#   MIT OR Apache-2.0: 89
#   Apache-2.0: 42
#   BSD-3-Clause: 8
#   Unlicense: 2
# 
# Python dependencies: 6 top-level, 34 transitive
#   All permissive
# 
# No copyleft dependencies detected. OK to release.
```

### 9.3 NOTICE 파일

Apache-2.0 요구사항. `~/myth/NOTICE`:

```
myth
Copyright (c) 2026 Jeffrey (Di-Vernon)

Licensed under either of Apache License 2.0 or MIT License.

This product includes software developed by:
  - gitleaks (https://github.com/gitleaks/gitleaks)
  - detect-secrets (https://github.com/Yelp/detect-secrets)
  - multilingual-e5-small (Microsoft, HuggingFace)
  - various Rust crates (see Cargo.lock)
  - various Python packages (see pyproject.toml)

See THIRD-PARTY.md for full attribution.
```

---

## 10. 변경 이력

| 날짜 | 변경 |
|---|---|
| 2026-04-19 | 초기 작성. gitleaks/detect-secrets/fastembed 주요 3종 귀속. |

---

## 참고

- **myth 본체 라이선스**: `~/myth/LICENSE` (MIT 전문과 Apache-2.0 전문 각각)
- **cargo-license 보고서**: `~/myth/LICENSES.txt` (빌드 시 자동 갱신)
- **NOTICE 파일**: `~/myth/NOTICE`
