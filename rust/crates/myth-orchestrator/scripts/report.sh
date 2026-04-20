#!/usr/bin/env bash
# lib/report.sh — Phase 4: 실행 보고서 (v0.2 간단 버전)
# pipeline.json과 output/ 을 읽어 터미널에 보고서를 출력한다.

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "lib/report.sh는 source로 로드해야 합니다" >&2
    exit 1
fi

# report_generate <out_base_dir>
report_generate() {
    local out_base="${1:?report_generate: out_base_dir 필수}"

    # 소요 시간 계산
    local started_at elapsed_sec mins secs
    started_at=$(state_get ".started_at")
    if [[ -n "$started_at" && "$started_at" != "null" ]]; then
        local start_epoch now_epoch
        start_epoch=$(date -d "$started_at" +%s 2>/dev/null || echo 0)
        now_epoch=$(date +%s)
        elapsed_sec=$(( now_epoch - start_epoch ))
    else
        elapsed_sec=0
    fi
    mins=$(( elapsed_sec / 60 ))
    secs=$(( elapsed_sec % 60 ))

    # task 상태 집계
    local total=0 done_c=0 failed_c=0 skipped_c=0 timeout_c=0
    local task_lines=""

    local task_ids tid
    task_ids=$(state_get '.tasks | keys[]' 2>/dev/null)
    for tid in $task_ids; do
        total=$((total + 1))
        local st
        st=$(state_get ".tasks.\"$tid\".status")
        case "$st" in
            done)    done_c=$((done_c + 1));      task_lines+="  ✅ [$tid] 완료"$'\n' ;;
            failed)  failed_c=$((failed_c + 1));   task_lines+="  ❌ [$tid] 실패"$'\n' ;;
            skipped) skipped_c=$((skipped_c + 1)); task_lines+="  ⏭️  [$tid] 스킵"$'\n' ;;
            timeout) timeout_c=$((timeout_c + 1)); task_lines+="  ⏰ [$tid] 타임아웃"$'\n' ;;
            *)       task_lines+="  ❓ [$tid] $st"$'\n' ;;
        esac

        # 실패/타임아웃 task의 에러 표시 (최대 3줄)
        if [[ "$st" == "failed" || "$st" == "timeout" ]]; then
            local efile="$out_base/$tid/error.txt"
            if [[ -f "$efile" && -s "$efile" ]]; then
                task_lines+="$(head -3 "$efile" | sed 's/^/       /')"$'\n'
            fi
        fi
    done

    # 변경된 파일 — 실행 시작 이후 수정된 것만 필터링
    local changed_lines=""
    local start_epoch_file="$HARNESS_BASE_DIR/.execute_start_epoch"
    if [[ -d "$HARNESS_WORK_DIR" ]]; then
        local changed=""
        if [[ -f "$start_epoch_file" ]]; then
            # 타임스탬프 마커 파일 생성 → find -newer 로 필터
            local marker
            marker=$(mktemp)
            touch -d "@$(cat "$start_epoch_file")" "$marker" 2>/dev/null
            changed=$(find "$HARNESS_WORK_DIR" -newer "$marker" \
                -not -path '*/.harness/*' -not -path '*/.git/*' \
                -not -path '*/node_modules/*' -not -path '*/__pycache__/*' \
                -type f 2>/dev/null | \
                sed "s|^$HARNESS_WORK_DIR/||" | sort | head -20)
            rm -f "$marker"
        else
            # 폴백: git diff
            if git -C "$HARNESS_WORK_DIR" rev-parse --git-dir >/dev/null 2>&1; then
                changed=$(cd "$HARNESS_WORK_DIR" && {
                    git diff --name-only 2>/dev/null
                    git ls-files --others --exclude-standard 2>/dev/null
                } | sort -u | head -20)
            fi
        fi
        if [[ -n "$changed" ]]; then
            while IFS= read -r f; do
                [[ -n "$f" ]] && changed_lines+="  • $f"$'\n'
            done <<< "$changed"
        fi
    fi

    # 요약 박스
    gum style \
        --border double \
        --border-foreground "$UI_COLOR_PRIMARY" \
        --padding "1 2" \
        --margin "1 0" \
        "📊 실행 보고서" \
        "" \
        "⏱️  소요 시간: ${mins}분 ${secs}초" \
        "📈 전체 $total / ✅ $done_c / ❌ $failed_c / ⏭️ $skipped_c / ⏰ $timeout_c"

    # task 상세
    if [[ -n "$task_lines" ]]; then
        printf '%s' "$task_lines"
    fi

    # 변경된 파일
    if [[ -n "$changed_lines" ]]; then
        echo ""
        gum style --foreground "$UI_COLOR_ACCENT" --bold "📁 변경된 파일:"
        printf '%s' "$changed_lines"
    fi

    echo ""
}
