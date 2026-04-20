#!/usr/bin/env bash
# lib/execute.sh — Phase 3: 실행 엔진 (v0.3)
# wave 내 task를 병렬로 실행한다 (tmux + worktree).
# 단일 task wave는 순차 경로 사용. tmux 미설치 시 순차 폴백.

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "lib/execute.sh는 source로 로드해야 합니다" >&2
    exit 1
fi

# ──────────────────────────────────────────────────────
# 초기화
# ──────────────────────────────────────────────────────

# pipeline.json에 모든 task를 pending 상태로 등록
_execute_init_tasks() {
    local plan_path="$1"
    local tid
    while IFS= read -r tid; do
        [[ -z "$tid" ]] && continue
        state_set ".tasks.\"$tid\".status" '"pending"'
    done < <(jq -r '[.waves[].tasks[].task_id] | .[]' "$plan_path")
}

# ──────────────────────────────────────────────────────
# 메인 진입점
# ──────────────────────────────────────────────────────

# execute_plan <plan_path> <out_base_dir>
execute_plan() {
    local plan_path="${1:?execute_plan: plan_path 필수}"
    local out_base="${2:?execute_plan: out_base_dir 필수}"

    mkdir -p "$out_base" || { echo "execute_plan: 출력 디렉토리 생성 실패" >&2; return 1; }

    # 실행 시작 타임스탬프 (report에서 변경 파일 필터링에 사용)
    date +%s > "$HARNESS_BASE_DIR/.execute_start_epoch"

    _execute_init_tasks "$plan_path"

    local main_branch
    main_branch=$(git -C "$HARNESS_WORK_DIR" branch --show-current 2>/dev/null || echo "main")

    local waves_count w
    waves_count=$(jq '.waves | length' "$plan_path")

    for ((w = 0; w < waves_count; w++)); do
        local wave_id tasks_count
        wave_id=$(jq -r ".waves[$w].wave_id" "$plan_path")
        tasks_count=$(jq ".waves[$w].tasks | length" "$plan_path")
        state_set ".current_wave" "$wave_id"

        ui_section "Wave $wave_id 실행 ($tasks_count tasks)"

        if (( tasks_count == 1 )) || ! command -v tmux >/dev/null 2>&1; then
            # 단일 task 또는 tmux 없음 → 순차 실행
            _execute_wave_sequential "$w" "$plan_path" "$out_base"
        else
            # 병렬 실행
            _execute_wave_parallel "$w" "$plan_path" "$out_base" "$main_branch"
        fi
    done
}

# ──────────────────────────────────────────────────────
# 순차 실행 (v0.2 호환)
# ──────────────────────────────────────────────────────

_execute_wave_sequential() {
    local wave_idx="$1" plan_path="$2" out_base="$3"

    local tasks_count t
    tasks_count=$(jq ".waves[$wave_idx].tasks | length" "$plan_path")

    for ((t = 0; t < tasks_count; t++)); do
        local task_json
        task_json=$(jq -c ".waves[$wave_idx].tasks[$t]" "$plan_path")
        _execute_task_sequential "$task_json" "$out_base"
    done
}

# 순차 모드: 단일 task 실행 (v0.2와 동일)
_execute_task_sequential() {
    local task_json="$1" out_base="$2"

    local task_id description
    task_id=$(printf '%s' "$task_json" | jq -r '.task_id')
    description=$(printf '%s' "$task_json" | jq -r '.description')

    # 의존 task 실패 확인
    if _should_skip_task "$task_json"; then
        return 0
    fi

    local method
    method=$(route_task "$task_json")

    local out_dir="$out_base/$task_id"
    mkdir -p "$out_dir"

    ui_info "[$task_id] ($method) $(echo "$description" | cut -c 1-70)"
    state_set ".tasks.\"$task_id\".status" '"running"'
    state_set_str ".tasks.\"$task_id\".started_at" "$(date -Is)"

    local rc=0
    case "$method" in
        bash_direct)       _execute_bash_direct "$task_json" "$out_dir" || rc=$? ;;
        claude_prompt)     _execute_claude_prompt "$task_json" "$out_dir" "$HARNESS_WORK_DIR" || rc=$? ;;
        sequential_chain)  _execute_sequential_chain "$task_json" "$out_dir" "$out_base" || rc=$? ;;
        *)                 echo "[$task_id] 알 수 없는 방법: $method" >&2; rc=1 ;;
    esac

    _finalize_task_status "$task_id" "$rc"
}

# ──────────────────────────────────────────────────────
# 병렬 실행 (v0.3 핵심)
# ──────────────────────────────────────────────────────

_execute_wave_parallel() {
    local wave_idx="$1" plan_path="$2" out_base="$3" main_branch="$4"

    local max_concurrent="${HARNESS_MAX_CONCURRENT:-3}"
    (( max_concurrent > 4 )) && max_concurrent=4

    # wave 내 모든 task 수집
    local tasks_count
    tasks_count=$(jq ".waves[$wave_idx].tasks | length" "$plan_path")

    local all_task_jsons=() all_task_ids=()
    local t
    for ((t = 0; t < tasks_count; t++)); do
        local tj
        tj=$(jq -c ".waves[$wave_idx].tasks[$t]" "$plan_path")
        all_task_jsons+=("$tj")
        all_task_ids+=("$(printf '%s' "$tj" | jq -r '.task_id')")
    done

    # 배치 처리 (max_concurrent 단위)
    local batch_start=0
    while (( batch_start < ${#all_task_jsons[@]} )); do
        local batch_ids=()
        local batch_wt_ids=()  # worktree가 있는 task (머지 필요)
        local i

        for ((i = batch_start; i < batch_start + max_concurrent && i < ${#all_task_jsons[@]}; i++)); do
            local task_json="${all_task_jsons[$i]}"
            local task_id="${all_task_ids[$i]}"

            # 의존 실패 → 스킵
            if _should_skip_task "$task_json"; then
                continue
            fi

            local method
            method=$(route_task "$task_json")

            local out_dir="$out_base/$task_id"
            mkdir -p "$out_dir"

            state_set ".tasks.\"$task_id\".status" '"running"'
            state_set_str ".tasks.\"$task_id\".started_at" "$(date -Is)"

            if [[ "$method" == "bash_direct" ]]; then
                # bash_direct: 백그라운드 프로세스 (worktree 불필요)
                _launch_bash_direct_bg "$task_json" "$out_dir" &
                disown
            else
                # claude_prompt / sequential_chain: tmux + worktree
                local wt_path
                wt_path=$(worktree_create "$task_id" "$main_branch" 2>/dev/null)
                if [[ -z "$wt_path" ]]; then
                    ui_warn "[$task_id] worktree 생성 실패 → 순차 폴백"
                    _execute_task_sequential "$task_json" "$out_base"
                    continue
                fi
                state_set_str ".tasks.\"$task_id\".worktree_path" "$wt_path"
                _launch_tmux_task "$task_json" "$out_dir" "$wt_path" "$out_base"
                batch_wt_ids+=("$task_id")
            fi

            batch_ids+=("$task_id")
        done

        # 모니터링: 배치 내 모든 task 완료 대기
        if (( ${#batch_ids[@]} > 0 )); then
            ui_monitor_panel "${batch_ids[@]}"
        fi

        # watchdog 정리
        for task_id in "${batch_ids[@]}"; do
            watchdog_stop "$task_id"
        done

        # worktree 머지 (순차적으로, 충돌 방지)
        for task_id in "${batch_wt_ids[@]}"; do
            local st
            st=$(state_get ".tasks.\"$task_id\".status")
            if [[ "$st" == "done" ]]; then
                if worktree_merge "$task_id" "$main_branch"; then
                    ui_success "[$task_id] 머지 완료"
                else
                    ui_error "[$task_id] 머지 실패"
                fi
            fi
            worktree_cleanup "$task_id"
        done

        # tmux 세션 정리
        for task_id in "${batch_ids[@]}"; do
            local sess_name
            sess_name=$(_tmux_session_name "$task_id")
            tmux kill-session -t "$sess_name" 2>/dev/null
        done

        batch_start=$(( batch_start + max_concurrent ))
    done
}

# ──────────────────────────────────────────────────────
# 병렬 런처
# ──────────────────────────────────────────────────────

# bash_direct를 백그라운드로 실행 + 상태 자동 업데이트
_launch_bash_direct_bg() {
    local task_json="$1" out_dir="$2"

    local task_id command
    task_id=$(printf '%s' "$task_json" | jq -r '.task_id')
    command=$(printf '%s' "$task_json" | jq -r '.command // ""')

    if [[ -z "$command" || "$command" == "null" ]]; then
        # command 없으면 runner 스크립트로 claude 폴백
        _write_task_runner "$task_json" "$out_dir" "$HARNESS_WORK_DIR" ""
        local runner="$HARNESS_LOG_DIR/.runner-${task_id}.sh"
        bash "$runner"
        return
    fi

    (
        cd "$HARNESS_WORK_DIR" || exit 1
        timeout 300 bash -c "$command" \
            > "$out_dir/stdout.txt" 2> "$out_dir/stderr.txt"
    )
    local rc=$?

    # output.txt 통합
    { cat "$out_dir/stdout.txt" 2>/dev/null
      [[ -s "$out_dir/stderr.txt" ]] && { echo "--- stderr ---"; cat "$out_dir/stderr.txt"; }
    } > "$out_dir/output.txt"
    (( rc != 0 )) && cp "$out_dir/stderr.txt" "$out_dir/error.txt" 2>/dev/null

    _finalize_task_status "$task_id" "$rc"
}

# tmux 세션에서 claude -p 실행
_launch_tmux_task() {
    local task_json="$1" out_dir="$2" wt_path="$3" out_base="$4"

    local task_id
    task_id=$(printf '%s' "$task_json" | jq -r '.task_id')

    # runner 스크립트 생성
    _write_task_runner "$task_json" "$out_dir" "$wt_path" "$out_base"
    local runner="$HARNESS_LOG_DIR/.runner-${task_id}.sh"
    local task_log="$HARNESS_LOG_DIR/${task_id}.log"

    local sess_name
    sess_name=$(_tmux_session_name "$task_id")

    # 기존 세션 정리
    tmux kill-session -t "$sess_name" 2>/dev/null

    # tmux detached 세션 시작
    if ! tmux new-session -d -s "$sess_name" \
        "bash '$runner' 2>&1 | tee '$task_log'; exit \${PIPESTATUS[0]}" 2>/dev/null; then
        echo "_launch_tmux_task: tmux 세션 시작 실패 ($task_id)" >&2
        state_set ".tasks.\"$task_id\".status" '"failed"'
        state_set_str ".tasks.\"$task_id\".finished_at" "$(date -Is)"
        return 1
    fi

    # tmux pane PID 가져오기
    sleep 0.5
    local pane_pid
    pane_pid=$(tmux list-panes -t "$sess_name" -F '#{pane_pid}' 2>/dev/null | head -1)

    # watchdog 시작 (하드 타임아웃 660초 = timeout 600 + 여유 60초)
    if [[ -n "$pane_pid" ]]; then
        watchdog_start "$pane_pid" "$task_id" "$task_log" 660 120
    fi
}

# runner 스크립트 생성 (tmux에서 독립 실행 가능)
_write_task_runner() {
    local task_json="$1" out_dir="$2" work_dir="$3" out_base="$4"

    local task_id description files_list method
    task_id=$(printf '%s' "$task_json" | jq -r '.task_id')
    description=$(printf '%s' "$task_json" | jq -r '.description')
    files_list=$(printf '%s' "$task_json" | jq -r '.scope.files // [] | join(", ")')
    method=$(route_task "$task_json")

    local runner="$HARNESS_LOG_DIR/.runner-${task_id}.sh"
    local prompt_file="$HARNESS_LOG_DIR/.prompt-${task_id}.txt"

    # 프롬프트 파일 생성
    if [[ "$method" == "sequential_chain" && -n "$out_base" ]]; then
        cat > "$prompt_file" << 'PROMPT_EOF'
이전 작업 결과를 참고하여 다음 작업을 수행하세요.
완료되면 수행한 내용을 간결하게 요약하세요.
PROMPT_EOF
        local dep dep_out
        local deps
        deps=$(printf '%s' "$task_json" | jq -r '.dependencies // [] | .[]')
        for dep in $deps; do
            dep_out="$out_base/$dep/output.txt"
            if [[ -f "$dep_out" ]]; then
                printf '\n=== 이전 작업 [%s] 결과 ===\n' "$dep" >> "$prompt_file"
                head -100 "$dep_out" >> "$prompt_file"
            fi
        done
        printf '\n## 현재 작업\n작업: %s\n' "$description" >> "$prompt_file"
    else
        cat > "$prompt_file" << 'PROMPT_EOF'
다음 작업을 작업 디렉토리에서 수행하세요.
완료되면 수행한 내용을 간결하게 요약하세요.
PROMPT_EOF
        printf '\n작업: %s\n' "$description" >> "$prompt_file"
    fi
    [[ -n "$files_list" ]] && printf '대상 파일: %s\n' "$files_list" >> "$prompt_file"

    # runner 스크립트 생성 (quoted heredoc — 변수 확장 방지)
    cat > "$runner" << 'RUNNER_HEADER'
#!/usr/bin/env bash
# 자동 생성된 task runner 스크립트
RUNNER_HEADER

    # 변수값을 printf로 안전하게 주입
    # PATH를 반드시 포함 (tmux 세션은 새 셸이므로 nvm 등 미로드)
    {
        printf 'export PATH=%q\n' "$PATH"
        printf 'export HARNESS_STATE_DIR=%q\n' "$HARNESS_STATE_DIR"
        printf 'export HARNESS_ROOT=%q\n' "$HARNESS_ROOT"
        printf 'TASK_ID=%q\n' "$task_id"
        printf 'WORK_DIR=%q\n' "$work_dir"
        printf 'PROMPT_FILE=%q\n' "$prompt_file"
        printf 'OUT_DIR=%q\n' "$out_dir"
    } >> "$runner"

    cat >> "$runner" << 'RUNNER_BODY'

# state.sh 로드 (상태 업데이트용)
source "$HARNESS_ROOT/lib/state.sh"

cd "$WORK_DIR" || exit 1
timeout 600 claude -p \
    --output-format text \
    --max-turns 10 \
    --no-session-persistence \
    --dangerously-skip-permissions \
    < "$PROMPT_FILE" \
    > "$OUT_DIR/output.txt" 2> "$OUT_DIR/error.txt"
RC=$?

NOW=$(date -Is)
if [ $RC -eq 0 ]; then
    state_set ".tasks.\"$TASK_ID\".status" '"done"'
elif [ $RC -eq 124 ]; then
    state_set ".tasks.\"$TASK_ID\".status" '"timeout"'
    echo "timeout 600초 초과" >> "$OUT_DIR/error.txt"
else
    state_set ".tasks.\"$TASK_ID\".status" '"failed"'
fi
state_set_str ".tasks.\"$TASK_ID\".finished_at" "$NOW"

exit $RC
RUNNER_BODY

    chmod +x "$runner"
}

# ──────────────────────────────────────────────────────
# 순차 모드 실행 함수 (v0.2 호환)
# ──────────────────────────────────────────────────────

_execute_bash_direct() {
    local task_json="$1" out_dir="$2"
    local task_id command
    task_id=$(printf '%s' "$task_json" | jq -r '.task_id')
    command=$(printf '%s' "$task_json" | jq -r '.command // ""')

    if [[ -z "$command" || "$command" == "null" ]]; then
        ui_info "[$task_id] command 필드 없음 → claude_prompt 폴백"
        _execute_claude_prompt "$task_json" "$out_dir" "$HARNESS_WORK_DIR"
        return $?
    fi

    (
        cd "$HARNESS_WORK_DIR" || exit 1
        timeout 300 bash -c "$command"
    ) > "$out_dir/stdout.txt" 2> "$out_dir/stderr.txt" &
    local pid=$!
    ui_spin_pid "[$task_id] 명령 실행 중..." "$pid"
    wait "$pid"
    local rc=$?

    { cat "$out_dir/stdout.txt"
      [[ -s "$out_dir/stderr.txt" ]] && { echo "--- stderr ---"; cat "$out_dir/stderr.txt"; }
    } > "$out_dir/output.txt"
    (( rc != 0 )) && cp "$out_dir/stderr.txt" "$out_dir/error.txt" 2>/dev/null
    return $rc
}

_execute_claude_prompt() {
    local task_json="$1" out_dir="$2" work_dir="${3:-$HARNESS_WORK_DIR}"
    local task_id description files_list
    task_id=$(printf '%s' "$task_json" | jq -r '.task_id')
    description=$(printf '%s' "$task_json" | jq -r '.description')
    files_list=$(printf '%s' "$task_json" | jq -r '.scope.files // [] | join(", ")')

    local prompt_file
    prompt_file=$(mktemp)
    cat > "$prompt_file" << 'PROMPT_EOF'
다음 작업을 작업 디렉토리에서 수행하세요.
완료되면 수행한 내용을 간결하게 요약하세요.
PROMPT_EOF
    printf '\n작업: %s\n' "$description" >> "$prompt_file"
    [[ -n "$files_list" ]] && printf '대상 파일: %s\n' "$files_list" >> "$prompt_file"

    (
        cd "$work_dir" || exit 1
        timeout 600 claude -p \
            --output-format text \
            --max-turns 10 \
            --no-session-persistence \
            --dangerously-skip-permissions \
            < "$prompt_file"
    ) > "$out_dir/output.txt" 2> "$out_dir/error.txt" &
    local pid=$!
    ui_spin_pid "[$task_id] claude -p 실행 중..." "$pid"
    wait "$pid"
    local rc=$?
    rm -f "$prompt_file"
    (( rc == 124 )) && echo "timeout 600초 초과" >> "$out_dir/error.txt"
    return $rc
}

_execute_sequential_chain() {
    local task_json="$1" out_dir="$2" out_base="$3"
    local task_id description files_list
    task_id=$(printf '%s' "$task_json" | jq -r '.task_id')
    description=$(printf '%s' "$task_json" | jq -r '.description')
    files_list=$(printf '%s' "$task_json" | jq -r '.scope.files // [] | join(", ")')

    local dep_context="" dep dep_out
    local deps
    deps=$(printf '%s' "$task_json" | jq -r '.dependencies // [] | .[]')
    for dep in $deps; do
        dep_out="$out_base/$dep/output.txt"
        if [[ -f "$dep_out" ]]; then
            dep_context+="=== 이전 작업 [$dep] 결과 ==="$'\n'
            dep_context+="$(head -100 "$dep_out")"$'\n\n'
        fi
    done

    local prompt_file
    prompt_file=$(mktemp)
    cat > "$prompt_file" << 'PROMPT_EOF'
이전 작업 결과를 참고하여 다음 작업을 수행하세요.
완료되면 수행한 내용을 간결하게 요약하세요.
PROMPT_EOF
    [[ -n "$dep_context" ]] && printf '\n## 이전 작업 결과\n%s\n' "$dep_context" >> "$prompt_file"
    printf '\n## 현재 작업\n작업: %s\n' "$description" >> "$prompt_file"
    [[ -n "$files_list" ]] && printf '대상 파일: %s\n' "$files_list" >> "$prompt_file"

    (
        cd "$HARNESS_WORK_DIR" || exit 1
        timeout 600 claude -p \
            --output-format text \
            --max-turns 10 \
            --no-session-persistence \
            --dangerously-skip-permissions \
            < "$prompt_file"
    ) > "$out_dir/output.txt" 2> "$out_dir/error.txt" &
    local pid=$!
    ui_spin_pid "[$task_id] claude -p (chain) 실행 중..." "$pid"
    wait "$pid"
    local rc=$?
    rm -f "$prompt_file"
    (( rc == 124 )) && echo "timeout 600초 초과" >> "$out_dir/error.txt"
    return $rc
}

# ──────────────────────────────────────────────────────
# 유틸
# ──────────────────────────────────────────────────────

# 의존 task 실패 시 스킵 판단
_should_skip_task() {
    local task_json="$1"
    local task_id dep dep_status
    task_id=$(printf '%s' "$task_json" | jq -r '.task_id')
    local deps
    deps=$(printf '%s' "$task_json" | jq -r '.dependencies // [] | .[]')
    for dep in $deps; do
        dep_status=$(state_get ".tasks.\"$dep\".status")
        if [[ "$dep_status" == "failed" || "$dep_status" == "skipped" || "$dep_status" == "timeout" ]]; then
            ui_warn "[$task_id] 의존 task($dep=$dep_status) 실패로 스킵"
            state_set ".tasks.\"$task_id\".status" '"skipped"'
            state_set_str ".tasks.\"$task_id\".finished_at" "$(date -Is)"
            return 0
        fi
    done
    return 1
}

# task 종료 후 상태 업데이트
_finalize_task_status() {
    local task_id="$1" rc="$2"
    local now
    now=$(date -Is)
    if (( rc == 0 )); then
        state_set ".tasks.\"$task_id\".status" '"done"'
        ui_success "[$task_id] 완료"
    elif (( rc == 124 )); then
        state_set ".tasks.\"$task_id\".status" '"timeout"'
        ui_error "[$task_id] timeout"
    else
        state_set ".tasks.\"$task_id\".status" '"failed"'
        ui_error "[$task_id] 실패 (rc=$rc)"
    fi
    state_set_str ".tasks.\"$task_id\".finished_at" "$now"
}

# tmux 세션 이름 생성
_tmux_session_name() {
    local task_id="$1"
    echo "harness-${task_id}"
}

# 모든 harness tmux 세션 정리
execute_cleanup_tmux() {
    tmux list-sessions -F '#{session_name}' 2>/dev/null | \
        grep '^harness-' | \
        while read -r s; do tmux kill-session -t "$s" 2>/dev/null; done
}

# 모든 worktree 정리
execute_cleanup_worktrees() {
    local wt_path
    while IFS= read -r wt_path; do
        [[ -z "$wt_path" ]] && continue
        local tid
        tid=$(basename "$wt_path")
        worktree_cleanup "$tid"
    done < <(worktree_list)
}
