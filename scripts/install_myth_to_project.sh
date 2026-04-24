#!/usr/bin/env bash
# install_myth_to_project.sh — Install myth hooks into a project's Claude Code settings
#
# Mode: coexist-selective (Option 3)
# - PreToolUse: myth only (replaces any existing security-gate equivalent)
# - PostToolUse, Stop, SessionStart: myth + existing bash hooks coexist
# - PostToolUseFailure, UserPromptSubmit: myth new (no bash equivalent)
#
# Detected coexistence hooks (harness-template):
# - auto-format.sh      PostToolUse (matcher: Write|Edit|MultiEdit)
# - review-loop.sh      Stop
# - session-init.sh     SessionStart (single-entry group with myth)
# - usage-tracker.sh    PostToolUse (no matcher, all tools)
#
# Usage:
#   bash install_myth_to_project.sh <project_path> [--dry-run]
#
# Safety:
# - Backs up settings file before modification
# - Renames (not deletes) existing security-gate.sh
# - Validates JSON before writing
# - Can be reversed via backup restore
#
# Known limitations (v0.2):
# - No `git status` check before install. If working tree is dirty, subsequent
#   review-loop.sh may flag installer's rename (security-gate.sh →
#   .disabled-by-myth-*) as a security regression. Mitigation: step 0 below
#   instructs user to commit immediately after install. (v0.3 TODO: warn pre-install)
# - No project-level SECURITY_CONTEXT.md injection. (v0.3 TODO: generalize
#   review-loop patch for myth-installed projects)

set -euo pipefail

# ─────────────────────────────────────────────────────────────────
# Arguments
# ─────────────────────────────────────────────────────────────────

if [ $# -lt 1 ]; then
    echo "Usage: bash $0 <project_path> [--dry-run]" >&2
    exit 1
fi

PROJECT_PATH="$(realpath "$1")"
DRY_RUN="false"
[ "${2:-}" = "--dry-run" ] && DRY_RUN="true"

CLAUDE_DIR="$PROJECT_PATH/.claude"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

# ─────────────────────────────────────────────────────────────────
# Preflight checks
# ─────────────────────────────────────────────────────────────────

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "myth project installer"
echo "target: $PROJECT_PATH"
echo "mode:   coexist-selective (option 3)"
echo "dry:    $DRY_RUN"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# project exists
if [ ! -d "$PROJECT_PATH" ]; then
    echo "ERROR: project path does not exist: $PROJECT_PATH" >&2
    exit 1
fi

# .claude/ exists
if [ ! -d "$CLAUDE_DIR" ]; then
    echo "ERROR: .claude/ directory not found in $PROJECT_PATH" >&2
    echo "       Is this a Claude Code project?" >&2
    exit 1
fi

# myth binaries available
MISSING_BINS=()
for h in pre-tool post-tool post-tool-failure user-prompt stop session-start; do
    if ! command -v "myth-hook-$h" > /dev/null 2>&1; then
        MISSING_BINS+=("myth-hook-$h")
    fi
done

if [ ${#MISSING_BINS[@]} -gt 0 ]; then
    echo "ERROR: missing myth hook binaries in PATH:" >&2
    printf '  - %s\n' "${MISSING_BINS[@]}" >&2
    echo "" >&2
    echo "Install myth first: cd ~/myth && cargo build --release && ./target/release/myth install" >&2
    exit 1
fi
echo "✓ myth binaries present in PATH"

# myth credentials (for future Tier 3)
if [ ! -f "$HOME/.myth/credentials" ]; then
    echo "⚠ warning: ~/.myth/credentials not found" >&2
    echo "  myth will work for Tier 0 (Bedrock/Foundation) but Tier 3 assessor will be inactive." >&2
    echo "  To set up: myth key set" >&2
else
    CRED_PERM=$(stat -c %a "$HOME/.myth/credentials")
    if [ "$CRED_PERM" != "600" ]; then
        echo "⚠ warning: ~/.myth/credentials permission is $CRED_PERM (should be 600)" >&2
    fi
    echo "✓ ~/.myth/credentials present (permission $CRED_PERM)"
fi

# jq available
if ! command -v jq > /dev/null 2>&1; then
    echo "ERROR: jq not found. Install: sudo apt install jq" >&2
    exit 1
fi
echo "✓ jq present"

# Determine target settings file
# Priority: settings.local.json > settings.json (local takes precedence in Claude Code)
# If neither exists, create settings.local.json
TARGET_FILE=""
if [ -f "$CLAUDE_DIR/settings.local.json" ]; then
    TARGET_FILE="$CLAUDE_DIR/settings.local.json"
    echo "✓ target file: settings.local.json (exists)"
elif [ -f "$CLAUDE_DIR/settings.json" ]; then
    TARGET_FILE="$CLAUDE_DIR/settings.json"
    echo "✓ target file: settings.json (exists, will be modified)"
    echo "  note: this file is typically committed to git. myth config will be visible to collaborators."
else
    TARGET_FILE="$CLAUDE_DIR/settings.local.json"
    echo "✓ target file: settings.local.json (will be created)"
fi

# Existing JSON valid?
if [ -f "$TARGET_FILE" ]; then
    if ! jq empty "$TARGET_FILE" > /dev/null 2>&1; then
        echo "ERROR: existing $TARGET_FILE has invalid JSON" >&2
        exit 1
    fi
    # Check if hooks already present
    if jq -e '.hooks' "$TARGET_FILE" > /dev/null 2>&1; then
        echo "⚠ target file already has 'hooks' section"
        if jq -e '.hooks.PreToolUse[]?.hooks[]? | select(.command | test("myth-hook-"))' "$TARGET_FILE" > /dev/null 2>&1; then
            echo "  myth hooks already registered — this appears to be a re-install"
            echo "  dry-run will show diff; actual install will add duplicate entries unless this is a fresh file"
        fi
    fi
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# ─────────────────────────────────────────────────────────────────
# Compute new settings content
# ─────────────────────────────────────────────────────────────────

# Load existing (or empty)
if [ -f "$TARGET_FILE" ]; then
    EXISTING=$(cat "$TARGET_FILE")
else
    EXISTING='{}'
fi

# Detect existing bash hooks for coexistence
HAS_AUTO_FORMAT="false"
HAS_REVIEW_LOOP="false"
HAS_SESSION_INIT="false"
HAS_USAGE_TRACKER="false"

[ -f "$CLAUDE_DIR/hooks/auto-format.sh" ] && HAS_AUTO_FORMAT="true"
[ -f "$CLAUDE_DIR/hooks/review-loop.sh" ] && HAS_REVIEW_LOOP="true"
[ -f "$CLAUDE_DIR/hooks/session-init.sh" ] && HAS_SESSION_INIT="true"
[ -f "$CLAUDE_DIR/hooks/usage-tracker.sh" ] && HAS_USAGE_TRACKER="true"

echo "detected existing bash hooks:"
echo "  auto-format.sh:    $HAS_AUTO_FORMAT"
echo "  review-loop.sh:    $HAS_REVIEW_LOOP"
echo "  session-init.sh:   $HAS_SESSION_INIT"
echo "  usage-tracker.sh:  $HAS_USAGE_TRACKER"

# Build myth hooks config (option 3: coexist-selective)
MYTH_HOOKS=$(jq -n \
    --argjson autoformat "$HAS_AUTO_FORMAT" \
    --argjson reviewloop "$HAS_REVIEW_LOOP" \
    --argjson sessioninit "$HAS_SESSION_INIT" \
    --argjson usagetracker "$HAS_USAGE_TRACKER" \
    '{
      PreToolUse: [
        { hooks: [{ type: "command", command: "myth-hook-pre-tool" }] }
      ],
      PostToolUse: (
        [{ hooks: [{ type: "command", command: "myth-hook-post-tool" }] }]
        + (if $autoformat then
            [{ matcher: "Write|Edit|MultiEdit",
               hooks: [{ type: "command", command: "bash .claude/hooks/auto-format.sh", timeout: 15 }] }]
          else [] end)
        + (if $usagetracker then
            [{ hooks: [{ type: "command", command: "bash .claude/hooks/usage-tracker.sh", timeout: 5 }] }]
          else [] end)
      ),
      PostToolUseFailure: [
        { hooks: [{ type: "command", command: "myth-hook-post-tool-failure" }] }
      ],
      UserPromptSubmit: [
        { hooks: [{ type: "command", command: "myth-hook-user-prompt" }] }
      ],
      Stop: (
        [{ hooks: [{ type: "command", command: "myth-hook-stop" }] }]
        + (if $reviewloop then
            [{ hooks: [{ type: "command", command: "bash .claude/hooks/review-loop.sh", timeout: 120 }] }]
          else [] end)
      ),
      SessionStart: (
        [{ hooks: [{ type: "command", command: "myth-hook-session-start" }] }]
        + (if $sessioninit then
            [{ hooks: [{ type: "command", command: "bash .claude/hooks/session-init.sh", timeout: 5 }] }]
          else [] end)
      )
    }')

# Merge with existing (myth hooks replace any existing hooks key)
NEW_CONTENT=$(echo "$EXISTING" | jq --argjson myth "$MYTH_HOOKS" '. + { hooks: $myth }')

# Validate new content
if ! echo "$NEW_CONTENT" | jq empty > /dev/null 2>&1; then
    echo "ERROR: computed new settings has invalid JSON" >&2
    exit 1
fi

# ─────────────────────────────────────────────────────────────────
# Show diff or apply
# ─────────────────────────────────────────────────────────────────

if [ "$DRY_RUN" = "true" ]; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "DRY RUN — no changes made"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "would write to: $TARGET_FILE"
    echo "would back up to: ${TARGET_FILE}.pre-myth-${TIMESTAMP}"
    echo ""
    echo "preview of new hooks section:"
    echo "$NEW_CONTENT" | jq '.hooks'
    echo ""
    echo "preview of full new file (first 60 lines):"
    echo "$NEW_CONTENT" | jq '.' | head -60
    echo ""
    echo "to apply, re-run without --dry-run"
    exit 0
fi

# ─────────────────────────────────────────────────────────────────
# Apply changes
# ─────────────────────────────────────────────────────────────────

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "APPLYING changes"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Backup
if [ -f "$TARGET_FILE" ]; then
    BACKUP_FILE="${TARGET_FILE}.pre-myth-${TIMESTAMP}"
    cp "$TARGET_FILE" "$BACKUP_FILE"
    echo "✓ backup: $BACKUP_FILE"
fi

# Rename security-gate.sh if exists (myth Bedrock supersedes it)
SECURITY_GATE="$CLAUDE_DIR/hooks/security-gate.sh"
if [ -f "$SECURITY_GATE" ]; then
    DISABLED_PATH="${SECURITY_GATE}.disabled-by-myth-${TIMESTAMP}"
    mv "$SECURITY_GATE" "$DISABLED_PATH"
    echo "✓ renamed: $SECURITY_GATE → ${DISABLED_PATH##*/}"
fi

# Write new content (pretty-printed)
echo "$NEW_CONTENT" | jq '.' > "$TARGET_FILE"
echo "✓ wrote: $TARGET_FILE"

# ─────────────────────────────────────────────────────────────────
# Verify
# ─────────────────────────────────────────────────────────────────

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "VERIFICATION"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# JSON valid
if jq empty "$TARGET_FILE" > /dev/null 2>&1; then
    echo "✓ JSON valid"
else
    echo "ERROR: written JSON is invalid. Restoring backup..." >&2
    [ -n "${BACKUP_FILE:-}" ] && cp "$BACKUP_FILE" "$TARGET_FILE"
    exit 1
fi

# Myth hooks present
MYTH_COUNT=$(jq '[.hooks[][]?.hooks[]? | select(.command | test("myth-hook-"))] | length' "$TARGET_FILE")
echo "✓ myth hooks registered: $MYTH_COUNT entries"

# Test invocation
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "TEST: invoking myth-hook-pre-tool with a safe command (ls)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

TEST_ENVELOPE=$(jq -n --arg cwd "$PROJECT_PATH" '{
    session_id: "550e8400-e29b-41d4-a716-446655440000",
    transcript_path: "/tmp/t.jsonl",
    cwd: $cwd,
    hook_event_name: "PreToolUse",
    tool_name: "Bash",
    tool_input: { command: "ls" }
}')

if TEST_RESULT=$(echo "$TEST_ENVELOPE" | myth-hook-pre-tool 2>&1); then
    echo "✓ test invocation succeeded"
    echo "  response (first 200 chars): ${TEST_RESULT:0:200}"
else
    TEST_RC=$?
    if [ "$TEST_RC" -eq 0 ] || [ "$TEST_RC" -eq 2 ]; then
        # 0 = allow, 2 = block — both are valid hook exits
        echo "✓ test invocation returned exit code $TEST_RC (valid)"
    else
        echo "⚠ test invocation exit code: $TEST_RC"
        echo "  response: ${TEST_RESULT:0:300}"
    fi
fi

# ─────────────────────────────────────────────────────────────────
# Post-install guidance
# ─────────────────────────────────────────────────────────────────

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "INSTALL COMPLETE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "next steps:"
echo "  0. commit the installation changes so review-loop.sh doesn't flag them"
echo "     as a security regression on the next session:"
echo "     cd $PROJECT_PATH && git add -A .claude .gitignore && \\"
echo "         git commit -m 'chore(claude): install myth hooks'"
echo ""
echo "  1. test in fresh Claude Code session:"
echo "     cd $PROJECT_PATH && claude"
echo "     (try a simple command like 'ls' or 'git status')"
echo ""
echo "  2. observe hook events:"
echo "     sqlite3 ~/.myth/state.db \"SELECT datetime(created_at,'unixepoch','localtime'),tool_name,verdict FROM hook_events ORDER BY created_at DESC LIMIT 10\""
echo ""
echo "  3. rollback if needed:"
[ -n "${BACKUP_FILE:-}" ] && echo "     cp $BACKUP_FILE $TARGET_FILE"
echo "     mv $CLAUDE_DIR/hooks/security-gate.sh.disabled-by-myth-${TIMESTAMP} $CLAUDE_DIR/hooks/security-gate.sh"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
