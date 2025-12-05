#!/usr/bin/env bash
set -euo pipefail

ROOT=${1:-/workspaces/0xv7}
PRIMARY=${PRIMARY:-cosmos-sultan}
SECOND=${SECOND:-cosmos-sultan-2}

OUT_DIR="$ROOT/security/reports"
mkdir -p "$OUT_DIR"

VALCONS=$(docker exec "$SECOND" wasmd tendermint show-address)
VALS_JSON=$(docker exec "$PRIMARY" wasmd query staking validators -o json)

VAL2=$(echo "$VALS_JSON" | jq -r '.validators[] | select(.description.moniker=="sultan-val2")')
STATUS=$(echo "$VAL2" | jq -r '.status')
TOKENS=$(echo "$VAL2" | jq -r '.tokens')
JAILED=$(echo "$VAL2" | jq -r '.jailed')
SIGN=$(docker exec "$PRIMARY" wasmd query slashing signing-info "$VALCONS" -o json | jq -r '.val_signing_info')

STAMP=$(date -u +%Y%m%dT%H%M%SZ)
jq -n \
  --arg valcons "$VALCONS" \
  --arg status "$STATUS" \
  --arg tokens "$TOKENS" \
  --arg jailed "$JAILED" \
  --argjson signing "$SIGN" \
  '{valcons:$valcons, staking: {status:$status, tokens: ($tokens|tonumber), jailed: ($jailed|test("true"))}, slashing: $signing}' \
  > "$OUT_DIR/validator_status_${STAMP}.json"

echo "✅ Wrote $OUT_DIR/validator_status_${STAMP}.json"
