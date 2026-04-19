# Archive Actions

myth 프로젝트로의 이관 완료 후 Jeffrey가 수동 실행할 아카이브 작업.

## 1. 기존 Ultraplan 아카이브

**대상**: `~/project/harness-orchestrator/MYTH-ULTRAPLAN.md` (v1, 2200줄)

이 파일은 myth v2.0 재작성의 원천이 됐지만, myth 프로젝트 자체는 이제 독립되어 있다. 원본 v1을 **삭제하지 않고 보존**한다 — 미래에 "왜 이렇게 바뀌었는지" 추적 가능.

```bash
cd ~/project/harness-orchestrator

# 이름 변경으로 아카이브
mv MYTH-ULTRAPLAN.md MYTH-ULTRAPLAN-v1.md

# Git commit
git add MYTH-ULTRAPLAN-v1.md
git rm --cached MYTH-ULTRAPLAN.md  # 이미 rename됐지만 안전용
git commit -m "archive: rename MYTH-ULTRAPLAN.md to v1 (superseded by ~/myth/)

v2.0+ of the plan lives in ~/myth/docs/09-CLAUDE-PROMPTS.md and related
design documents. This v1 file is retained for historical reference.
"
```

## 2. 기존 harness-orchestrator의 myth 관련 파일

**검토 후 결정**: `harness-orchestrator`에 myth와 중복되는 것이 있는지.

- `CONSTITUTION.md` (v2.2) — `~/myth/CONSTITUTION.md` (v2.3)로 이관됨
  - **원본 유지**: harness-orchestrator는 자체 거버넌스에 여전히 유효
  - myth 발전은 `~/myth/CONSTITUTION.md`에서만
  - 두 파일이 **의도적으로 분기**된 상태 (harness-orchestrator는 v2.2 cement, myth는 v2.3+ 진화)

- `lib/execute.sh` 등 병렬 실행 쉘 — `~/myth/rust/crates/myth-orchestrator/scripts/`에 복사됨
  - **원본 유지**: Jeffrey가 harness-orchestrator로 독립 작업도 계속 가능

해야 할 것:
- 특별한 이관 없음. 양쪽 공존.

## 3. myth 프로젝트 Git 초기화

myth는 이미 작업 디렉토리 `\\wsl$\Ubuntu\home\miirr\myth\`에 존재. 아직 Git 저장소가 아니라면:

```bash
cd ~/myth
git init

# .gitignore
cat > .gitignore <<'EOF'
# Build artifacts
rust/target/
python/build/
python/dist/
python/*.egg-info/
*.pyc
__pycache__/

# Lock files (workspace root Cargo.lock은 포함)
python/poetry.lock  # 선택

# Runtime data (이것들은 ~/.myth/에 있어야 하지만 방지)
.myth/
state.db
*.jsonl
caselog.*
audit.*

# Editor
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db
EOF

git add .gitignore
git add README.md CONSTITUTION.md ARCHITECTURE.md DECISIONS.md PROTOCOL.md WSL2-SETUP.md THIRD-PARTY.md
git add docs/
git commit -m "initial: myth design documents complete

28 design documents spanning 6 phases:
- Phase 1: Foundation (CONCEPTS, DECISIONS, ARCHITECTURE)
- Phase 2: Structure (OVERVIEW, DIRECTORY, CRATES overview)
- Phase 3: Crate details (10 Rust crates)
- Phase 4: Python + infrastructure
- Phase 5: Execution (BUILD-SCOPE, CLAUDE-PROMPTS, VALIDATION, RISKS, DEPLOYMENT)
- Phase 6: Index, README, CONSTITUTION

Ready for Day-0: Claude Code implementation.

Refs: Decision 1-9 (~/myth/DECISIONS.md)
"
```

## 4. GitHub repo 생성 (선택)

```bash
# GitHub CLI가 설치돼 있으면
gh repo create Di-Vernon/myth --private --source=~/myth --remote=origin
git push -u origin main
```

또는 GitHub 웹에서 repo 만들고:
```bash
cd ~/myth
git remote add origin git@github.com:Di-Vernon/myth.git
git branch -M main
git push -u origin main
```

## 5. 설치 준비 (Day-0 신호)

이 아카이브 작업이 끝나면 myth 프로젝트가 **Day-0 상태**다. 그 다음은:

```bash
# Jeffrey가 ~/myth/docs/09-CLAUDE-PROMPTS.md를 Claude Code에 전달
cd ~/myth
cat docs/09-CLAUDE-PROMPTS.md
# Claude Code 세션 시작 → 이 내용 붙여넣기
# → Claude Code가 Wave 0 ~ Wave 8 순차 실행
# → Day-1 완료
```

## 6. 작업 완료 체크리스트

- [ ] `~/project/harness-orchestrator/MYTH-ULTRAPLAN.md` → `MYTH-ULTRAPLAN-v1.md` 이름 변경
- [ ] harness-orchestrator Git commit 완료
- [ ] `~/myth/` 내부 Git init 완료
- [ ] `.gitignore` 추가
- [ ] 초기 commit 완료
- [ ] (선택) GitHub repo 생성 + push
- [ ] `~/myth/CONSTITUTION.md` v2.3 개정 완료 (별도 가이드 참조)

완료 시 이 파일(`ARCHIVE.md`)은 삭제해도 무방.

---

**이 파일은 일회성 작업 가이드**. 아카이브 완료 후 Jeffrey가 삭제하거나 `docs/99-archive-log.md` 같은 이력 폴더로 이동 가능.
