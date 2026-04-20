#!/usr/bin/env bash
# lib/watchdog.sh — 프로세스 모니터링 (v0.3)
# 하드 타임아웃 + 출력 정체(stale) 감지.
# 각 task에 대해 백그라운드 감시 프로세스를 실행한다.

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "lib/watchdog.sh는 source로 로드해야 합니다" >&2
    exit 1
fi

# watchdog_start <pid> <task_id> <log_path> [max_seconds] [stale_threshold]
# 백그라운드에서 target PID를 모니터링한다.
# - 하드 타임아웃: max_seconds 초과 → TERM → KILL
# - 출력 정체: log_path mtime이 stale_threshold초 동안 안 변하면 kill
watchdog_start() {
    local target_pid="${1:?watchdog_start: pid 필수}"
    local task_id="${2:?watchdog_start: task_id 필수}"
    local log_path="${3:?watchdog_start: log_path 필수}"
    local max_seconds="${4:-600}"
    local stale_threshold="${5:-120}"

    local pidfile="$HARNESS_LOG_DIR/.watchdog-${task_id}.pid"

    # 이미 실행 중이면 스킵
    if [[ -f "$pidfile" ]] && kill -0 "$(cat "$pidfile")" 2>/dev/null; then
        return 0
    fi

    # 환경변수를 값으로 캡처 (서브셸에서 참조)
    local _state_dir="$HARNESS_STATE_DIR"
    local _harness_root="$HARNESS_ROOT"

    (
        local start_time
        start_time=$(date +%s)

        while kill -0 "$target_pid" 2>/dev/null; do
            sleep 5

            local now elapsed
            now=$(date +%s)
            elapsed=$(( now - start_time ))

            # 하드 타임아웃
            if (( elapsed > max_seconds )); then
                echo "[watchdog] $task_id: 하드 타임아웃 ${max_seconds}s 초과" >&2
                kill -TERM "$target_pid" 2>/dev/null
                sleep 2
                kill -KILL "$target_pid" 2>/dev/null
                # 상태 업데이트 (state.sh를 직접 사용할 수 없으므로 jq 직접 호출)
                _watchdog_set_timeout "$_state_dir" "$task_id"
                break
            fi

            # 출력 정체 감지
            if [[ -f "$log_path" ]]; then
                local mtime stale
                mtime=$(stat -c %Y "$log_path" 2>/dev/null || echo "$now")
                stale=$(( now - mtime ))
                if (( stale > stale_threshold )); then
                    echo "[watchdog] $task_id: 출력 정체 ${stale}s 초과 (임계=${stale_threshold}s)" >&2
                    kill -TERM "$target_pid" 2>/dev/null
                    sleep 2
                    kill -KILL "$target_pid" 2>/dev/null
                    _watchdog_set_timeout "$_state_dir" "$task_id"
                    break
                fi
            fi
        done

        rm -f "$pidfile"
    ) &

    echo $! > "$pidfile"
}

# watchdog_stop <task_id>
# 모니터링 프로세스 중단
watchdog_stop() {
    local task_id="${1:?watchdog_stop: task_id 필수}"
    local pidfile="$HARNESS_LOG_DIR/.watchdog-${task_id}.pid"

    if [[ -f "$pidfile" ]]; then
        local wpid
        wpid=$(cat "$pidfile" 2>/dev/null)
        if [[ -n "$wpid" ]]; then
            kill "$wpid" 2>/dev/null
            wait "$wpid" 2>/dev/null
        fi
        rm -f "$pidfile"
    fi
}

# watchdog_stop_all — 모든 활성 watchdog 중단
watchdog_stop_all() {
    local pidfile
    for pidfile in "$HARNESS_LOG_DIR"/.watchdog-*.pid; do
        [[ -f "$pidfile" ]] || continue
        local wpid
        wpid=$(cat "$pidfile" 2>/dev/null)
        [[ -n "$wpid" ]] && kill "$wpid" 2>/dev/null
        rm -f "$pidfile"
    done
}

# 내부: watchdog 서브셸에서 pipeline.json을 직접 업데이트
# (state.sh 함수는 서브셸에서 접근 불가하므로 jq 직접 사용)
_watchdog_set_timeout() {
    local state_dir="$1"
    local task_id="$2"
    local f="$state_dir/pipeline.json"
    local lock="$state_dir/.pipeline.lock"
    local now
    now=$(date -Is)

    (
        flock -x 200 || exit 1
        local tmp
        tmp=$(mktemp)
        jq --arg tid "$task_id" --arg now "$now" '
            .tasks[$tid].status = "timeout"
            | .tasks[$tid].finished_at = $now
        ' "$f" > "$tmp" && mv "$tmp" "$f"
    ) 200>"$lock"
}
