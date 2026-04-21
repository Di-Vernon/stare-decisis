# myth — Python 레이어

## 역할

myth의 **LLM 호출·판단 계층**을 Python으로 구현한다. Rust 레이어는 **hook 임계 경로**에 집중하고, LLM 호출 같은 **유연성이 필요한 작업**은 Python이 맡는다.

두 주요 모듈:
- **Assessor** — 실패 직후 Claude Haiku 호출, Level·Category 판정
- **Observer** — 주간 분석, Claude Sonnet 호출, brief.md 재생성

**위치**: `~/myth/python/myth_py/`

## 패키지 구조

```
~/myth/python/myth_py/
├── pyproject.toml           # Poetry 또는 pip
├── __init__.py
├── assessor/
│   ├── __init__.py
│   ├── cli.py               # entry: myth-assessor
│   ├── classifier.py        # Tier 0 deterministic
│   ├── dispatcher.py        # Tier 3 Anthropic SDK (Milestone A)
│   ├── templates.py         # Variant A/B/C
│   ├── schema.py            # Pydantic 모델
│   ├── state.py             # lesson-state.jsonl 읽기/쓰기
│   └── subagent_runner.py   # Task subagent 프롬프트 처리
└── observer/
    ├── __init__.py
    ├── cli.py               # entry: myth-observer
    ├── analyzer.py          # caselog 분석
    ├── brief_gen.py         # brief.md 생성
    ├── migration.py         # Migration Readiness 평가
    ├── report.py            # 주간 리포트 포맷
    └── lapse.py             # Lapse 계산 + status 전환
```

## `pyproject.toml`

```toml
[tool.poetry]
name = "myth_py"
version = "0.1.0"
description = "myth Python layer (Assessor, Observer)"
authors = ["Jeffrey"]
license = "MIT OR Apache-2.0"

[tool.poetry.dependencies]
python = "^3.11"
anthropic = "^0.42"           # Milestone A 이후 Tier 3 활성
pydantic = "^2.7"
typer = "^0.12"
pyyaml = "^6.0"
rich = "^13.7"                # 터미널 출력
python-dateutil = "^2.9"

[tool.poetry.dev-dependencies]
pytest = "^8.2"
pytest-asyncio = "^0.23"
mypy = "^1.10"
ruff = "^0.5"

[tool.poetry.scripts]
myth-assessor = "myth_py.assessor.cli:app"
myth-observer = "myth_py.observer.cli:app"

[build-system]
requires = ["poetry-core"]
build-backend = "poetry.core.masonry.api"
```

## Assessor

### 역할 분담

Python Assessor 레이어는 **세 경로**에서 호출된다:

1. **Task subagent 호출** (가장 흔함): Claude 본체가 `.claude/agents/assessor.md` 정의된 Haiku subagent를 Task tool로 호출. 이 과정은 **Claude Code 내부에서 완결**되며 Python 레이어는 개입하지 않음.

2. **Rust hook에서 직접 호출** (Tier 0 보조): deterministic classify가 애매할 때, Python `classifier.py` 호출하여 더 정교한 규칙 기반 판정. LLM 없이 처리.

3. **Tier 3 dispatcher** (Milestone A 이후): Tier 1 준수율 <70% 시 활성. Anthropic SDK로 직접 Haiku 호출.

### `.claude/agents/assessor.md` — Subagent 정의

```markdown
---
name: assessor
description: Analyze a tool failure and produce structured JSON verdict.
model: claude-haiku-4-5-20251001
tools: []
---

You are the myth Assessor. When invoked, you analyze a single tool failure
from a Claude Code session and return a strict JSON verdict.

## Input

The user message will contain:
- Failure payload (tool_name, tool_input, error, context)
- A reminder_id (UUID)

## Your task

1. Read the failure payload
2. Decompose along 4+1 axes:
   - Blast radius (local/file/process/system/env)
   - Reversibility (trivial/possible/difficult/impossible)
   - Trigger likelihood (low/medium/high)
   - Category (security/correctness/process/data_safety/temporal)
   - Uplift? (is this a security|data_safety|temporal case → +1 level)
3. Compute Level (1-5) using max-of-axes + uplift
4. Write a one-line rationale (< 80 chars)
5. Write a one-paragraph description (100-300 chars) for lesson

## Output

Return ONLY this JSON, no preamble:

{
  "reminder_id": "<echo back>",
  "level": 1-5,
  "category": "security|correctness|process|data_safety|temporal",
  "axes": {
    "blast_radius": "local|file|process|system|env",
    "reversibility": "trivial|possible|difficult|impossible",
    "trigger_likelihood": "low|medium|high"
  },
  "uplift_applied": true|false,
  "rationale": "<short>",
  "description": "<1 paragraph>",
  "recommended_action": "<what Claude should do next>"
}

## Constraints

- If ambiguous, prefer lower Level (Rule of Lenity, +0.7 weight toward leniency)
- Never include markdown fences
- Never add explanatory text outside JSON
```

### `assessor/classifier.py` — Tier 0

Rust `myth-hooks`에서 Tier 0 커버 못한 경우 호출되는 Python fallback.

```python
from dataclasses import dataclass
from enum import Enum
import re

class Level(Enum):
    INFO = 1
    LOW = 2
    MEDIUM = 3
    HIGH = 4
    CRITICAL = 5

class Category(Enum):
    SECURITY = "security"
    CORRECTNESS = "correctness"
    PROCESS = "process"
    DATA_SAFETY = "data_safety"
    TEMPORAL = "temporal"

@dataclass
class Classification:
    level: Level
    category: Category
    rationale: str
    confidence: float  # 0.0 ~ 1.0

# Tier 0 시그니처 패턴 (Rust와 동일 규칙)
PATTERNS = [
    # (regex, level, category, rationale, confidence)
    (re.compile(r"(timeout|timed out|ETIMEDOUT)", re.I), 
     Level.LOW, Category.PROCESS, "transient_network", 0.9),
    (re.compile(r"(429|rate limit|too many requests)", re.I),
     Level.LOW, Category.PROCESS, "rate_limit", 0.95),
    (re.compile(r"(ENOENT|no such file|file not found)", re.I),
     Level.MEDIUM, Category.CORRECTNESS, "file_not_found", 0.85),
    (re.compile(r"(EACCES|permission denied)", re.I),
     Level.MEDIUM, Category.SECURITY, "permission_denied", 0.8),
    (re.compile(r"(SyntaxError|ParseError)"),
     Level.MEDIUM, Category.CORRECTNESS, "syntax_error", 0.9),
    (re.compile(r"(<<<<<<< HEAD|=======|>>>>>>> )", re.M),
     Level.HIGH, Category.DATA_SAFETY, "merge_conflict_artifact", 0.95),
    # ... 더 많은 패턴
]

def classify(tool_input: dict, error: str) -> Classification | None:
    """
    Returns classification if confident, None otherwise (escalate to Tier 1).
    """
    for regex, level, category, rationale, confidence in PATTERNS:
        if regex.search(error):
            return Classification(level, category, rationale, confidence)
    return None
```

### `assessor/templates.py` — Variant 렌더링

Rust `myth-hooks::templates::variant_b`와 **중복 없이 공유**하려면? 파일 하나를 **include 원천**으로 두고, Rust는 `include_str!`, Python은 파일 읽기로 같은 텍스트를 사용.

```python
from pathlib import Path
import re

TEMPLATE_DIR = Path.home() / "myth" / "templates" / "assessor-variants"

def render_variant_b(tool_name: str, compact_json: dict, reminder_id: str) -> str:
    template = (TEMPLATE_DIR / "variant_b.md").read_text()
    return (
        template
        .replace("{tool_name}", tool_name)
        .replace("{compact_json}", _compact(compact_json))
        .replace("{rid}", reminder_id)
    )

def _compact(obj: dict) -> str:
    import json
    return json.dumps(obj, separators=(",", ":"), ensure_ascii=False)[:500]
```

### `assessor/dispatcher.py` — Tier 3 (Milestone A 이후)

```python
from anthropic import Anthropic
from pathlib import Path
import os

API_KEY_PATH = Path.home() / ".config" / "myth" / "api_key"

def load_api_key() -> str:
    # 1. 환경 변수 MYTH_ANTHROPIC_API_KEY
    key = os.getenv("MYTH_ANTHROPIC_API_KEY")
    if key:
        return key
    # 2. ~/.config/myth/api_key 파일
    if API_KEY_PATH.exists():
        return API_KEY_PATH.read_text().strip()
    raise RuntimeError("No API key configured. Run `myth key set` first.")

def dispatch_haiku(prompt: str, max_tokens: int = 1000) -> str:
    client = Anthropic(api_key=load_api_key())
    response = client.messages.create(
        model="claude-haiku-4-5-20251001",
        max_tokens=max_tokens,
        messages=[{"role": "user", "content": prompt}],
    )
    return response.content[0].text
```

로그:
```python
def log_dispatch(request_id: str, tokens_in: int, tokens_out: int, cost_usd: float):
    log_path = Path.home() / ".local/state/myth/tier3-dispatch.jsonl"
    record = {
        "ts": datetime.utcnow().isoformat() + "Z",
        "request_id": request_id,
        "tokens_in": tokens_in,
        "tokens_out": tokens_out,
        "cost_usd": cost_usd,
    }
    with log_path.open("a") as f:
        f.write(json.dumps(record) + "\n")
```

### `assessor/schema.py` — Pydantic 검증

Claude Haiku가 반환한 JSON이 스펙에 맞는지 검증:

```python
from pydantic import BaseModel, Field
from typing import Literal

class AssessorAxes(BaseModel):
    blast_radius: Literal["local", "file", "process", "system", "env"]
    reversibility: Literal["trivial", "possible", "difficult", "impossible"]
    trigger_likelihood: Literal["low", "medium", "high"]

class AssessorVerdict(BaseModel):
    reminder_id: str
    level: int = Field(ge=1, le=5)
    category: Literal["security", "correctness", "process", "data_safety", "temporal"]
    axes: AssessorAxes
    uplift_applied: bool
    rationale: str = Field(max_length=80)
    description: str = Field(min_length=100, max_length=300)
    recommended_action: str

def parse_verdict(json_str: str) -> AssessorVerdict:
    return AssessorVerdict.model_validate_json(json_str)
```

Pydantic이 필드 타입·범위·enum 검증 자동.

## Observer

### `observer/cli.py`

```python
import typer
from rich.console import Console

app = typer.Typer(no_args_is_help=True)
console = Console()

@app.command()
def run(dry: bool = typer.Option(False, "--dry", help="Dry run without writing brief.md")):
    """Run weekly Observer analysis."""
    console.print("[cyan]Running Observer analysis...[/cyan]")
    
    from .analyzer import run_analysis
    from .brief_gen import generate_brief
    from .migration import compute_all_milestones
    from .lapse import update_lapse_scores
    
    analysis = run_analysis()
    console.print(f"  Analyzed {analysis.total_caselog_entries} caselog entries")
    console.print(f"  Found {len(analysis.new_lessons)} new lessons")
    console.print(f"  Recurrence increments: {analysis.recurrence_increments}")
    
    lapse_result = update_lapse_scores()
    console.print(f"  Lapse transitions: {lapse_result.new_lapsed_count}")
    
    milestones = compute_all_milestones()
    for m in milestones:
        status = "[red]TRIGGERED[/red]" if m.triggered else "[green]OK[/green]"
        console.print(f"  Milestone {m.id}: {status} ({m.current_value})")
    
    brief = generate_brief(analysis, lapse_result, milestones)
    
    if not dry:
        brief_path = Path.home() / ".myth" / "brief.md"
        brief_path.write_text(brief)
        console.print(f"[green]Brief written to {brief_path}[/green]")
    else:
        console.print("[yellow]Dry run: brief not written[/yellow]")
        console.print(brief)

if __name__ == "__main__":
    app()
```

### `observer/analyzer.py` — caselog 분석

```python
import json
from dataclasses import dataclass, field
from pathlib import Path
from datetime import datetime, timedelta
from collections import defaultdict

@dataclass
class WeeklyAnalysis:
    total_caselog_entries: int = 0
    new_lessons: list[str] = field(default_factory=list)
    recurrence_increments: int = 0
    level_distribution: dict[int, int] = field(default_factory=lambda: defaultdict(int))
    category_distribution: dict[str, int] = field(default_factory=lambda: defaultdict(int))
    bedrock_matches: int = 0
    tier_1_compliance_rate: float = 0.0
    tier_3_cost_usd: float = 0.0

def run_analysis() -> WeeklyAnalysis:
    caselog_path = Path.home() / ".myth" / "caselog.jsonl"
    cutoff = datetime.utcnow() - timedelta(days=7)
    
    result = WeeklyAnalysis()
    
    with caselog_path.open() as f:
        for line in f:
            entry = json.loads(line)
            ts = datetime.fromisoformat(entry["ts"].replace("Z", "+00:00"))
            if ts < cutoff:
                continue
            
            result.total_caselog_entries += 1
            result.level_distribution[entry.get("level", 1)] += 1
            result.category_distribution[entry.get("category", "unknown")] += 1
            
            if entry.get("bedrock_match"):
                result.bedrock_matches += 1
    
    # Shadow metrics로 Tier 1 compliance 계산
    shadow_path = Path.home() / ".myth" / "metrics" / "reflector-shadow.jsonl"
    if shadow_path.exists():
        result.tier_1_compliance_rate = _compute_tier1_compliance(shadow_path, cutoff)
    
    # Tier 3 cost
    tier3_log = Path.home() / ".local/state/myth/tier3-dispatch.jsonl"
    if tier3_log.exists():
        result.tier_3_cost_usd = _sum_tier3_costs(tier3_log, cutoff)
    
    return result
```

### `observer/brief_gen.py` — brief.md 생성

```python
def generate_brief(
    analysis: WeeklyAnalysis,
    lapse_result: LapseResult,
    milestones: list[MilestoneStatus],
) -> str:
    week = datetime.utcnow().strftime("%Y-W%V")
    
    sections = []
    
    # 1. 헤더
    sections.append(f"# myth Brief — {week}\n")
    sections.append(f"_Generated: {datetime.utcnow().isoformat()}Z_\n\n")
    
    # 2. 요약
    sections.append("## Summary\n")
    sections.append(f"- Analyzed {analysis.total_caselog_entries} events\n")
    sections.append(f"- {len(analysis.new_lessons)} new lessons\n")
    sections.append(f"- {analysis.recurrence_increments} recurrence increments\n")
    sections.append(f"- {lapse_result.new_lapsed_count} lessons lapsed\n")
    sections.append(f"- Bedrock matches: {analysis.bedrock_matches}\n\n")
    
    # 3. 활성 Lesson Top 10
    sections.append("## Active Lessons (top 10 by recurrence)\n\n")
    for l in top_active_lessons(10):
        sections.append(f"- **{l.id}** (L{l.level.value} {l.level.name}, recurrence {l.recurrence})\n")
        sections.append(f"  {l.rationale}\n")
    sections.append("\n")
    
    # 4. Migration Readiness
    sections.append("## Migration Readiness\n\n")
    for m in milestones:
        mark = "⚠" if m.triggered else " "
        sections.append(f"- {mark} **Milestone {m.id}** — {m.title}\n")
        sections.append(f"  Current: {m.current_value} / Threshold: {m.threshold}\n")
        for note in m.notes:
            sections.append(f"  - {note}\n")
    sections.append("\n")
    
    # 5. Tier 1 Compliance
    sections.append(f"## Assessor Tier 1 Compliance\n\n")
    sections.append(f"Rate: {analysis.tier_1_compliance_rate:.1%}\n\n")
    if analysis.tier_1_compliance_rate < 0.70:
        sections.append("⚠ Below 70% — Consider enabling Tier 2/3 (see Milestone A)\n\n")
    
    # 6. Tier 3 비용 (Milestone A 이후)
    if analysis.tier_3_cost_usd > 0:
        sections.append(f"## Tier 3 Cost (this week)\n\n")
        sections.append(f"${analysis.tier_3_cost_usd:.2f}\n\n")
    
    # 7. Observer 권고 (Grid 조정 등)
    sections.append("## Observer Recommendations\n\n")
    recs = generate_recommendations(analysis, lapse_result)
    for r in recs:
        sections.append(f"- {r}\n")
    
    return "".join(sections)
```

### `observer/migration.py` — Milestone 평가

```python
@dataclass
class MilestoneStatus:
    id: str  # "A" ~ "E"
    title: str
    triggered: bool
    current_value: str
    threshold: str
    notes: list[str]

def compute_milestone_c() -> MilestoneStatus:
    """Gavel daemon migration."""
    latency_path = Path.home() / ".local/state/myth/hook-latency.ndjson"
    cutoff = datetime.utcnow() - timedelta(days=14)
    
    latencies = []
    with latency_path.open() as f:
        for line in f:
            entry = json.loads(line)
            ts = datetime.fromisoformat(entry["ts"].replace("Z", "+00:00"))
            if ts < cutoff:
                continue
            if entry["event"] == "pre_tool":
                latencies.append(entry["latency_ms"])
    
    if not latencies:
        return MilestoneStatus("C", "Gavel daemon", False,
                               "no data", "15ms",
                               ["Insufficient data for 14-day evaluation"])
    
    p99 = sorted(latencies)[int(len(latencies) * 0.99)]
    
    triggered = p99 > 15.0 and len(latencies) >= 14 * 10  # 최소 하루 10 이벤트 기준
    
    return MilestoneStatus(
        id="C",
        title="The Gavel daemon migration",
        triggered=triggered,
        current_value=f"P99: {p99:.1f}ms",
        threshold="15ms (sustained 14d)",
        notes=[
            f"Sample size: {len(latencies)} events",
            f"Duration: {(datetime.utcnow() - cutoff).days} days",
            "Other conditions: build profile applied, WSL2 green, PGO attempted",
        ],
    )

def compute_all_milestones() -> list[MilestoneStatus]:
    return [
        compute_milestone_a(),  # Assessor Tier review (Day+21)
        compute_milestone_b(),  # Vector store
        compute_milestone_c(),  # Gavel daemon
        compute_milestone_d(),  # Semantic detection
        compute_milestone_e(),  # AST validation
    ]
```

### `observer/lapse.py` — Lapse 계산

```python
from myth_py.db import SqliteLessonStore  # Python↔Rust SQLite 공유

@dataclass
class LapseResult:
    new_lapsed_count: int
    revived_count: int
    archived_count: int

def update_lapse_scores() -> LapseResult:
    store = SqliteLessonStore.open()
    now = datetime.utcnow()
    result = LapseResult(0, 0, 0)
    
    for lesson in store.list_active():
        idle_days = (now - lesson.last_seen).days
        score = lesson.missed_hook_count * 1.0 + idle_days * 10.0
        
        threshold = {
            1: 50, 2: 50,
            3: 200, 4: 200,
            5: None,  # Bedrock 면제
        }.get(lesson.level.value)
        
        if threshold is None:
            continue
        
        if score >= threshold:
            store.mark_status(lesson.id, "lapsed")
            result.new_lapsed_count += 1
            
            # 아주 오래된 lapsed는 archive
            if idle_days >= 180:
                store.mark_status(lesson.id, "archived")
                result.archived_count += 1
    
    return result
```

## Rust ↔ Python 통합

### SQLite 공유

SQLite는 **프로세스 간 안전**. Python도 같은 `state.db`를 `sqlite3` 모듈로 열 수 있다.

```python
# myth_py/db.py (간단한 wrapper)
import sqlite3
from pathlib import Path

def open_db() -> sqlite3.Connection:
    path = Path.home() / ".myth" / "state.db"
    conn = sqlite3.connect(path, isolation_level=None)
    conn.execute("PRAGMA busy_timeout = 5000")
    conn.execute("PRAGMA journal_mode = WAL")
    return conn
```

**Write 경쟁 주의**: Rust hook과 Python observer가 동시 write 가능. WAL + busy_timeout으로 처리. 실제로 observer는 분·주 단위 실행이라 hook과 충돌 가능성 낮음.

### JSONL 공유

`caselog.jsonl`, `lesson-state.jsonl`은 append-only. Python이 **읽기만** 하거나, Python이 쓸 때도 같은 fcntl flock 규약.

```python
import fcntl

def append_jsonl(path: Path, record: dict):
    with path.open("a") as f:
        fcntl.flock(f, fcntl.LOCK_EX)
        f.write(json.dumps(record, ensure_ascii=False) + "\n")
        fcntl.flock(f, fcntl.LOCK_UN)
```

### myth-embed 호출

Python이 임베딩 필요 시 Unix socket 호출:

```python
import socket
import struct
import bincode  # PyPI: bincode (또는 수동 구현)

def query_embed(text: str) -> list[float]:
    path = Path.home() / ".local/state" / "myth" / "embed.sock"
    # 또는 $XDG_RUNTIME_DIR 경로
    
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
        s.connect(str(path))
        
        req = {"version": 1, "id": str(uuid.uuid4()), "op": {"Embed": {"text": text}}}
        payload = bincode.serialize(req)
        s.sendall(struct.pack("<I", len(payload)) + payload)
        
        length = struct.unpack("<I", _recv_exact(s, 4))[0]
        resp_bytes = _recv_exact(s, length)
        resp = bincode.deserialize(resp_bytes)
        
        return resp["result"]["Embedded"]["vector"]
```

**단순성 대안**: `subprocess.run(["myth-embed", "probe", text], capture_output=True)` — probe 출력을 파싱. 성능 저하 있지만 구현 간단. Observer가 드물게 호출하는 경우 용인 가능.

## 테스트

```
python/tests/
├── assessor/
│   ├── test_classifier.py     # Tier 0 패턴 매칭
│   ├── test_schema.py         # Pydantic 검증
│   ├── test_templates.py
│   └── test_dispatcher.py     # Anthropic SDK mock
└── observer/
    ├── test_analyzer.py
    ├── test_brief_gen.py      # 스냅샷 테스트
    ├── test_migration.py
    └── test_lapse.py
```

`pytest` + `pytest-asyncio`. Anthropic SDK는 mock (실제 과금 방지).

## 실행 지점

| 실행 시점 | 누가 호출 | 경로 |
|---|---|---|
| tool 실패 직후 | Rust `myth-hook-post-tool-failure` | Variant B template 삽입 (LLM 호출 안 함) |
| 다음 turn | Claude 본체가 Task(assessor) | `.claude/agents/assessor.md` → Haiku 내부 호출 |
| 다음 turn의 UserPromptSubmit | Rust `myth-hook-user-prompt` | Python 없음, compliance만 감시 |
| Observer 주간 실행 | `myth observer run` CLI → Python `observer.cli` | `analyzer → brief_gen → write brief.md` |
| Milestone A 활성 시 | Python `assessor.dispatcher` | Anthropic SDK로 Haiku 직접 호출 |

## 관련 결정

- Decision 3 (Assessor Hybrid 2-Tier): Tier 0 + Tier 1 (Day-1), Tier 2/3 코드 존재
- Decision 4 (Tier 3 SDK): `dispatcher.py` 골격만 Day-1, 실제 호출은 Milestone A 이후
- 카테고리 1 (Assessor/Observer): Python 모듈 이름 일치
- 카테고리 7 (brief.md): `brief_gen.py` 출력 파일명 확정

---

## Wave 6 drift 박제 (Wave 8 Task 8.4 sync)

Wave 6 커밋 `adf78f4`가 authoritative. 본 docs/05는 drift 6건을 실제 구현에
정렬. 코드 수정 0건, 문서 참조만.

- **drift 1** — `assessor/subagent_runner.py`: 미정의 파일. 생성하지 않음.
  Day-1 Task subagent 경로는 Claude Code 내장으로 충분.
- **drift 2** — `pyproject.toml` 포맷: docs/05 §pyproject Poetry 표기 vs 실제
  PEP 621 (Wave 0 산출물 + hatchling backend). PEP 621 채택.
- **drift 3** — `observer/report.py`: 미정의 파일. 생성하지 않음. Observer
  Day-1 출력은 `brief_gen.py` 단일 엔트리로 통합.
- **drift 4** — `assessor/state.py`: 미정의 파일. 생성하지 않음. Day-1
  observer의 상태 추적은 brief-gen cycle에 흡수됨.
- **drift 5** — `assessor/cli.py`: docs/05 본문 섹션 부재, pyproject
  `[project.scripts]` entry 제약. Wave 6에서 `run` stub 커맨드만, Wave 8
  Task 8.3에서 `classify --input` 추가 (아래 Wave 8 sync 참조).
- **drift 6** — `observer/lapse.py` sqlite3 인라인 (`myth_py.db` 모듈 도입
  회피). docs/05 내부 모순 해소: "DB 접근은 Rust 경유"를 관찰성 쿼리 읽기
  전용에는 예외 적용. 원문은 그대로 유지, 실구현이 단일 예외로 inline.
- drift 7 (click 8.3 비호환): 커밋 `c9e54a7`로 closed. `click>=8.1,<8.2` 고정.

## Wave 8 Task 8.3 — assessor CLI 추가 (carry-forward #3 해소)

- `myth_py/assessor/cli.py`에 `classify --input <path>` 서브커맨드 추가.
- Day-1: 고정 JSON `{"status":"not_enabled", "reason":"Tier 3 dispatch is
  inactive on Day-1 (Milestone A gate, Decision 4)"}` + exit 0.
- Rust `myth-hooks/src/tier3_dispatch.rs::maybe_tier3_dispatch`가 호출
  대상. 호출 gate (`tier3_gate_active()`)는 Day-1 `false`, Milestone A
  flip. Python `classify` 시그니처는 Milestone A에서 동일하게 유지 —
  Rust 호출부 재수정 불필요.
- `run` 서브커맨드 메시지: "Wave 8" → "Milestone A" (직접 interactive
  호출은 Milestone A).
