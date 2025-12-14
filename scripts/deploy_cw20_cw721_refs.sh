#!/usr/bin/env bash
set -euo pipefail
# Deploy reference CW20 and CW721 contracts to the running wasmd chain for compatibility testing.
# SLTN remains a native token; these are supplemental.

CONTAINER=${CONTAINER:-cosmos-sultan}
CHAIN_ID=${CHAIN_ID:-sultan-1}
KEYRING=${KEYRING:-test}
CWPLUS_VERSION=${CWPLUS_VERSION:-v1.1.0}

have_jq() { command -v jq >/dev/null 2>&1; }

if ! docker ps | grep -q "${CONTAINER}"; then
  echo "❌ Container ${CONTAINER} not running"; exit 1; fi

if ! curl -sf http://127.0.0.1:26657/status >/dev/null; then
  echo "❌ RPC not responding on 26657"; exit 1; fi

VAL=$(docker exec "$CONTAINER" wasmd keys show validator -a --keyring-backend "$KEYRING")
NEW_ADDR_JSON=$(docker exec "$CONTAINER" sh -lc 'wasmd keys add cwuser --keyring-backend '"$KEYRING"' --output json 2>/dev/null || wasmd keys show cwuser --keyring-backend '"$KEYRING"' --output json')
CWUSER=$(echo "$NEW_ADDR_JSON" | jq -r .address)

printf "Validator: %s\nCW User: %s\n" "$VAL" "$CWUSER"

# Helper: wait for next block
wait_next_block() {
  local h1 h2
  h1=$(curl -sf http://127.0.0.1:26657/status | jq -r '.result.sync_info.latest_block_height') || h1=0
  for _ in $(seq 1 15); do
    sleep 1
    h2=$(curl -sf http://127.0.0.1:26657/status | jq -r '.result.sync_info.latest_block_height') || h2=0
    [ "$h2" != "$h1" ] && break
  done
}

# Download artifacts inside container (avoids host permission issues)
ART_DIR=/root/wasm_artifacts
docker exec "$CONTAINER" sh -lc 'mkdir -p '"$ART_DIR"''

echo "⬇️ Downloading cw-plus artifacts (cw20_base, cw721_base) version ${CWPLUS_VERSION}";
SKIP_CW721=0
for file in cw20_base.wasm cw721_base.wasm; do
  # Prefer local artifact if available
  if [ -f "/workspaces/0xv7/cw-artifacts/$file" ]; then
    echo "📦 Using local artifact: $file"
    docker cp "/workspaces/0xv7/cw-artifacts/$file" "$CONTAINER:$ART_DIR/$file"
  elif [ "$file" = "cw721_base.wasm" ]; then
  if [ "$file" = "cw721_base.wasm" ]; then
    # Try release asset first
    docker exec "$CONTAINER" sh -lc 'curl -fsSL -o '"$ART_DIR/$file"' https://github.com/CosmWasm/cw-plus/releases/download/'"$CWPLUS_VERSION"'/'"$file"' || curl -fsSL -o '"$ART_DIR/$file"' https://raw.githubusercontent.com/CosmWasm/cw-plus/'"$CWPLUS_VERSION"'/artifacts/'"$file"' || curl -fsSL -o '"$ART_DIR/$file"' https://raw.githubusercontent.com/CosmWasm/cw-nfts/'"$CWPLUS_VERSION"'/artifacts/'"$file"'' || true
  else
    docker exec "$CONTAINER" sh -lc 'curl -fsSL -o '"$ART_DIR/$file"' https://github.com/CosmWasm/cw-plus/releases/download/'"$CWPLUS_VERSION"'/'"$file"' || curl -fsSL -o '"$ART_DIR/$file"' https://raw.githubusercontent.com/CosmWasm/cw-plus/'"$CWPLUS_VERSION"'/artifacts/'"$file"'' || true
  fi
  docker exec "$CONTAINER" sh -lc 'ls -lh '"$ART_DIR/$file"' || true'
  SIZE=$(docker exec "$CONTAINER" sh -lc 'stat -c %s '"$ART_DIR/$file"' 2>/dev/null || echo 0')
  if [ "$file" = "cw721_base.wasm" ] && [ "$SIZE" -lt 50000 ]; then
     echo "⚠️ cw721_base.wasm appears invalid (size=$SIZE); will skip CW721 deployment"
     SKIP_CW721=1
  else
     docker exec "$CONTAINER" sh -lc 'sha256sum '"$ART_DIR/$file"' || true'
  fi
done

store_contract() {
  local wasm_file=$1
  echo "🗄 Storing $wasm_file"
  local TX
  TX=$(docker exec "$CONTAINER" sh -lc 'wasmd tx wasm store '"$ART_DIR/$wasm_file"' --from validator --chain-id '"$CHAIN_ID"' --keyring-backend '"$KEYRING"' --gas 2500000 --fees 0usltn --yes -o json --broadcast-mode sync' || true)
  if have_jq; then echo "$TX" | jq -r '"code="+(.code|tostring)+" log="+(.raw_log//"")'; fi
  if have_jq; then CODE_ID=$(echo "$TX" | jq -r '.code_id // .logs[0].events[]?|select(.type=="store_code").attributes[]?|select(.key=="code_id").value' | grep -E '^[0-9]+$' | head -n1); fi
  if [ -z "${CODE_ID:-}" ]; then
    echo "❌ Failed to extract code_id"; echo "$TX"; exit 1; fi
  echo "$CODE_ID"
}

instantiate_cw20() {
  local code_id=$1
  echo "🚀 Instantiating CW20 (code_id=$code_id)"
  docker exec -i "$CONTAINER" sh -s <<'EOF'
set -e
VAL_ADDR=$(wasmd keys show validator -a --keyring-backend test)
cat >/tmp/init_cw20.json <<JSON
{
  "name": "Sultan Token",
  "symbol": "SLTNREF",
  "decimals": 6,
  "initial_balances": [{"address": "${VAL_ADDR}", "amount": "5000000"}],
  "mint": {"minter": "${VAL_ADDR}"},
  "marketing": {}
}
JSON
EOF
  local TX=$(docker exec "$CONTAINER" sh -lc 'wasmd tx wasm instantiate '"$code_id"' /tmp/init_cw20.json --label sltn-cw20-ref --admin '"$VAL"' --from validator --chain-id '"$CHAIN_ID"' --keyring-backend '"$KEYRING"' --fees 0usltn --gas 800000 --yes -o json --broadcast-mode sync' || true)
  if have_jq; then echo "$TX" | jq -r '"code="+(.code|tostring)+" log="+(.raw_log//"")'; fi
  wait_next_block
  local Q=$(docker exec "$CONTAINER" wasmd query wasm list-contract-by-code "$code_id" -o json)
  local ADDR=$(echo "$Q" | jq -r '.contracts[-1]')
  echo "$ADDR"
}

instantiate_cw721() {
  local code_id=$1
  echo "🚀 Instantiating CW721 (code_id=$code_id)"
  local INIT='{"name":"Sultan Collectibles","symbol":"SLTNFT"}'
  local TX=$(docker exec "$CONTAINER" sh -lc 'wasmd tx wasm instantiate '"$code_id"' '"$INIT"' --label sltn-cw721-ref --admin '"$VAL"' --from validator --chain-id '"$CHAIN_ID"' --keyring-backend '"$KEYRING"' --fees 0usltn --gas 800000 --yes -o json --broadcast-mode sync' || true)
  if have_jq; then echo "$TX" | jq -r '"code="+(.code|tostring)+" log="+(.raw_log//"")'; fi
  wait_next_block
  local Q=$(docker exec "$CONTAINER" wasmd query wasm list-contract-by-code "$code_id" -o json)
  local ADDR=$(echo "$Q" | jq -r '.contracts[-1]')
  echo "$ADDR"
}

transfer_cw20() {
  local contract=$1
  echo "💸 CW20 transfer 1234 SLTNREF to $CWUSER"
  docker exec -i "$CONTAINER" sh -s <<'EOF'
set -e
RECIP="'$CWUSER'"
cat >/tmp/exec_transfer.json <<JSON
{"transfer":{"recipient":'$CWUSER',"amount":"1234"}}
JSON
EOF
  local TX=$(docker exec "$CONTAINER" sh -lc 'wasmd tx wasm execute '"$contract"' /tmp/exec_transfer.json --from validator --chain-id '"$CHAIN_ID"' --keyring-backend '"$KEYRING"' --fees 0usltn --gas 400000 --yes -o json --broadcast-mode sync' || true)
  if have_jq; then echo "$TX" | jq -r '"code="+(.code|tostring)+" log="+(.raw_log//"")'; fi
  wait_next_block
  local BAL=$(docker exec "$CONTAINER" wasmd query wasm contract-state smart "$contract" '{"balance":{"address":"'$CWUSER'"}}' -o json | jq -r '.data.balance // .data')
  echo "Balance cwuser: $BAL"
}

# MAIN FLOW
echo "🔍 Checking existing stored codes for CW20 hash"
CW20_HASH=$(docker exec "$CONTAINER" sha256sum "$ART_DIR/cw20_base.wasm" | awk '{print toupper($1)}')
EXISTING_ID=$(docker exec "$CONTAINER" wasmd query wasm list-code -o json | jq -r '.code_infos[] | select(.data_hash=="'"$CW20_HASH"'") | .code_id' | head -n1)
if [ -n "${EXISTING_ID:-}" ]; then
  echo "✅ CW20 code already stored (code_id=${EXISTING_ID})"
  CW20_CODE_ID=$EXISTING_ID
else
  CW20_CODE_ID=$(store_contract cw20_base.wasm)
  echo "cw20 code_id=$CW20_CODE_ID"; wait_next_block
fi
CW20_ADDR_LIST=$(docker exec "$CONTAINER" wasmd query wasm list-contract-by-code "$CW20_CODE_ID" -o json | jq -r '.contracts[]?' || true)
if [ -n "${CW20_ADDR_LIST}" ]; then
  CW20_ADDR=$(echo "$CW20_ADDR_LIST" | tail -n1)
  echo "✅ Existing CW20 contract detected: $CW20_ADDR"
else
  CW20_ADDR=$(instantiate_cw20 "$CW20_CODE_ID")
  echo "cw20 addr=$CW20_ADDR"; wait_next_block
fi
transfer_cw20 "$CW20_ADDR"

echo "🔎 Query CW20 token_info"
docker exec "$CONTAINER" wasmd query wasm contract-state smart "$CW20_ADDR" '{"token_info":{}}' -o json | jq -r '.data.name // .data'

if [ "$SKIP_CW721" -eq 0 ]; then
  CW721_CODE_ID=$(store_contract cw721_base.wasm)
  echo "cw721 code_id=$CW721_CODE_ID"; wait_next_block
  CW721_ADDR=$(instantiate_cw721 "$CW721_CODE_ID")
  echo "cw721 addr=$CW721_ADDR"; wait_next_block
  INFO=$(docker exec "$CONTAINER" wasmd query wasm contract "$CW721_ADDR" -o json || true)
  if have_jq; then echo "$INFO" | jq -r '.contract_info.label'; fi
else
  echo "⏭ Skipped CW721 deployment due to missing valid artifact"
fi

if [ "$SKIP_CW721" -eq 0 ]; then
  echo "✅ CW20 & CW721 reference deployment complete"
else
  echo "✅ CW20 reference deployment complete (CW721 skipped – artifact unavailable)"
fi
