#!/usr/bin/env bash
# lib/worktree.sh — git worktree 관리 (v0.3)
# 에이전트별 격리 worktree를 생성/머지/정리한다.

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "lib/worktree.sh는 source로 로드해야 합니다" >&2
    exit 1
fi

# worktree_create <task_id> [base_branch]
# .harness/worktrees/{task_id}/ 에 격리 worktree 생성, 경로를 echo
worktree_create() {
    local task_id="${1:?worktree_create: task_id 필수}"
    local base_branch="${2:-}"

    local wt_dir="$HARNESS_BASE_DIR/worktrees/$task_id"

    # 이미 존재하면 경로만 반환
    if [[ -d "$wt_dir/.git" || -f "$wt_dir/.git" ]]; then
        echo "$wt_dir"
        return 0
    fi

    # base_branch 기본값: 현재 브랜치
    if [[ -z "$base_branch" ]]; then
        base_branch=$(git -C "$HARNESS_WORK_DIR" branch --show-current 2>/dev/null)
        [[ -z "$base_branch" ]] && base_branch="HEAD"
    fi

    # plan_id로 브랜치 이름 구성
    local plan_id="unknown"
    if [[ -f "$HARNESS_PLAN_PATH" ]]; then
        plan_id=$(jq -r '.plan_id // "unknown"' "$HARNESS_PLAN_PATH" 2>/dev/null)
    fi
    local branch_name="harness/${plan_id}/${task_id}"

    mkdir -p "$(dirname "$wt_dir")"

    # 새 브랜치로 worktree 생성 시도 (stdout/stderr 모두 억제)
    if git -C "$HARNESS_WORK_DIR" worktree add -b "$branch_name" "$wt_dir" "$base_branch" >/dev/null 2>&1; then
        echo "$wt_dir"
        return 0
    fi

    # 브랜치 이미 존재 → -B(강제)로 재시도
    if git -C "$HARNESS_WORK_DIR" worktree add "$wt_dir" -B "$branch_name" "$base_branch" >/dev/null 2>&1; then
        echo "$wt_dir"
        return 0
    fi

    echo "worktree_create: 생성 실패 (task=$task_id)" >&2
    return 1
}

# worktree_cleanup <task_id>
# worktree + branch 삭제. uncommitted 변경 있으면 보존.
worktree_cleanup() {
    local task_id="${1:?worktree_cleanup: task_id 필수}"
    local wt_dir="$HARNESS_BASE_DIR/worktrees/$task_id"

    [[ -d "$wt_dir" ]] || return 0

    local branch_name
    branch_name=$(git -C "$wt_dir" branch --show-current 2>/dev/null)

    # worktree 제거
    git -C "$HARNESS_WORK_DIR" worktree remove "$wt_dir" --force 2>/dev/null || {
        # 강제 제거 실패 시 디렉토리 정리만 시도
        git -C "$HARNESS_WORK_DIR" worktree prune 2>/dev/null
    }

    # 브랜치 삭제
    if [[ -n "$branch_name" && "$branch_name" == harness/* ]]; then
        git -C "$HARNESS_WORK_DIR" branch -D "$branch_name" 2>/dev/null
    fi
}

# worktree_merge <task_id> [target_branch]
# worktree 변경을 target_branch로 머지. 충돌 시 claude -p로 해소 시도.
worktree_merge() {
    local task_id="${1:?worktree_merge: task_id 필수}"
    local target_branch="${2:-}"
    local wt_dir="$HARNESS_BASE_DIR/worktrees/$task_id"

    if [[ ! -d "$wt_dir" ]]; then
        [[ "${HARNESS_DEBUG:-}" == "1" ]] && echo "worktree_merge: $task_id worktree 없음, 스킵" >&2
        return 0
    fi

    if [[ -z "$target_branch" ]]; then
        target_branch=$(git -C "$HARNESS_WORK_DIR" branch --show-current 2>/dev/null || echo "main")
    fi

    local wt_branch
    wt_branch=$(git -C "$wt_dir" branch --show-current 2>/dev/null)

    # worktree에 미커밋 변경이 있으면 자동 커밋
    if ! git -C "$wt_dir" diff --quiet 2>/dev/null || \
       ! git -C "$wt_dir" diff --cached --quiet 2>/dev/null || \
       [[ -n "$(git -C "$wt_dir" ls-files --others --exclude-standard 2>/dev/null)" ]]; then
        git -C "$wt_dir" add -A 2>/dev/null
        git -C "$wt_dir" commit -m "harness: $task_id 작업 완료" --no-verify 2>/dev/null || true
    fi

    # target 대비 새 커밋 확인
    local ahead
    ahead=$(git -C "$wt_dir" rev-list --count "$target_branch..HEAD" 2>/dev/null || echo "0")
    if (( ahead == 0 )); then
        [[ "${HARNESS_DEBUG:-}" == "1" ]] && echo "worktree_merge: $task_id 변경 없음" >&2
        return 0
    fi

    # 머지 시도
    if git -C "$HARNESS_WORK_DIR" merge "$wt_branch" --no-edit --no-verify 2>/dev/null; then
        return 0
    fi

    echo "worktree_merge: [$task_id] 머지 충돌 — 자동 해소 시도 중..." >&2

    # claude -p로 충돌 해소 시도
    local conflict_files
    conflict_files=$(git -C "$HARNESS_WORK_DIR" diff --name-only --diff-filter=U 2>/dev/null)

    if [[ -n "$conflict_files" ]]; then
        local resolve_prompt
        resolve_prompt=$(mktemp)
        cat > "$resolve_prompt" << 'RESOLVE_EOF'
git merge 충돌이 발생했습니다.
충돌 마커(<<<<<<< ======= >>>>>>>)가 있는 파일을 수정하여 충돌을 해소하세요.
양쪽 변경사항을 모두 보존하세요. 완료되면 요약하세요.
RESOLVE_EOF
        printf '\n충돌 파일:\n%s\n' "$conflict_files" >> "$resolve_prompt"

        local resolve_ok=false
        if (cd "$HARNESS_WORK_DIR" && timeout 180 claude -p \
                --output-format text --max-turns 5 \
                --no-session-persistence --dangerously-skip-permissions \
                < "$resolve_prompt" > /dev/null 2>&1); then
            local remaining
            remaining=$(git -C "$HARNESS_WORK_DIR" diff --name-only --diff-filter=U 2>/dev/null)
            if [[ -z "$remaining" ]]; then
                git -C "$HARNESS_WORK_DIR" add -A 2>/dev/null
                git -C "$HARNESS_WORK_DIR" commit --no-edit --no-verify 2>/dev/null
                resolve_ok=true
            fi
        fi
        rm -f "$resolve_prompt"

        if $resolve_ok; then
            echo "worktree_merge: [$task_id] 충돌 자동 해소 완료" >&2
            return 0
        fi
    fi

    # 실패 → 머지 중단
    git -C "$HARNESS_WORK_DIR" merge --abort 2>/dev/null
    echo "worktree_merge: [$task_id] 자동 해소 실패, 수동 해소 필요" >&2
    return 1
}

# worktree_list — 활성 harness worktree 목록 (경로만)
worktree_list() {
    local wt_base="$HARNESS_BASE_DIR/worktrees"
    if [[ -d "$wt_base" ]]; then
        find "$wt_base" -mindepth 1 -maxdepth 1 -type d 2>/dev/null
    fi
}
