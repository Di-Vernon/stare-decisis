#!/usr/bin/env bash
# ~/myth/scripts/bench-hooks.sh
#
# Hook P99 measurement protocol — locked for apples-to-apples comparison
# across Task 3.6 Step e (wire-through 전 baseline) and Step c
# (wire-through 후 재측정).
#
# !!! 변경 리뷰 필수 !!!
# HF_FLAGS / WARMUP / RUNS 및 각 시나리오의 fixture 경로는 Step e
# baseline과 Step c 재측정의 apples-to-apples 비교를 보장하는 상수다.
# 이 값들을 수정하면 Task 3.6 측정 대조 관계가 무효화된다. 변경은 커밋
# 단위로 리뷰하고 Step e baseline 전체를 재산출해야 한다.
#
# 근거:
#   - 원칙 #1 (docs-first) / Step e 보고 §§2
#     (ARCHITECTURE.md §4 line 264 엄격 예산 50ms)
#   - Step e 보고 §§4 (측정 프로토콜 재현성 고정)
#   - CONSTITUTION Article 8 Falsifiability Requirement
#     (pre-intervention baseline이 개입 효과 반증 가능성의 선결 조건)
#
# 실행:   bash ~/myth/scripts/bench-hooks.sh
# 환경:   BENCH_OUT_DIR 환경변수로 결과 디렉토리 오버라이드 가능
# 의존:   hyperfine >= 1.12 (--input 옵션), jq, release build of myth-hooks

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FIX="$ROOT/rust/crates/myth-hooks/tests/fixtures/envelopes"
REL="$ROOT/rust/target/release"
OUT_DIR="${BENCH_OUT_DIR:-/tmp/myth-bench-hooks/$(date +%Y%m%d-%H%M%S)}"

# --- 재측정 프로토콜 상수 (변경 금지) ---
WARMUP=5
RUNS=100
# -N (--shell=none): bash startup ~3ms가 5ms 미만 hot path를 왜곡하므로
#   필수. Step e 첫 시도에서 shell=bash로 pre_tool P99 9.63ms 측정 →
#   Task 3.4의 5.70ms와 역전 → -N 재측정 시 6.00ms 정합 회복.
# --input: -N 하에서 stdin redirection 불가 → fixture 파일을 --input으로.
# export-json: P99 추출(times 배열 sorted[98]*1000)을 위한 원시 샘플 보존.
HF_FLAGS=(--warmup "$WARMUP" --runs "$RUNS" -N)

# 의존성 사전 체크
if [ ! -x "$REL/myth-hook-pre-tool" ]; then
    echo "error: release build missing at $REL" >&2
    echo "       run: (cd $ROOT/rust && cargo build --release -p myth-hooks)" >&2
    exit 1
fi
command -v hyperfine >/dev/null 2>&1 || { echo "error: hyperfine not installed" >&2; exit 1; }
command -v jq >/dev/null 2>&1        || { echo "error: jq not installed"        >&2; exit 1; }

mkdir -p "$OUT_DIR"

# Tier 1 synth envelope — Tier 0 regex(timeout / rate_limit / file_not_found)를
# 회피하도록 AssertionError 메시지. fixture 디렉토리에는 실측 probe 레코드만
# 두기로 했으므로 합성 케이스는 OUT_DIR에 런타임 생성.
cat > "$OUT_DIR/tier1_env.json" <<'EOF'
{"session_id":"11111111-0000-4000-8000-000000000001","transcript_path":"/tmp/t","cwd":"/tmp/t","permission_mode":"default","hook_event_name":"PostToolUseFailure","tool_name":"Bash","tool_input":{"command":"python -c \"assert 2==3\""},"tool_use_id":"toolu_bench_tier1","error":"Exit code 1\nAssertionError: unclassifiable for tier 0","is_interrupt":false}
EOF

run_scn() {
    local label=$1 bin=$2 envfile=$3 prep_brief=${4:-}
    local TMP
    TMP=$(mktemp -d)
    mkdir -p "$TMP/.myth"
    [ "$prep_brief" = "yes" ] && printf '# Active lessons\n- L-0001: demo\n' > "$TMP/.myth/brief.md"
    (
        # HOME / XDG 격리.
        # `env -u FOO HOME=X bin` 순서 버그(env가 -u를 명령으로 해석)를
        # 피하기 위해 subshell에서 export + unset 후 hyperfine 호출.
        export HOME="$TMP"
        unset XDG_STATE_HOME XDG_CONFIG_HOME XDG_DATA_HOME \
              CLAUDE_REVIEW_ACTIVE MYTH_DISABLE
        hyperfine "${HF_FLAGS[@]}" \
            --input "$envfile" \
            --export-json "$OUT_DIR/$label.json" \
            "$REL/$bin" >/dev/null 2>&1
    )
    rm -rf "$TMP"
}

echo "myth hook P99 benchmark — $(date -Iseconds)"
echo "output dir: $OUT_DIR"
echo

run_scn pre_tool                myth-hook-pre-tool          "$FIX/pre_tool_use.json"
run_scn post_tool               myth-hook-post-tool         "$FIX/post_tool_use.json"
run_scn post_tool_failure_tier0 myth-hook-post-tool-failure "$FIX/post_tool_use_failure.json"
run_scn post_tool_failure_tier1 myth-hook-post-tool-failure "$OUT_DIR/tier1_env.json"
run_scn user_prompt             myth-hook-user-prompt       "$FIX/user_prompt_submit.json"
run_scn stop                    myth-hook-stop              "$FIX/stop.json"
run_scn session_start_nobrief   myth-hook-session-start     "$FIX/session_start.json"
run_scn session_start_withbrief myth-hook-session-start     "$FIX/session_start.json" yes

# P99 = sorted[98] * 1000 (hyperfine times가 초 단위이므로 ms 변환).
# N=100이므로 sorted[98]은 99번째 sample = 통상적 P99.
printf '%-35s %8s %8s %8s %8s %8s %8s\n' scenario min p50 p90 p99 max mean
for label in pre_tool post_tool post_tool_failure_tier0 post_tool_failure_tier1 \
             user_prompt stop session_start_nobrief session_start_withbrief; do
    jq -r --arg l "$label" '
        .results[0].times | sort as $s |
        [$l, ($s[0]*1000), ($s[49]*1000), ($s[89]*1000), ($s[98]*1000), ($s[99]*1000), (add/length*1000)] |
        "\(.[0]) \(.[1]) \(.[2]) \(.[3]) \(.[4]) \(.[5]) \(.[6])"
    ' "$OUT_DIR/$label.json" \
    | awk '{printf "%-35s %8.3f %8.3f %8.3f %8.3f %8.3f %8.3f\n", $1, $2, $3, $4, $5, $6, $7}'
done
