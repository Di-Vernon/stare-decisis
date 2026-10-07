# ADR-0014 — 저장 형식: `~/.myth/` 불변 파일 event sourcing

- 날짜: 2026-10-07
- 상태: 확정 (Jeffrey 비준)
- 관련: 핸드오프 §9-3, ADR-0013

## 맥락

mod 환경(ADR-0013)에서는 SQLite를 쓸 수 없다. 저장에 쓸 수 있는 두 API에는 다음 제약이 있다(Claude Code 공식 문서, v2.1.287 기준).

| 저장소 | 한도 | 제약 |
|---|---|---|
| `$.store` | 전체 4 MiB JSON | 어떤 세션도 `cleanupPeriodDays`(기본 30일) 동안 읽거나 쓰지 않으면 삭제된다. get→set이 원자적이지 않아 세션 간에는 마지막 쓰기가 이긴다 |
| `$.fs` | 파일당 4 MiB, 파일 수 제한 없음 | 쓰기가 원자적이지 않다. append, rename, delete가 없다 |

데이터량은 다음과 같이 추정했다. SWE-chat 기준으로 세션당 tool call이 약 59회(355,000 ÷ 6,000)다. 이벤트 1건을 약 300 B로 잡으면(설계 목표치, 실측 대상) 세션당 수십 KB이고, 하루 10세션을 써도 수백 KB 수준이다. 이 규모라면 SQLite의 트랜잭션·집계 쿼리·동시성 제어가 필요하지 않다.

## 선택

### 1. 위치: 사용자 전역 `~/.myth/`

- 프로젝트 안에 두지 않는다. 사용자 발언이 repo에 커밋될 위험을 없애고, 프로젝트를 가로지르는 패턴 분석을 가능하게 하기 위해서다.
- 팀 공유는 범위 밖이다. 개인 사용 경험이 쌓인 뒤 승인된 precedent만 내보내는 기능으로 확장한다.

### 2. 누적 단위: 사용자 하나

- 저장과 집계를 프로젝트별로 나누지 않는다. 대부분의 실수는 사용자의 작업 습관과 소통 방식에 붙어 다닌다.
- 대신 holding에 **scope 태그**를 둔다.
  - 기본값은 `global`이다.
  - holding이 특정 경로, 명령, 도구를 직접 가리킬 때만 project scope를 붙인다.
  - project key는 git 저장소 루트 경로의 해시이고, 정규화한 remote URL을 별칭으로 둔다. git이 아니면 작업 디렉터리 경로의 해시를 쓴다.
- 근거: 영속 메모리의 맥락 밖 과잉 적용이 최대 86.48%였다(BenchPreS, arXiv 2603.16557). 저장소 특화 메모리는 negative transfer를 일으켰다(arXiv 2604.14004). (R1)

### 3. 저장 내용: 원문 저장, 비밀값만 가림

- 내용은 원문 그대로 저장한다. 길이 상한은 파싱에 부담이 가지 않는 선에서만 둔다.
- **비밀값 패턴만 가린다**: API 키, 토큰, 개인 키, 비밀번호 할당문, 자격 증명이 포함된 DB URI. 기본으로 켜고, 설정으로 끌 수 있다.
- 가리는 이유는 세 가지다.
  - Claude Code transcript는 기본 30일 뒤 지워지지만 case file은 영구 보존된다.
  - case file 내용은 holding 추출 때 모델로 가고, 이후 context에도 다시 주입된다.
  - `~/.myth/`는 백업이나 동기화에 딸려 갈 수 있다.
- 비용은 정규식 한 번 통과 수준이고, holding 품질에는 영향이 없다.
- 패턴은 v1 NOTICE의 gitleaks 기반 패턴과 tellonce의 redaction 패턴(MIT)을 데이터로 옮겨 쓴다.

### 4. 구조와 형식

```
~/.myth/
├── VERSION
├── events/<날짜>/<session>-<chunk>.jsonl   해당 세션만 씀, 턴 단위 청크
├── cases/<case-id>.json                    사건 1건 = 파일 1개
├── decisions/<decision-id>.json            결정 1건 = 파일 1개, 덮어쓰지 않음
├── counters/<session>.json                 세션별 증분, 읽을 때 합산
└── audit/                                  A1 라벨 (ADR-0012)
```

- 모든 레코드는 같은 envelope를 쓴다: `{ v, id, ts, session, project, type, payload, sha256 }`.
  - `id`는 시간순으로 정렬되는 고유 ID다. 파일 이름이 겹치지 않으므로 세션 간 충돌이 없다.
  - `sha256`이 맞지 않는 파일은 잘린 쓰기로 보고 무시한다. 관찰 기록은 실패 시 degrade한다.
- append가 없으므로 세션 메모리에 버퍼링했다가 턴 단위 청크 파일로 새로 쓴다.
- 파일 하나에는 쓰는 주체가 하나뿐이다. 이벤트·counters는 해당 세션만 쓰고, case와 decision은 한 번 쓰면 바꾸지 않는다.
- 스키마가 바뀌면 이벤트는 그대로 두고, 읽을 때 변환해 projection을 다시 만든다.

### 5. `$.store`의 역할: 재생성 가능한 캐시

- 현재 상태 projection(활성 precedent, 합산 counters)과 인덱스만 둔다.
- 다른 세션이 새로 쓴 파일은 마지막으로 읽은 `id` 이후분만 읽어 반영한다.
- 캐시가 사라지거나 스키마 버전이 바뀌면 파일에서 전부 다시 만든다.

### 6. 보존과 정리

| 데이터 | 보존 |
|---|---|
| 원시 이벤트 | 설정 가능한 보존 기간. 근거 있는 기본값이 없으므로 spike S1의 실측 용량을 보고 정한다 |
| case, decision | 영구 보존. 양이 적고 holding의 근거이기 때문이다 |
| 정리 | delete API가 없으므로, OS 명령을 호출하는 `/myth gc` 명령을 선택 기능으로 둔다 |

## 연기한 것: snapshot

projection 재구성을 빠르게 하려는 압축 스냅샷은 초기 버전에서 생략한다.

- 평소에는 `$.store` 캐시와 증분 반영으로 충분하다.
- 전체 재구성은 캐시 삭제(30일 미사용)나 스키마 변경 때만 일어나고, 대상 데이터는 수 MB 수준이다.
- 재구성 시간이 실제 병목으로 측정되면 추가한다. 형식은 `snapshots/<id>.json`을 새 이름으로 쓰고, 최신 유효본을 채택하는 방식이다.

## 참조

- https://code.claude.com/docs/en/plugins/mods/reference (Limits)
- https://code.claude.com/docs/en/plugins/mods/interface (Keep state)
- https://code.claude.com/docs/en/costs (cleanupPeriodDays)
- R1: `REPORT-v2-R1.md` Q1-d
