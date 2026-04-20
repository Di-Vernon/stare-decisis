#!/usr/bin/env bash
# lib/plan.sh — Phase 1: Ultraplan 생성
# 사용자 고수준 명령 + 프로젝트 컨텍스트를 meta prompt에 주입하고
# claude -p를 호출하여 plan.json을 생성한다.

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "lib/plan.sh는 source로 로드해야 합니다" >&2
    exit 1
fi

# 프로젝트 컨텍스트 수집 — git/파일구조/CLAUDE.md
# 너무 길어지지 않도록 각 섹션에 head 적용
plan_collect_context() {
    local work_dir="${HARNESS_WORK_DIR:-$PWD}"
    local ctx=""

    ctx+="### 작업 디렉토리"$'\n'
    ctx+="$work_dir"$'\n\n'

    ctx+="### Git 상태"$'\n'
    if git -C "$work_dir" rev-parse --git-dir >/dev/null 2>&1; then
        local branch
        branch=$(git -C "$work_dir" branch --show-current 2>/dev/null || echo "(detached)")
        ctx+="branch: $branch"$'\n'
        ctx+="status:"$'\n'
        ctx+="$(git -C "$work_dir" status --short 2>&1 | head -20)"$'\n'
        ctx+="최근 커밋:"$'\n'
        ctx+="$(git -C "$work_dir" log --oneline -5 2>&1)"$'\n'
    else
        ctx+="(git 저장소 아님)"$'\n'
    fi
    ctx+=$'\n'

    ctx+="### 파일 구조 (depth 2)"$'\n'
    if command -v tree >/dev/null 2>&1; then
        ctx+="$(cd "$work_dir" && tree -L 2 -I 'node_modules|.git|dist|build|__pycache__|.venv|.harness|target' --dirsfirst 2>/dev/null | head -60)"$'\n'
    else
        ctx+="$(cd "$work_dir" && find . -maxdepth 2 \
            -not -path '*/\.*' \
            -not -path '*/node_modules*' 2>/dev/null | head -40)"$'\n'
    fi
    ctx+=$'\n'

    if [[ -f "$work_dir/CLAUDE.md" ]]; then
        ctx+="### CLAUDE.md (앞부분 40줄)"$'\n'
        ctx+="$(head -40 "$work_dir/CLAUDE.md")"$'\n'
    fi

    printf '%s' "$ctx"
}

# 메타프롬프트 조립 — {user_input}, {project_context} 치환
plan_build_prompt() {
    local user_input="$1"
    local context="$2"
    local template_file="${HARNESS_ROOT:?}/.claude/prompts/plan-meta.md"

    if [[ ! -f "$template_file" ]]; then
        echo "plan_build_prompt: 메타프롬프트 템플릿 없음: $template_file" >&2
        return 1
    fi

    # awk로 치환 (sed는 특수문자 이스케이프가 번거로움)
    awk -v input="$user_input" -v ctx="$context" '
        {
            gsub(/\{user_input\}/, input)
            gsub(/\{project_context\}/, ctx)
            print
        }
    ' "$template_file"
}

# 응답에서 JSON 본문만 추출
# 1) 원본 그대로 유효 → 그대로 사용
# 2) ```json ... ``` 코드 펜스 제거 후 시도
# 3) 첫 '{' 이후부터 마지막 '}' 까지 캡처
plan_extract_json() {
    local raw="$1"

    if printf '%s' "$raw" | jq empty 2>/dev/null; then
        printf '%s' "$raw"
        return 0
    fi

    local stripped
    stripped=$(printf '%s' "$raw" | sed -e 's/```json//g' -e 's/```//g')
    if printf '%s' "$stripped" | jq empty 2>/dev/null; then
        printf '%s' "$stripped"
        return 0
    fi

    # 첫 중괄호 위치부터 끝까지 캡처 후 jq로 검증
    local trimmed
    trimmed=$(printf '%s' "$stripped" | awk '
        BEGIN { started = 0 }
        {
            if (!started) {
                idx = index($0, "{")
                if (idx > 0) {
                    started = 1
                    print substr($0, idx)
                }
            } else {
                print
            }
        }
    ')
    if printf '%s' "$trimmed" | jq empty 2>/dev/null; then
        printf '%s' "$trimmed"
        return 0
    fi

    return 1
}

# Plan 생성 — plan.json 파일을 만들고 리턴 코드로 결과 보고
# 사용법: plan_generate <user_input> <out_path> [feedback]
plan_generate() {
    local user_input="${1:?plan_generate: user_input 필수}"
    local out_path="${2:?plan_generate: out_path 필수}"
    local feedback="${3:-}"

    local context prompt
    context=$(plan_collect_context) || {
        echo "plan_generate: 컨텍스트 수집 실패" >&2
        return 1
    }
    prompt=$(plan_build_prompt "$user_input" "$context") || {
        echo "plan_generate: 프롬프트 조립 실패" >&2
        return 1
    }

    if [[ -n "$feedback" ]]; then
        prompt+=$'\n\n## 이전 Plan에 대한 피드백\n'"$feedback"
        prompt+=$'\n\n위 피드백을 반영하여 Plan을 다시 생성하세요. 규칙과 JSON 스키마는 그대로 지켜야 합니다.\n'
    fi

    local prompt_file out_file err_file raw json
    prompt_file=$(mktemp) || return 1
    out_file=$(mktemp) || return 1
    err_file=$(mktemp) || return 1
    printf '%s' "$prompt" > "$prompt_file"

    # --max-turns 1: 응답 1회 후 즉시 종료 (tool loop 방지)
    # --no-session-persistence: 세션 저장을 건너뛰어 응답 후 hang 방지
    # 두 플래그 모두 없으면 claude -p가 stdout 출력 후 프로세스를 종료하지 않는 버그 발생
    timeout 180 claude -p \
            --output-format text \
            --max-turns 3 \
            --no-session-persistence \
            < "$prompt_file" \
            > "$out_file" 2> "$err_file"
    local rc=$?
    if (( rc != 0 )); then
        echo "plan_generate: claude -p 실행 실패 (rc=$rc)" >&2
        if (( rc == 124 )); then
            echo "  → timeout 180초 초과" >&2
        fi
        echo "--- stderr ---" >&2
        head -20 "$err_file" >&2
        echo "--- stdout ---" >&2
        head -20 "$out_file" >&2
        rm -f "$prompt_file" "$out_file" "$err_file"
        return 1
    fi

    raw=$(cat "$out_file")
    rm -f "$prompt_file" "$out_file" "$err_file"

    if [[ -z "$raw" ]]; then
        echo "plan_generate: claude 응답이 비어있음" >&2
        return 1
    fi

    json=$(plan_extract_json "$raw") || {
        echo "plan_generate: 응답에서 JSON을 추출할 수 없음" >&2
        echo "--- 응답 앞부분 ---" >&2
        printf '%s\n' "$raw" | head -30 >&2
        return 1
    }

    # created_at 주입 + 메타데이터 자동 계산
    local now
    now=$(date -Is)
    mkdir -p "$(dirname "$out_path")" || return 1
    if ! printf '%s' "$json" | jq --arg now "$now" '
        .created_at = $now
        | .metadata = (.metadata // {})
        | .metadata.total_tasks = ([.waves[].tasks[]] | length)
        | .metadata.total_waves = (.waves | length)
    ' > "$out_path"; then
        echo "plan_generate: plan.json 저장 실패" >&2
        return 1
    fi

    return 0
}

# Plan 검증 — 스키마/충돌/순환
plan_validate() {
    local plan_path="${1:?plan_validate: path 필수}"

    if [[ ! -f "$plan_path" ]]; then
        echo "plan_validate: 파일 없음: $plan_path" >&2
        return 1
    fi
    if ! jq empty "$plan_path" 2>/dev/null; then
        echo "plan_validate: 유효한 JSON 아님" >&2
        return 1
    fi

    # 필수 필드 확인
    local missing
    missing=$(jq -r '
        [
            (if (.objective // "") == "" then "objective" else empty end),
            (if (.waves // []) == [] then "waves" else empty end)
        ] | join(", ")
    ' "$plan_path")
    if [[ -n "$missing" ]]; then
        echo "plan_validate: 필수 필드 누락: $missing" >&2
        return 1
    fi

    # 각 task에 task_id와 description이 있는지
    local task_err
    task_err=$(jq -r '
        [.waves[] | .wave_id as $w | .tasks[] |
         select((.task_id // "") == "" or (.description // "") == "") |
         "wave \($w): task_id 또는 description 누락"] | join("\n")
    ' "$plan_path")
    if [[ -n "$task_err" ]]; then
        echo "plan_validate: $task_err" >&2
        return 1
    fi

    # 동일 wave 내 파일 중복 체크
    local conflicts
    conflicts=$(jq -r '
        [.waves[] | . as $w |
         ([.tasks[].scope.files // [] | .[]]) as $files |
         ($files | group_by(.) | map(select(length > 1) | .[0])) as $dup |
         if ($dup | length) > 0 then
             "wave \($w.wave_id): 중복 파일 \($dup | join(", "))"
         else empty end
        ] | join("\n")
    ' "$plan_path")
    if [[ -n "$conflicts" ]]; then
        echo "plan_validate: 파일 충돌:" >&2
        echo "$conflicts" >&2
        return 1
    fi

    # task_id 전체 중복 체크
    local dup_tid
    dup_tid=$(jq -r '
        [.waves[].tasks[].task_id] | group_by(.) | map(select(length > 1) | .[0]) | join(", ")
    ' "$plan_path")
    if [[ -n "$dup_tid" ]]; then
        echo "plan_validate: task_id 중복: $dup_tid" >&2
        return 1
    fi

    # 의존성 검증 — 존재하는 task_id + 이전 wave에 있어야 함
    # tid → wave_id 맵을 먼저 만들고, 각 dep이 strictly earlier wave인지 확인
    local dep_err
    dep_err=$(jq -r '
        ([.waves[] | .wave_id as $w | .tasks[] | {tid: .task_id, wid: $w}]
         | map({key: .tid, value: .wid}) | from_entries) as $tid2wave |
        [.waves[] | .wave_id as $wid | .tasks[] |
         . as $t |
         (.dependencies // [])[] | . as $d |
         if $tid2wave[$d] == null then
             "\($t.task_id): 존재하지 않는 의존 \($d)"
         elif $tid2wave[$d] >= $wid then
             "\($t.task_id)(wave \($wid)): 의존 \($d)이 같은/나중 wave (\($tid2wave[$d]))에 있음"
         else empty end
        ] | join("\n")
    ' "$plan_path")
    if [[ -n "$dep_err" ]]; then
        echo "plan_validate: 의존성 오류:" >&2
        echo "$dep_err" >&2
        return 1
    fi

    return 0
}
