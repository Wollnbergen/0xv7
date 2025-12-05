#!/usr/bin/env bash
set -euo pipefail

# Validator lifecycle + groundwork for slashing (downtime) test.
# Steps:
# 1. Ensure primary validator container running.
# 2. Launch second node container and join existing network (reuse p2p script logic simplified).
# 3. Create key val2 on second node.
# 4. Fund val2 with stake + usltn from primary.
# 5. Create second validator (val2) using its consensus pubkey.
# 6. Delegate additional stake to val2 (increase voting power for later downtime slash simulation).
# 7. Begin (optional) downtime prep: instructions for pausing primary later while chain continues.
#
# Note: To later test downtime slashing, val2 must hold >2/3 of total voting power so blocks continue when primary stops.
# This script increases val2 power via extra delegation.

PRIMARY=${PRIMARY:-cosmos-sultan}
SECOND=${SECOND:-cosmos-sultan-2}
CHAIN_ID=${CHAIN_ID:-sultan-1}
KEYRING=${KEYRING:-test}
STAKE_DENOM=${STAKE_DENOM:-stake}
FEE_DENOM=${FEE_DENOM:-usltn}
VAL_KEY=${VAL_KEY:-validator}
VAL2_KEY=${VAL2_KEY:-val2}
NET_NAME=${NET_NAME:-sultan-net}

fail() { echo "❌ $1"; exit 1; }
info() { echo "➡️  $1"; }

command -v jq >/dev/null || fail "jq required"

info "Check primary container running"
docker ps --format '{{.Names}}' | grep -qw "$PRIMARY" || fail "Primary container $PRIMARY not running"

RPC_PRIMARY=${RPC_PRIMARY:-http://127.0.0.1:26657}
curl -sf "$RPC_PRIMARY/status" >/dev/null || fail "Primary RPC unreachable"

info "Create/join docker network $NET_NAME"
docker network inspect "$NET_NAME" >/dev/null 2>&1 || docker network create "$NET_NAME"
set +e; docker network connect "$NET_NAME" "$PRIMARY" 2>/dev/null; set -e

info "Remove old second container if exists"
docker rm -f "$SECOND" >/dev/null 2>&1 || true

SECOND_DATA_HOST=${SECOND_DATA_HOST:-/workspaces/0xv7/cosmos-data-2}
mkdir -p "$SECOND_DATA_HOST"

info "Start second node base container (persistent)"
docker run -d --name "$SECOND" --network "$NET_NAME" \
  -v "$SECOND_DATA_HOST":/root/.wasmd \
  cosmwasm/wasmd:latest sh -c 'while true; do sleep 3600; done' >/dev/null

info "Init second node (chain-id) and enable APIs"
docker exec "$SECOND" sh -lc 'wasmd init sultan-node-2 --chain-id '"$CHAIN_ID"' >/dev/null 2>&1 || true'
docker exec "$SECOND" sh -lc 'sed -i "s#laddr = \"tcp://127.0.0.1:26656\"#laddr = \"tcp://0.0.0.0:26656\"#" $HOME/.wasmd/config/config.toml || true'
docker exec "$SECOND" sh -lc 'sed -i "s#laddr = \"tcp://127.0.0.1:26657\"#laddr = \"tcp://0.0.0.0:26657\"#" $HOME/.wasmd/config/config.toml || true'
docker exec "$SECOND" sh -lc 'sed -i "s/^enable = false/enable = true/" $HOME/.wasmd/config/app.toml || true'
docker exec "$SECOND" sh -lc 'grep -q "minimum-gas-prices" $HOME/.wasmd/config/app.toml && sed -i "s/^minimum-gas-prices.*/minimum-gas-prices = \"0usltn\"/" $HOME/.wasmd/config/app.toml || echo "minimum-gas-prices = \"0usltn\"" >> $HOME/.wasmd/config/app.toml'

info "Copy genesis from primary"
TMP_GEN=$(mktemp)
if docker cp "$PRIMARY:/root/.wasmd/config/genesis.json" "$TMP_GEN" 2>/dev/null; then
  docker cp "$TMP_GEN" "$SECOND:/root/.wasmd/config/genesis.json"
else
  fail "Could not copy genesis from primary"
fi
rm -f "$TMP_GEN"

info "Fetch primary node id"
PRIMARY_ID=$(docker exec "$PRIMARY" wasmd tendermint show-node-id)
PEERS_VAL="${PRIMARY_ID}@${PRIMARY}:26656"
docker exec "$SECOND" sh -lc 'CFG=$HOME/.wasmd/config/config.toml; if grep -q "^persistent_peers" "$CFG"; then sed -i "s#^persistent_peers = \".*\"#persistent_peers = \"'"$PEERS_VAL"'\"#" "$CFG"; else echo "persistent_peers = \"'"$PEERS_VAL"'\"" >> "$CFG"; fi'

info "Start second node wasmd"
docker exec -d "$SECOND" sh -lc 'wasmd start --minimum-gas-prices 0usltn'
sleep 5

info "Create val2 key on second node"
docker exec "$SECOND" sh -lc 'wasmd keys add '"$VAL2_KEY"' --keyring-backend '"$KEYRING"' --output json' > /tmp/val2_key.json 2>/dev/null || true
VAL2_ADDR=$(docker exec "$SECOND" wasmd keys show "$VAL2_KEY" -a --keyring-backend "$KEYRING")
[ -n "$VAL2_ADDR" ] || fail "val2 address missing"
echo "val2 address: $VAL2_ADDR"

info "Fund val2 from validator (stake + usltn)"
VAL_ADDR=$(docker exec "$PRIMARY" wasmd keys show "$VAL_KEY" -a --keyring-backend "$KEYRING")
SEND_AMTS="1000000${STAKE_DENOM},1000000${FEE_DENOM}"
TX_FUND=$(docker exec "$PRIMARY" wasmd tx bank send "$VAL_KEY" "$VAL2_ADDR" "$SEND_AMTS" --chain-id "$CHAIN_ID" --fees 0${FEE_DENOM} --yes --broadcast-mode sync --keyring-backend "$KEYRING" -o json || true)
if [ "$(echo "$TX_FUND" | jq -r .code)" != "0" ]; then echo "$TX_FUND" | jq '.'; fail "Funding val2 failed"; fi

info "Wait one block after funding"
H_BEFORE=$(curl -sf "$RPC_PRIMARY/status" | jq -r '.result.sync_info.latest_block_height')
TARGET=$((H_BEFORE+1))
for i in $(seq 1 20); do H_CUR=$(curl -sf "$RPC_PRIMARY/status" | jq -r '.result.sync_info.latest_block_height'); [ "$H_CUR" -ge "$TARGET" ] && break || sleep 1; done

info "Query val2 balance"
BAL_VAL2=$(docker exec "$SECOND" wasmd query bank balances "$VAL2_ADDR" -o json | jq -r '.balances[]? | select(.denom=="'"$STAKE_DENOM"'") | .amount')
echo "val2 stake balance: $BAL_VAL2"

info "Fetch second node consensus pubkey"
VAL2_PUB=$(docker exec "$SECOND" wasmd tendermint show-validator)
[ -n "$VAL2_PUB" ] || fail "Consensus pubkey missing"

SELF_DELEGATE_AMT=${SELF_DELEGATE_AMT:-500000${STAKE_DENOM}}
info "Create second validator (self-delegate $SELF_DELEGATE_AMT)"
TMP_JSON=$(mktemp)
cat > "$TMP_JSON" <<JSON
{
  "pubkey": $VAL2_PUB,
  "amount": "$SELF_DELEGATE_AMT",
  "moniker": "sultan-val2",
  "identity": "",
  "website": "",
  "security": "",
  "details": "secondary validator",
  "commission-rate": "0.10",
  "commission-max-rate": "0.20",
  "commission-max-change-rate": "0.01",
  "min-self-delegation": "1"
}
JSON
docker cp "$TMP_JSON" "$SECOND:/root/validator_val2.json"
rm -f "$TMP_JSON"
CREATE_TX=$(docker exec "$SECOND" wasmd tx staking create-validator /root/validator_val2.json --from "$VAL2_KEY" --chain-id "$CHAIN_ID" --fees 0${FEE_DENOM} --gas auto --gas-adjustment 1.3 --yes --broadcast-mode sync --keyring-backend "$KEYRING" -o json || true)
if [ "$(echo "$CREATE_TX" | jq -r .code)" != "0" ]; then 
  RAW=$(echo "$CREATE_TX" | jq -r .raw_log)
  if echo "$RAW" | grep -qi "validator already exist"; then
    info "Validator already exists for $VAL2_KEY; continuing"
  else
    echo "$CREATE_TX" | jq '.'
    fail "create-validator failed"
  fi
fi

info "Wait block after create-validator"
H_BEFORE2=$(curl -sf "$RPC_PRIMARY/status" | jq -r '.result.sync_info.latest_block_height')
TARGET2=$((H_BEFORE2+1))
for i in $(seq 1 20); do H_CUR=$(curl -sf "$RPC_PRIMARY/status" | jq -r '.result.sync_info.latest_block_height'); [ "$H_CUR" -ge "$TARGET2" ] && break || sleep 1; done

info "List validators (expect val2 present)"
docker exec "$PRIMARY" wasmd query staking validators -o json | jq -r '.validators[] | "- " + .description.moniker + " voting_power=" + .tokens'

EXTRA_DELEGATE_AMT=${EXTRA_DELEGATE_AMT:-300000${STAKE_DENOM}}
info "Delegate extra power to val2 from primary (amt=$EXTRA_DELEGATE_AMT)"
VAL2_VALOPER=$(docker exec "$SECOND" wasmd keys show "$VAL2_KEY" --bech val -a --keyring-backend "$KEYRING")
DELEG_TX=$(docker exec "$PRIMARY" wasmd tx staking delegate "$VAL2_VALOPER" "$EXTRA_DELEGATE_AMT" --from "$VAL_KEY" --chain-id "$CHAIN_ID" --fees 0${FEE_DENOM} --yes --broadcast-mode sync --keyring-backend "$KEYRING" -o json || true)
if [ "$(echo "$DELEG_TX" | jq -r .code)" != "0" ]; then echo "$DELEG_TX" | jq '.'; fail "Extra delegate failed"; fi

info "Wait block after extra delegation"
H_BEFORE3=$(curl -sf "$RPC_PRIMARY/status" | jq -r '.result.sync_info.latest_block_height')
TARGET3=$((H_BEFORE3+1))
for i in $(seq 1 20); do H_CUR=$(curl -sf "$RPC_PRIMARY/status" | jq -r '.result.sync_info.latest_block_height'); [ "$H_CUR" -ge "$TARGET3" ] && break || sleep 1; done

info "Validator powers (post extra delegation)"
docker exec "$PRIMARY" wasmd query staking validators -o json | jq -r '.validators[] | "* " + .description.moniker + ": tokens=" + .tokens'

echo "----- Slashing Prep Guidance -----"
echo "To simulate downtime slash: ensure val2 now holds >2/3 power. Stop primary: docker stop $PRIMARY. Observe blocks continue (if power threshold satisfied). After configured missed blocks window, val2 would be jailed if it were the one stopped. For double-sign test, requires forging duplicate vote with same height/round from different priv_validator_state (advanced)."
echo "Script complete: second validator established and powered up."
echo "✅ Validator lifecycle test finished"

info "Unbond a portion from val2 (test unbond path)"
UNBOND_AMT=${UNBOND_AMT:-100000${STAKE_DENOM}}
VAL2_VALOPER=$(docker exec "$SECOND" wasmd keys show "$VAL2_KEY" --bech val -a --keyring-backend "$KEYRING")
UNBOND_TX=$(docker exec "$SECOND" wasmd tx staking unbond "$VAL2_VALOPER" "$UNBOND_AMT" --from "$VAL2_KEY" --chain-id "$CHAIN_ID" --fees 0${FEE_DENOM} --yes --broadcast-mode sync --keyring-backend "$KEYRING" -o json || true)
if [ "$(echo "$UNBOND_TX" | jq -r .code)" != "0" ]; then echo "$UNBOND_TX" | jq '.'; echo "⚠️ Unbond tx failed (may require more time since delegation)"; else info "Unbond tx accepted"; fi
info "Wait one block after unbond"
H_U=$(curl -sf "$RPC_PRIMARY/status" | jq -r '.result.sync_info.latest_block_height'); TARGET_U=$((H_U+1)); for i in $(seq 1 15); do H_CUR=$(curl -sf "$RPC_PRIMARY/status" | jq -r '.result.sync_info.latest_block_height'); [ "$H_CUR" -ge "$TARGET_U" ] && break || sleep 1; done

info "Redelegate from primary to val2 (test redelegation path)"
PRIMARY_VALOPER=$(docker exec "$PRIMARY" wasmd keys show "$VAL_KEY" --bech val -a --keyring-backend "$KEYRING")
REDELEGATE_AMT=${REDELEGATE_AMT:-50000${STAKE_DENOM}}
REDEL_TX=$(docker exec "$PRIMARY" wasmd tx staking redelegate "$PRIMARY_VALOPER" "$VAL2_VALOPER" "$REDELEGATE_AMT" --from "$VAL_KEY" --chain-id "$CHAIN_ID" --fees 0${FEE_DENOM} --yes --broadcast-mode sync --keyring-backend "$KEYRING" -o json || true)
if [ "$(echo "$REDEL_TX" | jq -r .code)" != "0" ]; then echo "$REDEL_TX" | jq '.'; echo "⚠️ Redelegate tx failed (cooldown / constraints?)"; else info "Redelegate tx accepted"; fi
info "Wait block after redelegation"
H_R=$(curl -sf "$RPC_PRIMARY/status" | jq -r '.result.sync_info.latest_block_height'); TARGET_R=$((H_R+1)); for i in $(seq 1 15); do H_CUR=$(curl -sf "$RPC_PRIMARY/status" | jq -r '.result.sync_info.latest_block_height'); [ "$H_CUR" -ge "$TARGET_R" ] && break || sleep 1; done

info "Final validator set snapshot"
docker exec "$PRIMARY" wasmd query staking validators -o json | jq -r '.validators[] | "• " + .description.moniker + " tokens=" + .tokens'

echo "✅ Extended lifecycle (unbond & redelegate) complete"
