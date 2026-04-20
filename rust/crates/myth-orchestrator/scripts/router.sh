#!/usr/bin/env bash
# lib/router.sh — 태스크 라우팅 로직
# task JSON을 받아 실행 방법(execution_method)을 결정한다.
# ARCHITECTURE.md §6 라우팅 규칙을 구현.

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "lib/router.sh는 source로 로드해야 합니다" >&2
    exit 1
fi

# route_task <task_json_string>
# stdout으로 실행 방법 출력: bash_direct | claude_prompt | sequential_chain
route_task() {
    local task_json="${1:?route_task: task JSON 필수}"

    # plan.json에 execution_method가 지정되어 있으면 우선 사용
    local existing
    existing=$(printf '%s' "$task_json" | jq -r '.execution_method // ""')
    if [[ -n "$existing" && "$existing" != "null" ]]; then
        echo "$existing"
        return 0
    fi

    local task_type complexity files_count deps_count
    task_type=$(printf '%s' "$task_json" | jq -r '.task_type // ""')
    complexity=$(printf '%s' "$task_json" | jq -r '.estimated_complexity // "moderate"')
    files_count=$(printf '%s' "$task_json" | jq '[.scope.files // [] | .[]] | length')
    deps_count=$(printf '%s' "$task_json" | jq '[.dependencies // [] | .[]] | length')

    # 규칙 기반 라우팅 (ARCHITECTURE.md §6)
    case "$task_type" in
        install|build|configuration)
            echo "bash_direct"
            return 0
            ;;
        test)
            echo "bash_direct"
            return 0
            ;;
    esac

    # code_edit + simple + 파일 3개 이하 → claude_prompt
    if [[ "$task_type" == "code_edit" && "$complexity" == "simple" ]] && (( files_count <= 3 )); then
        echo "claude_prompt"
        return 0
    fi

    # 의존성 있으면 sequential_chain
    if (( deps_count > 0 )); then
        echo "sequential_chain"
        return 0
    fi

    # 기본값
    echo "claude_prompt"
}
