#!/usr/bin/env bash
set -euo pipefail

# Heuristic secret scan. Not a substitute for full tooling.

ROOT=${1:-/workspaces/0xv7}
OUT_DIR="$ROOT/security/reports"
OUT_FILE="$OUT_DIR/secret_scan.json"
mkdir -p "$OUT_DIR"

echo "🔐 Scanning for potential secrets"
PATTERNS='(PRIVATE_KEY|SECRET_KEY|MNEMONIC|seed phrase|password=|api[_-]?key)'
# Exclude VCS and runtime data dirs to reduce noise and permissions errors
EXCLUDES=(--exclude-dir=.git --exclude-dir=cosmos-data --exclude-dir=cosmos-data-2 --exclude-dir=sultan-cosmos)
RAW=$(grep -RInE "$PATTERNS" "$ROOT" "${EXCLUDES[@]}" 2>/dev/null || true)

# Long base64 strings heuristic (skip .git and runtime data)
RAW_B64=$(grep -RInE '[A-Za-z0-9+/]{40,}={0,2}' "$ROOT" "${EXCLUDES[@]}" 2>/dev/null || true)

command -v jq >/dev/null 2>&1 || { echo "❌ jq not installed"; exit 1; }

# Use temporary files to avoid extremely long argument expansions
TMP1=$(mktemp)
TMP2=$(mktemp)
echo "$RAW" > "$TMP1"
echo "$RAW_B64" > "$TMP2"
jq -n --rawfile matches "$TMP1" --rawfile b64 "$TMP2" '{keyword_matches: $matches, base64_candidates: $b64}' > "$OUT_FILE"
rm -f "$TMP1" "$TMP2"
echo "✅ Secret scan complete -> $OUT_FILE"
echo "Review candidates above for false positives."
