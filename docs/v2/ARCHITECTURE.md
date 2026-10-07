# myth v2 — Architecture

- 상태: 초안 (Phase B 산출물). Phase C 독립 리뷰 전이다.
- 날짜: 2026-10-07
- 기준 자료: `MYTH-V2-HANDOFF.md`(이하 핸드오프), `REPORT-v2-R1.md`(R1), `REPORT-v2-R2.md`(R2), [ADR-0001 ~ ADR-0017](README.md)
- 용어: myth 신조어는 원문으로 쓴다([ADR-0004](adr/0004-terminology.md)). 이 문서에서 새로 쓰는 용어는 §13에 따로 모았다.

## 0. 이 문서를 읽는 법

이 문서는 ADR들을 하나의 동작하는 시스템으로 엮는다. ADR과 이 문서가 다르면 ADR이 우선한다. 이 문서가 결정을 뒤집을 일이 생기면 새 ADR을 쓴다.

문서의 각 내용에는 출처를 붙였다.

| 표기 | 뜻 |
|---|---|
| `[ADR-NNNN]` | 확정된 결정 |
| `[H§n]` | 핸드오프의 합의된 방향. 아직 ADR로 확정하지 않았다 |
| `[제안]` | 이 문서에서 처음 제안하는 통합안. ADR로 확정하기 전까지는 잠정이다 |
| `[열림 Kn]` | 결정이 필요한 공백 또는 충돌. §12에 모았다 |

## 1. 의도 (불변) [H§1]

myth는 로컬에서 Claude Code가 **스스로 진화·수정·발전**하게 하는 **second brain이자 구속구**다.

- 치명적 실수의 재발을 0으로 만들지 않는다. **재발 빈도를 떨어뜨린다.**
- 실수를 **미연에 방지**한다.
- **발생 패턴을 스스로 분석**해서, 비슷한 작업을 할 때 **더 주의**하게 한다.
- 1차 사용자는 공개 OSS 사용자다. 일정 제약은 없고, 설계 품질을 우선한다.

## 2. 불변 원칙 [H§2]

- **Master Principle**: "완벽은 도달이 아니라 수렴이다. 수렴은 우연이 아니라 법이다."
  - Test 1: 수렴을 이끄는가, 자의성을 허용하는가?
  - Test 2: 체계를 강화하는가, 체계 밖 예외를 만드는가?
- **임의 수치 금지**: 모든 수치에 출처를 단다. 출처가 없으면 "설정 가능 파라미터 + 보정 실험"으로 둔다(§10).
- **실패 처리**: 보안 판정은 실패하면 deny한다. 관찰 기록은 실패하면 degrade한다.
- **측정된 병목만 최적화한다.**
- docs-first. 커밋 1개 = 결정 1개.

## 3. 정체성: common-law compiler [H§8]

myth는 집행기가 아니다. **집행은 플랫폼(Claude Code)에 맡긴다.** myth가 하는 일은 다음 넷이다.

1. **사건 수집**: evidence sensor로 실수의 증거를 모아 case file로 남긴다.
2. **판례 추출**: 같은 패턴이 반복되면 holding을 만들고, 사용자 ratification을 거쳐 precedent로 만든다.
3. **성문화**: precedent를 codification ladder에서 가장 강하면서도 맞는 자리에 컴파일한다.
4. **승급·강등**: 재발, override, 폐용(disuse)을 측정해서 ladder를 오르내린다.

법 은유는 규칙의 생애주기를 설명하는 설계 언어다. 런타임 컴포넌트 이름으로는 쓰지 않는다.

### 의도와 메커니즘의 대응

| 의도 [H§1] | 메커니즘 | M0 포함 |
|---|---|---|
| 재발 빈도를 떨어뜨린다 | precedent → ladder 집행, 재발률 측정 | O (L2까지) |
| 미연에 방지한다 | L2 선택적 경고, L3 확인, L4 차단, restraint lock | 일부 (L2, restraint lock) |
| 보고 전에 잡는다 | audit A0 (claim과 증거 대조) | O |
| 결과를 빨리 판단하게 한다 | completion report, change tree | O |
| 패턴을 스스로 분석한다 | 구조 signature 군집, 패턴 분석기, 위험 프로필 | 군집만 [열림 K1, K6] |
| 비슷한 작업에서 더 주의한다 | 기억(recall) 채널, 위험 프로필 주입 | X [열림 K6] |

## 4. 전체 구조

```
┌──────────────────────────── Claude Code (v2.1.287+) ─────────────────────────────┐
│                                                                                  │
│  정적 설정 (mod가 없어도 Claude Code가 직접 읽음)                                 │
│   ~/.claude/rules/myth/*.md ......... L1                                         │
│   ~/.claude/settings.json                                                        │
│     permissions.deny ................ L4, restraint lock                         │
│     autoMode.soft_deny / hard_deny .. L3 / L4                                    │
│                                                                                  │
│  myth plugin                                                                     │
│   ┌─ mod adapter (얇음, `$` 의존) ────────────────────────────────────────────┐  │
│   │ session.start · prompt.submit · tool.check · tool.call · classic.*        │  │
│   │ turn.complete · session.compact · session.end · /myth 명령 · $.ui         │  │
│   └──────────────┬────────────────────────────────────────────────────────────┘  │
│                  ▼                                                               │
│   ┌─ core (순수 TS, `$` 비의존) ──────────────────────────────────────────────┐  │
│   │ recorder → signals → cases → grouping → promotion → ladder → mirror       │  │
│   │ audit(A0/A1) · report(completion report, change tree) · enforce(L2~L4)    │  │
│   │ counters · budget · redaction · config                                    │  │
│   └──────────────┬────────────────────────────────────────────────────────────┘  │
│                  ▼                                                               │
│   $.store (재생성 가능한 캐시)          $.model.fork (holding 초안만)            │
└──────────────────┼───────────────────────────────────────────────────────────────┘
                   ▼
          ~/.myth/ (진실원, 불변 파일 event sourcing)
```

- **배포**: plugin 하나에 mod, skills, 필요한 settings hook을 담는다. marketplace의 `/plugin install`로 설치한다. 외부 런타임은 없고, 의존성은 0을 원칙으로 한다. [ADR-0013]
- **core / adapter 분리**: 도메인 로직은 `$`에 의존하지 않는 순수 TS다. mod는 이벤트를 core로 넘기는 adapter만 맡는다. mod API가 바뀌어도 adapter만 고치면 된다. [ADR-0013]
- **v1 코드는 재사용하지 않는다.** v1에서 얻은 교훈(redaction 패턴, 운영 입력 형태로 검증하기, fail-safe/degrade 구분)만 옮긴다. [ADR-0013, H§4]
- **이식성**: Claude Code 전용이다. [ADR-0013]
- core 안의 모듈 경계(recorder, signals 등)는 [제안]이다. 구현에서 이름과 경계가 바뀔 수 있다.

## 5. 데이터 모델

### 5.1 저장 위치와 원칙 [ADR-0014]

```
~/.myth/
├── VERSION
├── events/<날짜>/<session>-<chunk>.jsonl   해당 세션만 씀. 턴 단위 청크
├── cases/<case-id>.json                    사건 1건 = 파일 1개
├── decisions/<decision-id>.json            결정 1건 = 파일 1개. 덮어쓰지 않음
├── counters/<session>.json                 세션별 증분. 읽을 때 합산
└── audit/                                  A1 라벨
```

- 사용자 전역이다. 프로젝트 안에 두지 않는다.
- 누적 단위는 사용자 하나다. holding에는 scope 태그를 단다. 기본값은 `global`이다.
- 모든 레코드는 같은 envelope를 쓴다: `{ v, id, ts, session, project, type, payload, sha256 }`.
  - `id`는 시간순으로 정렬되는 고유 ID다.
  - `sha256`이 맞지 않는 파일은 잘린 쓰기로 보고 무시한다(관찰은 degrade).
- 파일 하나에는 쓰는 주체가 하나뿐이다. 그래서 잠금이 필요 없다.
- `$.store`는 재생성 가능한 캐시다. 활성 precedent와 합산 counters 같은 projection만 둔다.
- 원문을 저장하되, 비밀값 패턴만 가린다. 기본으로 켜져 있고 끌 수 있다.
- 원시 이벤트는 설정한 기간만 보존한다(기본값은 S1 실측 후 결정). case와 decision은 영구 보존한다.
- `project` 필드: git 저장소 루트 경로의 해시. 정규화한 remote URL을 별칭으로 둔다. git이 아니면 작업 디렉터리 경로의 해시를 쓴다.

### 5.2 event

hook 이벤트 하나를 정규화한 기록이다. 세션 메모리에 버퍼링했다가 턴이 끝나면 청크 파일 하나로 쓴다. [ADR-0014]

| `type` (예) | 출처 이벤트 | payload 요지 |
|---|---|---|
| `prompt` | prompt.submit | 사용자 원문(가림 적용), 프롬프트에 명시된 경로·파일명 [ADR-0007] |
| `tool_pre` | tool.check / tool.call | 도구, 입력, matcher 적중 결과, 내린 판정 |
| `tool_post` | classic.PostToolUse / Failure | exit code, 변경 파일, 오류 여부, interrupt 여부 |
| `permission_denied` | classic.PermissionDenied | 거절 사유 |
| `turn_end` | turn.complete / classic.Stop | 응답 원문, completion report 파싱 결과, change tree 데이터 |
| `signal` | core.signals | 감지된 행동 신호(§6.2) |
| `audit_flag` | core.audit | A0 검사 결과(§7) |

### 5.3 case file

사건 하나의 기록이다. 한 번 쓰면 바꾸지 않는다. [ADR-0004, ADR-0005, ADR-0014, H§8.2]

```jsonc
{
  "id": "…",
  "provenance": "user_signal | permission | interrupt | verification | audit",
  "trust": "normal | low",            // audit이면 low [ADR-0005]
  "signals": ["redo", "revert"],      // 신호별로 따로 기록. 하나의 점수로 합치지 않음 [H§8.1]
  "facts": ["<event id>", "…"],       // 증거 포인터
  "evidence_spans": ["<사용자 원문 정확 인용>"],
  "signature": "…",                   // 구조 signature [열림 K1]
  "project": "…",
  "session": "…"
}
```

- case file은 **holding을 갖지 않는다.** holding은 승격 제안 시점에 여러 case를 묶어서 만든다(§6.4). 이것은 핸드오프 §8.2의 "case file 안에 holding" 구조와 다르다. ADR-0015에서 LLM 사용처를 승격 시점 한 곳으로 정했기 때문이다.

### 5.4 holding / precedent

holding은 같은 signature로 묶인 case들에서 만든 초안이다. ratification을 받으면 precedent가 된다. [ADR-0004, H§8.2]

```jsonc
{
  "id": "…",
  "holding": "<명령형 한 문장>",
  "polarity": "constrain | prescribe | restrain",
  "persistence": "once | session | durable",
  "scope": { "kind": "global | project", "project": "…", "paths": ["…"], "tools": ["…"], "commands": ["…"] },
  "matcher": { "tool": "…", "path_glob": "…", "command_pattern": "…" },   // 결정적 matcher
  "cases": ["<case id>", "…"],
  "evidence_spans": ["<사용자 원문 정확 인용>"],                          // 없거나 원문과 다르면 무효 [ADR-0015]
  "counter_examples": ["…"],
  "rung": "L0 | L1 | L2 | L3 | L4",
  "state": "proposed | active | demoted | retired | regressed | rejected",
  "counters": { "opportunity": 0, "exposure": 0, "suppressed": 0,
                "recurrence_user": 0, "recurrence_intercepted": 0, "override": 0 },
  "evaluation": "holdout | user_judgment"                                    // [H§8.8]
}
```

- precedent의 상태 변화는 `decisions/`에 decision 레코드로 남긴다. 현재 상태는 decision들을 순서대로 적용한 projection이다. [ADR-0014]
- scope 기본값은 `global`이다. holding이 특정 경로, 명령, 도구를 직접 가리킬 때만 project scope를 단다. [ADR-0014]

### 5.5 counters [H§8.2, ADR-0012, 정의는 제안]

| counter | 정의 [제안] |
|---|---|
| opportunity | precedent의 matcher가 tool call에 적중한 횟수. 개입 여부와 무관하다 |
| exposure | 적중해서 실제로 개입(L2 경고, L3 확인, L4 차단)이 나간 횟수 |
| suppressed | 적중했지만 intervention budget을 넘어 개입을 억제한 횟수. exposure와 따로 센다 [ADR-0012] |
| recurrence_user | precedent가 active인 동안, 같은 signature의 case가 사용자 신호로 새로 생긴 횟수 [ADR-0012] |
| recurrence_intercepted | 같은 signature의 사건을 myth의 개입이나 audit이 먼저 잡은 횟수 [ADR-0012] |
| override | L3: 사용자가 확인 요청을 승인한 횟수. L2: 경고 뒤 같은 행동이 그대로 실행된 횟수. L2 override의 해석은 [열림 K9] |

- **1급 지표는 재발률 = 재발 ÷ opportunity다.** [H§8.8]
- counters는 세션별 파일로 쓰고, 읽을 때 합산한다. [ADR-0014]

## 6. 핵심 루프: 사건 → precedent

```
행동 신호 ──► case file ──► signature 군집 ──► 승격 조건 충족
 (매 턴, 결정적)  (조용히 쌓임)   (결정적)            │
                                                 ▼
                         $.model.fork로 holding·matcher 초안 (드묾)
                                                 │
                         evidence span 검증 · matcher 검증 (결정적)
                                                 │
                         ratification (사용자, intervention budget 안에서)
                                                 │
                         precedent ──► ladder 컴파일 ──► 집행 ──► counters
                                                 ▲                      │
                                                 └──── 승급·강등 ◄──────┘
```

### 6.1 M0 최소 루프

M0는 닫힌 피드백 루프 하나부터 만든다. [H§10, ADR-0007]

> 행동 신호 감지 → case file → signature 군집 → 승격 제안(fork 초안) → ratification → L2 경고 → 재발률 측정

핸드오프의 원래 루프는 "교정 감지 → case 기록 → holding 추출 → 사용자 비준 → L2 경고 → 재발률 계측"이다. ADR-0015와 ADR-0016을 반영하면 위와 같이 바뀐다. 여기에 audit A0·A1을 병행한다. [ADR-0007]

### 6.2 감지: 행동 신호만 [ADR-0016]

초기 버전은 모든 언어에서 어휘 사전 없이 행동 신호만 쓴다.

| 신호 | 감지 기준 | 대상 |
|---|---|---|
| audit A0 | claim과 증거의 대조(§7) | 거짓 완료 |
| 오류 붙여넣기 | 사용자 프롬프트 안의 오류 출력·스택 트레이스 구조 패턴 | 거짓 완료, 실패 보고 |
| rework | 완료 claim 직후 같은 파일을 다시 수정하거나 같은 테스트를 다시 실행 | 거짓 완료 |
| redo | 사용자 턴 직후, 직전 행동을 같은 도구·같은 대상에 다른 인자로 다시 함 | 말로 한 교정 |
| revert | `git restore` / `checkout` / `revert`, 직전 변경 원복 | 교정, 거부 |
| interrupt | 사용자 중단 | 거부 |
| permission denial | auto mode 차단, 권한 거절. 차단 뒤 승인은 오탐 신호로 따로 기록 [H§8.1] | 거부, 오탐 |
| 검증 결과 | test/lint/build가 통과→실패로 바뀜, 같은 파일 반복 수정, 같은 시도 3회 이상 | 결함 [H§8.1] |

- 도구 오류는 case로 만들지 않는다. 루프 감지 신호로만 쓴다. [H§8.1]
- 신호는 하나의 점수로 합치지 않는다. 신호별로 따로 기록한다. [H§8.1]
- 행동 신호는 정상적인 반복 개발과도 겹친다. 그래서 후보의 정밀도는 낮다. 후보는 조용히 쌓이고, 승격에는 반복과 ratification이 필요하므로 이 비용은 감당할 수 있다. [ADR-0016]
- 말로만 드러난 교정은 놓친다. 그 비율은 E2a로 추정한다. [ADR-0016]
- 감지 단계에는 LLM 호출도, 사용자 질문도 없다. [ADR-0015]

### 6.3 군집: 구조 signature

- "같은 패턴인가"는 결정적으로 판단한다. 기준은 "Claude가 무엇을 했는가"의 구조 사실(도구, 경로, 명령)이다. [ADR-0015]
- signature의 구체적 정의(어떤 필드를 어떻게 정규화할지)는 아직 없다. [열림 K1]
- audit incident 하나만으로는 holding 후보가 되지 않는다. 같은 signature에 독립 신호(사용자 신호, PermissionDenied 후 승인, interrupt, 검증 결과)가 1개 이상 있어야 한다. [ADR-0005]

### 6.4 승격 제안과 holding 초안 [ADR-0015]

1. 같은 signature의 case가 승격 조건을 채우면 제안 큐에 넣는다. 조건은 L0→L1 기준(같은 패턴 사건 ≥3건)을 따른다. [H§8.9]
2. 턴이 끝난 뒤 백그라운드에서 `$.model.fork`로 holding과 matcher 초안을 만든다.
   - fork는 세션 transcript와 prompt cache를 공유한다. 그래서 맥락을 갖고 있고, 비용이 낮다.
   - myth 코드에는 모델 ID를 하드코딩하지 않는다.
   - 초안에는 polarity, persistence, scope도 포함한다.
   - 기존 precedent와 겹치면 lifecycle 연산(NOOP / UPDATE / SUPERSEDE / SPLIT / NEW / NEEDS_USER / REJECT)을 함께 제안한다. [ADR-0001 차용 4]
3. 초안을 결정적으로 검증한다.
   - **evidence span**: 사용자 원문의 정확한 인용이 없거나, 인용이 원문과 다르면 무효다. [ADR-0015, ADR-0001 차용 1]
   - **matcher 검증**: 근거 case들의 실제 hook 입력(운영 입력 형태)에 matcher를 돌려서, 근거 case에 적중하는지 확인한다. v1 결함 #1(직렬화 문자열에 regex를 매칭함)과 근본 원인 6(검증 도메인 ≠ 운영 입력 형태)에 대한 대응이다. [제안, H§4, ADR-0001 차용 5]
   - **위험 규칙 거부**: 안전장치 해제, 파괴적 삭제 허용 같은 durable 규칙은 REJECT한다. [ADR-0001 차용 3]
4. fork가 null을 반환하면(cold snapshot, API 오류) 제안을 보류한다. 다음 턴이 끝난 뒤 다시 시도한다. [ADR-0015]
5. LLM 예산 [ADR-0015]
   - 구독 사용자: `$.session.usage().rateLimits` 사용률이 기준값 이상이면 미룬다.
   - API key 사용자: 일일 비용 상한으로 판단한다.
   - 세션당 호출 상한을 둔다.
   - 기준값은 모두 설정값이고, 초기값은 S1 실측 후 정한다.

### 6.5 ratification

- 사용자가 확정해야 precedent가 된다. [ADR-0015, H§8.7]
- 제안을 보여 주는 횟수는 intervention budget을 따른다. [ADR-0012, ADR-0015]
- 표시 방법 [제안]
  - 턴이 끝날 때 `$.ui.ask`로 승인 / 거부 / 수정을 묻는다.
  - budget을 넘었거나 `$.ui.ask`를 쓸 수 없는 환경(`claude -p`, SDK)이면 큐에 남긴다. `/myth review`로 나중에 처리할 수 있다.
- 비대칭 자율성 [H§8.7]

| 자동 | ratification 필요 |
|---|---|
| 기록, 분석 | durable 확정 |
| 승격 제안 | L3/L4 승격 |
| 효과 없는 경고 강등 | 규칙 폐기(retire) |
| regressed 복원 | 설정 변경 |

### 6.6 persistence

- `once`가 기본값이다. [H§8.2]
- `durable` 조건은 원래 "서로 다른 세션 2개 이상에서 독립 추출"이다. 그런데 추출은 승격 시점에 한 번만 일어난다. 그래서 "근거 case가 서로 다른 세션 2개 이상에서 나옴"으로 다시 정의해야 한다. [열림 K3]
- `session` persistence가 무엇을 하는지(세션 내 재주입, 특히 compaction 뒤)는 [열림 K5]와 함께 정한다.

## 7. audit

audit은 사용자 교정 없이 claim과 실행 증거를 대조하는 evidence sensor다.

### 7.1 지위 [ADR-0005, ADR-0006]

- 별도의 규칙집을 갖지 않는다. 규칙은 precedent에서만 나온다.
- 개입(피드백·차단)은 codification ladder의 한 칸으로 다룬다. 다른 칸과 같은 측정·강등 규칙을 적용한다.
- 산출물은 audit incident다(`provenance: audit`, `trust: low`).
- L3/L4 승격 근거를 셀 때 audit incident는 넣지 않는다.
- intent 오독은 audit 대상에서 제외한다. 대신 completion report와 change tree로 사용자가 직접 판단하게 한다.

### 7.2 단계 [ADR-0007]

| 단계 | 내용 | LLM 호출 | 시점 |
|---|---|---|---|
| A0 | 결정적 검사 | 0 | M0 |
| A1 | 라벨 수집과 정밀도 측정 | 0 | M0 |
| A2 | intent contract(필요할 때만 추출), scope 검사, LLM 2단계 판정, 실행 전 확인 질문 | 필요할 때만 | A1 데이터가 필요성을 보일 때 |

### 7.3 completion report [ADR-0009, ADR-0017]

작업 턴(파일 변경이 있었거나 완료 claim을 한 턴)에 Claude는 고정 형식으로 보고한다. 칸 제목은 영어로 고정하고, 서술은 사용자 언어로 쓴다.

```
## Done
## Verified
- `npm test` → exit 0
## Not done / Unverified
## Assumptions
```

- Verified에는 명령 실행으로 확인한 것만 쓴다. 한 줄에 하나씩, `` `명령` → exit N `` 형식이다.
- 형식 안내는 SessionStart에 한 번 짧게 넣는다. 매 프롬프트 상시 주입은 하지 않는다.
- 질문·답변만 오간 턴에는 형식을 요구하지 않는다.
- 변경 파일 목록은 Claude가 다시 쓰지 않는다. change tree를 참조한다.

### 7.4 A0 검사 [ADR-0008, ADR-0017]

| # | 검사 | M0 개입 수준 |
|---|---|---|
| 1 | Verified의 명령이 같은 턴의 Bash 기록에 없음, 또는 exit code가 다름 | 턴당 1회 피드백 |
| 2 | 작업 턴인데 칸 제목 4개가 없음 | 턴당 1회 피드백 |
| 3 | 완료 claim이 있는데 해당 턴에 테스트 실행 0회 | shadow mode |
| 4 | 피드백 직후 테스트 파일 변경 | shadow mode. gaming 의심으로 기록 |
| 5 | 같은 파일 반복 수정, 같은 시도 3회 이상 | 맥락 정보로만 사용 |

- 피드백은 Stop 시점의 `additionalContext`로 준다. 사실과 증거 위치만 적는다. 명령조가 아니라 사실 진술로 쓴다.
- 명세상 불가능하면 BLOCKED-IMPOSSIBLE과 근거를 보고해도 된다는 출구를 함께 준다. 테스트 기댓값이나 "통과시켜라" 같은 목표는 넘기지 않는다.
- `stop_hook_active=true`이면 피드백을 생략한다. 상한(턴당 1회, 세션당 3회)은 myth 자체 카운터로 강제한다. 플랫폼 Stop 상한(연속 8회)보다 항상 낮게 유지한다.
- Stop hook 하나가 #1~#4를 수행한다. LLM 호출은 없다.

### 7.5 change tree [ADR-0010]

파일 변경이 있었던 턴에는 myth가 Stop 시점에 change tree를 렌더링해 사용자에게만 보여 준다(`systemMessage`).

```
Changes: 3 files, +42 −7
src/
├── auth/
│   ├── login.ts      M  +30 −5
│   └── validate.ts   A  +12
└── legacy/
    └── old.ts        D  −2
```

- git 저장소: 턴 시작 기준점 대비 `--name-status`, `--numstat`
- git이 아닌 디렉터리: 이벤트 로그의 Edit/Write 기록. Bash로 바뀐 파일이 빠질 수 있다는 사실을 트리 하단에 표시한다.
- intervention budget에서 차감하지 않는다. Claude의 행동을 바꾸지 않기 때문이다.

### 7.6 A1 라벨 [ADR-0012]

- **수동 라벨**: audit flag 뒤 N턴(초기값 5) 안의 사용자 교정·되돌림·반대 발언을 자동으로 연결한다.
- **능동 라벨**: 세션 종료 시 묻는 기능이다. M0에서는 기본으로 꺼 둔다.
- 해석: flag 뒤 교정 → 진양성. flag 뒤 반대·되돌림 → 위양성. "flag 없음 ∧ 교정 없음"은 진음성으로 보지 않는다.
- 차단 권한 해제(A2 이후): 검사 항목별로 n≥25 ∧ 오탐 ≤1이면 1회 차단 후보, n≥55 ∧ 오탐 ≤1이면 반복 차단 후보다. 실제 해제는 ratification을 거친다. 최근 20건 이동 창에서 기준 아래로 떨어지면 자동으로 shadow mode로 돌린다.

## 8. codification ladder와 집행

### 8.1 단계와 컴파일 대상 [H§8.4, ADR-0013]

| 단계 | 내용 | mod 런타임 | 정적 미러 (mod가 없어도 동작) |
|---|---|---|---|
| L0 | 기록 | — | — |
| L1 | priming 규칙 | (해당 없음) | `~/.claude/rules/myth/<id>.md` + `paths` frontmatter [열림 K7] |
| L2 | 선택적 경고 | PreToolUse 시점 context 주입. matcher가 맞을 때만 | 없음 |
| L3 | 확인 | `tool.check` → `ask` | `autoMode.soft_deny` |
| L4 | 차단 | `tool.call` → `{ deny }` | `permissions.deny`, `autoMode.hard_deny` |

- polarity별 경로 [H§8.4]
  - `constrain`: 텍스트로도 효과가 있다. L1부터 시작할 수 있다.
  - `prescribe`: 상시 텍스트는 해가 될 수 있다. L2와 Stop 검증이 중심이다.
  - `restrain`: 텍스트로는 효과가 없다. L0에서 바로 L3 후보로 간다. 제안 시점의 정의는 [열림 K4]다.
- `autoMode.*`는 user/managed 범위에서만 읽는다. 그래서 myth는 `~/.claude/settings.json`을 편집한다. 동의, 백업, 되돌리기가 필요하다. [H§5]
- L2의 context 주입을 mod 이벤트로 할지 `classic.PreToolUse`의 `additionalContext`로 할지는 S1에서 확인한다. [제안]
- M0는 L2까지만 컴파일한다. ratification 직후 바로 L2로 가는 M0 루프는 §8.9의 L1→L2 조건(≥3 세션)과 충돌한다. [열림 K2]

### 8.2 승급·강등 파라미터 [H§8.9]

모두 설정 가능하고, 보정 대상이다.

| 전이 | 잠정 조건 | 근거 등급 | 보정 |
|---|---|---|---|
| L0→L1 | 같은 패턴 사건 ≥3건 | [D] Ratchet 전이 | — |
| L1→L2 | ≥3개 세션에서 확인 | [C] claude-learn | — |
| L2→L3 | L2 이후 재발 ≥2건, 또는 restrain 유형 | [D] | E1 |
| 강등 | opportunity ≥20회 동안 재발 0 | rule of three | E4 |
| 폐기 판단 | ≥100회 누적 | [A] 전이 | — |
| 상시 로드 규칙 상한 | ≤50 | [A] 단일 모델 | — |
| override 강등 임계값 | 근거 없음 | — | E3 |

- **regressed 복원**: 강등되거나 폐기된 precedent가 재발하면 즉시 이전 단계로 복원한다. 자동이다. [R1 Q1-e, H§8.7]
- **효과 측정**: holdout은 고빈도 precedent(주 10회 이상)에만 쓴다. restrain형 L4는 보류하지 않는다. 나머지는 사용자 판정으로 처리하고, 그 사실을 `evaluation` 필드에 남긴다. [H§7 보정 4, H§8.8, R1 Q1-f]
- M0 범위에서는 holdout을 구현하지 않는다. [제안]

### 8.3 intervention budget [ADR-0012]

- 세션 하나에서 사용자에게 보이는 개입의 합계 상한이다. 초기값은 6이다(근거 없음 → E-A4).
- 합산 대상: audit 피드백, L2 경고, L3 확인, ratification 제안 [ADR-0012, ADR-0015]
- 차감하지 않는 것: change tree(정보 제공), L4 차단과 restraint lock(보안 판정)
- 넘으면 새 개입은 shadow mode로 돌리고, `suppressed`로 따로 기록한다.

### 8.4 restraint lock [ADR-0011, H§8.6]

작업 중인 에이전트가 myth를 무력화하지 못하게 한다.

- **항상 막는 대상**: myth 설정·데이터(`~/.myth/**`, `~/.claude/rules/myth/**`), `.claude/settings*.json`, (A2 이후) intent contract 파일
  - mod의 `tool.call` deny와 정적 `permissions.deny`를 함께 건다. 정적 쪽이 degraded mode의 미러다.
  - 보안 판정이므로 판정이 실패하면 deny한다.
  - Bash를 통한 우회를 어디까지 막을 수 있는지는 [열림 K10]이다.
  - `~/.claude/rules/myth/**`를 포함하는 것은 [제안]이다. L1 정적 미러가 그 위치에 생기기 때문이다.
- **변경 경로**: myth의 변경은 Claude의 도구가 아니라 mod의 `$.fs`와 `/myth` 명령으로만 한다. 에이전트가 호출할 수 있는 자기 승인 CLI는 두지 않는다. [ADR-0001 피할 것]
- **설정 변경 감시**: `ConfigChange` hook으로 project/local 설정 변경을 막고 기록한다.
- **변조 점검**: SessionStart에서 myth 설정과 hook 스크립트의 해시를 대조한다. 불일치하면 변조 의심으로 표시한다.
- **설치 위치**: user scope.
- **테스트 파일은 막지 않는다.** 피드백 직후의 테스트 파일 변경을 gaming 의심으로 기록한다.

### 8.5 degraded mode [ADR-0013]

관리자 설정(`allowManagedModsOnly`, `allowManagedHooksOnly`, managed `disableAllHooks`)이나 실행 환경(Desktop 앱의 WSL 세션) 때문에 mod가 로드되지 않을 수 있다.

| 남는 것 (정적 설정) | 멈추는 것 |
|---|---|
| L1 규칙 파일 | evidence 수집, case file, 승격, ratification |
| L3 `autoMode.soft_deny` | L2 경고 |
| L4 `permissions.deny`, `autoMode.hard_deny` | audit, completion report 검사, change tree |
| restraint lock의 `permissions.deny` | counters, 측정 |

- 관리자에게는 조직 mod로 배포하는 방법(관리 디렉터리 marketplace + `prependPlugins`)을 안내한다.
- 정적 미러를 쓰는 경로는 안전해야 한다. `~/.claude/settings.json`은 원자적 쓰기가 없어서, Claude Code와 동시에 쓰면 손상될 수 있다. 쓰기 방법은 S1에서 정한다. [ADR-0013]

## 9. 이벤트별 처리 경로

각 경로의 hook 자체 실행 시간은 10초 제한이 있다. 무거운 작업은 타이머와 `$.model.*` 대기로 넘긴다. [ADR-0013]

| 이벤트 | 처리 | 실패 시 |
|---|---|---|
| `session.start` | 버전·해시 대조(변조 점검) · `$.store` 캐시 확인, 없거나 스키마가 다르면 `~/.myth/`에서 재구성 · 다른 세션이 쓴 새 파일 반영 · completion report 형식 안내 1회 주입 · 미처리 제안 큐 확인 | 관찰 → degrade. 변조 의심은 표시만 |
| `prompt.submit` | 프롬프트 기록(가림) · 명시 경로·파일명 추출 · 오류 붙여넣기 신호 · 직전 턴 행동과의 비교 준비(redo·revert 판정용) · A1 수동 라벨 연결 | degrade |
| `tool.check` | restraint lock 판정 · L4/L3 precedent 판정(`deny`/`ask`) · opportunity·exposure 기록 | **restraint lock·L4는 deny**. L3는 ask |
| `tool.call` | restraint lock deny · L2 matcher 판정과 경고(budget 확인) · 사전 기록 | restraint lock은 deny. L2는 생략 |
| `classic.PostToolUse` / `Failure` | 결과 기록(exit code, 변경 파일) · 검증 결과 전이 · 반복 수정·재시도 감지 · interrupt 감지 | degrade |
| `classic.PermissionDenied` | permission denial 신호 · 이후 승인 시 오탐 신호 | degrade |
| `turn.complete` / `classic.Stop` | completion report 파싱 · A0 검사와 피드백 · change tree 렌더링 · rework·redo·revert 신호 확정 · case file 생성 · signature 군집과 승격 조건 확인 · 이벤트 청크·counters 쓰기 · 승격 초안 작업 예약(백그라운드) | degrade. 피드백 생략 |
| `session.compact` | 기록만 [제안]. 재주입은 [열림 K5] | degrade |
| `classic.ConfigChange` | project/local 설정 변경 차단과 기록 | deny |
| `session.end` | 남은 버퍼 쓰기 · counters 최종 쓰기 · (설정 시) A1 능동 라벨 | degrade |
| 백그라운드 타이머 | `$.model.fork` 초안 → 결정적 검증 → ratification 대기 큐 | 보류 후 재시도 |

## 10. 파라미터

임의 수치 금지 원칙에 따라, 출처가 없는 값은 모두 설정 가능하게 두고 보정 실험을 붙였다.

| 파라미터 | 초기값 | 근거 | 보정 |
|---|---|---|---|
| L0→L1 사건 수 | ≥3 | [D] Ratchet | — |
| L1→L2 세션 수 | ≥3 | [C] | — |
| L2→L3 재발 수 | ≥2 | [D] | E1 |
| 강등 opportunity | ≥20, 재발 0 | rule of three | E4 |
| 폐기 판단 누적 | ≥100 | [A] 전이 | — |
| 상시 로드 규칙 상한 | ≤50 | [A] | — |
| holdout 대상 빈도 | 주 10회 이상 | [D] 계산 | E1 |
| override 강등 임계값 | 없음 | — | E3 |
| audit 피드백 상한 | 턴당 1, 세션당 3 | 근거 없음 | E-A4 |
| intervention budget | 세션당 6 | 근거 없음 | E-A4 |
| A1 연결 창 N | 5턴 | 근거 없음 | E-A5 |
| 1회 차단 후보 | n≥25, 오탐 ≤1 (Wilson ≥0.80) | [D] 계산. 목표 정밀도는 근거 없음 | E-A1, E-A5 |
| 반복 차단 후보 | n≥55, 오탐 ≤1 (Wilson ≥0.90) | 같음 | 같음 |
| 강등 이동 창 | 최근 20건 | 근거 없음 | E-A5 |
| LLM 예산 (rateLimits 기준, 일일 비용, 세션당 호출) | S1 후 | 근거 없음 | S1 |
| 원시 이벤트 보존 기간 | S1 후 | 근거 없음 | S1 |
| 비밀값 가림 | 켬 | [ADR-0014] | — |
| A1 능동 라벨 | 끔 | [ADR-0012] | E-A5 |

## 11. 실험과 spike

| ID | 목적 | 출처 | 시점 |
|---|---|---|---|
| **S1** | 세션당 저장 바이트 · 타이머 콜백 시간 제한 · `$.fs` 잘린 쓰기 · 정적 미러 쓰기 경로(`settings.json` 동시 쓰기) · npm vendoring · **user-level rules의 `paths` 동작** · **L2 주입 수단** · **fork 비용·지연** | ADR-0013. 굵은 항목은 [제안] | M0 첫 작업 |
| E2a | SWE-chat 영어 데이터에서 반발 턴 중 행동 신호가 하나도 없는 비율 → 사전을 뺀 비용 추정 | ADR-0016 | 구현 단계 (HF 이용 조건 동의, read 토큰) |
| E5 | 오탐 precedent의 과잉 적용 비용(matcher를 shadow로 돌려 잠재 노출 측정) | R1 §6 | M0 직후 |
| E1 | 같은 precedent의 L1 / L2 / L3 재발률 비교 | R1 §6 | 고빈도 precedent가 생긴 뒤 |
| E3 | override율 기반 강등 임계값 | R1 §6 | L2 precedent 노출 ≥20회 뒤 |
| E4 | 강등 증거 하한 (≥10 vs ≥20) | R1 §6 | 강등 후보가 생긴 뒤 |
| E-A1 | audit 검사 항목별 정밀도 | R2 §11 | M0 (A1 데이터) |
| E-A4 | 피드백 뒤 수정률, gaming, 상한값, BLOCKED-IMPOSSIBLE 사용률 | R2 §11 | M0 이후 |
| E-A5 | 라벨 수집량 | R2 §11 | M0 |
| E-A8 | 변조 방어 점검 | R2 §11 | M0 |

R1의 E2(한·영 하이브리드 감지 검증)는 ADR-0016으로 E2a로 대체됐다. E-A2, E-A3, E-A6는 A2 단계에서 다룬다.

## 12. 열린 항목

ADR들을 통합하면서 드러난 공백과 충돌이다. **M0 구현 전**에 정해야 하는 것은 K1~K4와 K10이다.

| # | 항목 | 내용 | 시점 |
|---|---|---|---|
| K1 | 구조 signature 정의 | "같은 패턴"을 판단할 필드와 정규화 규칙이 없다(ADR-0015는 원칙만 정함). 신호 종류, 도구, 대상 경로의 일반화 수준(파일 / 디렉터리 / glob), 명령의 정규화(앞 토큰, 플래그) 등을 정해야 한다. 너무 좁으면 3건이 모이지 않고, 너무 넓으면 다른 실수가 섞인다 | M0 전 |
| K2 | M0 루프 vs L1→L2 조건 | M0 루프는 ratification 직후 L2로 간다. §8.9는 L1→L2에 "≥3 세션 확인"을 요구한다. 선택지: (a) ratification 시 polarity에 따라 첫 단계를 정한다(constrain→L1+L2, prescribe→L2, restrain→L3 제안). (b) M0에서만 L1을 건너뛴다. (c) §8.9를 그대로 따르고 M0 루프를 고친다 | M0 전 |
| K3 | durable 조건 재정의 | 추출이 승격 시점 1회뿐이므로, "근거 case가 서로 다른 세션 2개 이상에서 나옴"으로 다시 정의한다 | M0 전 |
| K4 | restrain 직행의 제안 시점 | polarity는 fork 초안이 나와야 알 수 있다. restrain 후보를 몇 건부터 제안할지(L0→L1과 같은 ≥3건인지, 더 적은지)가 없다 | M0 전 |
| K5 | compaction 뒤 재주입 | 핸드오프 §5는 "대화 중 boundary는 저장되지 않고 compaction 때 소실될 수 있다"를 myth의 핵심 존재 이유로 들었다. 이를 다룬 ADR이 없다. 감지 단계에 LLM이 없으므로(ADR-0015), 재주입할 내용을 무엇으로 할지(교정 신호가 붙은 턴의 사용자 원문 인용 등)가 정해지지 않았다. `session` persistence의 의미도 함께 정한다 | M0 범위 여부부터 결정 |
| K6 | 기억 채널, 패턴 분석기, 위험 프로필 | 핸드오프 §8.3, §8.5의 방향은 있지만 ADR이 없다. 오탐 주입이 누락보다 해롭다는 근거가 있으므로, 도입하더라도 shadow로 먼저 측정해야 한다(E5와 같은 방식) | M0 이후 |
| K7 | L1의 위치와 project scope | user-level `~/.claude/rules/`는 공식 지원된다. 그러나 user-level 규칙의 `paths` frontmatter가 무시된다는 버그 보고가 있다(anthropics/claude-code#21858, 2.1.25 기준, open). 또 user-level 규칙의 `paths`는 모든 프로젝트에 적용되므로, project scope precedent를 L1로 내릴 위치가 없다. 프로젝트 안 `.claude/rules/`에 쓰면 ADR-0014(프로젝트 안에 두지 않음)와 충돌한다 | S1 후 |
| K8 | 정적 미러 쓰기 안전성 | `~/.claude/settings.json`에 원자적 쓰기가 없다. 백업, 되돌리기, 동시 쓰기 감지 방법이 필요하다 | S1 |
| K9 | override 정의 | L2 경고는 context일 뿐이어서, 경고 뒤 같은 행동이 실행된 것이 정당한 override인지 재발인지 구분하기 어렵다 | E3 전 |
| K10 | restraint lock의 Bash 우회 | 경로 기반 `permissions.deny`(Edit/Write 규칙)는 내장 파일 도구에 적용된다. Bash의 리다이렉션이나 `cp`·`sed -i`로 쓰는 것까지 막는다고 가정할 수 없다(플랫폼 사실 재확인 필요). mod의 `tool.call`에서 Bash 명령을 파싱해 보호 경로를 찾을 수는 있지만, 셸 파싱은 완전할 수 없다. 남는 방어는 SessionStart 해시 대조(탐지)다. sandbox 설정을 쓸지, 탐지로 만족할지 정해야 한다 | M0 전 |

참고로, 통합 과정에서 다음은 **충돌이 아님**을 확인했다.

- restraint lock의 정적 미러: ADR-0011이 이미 `permissions.deny`를 함께 걸도록 정했다. degraded mode에서도 남는다.
- ADR-0001의 차용 항목 중 "SQLite 진실원", "inbox + detached worker", "`claude -p` 재귀 방지"는 ADR-0013·0014로 형태가 바뀌었다. 각각 불변 파일 event sourcing, mod 타이머와 `$.model.fork`(하위 프로세스 없음)로 대체된다.
- ADR-0015의 분석 표에 남은 "어휘 패턴"은 ADR-0016으로 빠졌다. ADR-0015 본문의 "E2"는 E2a를 가리킨다.

## 13. 이 문서에서 처음 쓰는 용어

ADR-0004 용어집에 추가할 후보다. [제안]

| 용어 | 뜻 |
|---|---|
| signature | case를 군집하는 결정적 키. Claude가 한 행동의 구조 사실로 만든다 |
| static mirror | mod 없이도 Claude Code가 직접 읽는 정적 설정으로 컴파일한 precedent 사본 |
| degraded mode | mod가 로드되지 않아 정적 미러만 동작하는 상태 |
| rework / redo / revert | ADR-0016의 행동 신호 이름 |

## 14. 정직한 한계 [H§8.10 + 이후 결정]

- 바뀌는 것은 맥락과 정책이지 모델 가중치가 아니다.
- 처음 보는 유형의 실수는 첫 발생을 막지 못한다. 이건 auto mode가 1차 방어선이다.
- 행동으로 드러나지 않는 추론 실수는 Stop 검증으로 "사후, 단 보고 전"까지만 앞당길 수 있다.
- intent 오독은 audit이 잡지 않는다. 판단은 사용자에게 남는다. [ADR-0006]
- 말로만 드러난 교정은 놓친다. 그 비율은 E2a로 추정한다. [ADR-0016]
- 결정적 감지의 재현율은 근거가 없다. [ADR-0015]
- fork는 세션의 메인 모델을 쓴다. 호출당 비용이 작은 모델보다 클 수 있다. [ADR-0015]
- mod API는 신생이다. 변경 위험을 core / adapter 분리로 줄일 뿐 없애지는 못한다. [ADR-0013]
- 대부분의 임계값은 [C]/[D] 등급이거나 근거가 없다. 사용자 한 명의 데이터로 보정하므로, 고빈도 precedent를 빼면 통계적 판정이 어렵다.
