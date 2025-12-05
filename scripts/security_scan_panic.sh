#!/usr/bin/env bash
set -euo pipefail

# Scan repository for panic-prone Rust patterns and write JSON report.

ROOT=${1:-/workspaces/0xv7}
OUT_DIR="$ROOT/security/reports"
OUT_FILE="$OUT_DIR/panic_scan.json"
mkdir -p "$OUT_DIR"

echo "🔍 Scanning for unwrap()/expect()/panic! in Rust sources"
# Exclude VCS and runtime data dirs to reduce noise and permissions errors
EXCLUDES=(--exclude-dir=.git --exclude-dir=cosmos-data --exclude-dir=cosmos-data-2 --exclude-dir=sultan-cosmos)
MATCHES=$(grep -RInE '\bunwrap\(|\bexpect\(|panic!\(' "$ROOT" --include='*.rs' "${EXCLUDES[@]}" 2>/dev/null || true)
TOTAL=$(printf "%s" "$MATCHES" | grep -c ':' || true)

command -v jq >/dev/null 2>&1 || { echo "❌ jq not installed"; exit 1; }

# Use temporary file for raw matches to avoid long arg expansions
TMP=$(mktemp)
printf "%s" "$MATCHES" > "$TMP"
jq -n --arg total "$TOTAL" --rawfile matches "$TMP" '{total_findings: ($total|tonumber), raw: $matches}' > "$OUT_FILE"
rm -f "$TMP"
echo "✅ Panic scan complete -> $OUT_FILE (findings=$TOTAL)"

if [ "$TOTAL" != "0" ]; then
  echo "⚠️  Consider replacing unwrap/expect with proper error handling."
else
  echo "👍 No panic-prone patterns detected."
fi
