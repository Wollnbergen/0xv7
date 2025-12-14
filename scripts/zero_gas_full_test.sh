#!/usr/bin/env bash
set -euo pipefail
# Aggregated zero-gas verification: SDK bank + staking, CW20 store/instantiate/transfer/mint.
# Assumes cosmos-sultan running locally.

CONTAINER=${CONTAINER:-cosmos-sultan}
CHAIN_ID=${CHAIN_ID:-sultan-1}
KEYRING=${KEYRING:-test}

have_jq() { command -v jq >/dev/null 2>&1; }
fail() { echo "❌ $1"; exit 1; }

echo "🔎 Checking container & RPC"
docker ps --format '{{.Names}}' | grep -qw "$CONTAINER" || fail "Container $CONTAINER not running"
curl -sf http://127.0.0.1:26657/status >/dev/null || fail "RPC 26657 unreachable"

VAL=$(docker exec "$CONTAINER" wasmd keys show validator -a --keyring-backend "$KEYRING")
RECIP_JSON=$(docker exec "$CONTAINER" sh -lc 'wasmd keys add zgasuser --keyring-backend '"$KEYRING"' --output json 2>/dev/null || wasmd keys show zgasuser --keyring-backend '"$KEYRING"' --output json')
RECIP=$(echo "$RECIP_JSON" | jq -r .address)
echo "Validator=$VAL Recipient=$RECIP"

H_BEFORE=$(curl -sf http://127.0.0.1:26657/status | jq -r '.result.sync_info.latest_block_height')
echo "Starting height=$H_BEFORE"

echo "💸 Bank send 1000usltn (fee 0usltn)"
TX_BANK=$(docker exec "$CONTAINER" wasmd tx bank send validator "$RECIP" 1000usltn --fees 0usltn --yes --chain-id "$CHAIN_ID" --broadcast-mode sync --keyring-backend "$KEYRING" -o json || true)
[ "$(echo "$TX_BANK" | jq -r .code)" = "0" ] || fail "Bank send failed: $(echo "$TX_BANK" | jq -r .raw_log)"

BAL_RECIP=$(docker exec "$CONTAINER" wasmd q bank balances "$RECIP" -o json | jq -r '.balances[]? | select(.denom=="usltn") | .amount')
echo "Recipient usltn balance=$BAL_RECIP"

echo "⏱ Waiting one block to finalize bank tx"
TARGET=$((H_BEFORE+1))
for i in $(seq 1 15); do
  H_CUR=$(curl -sf http://127.0.0.1:26657/status | jq -r '.result.sync_info.latest_block_height')
  [ "$H_CUR" -ge "$TARGET" ] && break || sleep 1
done
H_AFTER_BANK=$(curl -sf http://127.0.0.1:26657/status | jq -r '.result.sync_info.latest_block_height')
echo "Height after bank=$H_AFTER_BANK"

echo "🪧 Staking delegate 50stake (fee 0usltn)"
VALOPER=$(docker exec "$CONTAINER" wasmd keys show validator --keyring-backend "$KEYRING" --bech val -a)
TX_STAKE=$(docker exec "$CONTAINER" wasmd tx staking delegate "$VALOPER" 50stake --from validator --chain-id "$CHAIN_ID" --fees 0usltn --yes --broadcast-mode sync --keyring-backend "$KEYRING" -o json || true)
[ "$(echo "$TX_STAKE" | jq -r .code)" = "0" ] || fail "Delegate failed: $(echo "$TX_STAKE" | jq -r .raw_log)"

DELEG=$(docker exec "$CONTAINER" wasmd q staking delegations "$VAL" -o json | jq -r '.delegation_responses[-1].delegation.shares')
echo "Latest delegation shares=$DELEG"

echo "📦 CW20 fresh store (avoid legacy incompatible code_ids)"
ART=/root/wasm_artifacts/cw20_latest.wasm
docker exec "$CONTAINER" sh -lc 'mkdir -p /root/wasm_artifacts'
if [ -f /workspaces/0xv7/cw-artifacts/cw20_base.wasm ]; then
  docker cp /workspaces/0xv7/cw-artifacts/cw20_base.wasm "$CONTAINER:$ART"
else
  docker exec "$CONTAINER" sh -lc 'command -v curl >/dev/null 2>&1 || apk add --no-cache curl; curl -fsSL -o '"$ART"' https://github.com/CosmWasm/cw-plus/releases/download/v1.1.0/cw20_base.wasm'
fi
SIZE=$(docker exec "$CONTAINER" stat -c %s "$ART")
[ "$SIZE" -gt 50000 ] || fail "CW20 artifact invalid size=$SIZE"
STORE_TX=$(docker exec "$CONTAINER" wasmd tx wasm store "$ART" --from validator --chain-id "$CHAIN_ID" --fees 0usltn --gas 2500000 --yes --broadcast-mode sync --keyring-backend "$KEYRING" -o json || true)
CODE_ID=$(docker exec "$CONTAINER" wasmd query wasm list-code -o json | jq -r '.code_infos[-1].code_id')
[ -n "$CODE_ID" ] || fail "No code_id after store"
echo "Stored new code_id=$CODE_ID"

echo "⏱ Waiting one block to finalize store tx"
H_STORE_WAIT_TARGET=$(( $(curl -sf http://127.0.0.1:26657/status | jq -r '.result.sync_info.latest_block_height') + 1 ))
for i in $(seq 1 15); do
  H_CUR=$(curl -sf http://127.0.0.1:26657/status | jq -r '.result.sync_info.latest_block_height')
  [ "$H_CUR" -ge "$H_STORE_WAIT_TARGET" ] && break || sleep 1
done

echo "🚀 CW20 instantiate (inline JSON, zero-fee)"
INIT='{"name":"ZeroGasToken","symbol":"ZGT","decimals":6,"initial_balances":[{"address":"'$VAL'","amount":"500000"}],"mint":{"minter":"'$VAL'"}}'
INST_TX=$(docker exec "$CONTAINER" wasmd tx wasm instantiate "$CODE_ID" "$INIT" --label zgt-inline --admin "$VAL" --from validator --fees 0usltn --gas 900000 --yes --broadcast-mode sync -o json --chain-id "$CHAIN_ID" --keyring-backend "$KEYRING" || true)
if [ "$(echo "$INST_TX" | jq -r .code)" != "0" ]; then
  echo "$INST_TX" | jq '.' || true
  fail "Instantiate failed"
fi
CONTRACT=$(docker exec "$CONTAINER" wasmd query wasm list-contract-by-code "$CODE_ID" -o json | jq -r '.contracts[-1]')
[ -n "$CONTRACT" ] || fail "No contract address after instantiate (legacy incompat?)"
echo "Contract=$CONTRACT"

echo "⏱ Waiting one block to finalize instantiate tx"
H_INST_WAIT_TARGET=$(( $(curl -sf http://127.0.0.1:26657/status | jq -r '.result.sync_info.latest_block_height') + 1 ))
for i in $(seq 1 15); do
  H_CUR=$(curl -sf http://127.0.0.1:26657/status | jq -r '.result.sync_info.latest_block_height')
  [ "$H_CUR" -ge "$H_INST_WAIT_TARGET" ] && break || sleep 1
done

echo "💸 CW20 transfer 1234 (zero-fee)"
TX_TRANSFER=$(docker exec "$CONTAINER" wasmd tx wasm execute "$CONTRACT" '{"transfer":{"recipient":"'$RECIP'","amount":"1234"}}' --from validator --fees 0usltn --gas 400000 --yes --broadcast-mode sync -o json --chain-id "$CHAIN_ID" --keyring-backend "$KEYRING" || true)
[ "$(echo "$TX_TRANSFER" | jq -r .code)" = "0" ] || fail "CW20 transfer failed"
echo "⏱ Waiting one block to finalize transfer tx"
H_XFER_WAIT_TARGET=$(( $(curl -sf http://127.0.0.1:26657/status | jq -r '.result.sync_info.latest_block_height') + 1 ))
for i in $(seq 1 15); do
  H_CUR=$(curl -sf http://127.0.0.1:26657/status | jq -r '.result.sync_info.latest_block_height')
  [ "$H_CUR" -ge "$H_XFER_WAIT_TARGET" ] && break || sleep 1
done
BAL_CW20_RECIP=$(docker exec "$CONTAINER" wasmd query wasm contract-state smart "$CONTRACT" '{"balance":{"address":"'$RECIP'"}}' -o json | jq -r '.data.balance')
echo "CW20 recipient balance=$BAL_CW20_RECIP"

echo "🪙 CW20 mint 5000 to recipient (zero-fee)"
TX_MINT=$(docker exec "$CONTAINER" wasmd tx wasm execute "$CONTRACT" '{"mint":{"recipient":"'$RECIP'","amount":"5000"}}' --from validator --fees 0usltn --gas 450000 --yes --broadcast-mode sync -o json --chain-id "$CHAIN_ID" --keyring-backend "$KEYRING" || true)
[ "$(echo "$TX_MINT" | jq -r .code)" = "0" ] || fail "CW20 mint failed"
echo "⏱ Waiting one block to finalize mint tx"
H_MINT_WAIT_TARGET=$(( $(curl -sf http://127.0.0.1:26657/status | jq -r '.result.sync_info.latest_block_height') + 1 ))
for i in $(seq 1 15); do
  H_CUR=$(curl -sf http://127.0.0.1:26657/status | jq -r '.result.sync_info.latest_block_height')
  [ "$H_CUR" -ge "$H_MINT_WAIT_TARGET" ] && break || sleep 1
done
BAL_CW20_RECIP2=$(docker exec "$CONTAINER" wasmd query wasm contract-state smart "$CONTRACT" '{"balance":{"address":"'$RECIP'"}}' -o json | jq -r '.data.balance')
SUPPLY=$(docker exec "$CONTAINER" wasmd query wasm contract-state smart "$CONTRACT" '{"token_info":{}}' -o json | jq -r '.data.total_supply')

H_AFTER=$(curl -sf http://127.0.0.1:26657/status | jq -r '.result.sync_info.latest_block_height')
echo "Final height=$H_AFTER (delta=$((H_AFTER-H_BEFORE)))"

echo "Summary:"
echo " Bank send code=0 balance=$BAL_RECIP"
echo " Delegate code=0 shares=$DELEG"
echo " CW20 contract=$CONTRACT transfer_balance=$BAL_CW20_RECIP mint_balance=$BAL_CW20_RECIP2 supply=$SUPPLY"

[[ "$BAL_CW20_RECIP" = "1234" && "$BAL_CW20_RECIP2" = "6234" ]] || fail "CW20 balances unexpected"
echo "✅ Zero-gas aggregated test PASSED"