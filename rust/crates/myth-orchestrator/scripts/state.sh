#!/usr/bin/env bash
# lib/state.sh — pipeline.json 상태 관리
# flock으로 동시 접근을 보호하고 jq로 필드를 read/write한다.

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "lib/state.sh는 source로 로드해야 합니다" >&2
    exit 1
fi

# state 파일 경로 — bin/harness에서 HARNESS_STATE_DIR을 설정해야 함
_state_file() {
    echo "${HARNESS_STATE_DIR:?HARNESS_STATE_DIR 미설정}/pipeline.json"
}

_state_lock() {
    echo "${HARNESS_STATE_DIR:?HARNESS_STATE_DIR 미설정}/.pipeline.lock"
}

# pipeline.json 초기화 — 없으면 기본 구조로 생성
state_init() {
    local f
    f=$(_state_file) || return 1
    mkdir -p "$(dirname "$f")" || {
        echo "state_init: 디렉토리 생성 실패" >&2
        return 1
    }
    if [[ ! -f "$f" ]]; then
        cat > "$f" <<'EOF'
{
  "state": "idle",
  "plan_path": null,
  "current_wave": 0,
  "started_at": null,
  "finished_at": null,
  "tasks": {}
}
EOF
    fi
}

# state_get <jq_path>
# 예: state_get ".state" → "planning"
state_get() {
    local path="${1:?state_get: jq path 필수}"
    local f lock
    f=$(_state_file) || return 1
    lock=$(_state_lock) || return 1

    if [[ ! -f "$f" ]]; then
        echo "null"
        return 0
    fi
    (
        flock -s 200 || {
            echo "state_get: 락 획득 실패" >&2
            exit 1
        }
        jq -r "$path" "$f"
    ) 200>"$lock"
}

# state_set <jq_path> <jq_value_expr>
# 예: state_set ".current_wave" "2"
# 예: state_set ".tasks.t1" '{"status":"running"}'
state_set() {
    local path="${1:?state_set: jq path 필수}"
    local value="${2:?state_set: jq value 필수}"
    local f lock tmp
    f=$(_state_file) || return 1
    lock=$(_state_lock) || return 1

    if [[ ! -f "$f" ]]; then
        state_init || return 1
    fi
    (
        flock -x 200 || {
            echo "state_set: 락 획득 실패" >&2
            exit 1
        }
        tmp=$(mktemp) || exit 1
        if ! jq "$path = $value" "$f" > "$tmp"; then
            echo "state_set: jq 업데이트 실패 (path=$path)" >&2
            rm -f "$tmp"
            exit 1
        fi
        mv "$tmp" "$f"
    ) 200>"$lock"
}

# state_set_str <jq_path> <string>
# 문자열 값을 안전하게 설정 (자동 JSON 이스케이프)
state_set_str() {
    local path="${1:?}"
    local str="${2-}"
    local json_val
    json_val=$(jq -nc --arg v "$str" '$v') || return 1
    state_set "$path" "$json_val"
}

# 상태 전이 — 허용된 값만 수락
state_transition() {
    local new_state="${1:?}"
    case "$new_state" in
        idle|planning|reviewing|executing|reporting|done|failed|cancelled) ;;
        *)
            echo "state_transition: 잘못된 상태 '$new_state'" >&2
            return 1
            ;;
    esac
    state_set_str ".state" "$new_state" || return 1
}

# 현재 상태 출력
state_current() {
    state_get ".state"
}
