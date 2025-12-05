#!/usr/bin/env bash
set -euo pipefail

# Probe downtime slashing by stopping the secondary validator briefly and
# observing missed blocks counter via slashing signing-info.

PRIMARY=${PRIMARY:-cosmos-sultan}
SECOND=${SECOND:-cosmos-sultan-2}
KEYRING=${KEYRING:-test}
RPC=${RPC:-http://127.0.0.1:26657}
BLOCKS_OFFLINE=${BLOCKS_OFFLINE:-15}

fail() { echo "❌ $1"; exit 1; }
info() { echo "➡️  $1"; }

command -v jq >/dev/null 2>&1 || fail "jq required"

docker ps --format '{{.Names}}' | grep -qw "$PRIMARY" || fail "Primary $PRIMARY not running"
curl -sf "$RPC/status" >/dev/null || fail "Primary RPC unreachable: $RPC"

if ! docker ps --format '{{.Names}}' | grep -qw "$SECOND"; then
  fail "Secondary $SECOND not running; run scripts/validator_lifecycle_test.sh first"
fi

VALCONS=$(docker exec "$SECOND" wasmd tendermint show-address)
[ -n "$VALCONS" ] || fail "Could not obtain valcons address from $SECOND"
info "valcons: $VALCONS"

BEFORE=$(docker exec "$PRIMARY" wasmd query slashing signing-info "$VALCONS" -o json || echo '{}')
MISS_BEFORE=$(echo "$BEFORE" | jq -r '.missed_blocks_counter // 0')
JAILED_BEFORE=$(echo "$BEFORE" | jq -r '.jailed_until // ""')
H0=$(curl -sf "$RPC/status" | jq -r '.result.sync_info.latest_block_height')

info "Stopping secondary validator $SECOND"
docker stop "$SECOND" >/dev/null

TARGET=$((H0 + BLOCKS_OFFLINE))
info "Waiting for $BLOCKS_OFFLINE blocks on primary (from $H0 to $TARGET)"
for i in $(seq 1 120); do
  H=$(curl -sf "$RPC/status" | jq -r '.result.sync_info.latest_block_height')
  [ "$H" -ge "$TARGET" ] && break
  sleep 1
done
H1=$(curl -sf "$RPC/status" | jq -r '.result.sync_info.latest_block_height')

AFTER=$(docker exec "$PRIMARY" wasmd query slashing signing-info "$VALCONS" -o json || echo '{}')
MISS_AFTER=$(echo "$AFTER" | jq -r '.missed_blocks_counter // 0')
JAILED_AFTER=$(echo "$AFTER" | jq -r '.jailed_until // ""')

info "Restarting secondary validator $SECOND"
docker start "$SECOND" >/dev/null
sleep 3

DELTA=$((MISS_AFTER - MISS_BEFORE))
echo "Summary: height_start=$H0 height_end=$H1 missed_before=$MISS_BEFORE missed_after=$MISS_AFTER delta=$DELTA jailed_before=$JAILED_BEFORE jailed_after=$JAILED_AFTER"

OUTDIR=/workspaces/0xv7/security/reports
mkdir -p "$OUTDIR"
STAMP=$(date -u +%Y%m%dT%H%M%SZ)
jq -n \
  --arg valcons "$VALCONS" \
  --arg h_start "$H0" \
  --arg h_end "$H1" \
  --arg miss_before "$MISS_BEFORE" \
  --arg miss_after "$MISS_AFTER" \
  --arg jailed_before "$JAILED_BEFORE" \
  --arg jailed_after "$JAILED_AFTER" \
  '{valcons:$valcons, height_start: ($h_start|tonumber), height_end: ($h_end|tonumber), missed_before: ($miss_before|tonumber), missed_after: ($miss_after|tonumber), jailed_before: $jailed_before, jailed_after: $jailed_after}' \
  > "$OUTDIR/slashing_probe_${STAMP}.json"

if [ "$DELTA" -gt 0 ]; then
  echo "✅ Downtime evidenced: missed_blocks increased by $DELTA (report saved)"
else
  echo "ℹ️  No increase observed. Increase BLOCKS_OFFLINE or check validator set."
fi
