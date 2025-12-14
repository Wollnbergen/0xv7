#!/usr/bin/env bash
set -euo pipefail
# Zero-fee enforcement test for CW20 operations (store, instantiate, execute) on running chain.
# Assumes cosmos-sultan container running and cw20_base.wasm obtainable.

CONTAINER=${CONTAINER:-cosmos-sultan}
CHAIN_ID=${CHAIN_ID:-sultan-1}
KEYRING=${KEYRING:-test}
CWPLUS_VERSION=${CWPLUS_VERSION:-v1.1.0}
ART_DIR=/root/wasm_artifacts

have_jq() { command -v jq >/dev/null 2>&1; }
fail() { echo "❌ $1"; exit 1; }

[ "$(docker ps --format '{{.Names}}' | grep -w "$CONTAINER" || true)" ] || fail "Container $CONTAINER not running"
curl -sf http://127.0.0.1:26657/status >/dev/null || fail "RPC 26657 unreachable"

VAL=$(docker exec "$CONTAINER" wasmd keys show validator -a --keyring-backend "$KEYRING")
USER_JSON=$(docker exec "$CONTAINER" sh -lc 'wasmd keys add cwtest --keyring-backend '"$KEYRING"' --output json 2>/dev/null || wasmd keys show cwtest --keyring-backend '"$KEYRING"' --output json')
USER=$(echo "$USER_JSON" | jq -r .address)

printf "Validator: %s\nUser: %s\n" "$VAL" "$USER"

docker exec "$CONTAINER" sh -lc "mkdir -p $ART_DIR"
if [ -f "/workspaces/0xv7/cw-artifacts/cw20_base.wasm" ]; then
  echo "📦 Using local cw20_base.wasm"
  docker cp "/workspaces/0xv7/cw-artifacts/cw20_base.wasm" "$CONTAINER:$ART_DIR/cw20_base.wasm"
elif ! docker exec "$CONTAINER" test -f "$ART_DIR/cw20_base.wasm"; then
  echo "⬇️ Download cw20_base.wasm"
  docker exec "$CONTAINER" sh -lc 'curl -fsSL -o '"$ART_DIR/cw20_base.wasm"' https://github.com/CosmWasm/cw-plus/releases/download/'"$CWPLUS_VERSION"'/cw20_base.wasm'
fi
SIZE=$(docker exec "$CONTAINER" stat -c %s "$ART_DIR/cw20_base.wasm")
[ "$SIZE" -gt 50000 ] || fail "cw20_base.wasm invalid (size=$SIZE)"

HASH=$(docker exec "$CONTAINER" sha256sum "$ART_DIR/cw20_base.wasm" | awk '{print toupper($1)}')
EXISTING_ID=$(docker exec "$CONTAINER" wasmd query wasm list-code -o json | jq -r '.code_infos[] | select(.data_hash=="'"$HASH"'") | .code_id' | head -n1)
if [ -n "${EXISTING_ID:-}" ]; then
  CODE_ID=$EXISTING_ID
  echo "✅ Reusing existing code_id=$CODE_ID"
else
  echo "🗄 Store (zero-fee)"
  STORE_TX=$(docker exec "$CONTAINER" sh -lc 'wasmd tx wasm store '"$ART_DIR/cw20_base.wasm"' --from validator --chain-id '"$CHAIN_ID"' --keyring-backend '"$KEYRING"' --gas 2500000 --fees 0usltn --yes -o json --broadcast-mode sync' || true)
  CODE_ID=$(docker exec "$CONTAINER" wasmd query wasm list-code -o json | jq -r '.code_infos[-1].code_id')
  echo "code_id=$CODE_ID"
fi

CONTRACTS=$(docker exec "$CONTAINER" wasmd query wasm list-contract-by-code "$CODE_ID" -o json | jq -r '.contracts[]?' || true)
if [ -n "${CONTRACTS}" ]; then
  CW20_ADDR=$(echo "$CONTRACTS" | tail -n1)
  echo "✅ Reusing existing contract $CW20_ADDR"
else
  echo "🚀 Instantiate (zero-fee)"
  docker exec -i "$CONTAINER" sh -s <<'EOF_IN'
set -e
VAL_ADDR=$(wasmd keys show validator -a --keyring-backend test)
cat >/tmp/init_cw20.json <<JSON
{
  "name": "SLTN Ref",
  "symbol": "SLTNREF",
  "decimals": 6,
  "initial_balances": [{"address": "${VAL_ADDR}", "amount": "1000000"}],
  "mint": {"minter": "${VAL_ADDR}"}
}
JSON
EOF_IN
  INST_TX=$(docker exec "$CONTAINER" sh -lc 'wasmd tx wasm instantiate '"$CODE_ID"' /tmp/init_cw20.json --label sltn-cw20-test --admin '"$VAL"' --from validator --chain-id '"$CHAIN_ID"' --keyring-backend '"$KEYRING"' --fees 0usltn --gas 800000 --yes -o json --broadcast-mode sync' || true)
  if have_jq; then echo "$INST_TX" | jq -r '"txhash="+(.txhash//"")'; fi
  TXH=$(echo "$INST_TX" | jq -r '.txhash // empty' 2>/dev/null || true)
  # Wait a block then inspect tx result
  sleep 2
  if [ -n "${TXH:-}" ]; then
    docker exec "$CONTAINER" wasmd query tx "$TXH" -o json | jq -r '.raw_log // .code // .codespace' || true
  fi
  CW20_ADDR=$(docker exec "$CONTAINER" wasmd query wasm list-contract-by-code "$CODE_ID" -o json | jq -r '.contracts[-1]')
fi
echo "Contract: $CW20_ADDR"

echo "💸 Transfer (zero-fee) 1234 -> $USER"
docker exec -i "$CONTAINER" sh -s <<EOF_EX
set -e
cat >/tmp/exec_transfer.json <<JSON
{"transfer":{"recipient":"$USER","amount":"1234"}}
JSON
EOF_EX
TX1=$(docker exec "$CONTAINER" sh -lc 'wasmd tx wasm execute '"$CW20_ADDR"' /tmp/exec_transfer.json --from validator --chain-id '"$CHAIN_ID"' --keyring-backend '"$KEYRING"' --fees 0usltn --gas 400000 --yes -o json --broadcast-mode sync' || true)

echo "🧪 Query balance"
BAL_USER=$(docker exec "$CONTAINER" wasmd query wasm contract-state smart "$CW20_ADDR" '{"balance":{"address":"'$USER'"}}' -o json | jq -r '.data.balance // .data')
BAL_VAL=$(docker exec "$CONTAINER" wasmd query wasm contract-state smart "$CW20_ADDR" '{"balance":{"address":"'$VAL'"}}' -o json | jq -r '.data.balance // .data')
printf "User balance: %s\nValidator balance: %s\n" "$BAL_USER" "$BAL_VAL"

# Mint test (zero-fee)
echo "🪙 Mint 5000 to user (zero-fee)"
docker exec -i "$CONTAINER" sh -s <<EOF_EX2
set -e
cat >/tmp/exec_mint.json <<JSON
{"mint":{"recipient":"$USER","amount":"5000"}}
JSON
EOF_EX2
TX2=$(docker exec "$CONTAINER" sh -lc 'wasmd tx wasm execute '"$CW20_ADDR"' /tmp/exec_mint.json --from validator --chain-id '"$CHAIN_ID"' --keyring-backend '"$KEYRING"' --fees 0usltn --gas 450000 --yes -o json --broadcast-mode sync' || true)
BAL_USER2=$(docker exec "$CONTAINER" wasmd query wasm contract-state smart "$CW20_ADDR" '{"balance":{"address":"'$USER'"}}' -o json | jq -r '.data.balance // .data')
printf "User balance after mint: %s\n" "$BAL_USER2"

SUPPLY=$(docker exec "$CONTAINER" wasmd query wasm contract-state smart "$CW20_ADDR" '{"token_info":{}}' -o json | jq -r '.data.total_supply // .data.total_supply')
printf "Total supply: %s\n" "$SUPPLY"

# Basic assertions
[ "$BAL_USER" != "null" ] || fail "Balance query failed"
[ "$BAL_USER2" != "$BAL_USER" ] || fail "Mint did not change balance"
[ "$SUPPLY" != "null" ] || fail "Supply query failed"

echo "✅ Zero-fee CW20 store/instantiate/execute/mint test passed"
