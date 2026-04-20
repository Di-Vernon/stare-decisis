#!/usr/bin/env bash
# lib/review.sh — Phase 2: Plan 검토 UI
# plan.json을 시각화하고 사용자의 승인/수정/취소 선택을 받는다.

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "lib/review.sh는 source로 로드해야 합니다" >&2
    exit 1
fi

# review_display <plan_path>
# plan을 시각적으로 출력 (ui_plan_display 래퍼 + 안전성 체크)
review_display() {
    local plan_path="${1:?review_display: plan_path 필수}"
    if ! ui_plan_display "$plan_path"; then
        ui_error "plan 렌더링 실패"
        return 1
    fi
}

# review_prompt
# 표준출력으로 decision만 내보낸다: approve | revise | cancel
# revise인 경우 피드백은 HARNESS_STATE_DIR/.last_feedback 파일에 저장된다.
# (파이프 파싱을 단순화하기 위해 콜론 구분 대신 파일 사용)
review_prompt() {
    local state_dir="${HARNESS_STATE_DIR:?HARNESS_STATE_DIR 미설정}"
    local feedback_file="$state_dir/.last_feedback"
    rm -f "$feedback_file"

    local choice
    # gum choose는 /dev/tty를 사용하므로 $(...) 캡처 안에서도 정상 동작
    choice=$(gum choose \
        --header "이 plan을 어떻게 할까요?" \
        "승인 — 실행 단계로 진행" \
        "수정 — 피드백을 입력하고 plan 재생성" \
        "취소 — 종료") || {
        echo "cancel"
        return 0
    }

    case "$choice" in
        "승인"*)
            echo "approve"
            ;;
        "수정"*)
            local fb
            fb=$(gum write \
                --placeholder "이 plan을 어떻게 고치고 싶으신가요? (예: 테스트 task 추가, wave 축소 등)" \
                --header "수정 피드백 (Ctrl+D로 완료, Esc로 취소)" \
                --width 100 \
                --height 10) || {
                echo "cancel"
                return 0
            }
            if [[ -z "${fb// }" ]]; then
                echo "cancel"
                return 0
            fi
            printf '%s' "$fb" > "$feedback_file"
            echo "revise"
            ;;
        "취소"*|*)
            echo "cancel"
            ;;
    esac
}

# review_last_feedback — review_prompt에서 저장한 피드백을 읽어서 출력
review_last_feedback() {
    local feedback_file="${HARNESS_STATE_DIR:?}/.last_feedback"
    if [[ -f "$feedback_file" ]]; then
        cat "$feedback_file"
    fi
}
