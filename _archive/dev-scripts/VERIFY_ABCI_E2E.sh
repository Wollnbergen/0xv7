#!/usr/bin/env bash
set -euo pipefail

echo "⏱ ABCI E2E verification (bank + staking, zero-fee)"

name_or_id=${1:-cosmos-sultan}

have_jq() { command -v jq >/dev/null 2>&1; }

echo "\n== Container =="
docker ps --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}' | grep -E "(^|\s)${name_or_id}(\s|$)" || {
  echo "❌ Container ${name_or_id} not running"; exit 1;
}

echo "\n== RPC height =="
curl -sf http://localhost:26657/status | { if have_jq; then jq -r '.result.sync_info.latest_block_height'; else cat; fi; } || echo "rpc down"

echo "\n== Addresses =="
VAL=$(docker exec "$name_or_id" wasmd keys show validator -a --keyring-backend test)
echo "validator: $VAL"
RECIP_JSON=$(docker exec "$name_or_id" sh -lc 'wasmd keys add bankuser --keyring-backend test --output json 2>/dev/null || wasmd keys show bankuser --keyring-backend test --output json')
RECIP=$(echo "$RECIP_JSON" | { if have_jq; then jq -r .address; else sed -n 's/.*"address"\s*:\s*"\([^"]\+\)".*/\1/p'; fi; })
echo "bankuser: $RECIP"

echo "\n== Pre-balances (validator) =="
docker exec "$name_or_id" wasmd q bank balances "$VAL" -o json | { if have_jq; then jq -r '.balances[]? | .denom+":"+.amount'; else cat; fi; }

echo "\n== Bank send 1000usltn (fees=0usltn) =="
TXR=$(docker exec "$name_or_id" sh -lc 'wasmd tx bank send validator '"$RECIP"' 1000usltn --fees 0usltn --yes --chain-id sultan-1 --broadcast-mode sync --keyring-backend test -o json' || true)
if have_jq; then echo "$TXR" | jq -r '"code="+(.code|tostring)+" log="+(.raw_log//"")'; else echo "$TXR"; fi

echo "\n== Recipient balances =="
docker exec "$name_or_id" wasmd q bank balances "$RECIP" -o json | { if have_jq; then jq -r '.balances[]? | .denom+":"+.amount'; else cat; fi; }

# Wait one block to avoid sequence mismatch between consecutive txs
echo "\n== Wait for next block =="
H_BEFORE=$(curl -sf http://localhost:26657/status | { if have_jq; then jq -r '.result.sync_info.latest_block_height'; else cat; fi; } || echo 0)
for i in $(seq 1 10); do
  H_NOW=$(curl -sf http://localhost:26657/status | { if have_jq; then jq -r '.result.sync_info.latest_block_height'; else cat; fi; } || echo 0)
  [ "$H_NOW" != "$H_BEFORE" ] && break
  sleep 1
done

echo "\n== Staking delegate 100stake (fees=0usltn) =="
VALOPER=$(docker exec "$name_or_id" wasmd keys show validator --keyring-backend test --bech val -a 2>/dev/null || docker exec "$name_or_id" wasmd q staking validators -o json | jq -r '.validators[0].operator_address')
echo "valoper: $VALOPER"
STX=$(docker exec "$name_or_id" sh -lc 'wasmd tx staking delegate '"$VALOPER"' 100stake --from validator --chain-id sultan-1 --fees 0usltn --yes --broadcast-mode sync --keyring-backend test -o json' || true)
if have_jq; then echo "$STX" | jq -r '"code="+(.code|tostring)+" log="+(.raw_log//"")'; else echo "$STX"; fi

echo "\n== Delegations (validator) =="
docker exec "$name_or_id" wasmd q staking delegations "$VAL" -o json | { if have_jq; then jq -r '.delegation_responses[]? | .delegation.shares'; else cat; fi; }

echo "\n== Final RPC height =="
curl -sf http://localhost:26657/status | { if have_jq; then jq -r '.result.sync_info.latest_block_height'; else cat; fi; } || echo "rpc down"

echo "\n✅ ABCI verification script finished"
