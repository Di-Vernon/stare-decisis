#!/usr/bin/env bash
# lib/ui.sh — gum 기반 UI 컴포넌트
# 일관된 테마를 환경변수로 설정하고, phase 전반에서 재사용할 헬퍼를 제공한다.

# 이 파일은 source로만 로드되어야 한다 (직접 실행 금지)
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "lib/ui.sh는 source로 로드해야 합니다" >&2
    exit 1
fi

# 색상 팔레트 (ANSI 256색)
readonly UI_COLOR_PRIMARY="212"    # 핑크
readonly UI_COLOR_ACCENT="99"      # 보라
readonly UI_COLOR_SUCCESS="46"     # 녹색
readonly UI_COLOR_WARN="214"       # 주황
readonly UI_COLOR_ERROR="196"      # 빨강
readonly UI_COLOR_MUTED="244"      # 회색

# gum 전역 환경변수 — 모든 phase에서 같은 테마 사용
export GUM_CHOOSE_CURSOR_FOREGROUND="$UI_COLOR_PRIMARY"
export GUM_CHOOSE_SELECTED_FOREGROUND="$UI_COLOR_PRIMARY"
export GUM_CHOOSE_HEADER_FOREGROUND="$UI_COLOR_ACCENT"
export GUM_CONFIRM_SELECTED_BACKGROUND="$UI_COLOR_PRIMARY"
export GUM_SPIN_SPINNER="dot"
export GUM_SPIN_SPINNER_FOREGROUND="$UI_COLOR_PRIMARY"
export GUM_SPIN_TITLE_FOREGROUND="$UI_COLOR_MUTED"
export GUM_INPUT_CURSOR_FOREGROUND="$UI_COLOR_PRIMARY"
export GUM_INPUT_PROMPT_FOREGROUND="$UI_COLOR_PRIMARY"
export GUM_INPUT_HEADER_FOREGROUND="$UI_COLOR_ACCENT"
export GUM_WRITE_CURSOR_FOREGROUND="$UI_COLOR_PRIMARY"
export GUM_WRITE_HEADER_FOREGROUND="$UI_COLOR_ACCENT"

# 헤더 박스 — 진입 시 로고/타이틀 표시
ui_header() {
    local title="${1:-Harness Orchestrator}"
    local subtitle="${2:-}"
    if [[ -n "$subtitle" ]]; then
        gum style \
            --border double \
            --border-foreground "$UI_COLOR_PRIMARY" \
            --padding "1 4" \
            --margin "1 0" \
            --align center \
            --bold \
            "🎯 $title" "$subtitle" || return 1
    else
        gum style \
            --border double \
            --border-foreground "$UI_COLOR_PRIMARY" \
            --padding "1 4" \
            --margin "1 0" \
            --align center \
            --bold \
            "🎯 $title" || return 1
    fi
}

# 섹션 구분자 — phase/단계 경계에 사용
ui_section() {
    local msg="$*"
    gum style \
        --foreground "$UI_COLOR_PRIMARY" \
        --bold \
        --margin "1 0 0 0" \
        "━━━ $msg ━━━"
}

# 메시지 스타일 (성공/에러/경고/정보)
ui_success() {
    gum style --foreground "$UI_COLOR_SUCCESS" --bold "✅ $*"
}

ui_error() {
    gum style --foreground "$UI_COLOR_ERROR" --bold "❌ $*" >&2
}

ui_warn() {
    gum style --foreground "$UI_COLOR_WARN" --bold "⚠️  $*" >&2
}

ui_info() {
    gum style --foreground "$UI_COLOR_MUTED" "ℹ️  $*"
}

# 확인 다이얼로그 래퍼
ui_confirm() {
    gum confirm "$@"
}

# 선택 다이얼로그 래퍼
ui_choose() {
    gum choose "$@"
}

# 텍스트 입력 래퍼 (한 줄)
ui_input() {
    gum input "$@"
}

# 멀티라인 입력 래퍼
ui_write() {
    gum write "$@"
}

# 백그라운드 PID 추적 스피너
# 사용법: ui_spin_pid "제목" <pid>
# 인자로 받은 PID가 종료될 때까지 스피너를 돌린다.
ui_spin_pid() {
    local title="$1"
    local pid="$2"
    if [[ -z "$pid" ]]; then
        ui_error "ui_spin_pid: PID가 비어있음"
        return 1
    fi
    gum spin \
        --spinner dot \
        --title "$title" \
        -- bash -c "while kill -0 $pid 2>/dev/null; do sleep 0.25; done"
}

# plan.json을 사람이 읽기 쉬운 형태로 렌더링
# wave 요약 박스 + 각 task를 카드로 표시
ui_plan_display() {
    local plan_path="$1"
    if [[ ! -f "$plan_path" ]]; then
        ui_error "plan 파일을 찾을 수 없음: $plan_path"
        return 1
    fi
    if ! jq empty "$plan_path" 2>/dev/null; then
        ui_error "plan 파일이 유효한 JSON이 아님: $plan_path"
        return 1
    fi

    local objective waves_count tasks_count est_min plan_id
    plan_id=$(jq -r '.plan_id // "(id 없음)"' "$plan_path")
    objective=$(jq -r '.objective // "(목표 없음)"' "$plan_path")
    waves_count=$(jq '.waves | length' "$plan_path")
    tasks_count=$(jq '[.waves[].tasks[]] | length' "$plan_path")
    est_min=$(jq -r '.metadata.estimated_minutes // "?"' "$plan_path")

    # 요약 박스
    gum style \
        --border rounded \
        --border-foreground "$UI_COLOR_PRIMARY" \
        --padding "1 2" \
        --margin "1 0" \
        "📋 Plan: $plan_id" \
        "🎯 목표: $objective" \
        "" \
        "📊 Wave: $waves_count    Task: $tasks_count    예상: ${est_min}분"

    # wave별 task 카드
    local w
    for ((w = 0; w < waves_count; w++)); do
        local wave_id tcount
        wave_id=$(jq -r ".waves[$w].wave_id" "$plan_path")
        tcount=$(jq ".waves[$w].tasks | length" "$plan_path")
        ui_section "Wave $wave_id  ($tcount tasks)"

        local t
        for ((t = 0; t < tcount; t++)); do
            local tid desc ttype method complexity files deps
            tid=$(jq -r ".waves[$w].tasks[$t].task_id" "$plan_path")
            desc=$(jq -r ".waves[$w].tasks[$t].description" "$plan_path")
            ttype=$(jq -r ".waves[$w].tasks[$t].task_type // \"?\"" "$plan_path")
            method=$(jq -r ".waves[$w].tasks[$t].execution_method // \"?\"" "$plan_path")
            complexity=$(jq -r ".waves[$w].tasks[$t].estimated_complexity // \"?\"" "$plan_path")
            files=$(jq -r ".waves[$w].tasks[$t].scope.files // [] | join(\", \")" "$plan_path")
            deps=$(jq -r ".waves[$w].tasks[$t].dependencies // [] | join(\", \")" "$plan_path")

            gum style \
                --border rounded \
                --border-foreground "$UI_COLOR_MUTED" \
                --padding "0 2" \
                --margin "0 0 0 2" \
                "[$tid] $(echo "$desc" | cut -c 1-72)" \
                "  type=$ttype  method=$method  복잡도=$complexity" \
                "  files: ${files:-(없음)}" \
                "  deps:  ${deps:-(없음)}"
        done
    done
}

# ──────────────────────────────────────────────────────
# 실시간 모니터링 패널 (v0.3)
# pipeline.json을 1초마다 폴링하며 task 상태를 갱신 표시.
# Synchronized Output으로 깜빡임 방지.
# ──────────────────────────────────────────────────────

ui_monitor_panel() {
    local task_ids=("$@")
    local n=${#task_ids[@]}
    (( n == 0 )) && return 0

    local panel_lines=$(( n + 2 ))  # 헤더 + task 행 + 푸터

    # 초기 빈 줄 확보
    local i
    for ((i = 0; i < panel_lines; i++)); do echo; done

    while true; do
        local all_done=true
        local lines=()

        for tid in "${task_ids[@]}"; do
            local st
            st=$(state_get ".tasks.\"$tid\".status" 2>/dev/null)
            local line=""

            case "$st" in
                running)
                    all_done=false
                    local elapsed=""
                    local started
                    started=$(state_get ".tasks.\"$tid\".started_at" 2>/dev/null)
                    if [[ -n "$started" && "$started" != "null" ]]; then
                        local s_epoch
                        s_epoch=$(date -d "$started" +%s 2>/dev/null || echo "$(date +%s)")
                        elapsed="$(( $(date +%s) - s_epoch ))s"
                    fi
                    line="  ⏳ [$tid] 실행 중 ($elapsed)"
                    ;;
                done)    line="  ✅ [$tid] 완료" ;;
                failed)  line="  ❌ [$tid] 실패" ;;
                timeout) line="  ⏰ [$tid] 타임아웃" ;;
                skipped) line="  ⏭️  [$tid] 스킵" ;;
                pending) line="  ⬜ [$tid] 대기"; all_done=false ;;
                *)       line="  ⬜ [$tid] $st"; all_done=false ;;
            esac
            lines+=("$line")
        done

        # Synchronized Output + 커서 이동
        printf '\033[?2026h'
        printf '\033[%dA' "$panel_lines"

        printf '\033[2K  ┌─── 실행 모니터 ──────────────────────┐\n'
        for line in "${lines[@]}"; do
            printf '\033[2K  │ %-38s│\n' "$line"
        done
        printf '\033[2K  └──────────────────────────────────────┘\n'

        printf '\033[?2026l'

        if $all_done; then break; fi
        sleep 1
    done
}
