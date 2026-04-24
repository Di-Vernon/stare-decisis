#!/usr/bin/env bash
# Migrate Anthropic API key from a source .env-style file into the
# canonical myth credentials store (~/.myth/credentials, 0600).
#
# Usage:
#   bash scripts/migrate_key.sh                       # default source = experiment .env
#   bash scripts/migrate_key.sh /path/to/source.env   # explicit source
#
# Idempotent: re-running with the same key just rewrites the file.
# Refuses to clobber an existing canonical key without --force.
#
# Piggy-backs on `myth key set --from-stdin`, so the same validation
# (sk-ant- prefix, 0600 chmod) applies. Direct file write is avoided so
# future format changes only need updating one code path.

set -euo pipefail

SOURCE="${1:-$HOME/myth/experiment/remand-prototype/.env}"
FORCE="${FORCE:-0}"

if [[ ! -f "$SOURCE" ]]; then
    echo "error: source file not found: $SOURCE" >&2
    exit 1
fi

KEY=$(grep -E '^ANTHROPIC_API_KEY=' "$SOURCE" | head -1 | cut -d= -f2-)
if [[ -z "$KEY" ]]; then
    echo "error: no ANTHROPIC_API_KEY=... line in $SOURCE" >&2
    exit 1
fi

TARGET="$HOME/.myth/credentials"
if [[ -f "$TARGET" && "$FORCE" != "1" ]]; then
    EXISTING=$(grep -E '^ANTHROPIC_API_KEY=' "$TARGET" 2>/dev/null | head -1 | cut -d= -f2- || true)
    if [[ "$EXISTING" == "$KEY" ]]; then
        echo "no-op: $TARGET already holds this key"
        exit 0
    fi
    echo "error: $TARGET exists with a different key. Re-run with FORCE=1 to overwrite." >&2
    exit 2
fi

# Resolve the myth binary: PATH first, then default install location.
MYTH="$(command -v myth 2>/dev/null || true)"
if [[ -z "$MYTH" && -x "$HOME/.local/bin/myth" ]]; then
    MYTH="$HOME/.local/bin/myth"
fi
if [[ -z "$MYTH" ]]; then
    echo "error: myth binary not on PATH and not at ~/.local/bin/myth" >&2
    echo "       run \`cargo install --path rust/crates/myth-cli\` or \`myth install\`" >&2
    exit 3
fi

echo "$KEY" | "$MYTH" key set --from-stdin >/dev/null
"$MYTH" key show
echo "migrated $SOURCE -> $TARGET (via $MYTH key set)"
