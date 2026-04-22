#!/usr/bin/env bash
# e2e-wave8.sh — Wave 8 Task 8.6 end-to-end scenario runner.
#
# Runs 7 `myth` commands against an isolated $HOME so the real
# ~/.myth and ~/.local/bin are not touched. Failure-tolerant for
# environment shortcomings (claude binary missing, system python3
# without myth_py) — prints a one-line verdict per step.
#
# Usage: bash scripts/e2e-wave8.sh
#
# Side effects:
#   - Creates and deletes /tmp/myth-wave8-home
#   - Creates and deletes /tmp/wave8-test
#   - Does NOT pollute $HOME or the real Python env.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MYTH_BIN="${REPO_ROOT}/rust/target/release/myth"
WAVE8_HOME=/tmp/myth-wave8-home
WAVE8_PROJECT=/tmp/wave8-test

if [ ! -x "$MYTH_BIN" ]; then
    echo "error: $MYTH_BIN not found. Run 'cargo build --release' first." >&2
    exit 1
fi

echo "=== Wave 8 Task 8.6 — end-to-end ($(date -Iseconds)) ==="
echo "myth binary: $MYTH_BIN"
echo "isolated HOME: $WAVE8_HOME"
echo "project dir:  $WAVE8_PROJECT"
echo

# Clean state.
rm -rf "$WAVE8_HOME" "$WAVE8_PROJECT"

# Seed isolated ~/.myth (install replacement — avoids Python env pollution).
mkdir -p "$WAVE8_HOME/.myth/metrics" "$WAVE8_HOME/.myth/archive" "$WAVE8_HOME/.local/bin"
cp "$REPO_ROOT/templates/bedrock-rules.yaml" "$WAVE8_HOME/.myth/"
cp "$REPO_ROOT/templates/foundation-rules.yaml" "$WAVE8_HOME/.myth/"
cp "$REPO_ROOT/templates/grid.yaml" "$WAVE8_HOME/.myth/"
cp "$REPO_ROOT/CONSTITUTION.md" "$WAVE8_HOME/.myth/"
printf 'version: 1\nitems: []\n' > "$WAVE8_HOME/.myth/surface-rules.yaml"

# Resolve the Python env that has myth_py installed so `myth observer run`
# can find the module. Falls back to system python3.
VENV_PY_DIR=""
if command -v uv >/dev/null 2>&1; then
    VENV_PY_DIR="$(cd "$REPO_ROOT/python" && uv run which python3 2>/dev/null | xargs -r dirname || true)"
fi
PY_PATH_PREFIX=""
if [ -n "$VENV_PY_DIR" ] && [ -d "$VENV_PY_DIR" ]; then
    PY_PATH_PREFIX="$VENV_PY_DIR:"
    echo "note: myth_py env found at $VENV_PY_DIR; prepending to PATH for observer run."
else
    echo "note: no myth_py venv detected; observer run may fall back to system python3."
fi

step() {
    local n="$1"; shift
    local label="$1"; shift
    echo "--- Step $n: $label"
    "$@"
    local rc=$?
    echo "    exit: $rc"
    echo
    return 0
}

# Step 1: install (SKIPPED — isolation)
echo "--- Step 1: myth install  [SKIPPED for isolation]"
echo "    Reason: invoking uv pip install -e .[dev] would modify the real"
echo "    Python env. Manual seed above reproduces the ~/.myth layout."
echo "    Seeded files:"
ls -1 "$WAVE8_HOME/.myth/" | sed 's/^/      /'
echo

# Step 2
step 2 "myth doctor" bash -c \
    "HOME=\"$WAVE8_HOME\" \"$MYTH_BIN\" doctor 2>&1 | tail -12"

# Step 3
mkdir -p "$WAVE8_PROJECT"
step 3 "myth init $WAVE8_PROJECT" bash -c \
    "HOME=\"$WAVE8_HOME\" \"$MYTH_BIN\" init \"$WAVE8_PROJECT\" 2>&1 | tail -8"

# Step 4
step 4 "myth run --help" bash -c \
    "HOME=\"$WAVE8_HOME\" \"$MYTH_BIN\" run --help 2>&1 | tail -10"

# Step 5
step 5 "myth lesson list" bash -c \
    "HOME=\"$WAVE8_HOME\" \"$MYTH_BIN\" lesson list 2>&1 | tail -5"

# Step 6
step 6 "myth observer run --dry" bash -c \
    "HOME=\"$WAVE8_HOME\" PATH=\"${PY_PATH_PREFIX}\$PATH\" \"$MYTH_BIN\" observer run --dry 2>&1 | tail -10"

# Step 7
step 7 "myth constitution (PAGER=cat)" bash -c \
    "HOME=\"$WAVE8_HOME\" PAGER=cat \"$MYTH_BIN\" constitution 2>&1 | head -10"

# Cleanup
rm -rf "$WAVE8_HOME" "$WAVE8_PROJECT"
echo "=== cleanup: $WAVE8_HOME and $WAVE8_PROJECT removed ==="
