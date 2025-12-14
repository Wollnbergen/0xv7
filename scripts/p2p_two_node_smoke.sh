#!/usr/bin/env bash
set -euo pipefail

PRIMARY_NAME=${PRIMARY_NAME:-cosmos-sultan}
SECOND_NAME=${SECOND_NAME:-cosmos-sultan-2}
NET_NAME=${NET_NAME:-sultan-net}
PRIMARY_DATA=${PRIMARY_DATA:-/workspaces/0xv7/cosmos-data}
SECOND_DATA=${SECOND_DATA:-/workspaces/0xv7/cosmos-data-2}

echo "🔗 Creating/using Docker network: $NET_NAME"
docker network inspect "$NET_NAME" >/dev/null 2>&1 || docker network create "$NET_NAME"

echo "🔗 Connecting primary node to $NET_NAME (if not already)"
set +e
docker network connect "$NET_NAME" "$PRIMARY_NAME" 2>/dev/null
set -e

echo "📄 Ensuring second node data dir exists"
mkdir -p "$SECOND_DATA/config"

echo "🧹 Removing any old second node container"
docker rm -f "$SECOND_NAME" >/dev/null 2>&1 || true

echo "🚀 Starting second node container (on $NET_NAME)"
docker run -d --name "$SECOND_NAME" \
  --network "$NET_NAME" \
  -p 26666:26656 -p 26667:26657 -p 1318:1317 -p 9091:9090 \
  -v "$SECOND_DATA":/root/.wasmd \
  cosmwasm/wasmd:latest sleep 10 >/dev/null

echo "🔑 Initialize node keys and config (chain-id sultan-1)"
docker exec -i "$SECOND_NAME" sh <<'INNERSH'
set -e
if [ ! -f "$HOME/.wasmd/config/config.toml" ]; then
  wasmd init sultan-node-2 --chain-id sultan-1 >/dev/null 2>&1 || true
fi
# Bind addresses
sed -i "s#^laddr = \".*26656\"#laddr = \"tcp://0.0.0.0:26656\"#" "$HOME/.wasmd/config/config.toml" || true
sed -i "s#^laddr = \".*26657\"#laddr = \"tcp://0.0.0.0:26657\"#" "$HOME/.wasmd/config/config.toml" || true
# Enable API and gRPC
if [ -f "$HOME/.wasmd/config/app.toml" ]; then
  sed -i "s/^enable = false/enable = true/" "$HOME/.wasmd/config/app.toml" || true
  sed -i "s#^address = \".*1317\"#address = \"tcp://0.0.0.0:1317\"#" "$HOME/.wasmd/config/app.toml" || true
  sed -i "s#^address = \".*:9090\"#address = \"0.0.0.0:9090\"#" "$HOME/.wasmd/config/app.toml" || true
  if grep -q '^minimum-gas-prices' "$HOME/.wasmd/config/app.toml"; then
    sed -i 's/^minimum-gas-prices.*/minimum-gas-prices = "0usltn"/' "$HOME/.wasmd/config/app.toml"
  else
    echo 'minimum-gas-prices = "0usltn"' >> "$HOME/.wasmd/config/app.toml"
  fi
fi
INNERSH

echo "📥 Copying genesis from primary to second container"
TMP_GEN=$(mktemp)
set +e
docker cp "$PRIMARY_NAME:/root/.wasmd/config/genesis.json" "$TMP_GEN" 2>/dev/null
CP_RC=$?
set -e
if [ $CP_RC -ne 0 ]; then
  echo "⚠️  docker cp failed; attempting host path copy (may require permissions)"
  if [[ -f "$PRIMARY_DATA/config/genesis.json" ]]; then
    cp "$PRIMARY_DATA/config/genesis.json" "$TMP_GEN"
  else
    echo "❌ Could not obtain genesis.json from container or host. Aborting."
    rm -f "$TMP_GEN"
    exit 1
  fi
fi
docker cp "$TMP_GEN" "$SECOND_NAME:/root/.wasmd/config/genesis.json"
rm -f "$TMP_GEN"

echo "🆔 Fetch primary node ID"
PRIMARY_ID=$(docker exec "$PRIMARY_NAME" wasmd tendermint show-node-id)
echo "primary id: $PRIMARY_ID"

echo "🔧 Configure persistent_peers to primary"
PEERS_VAL="${PRIMARY_ID}@${PRIMARY_NAME}:26656"
docker exec --env PEERS_VAL="$PEERS_VAL" -i "$SECOND_NAME" sh <<'INNERSH'
set -e
CFG="$HOME/.wasmd/config/config.toml"
if grep -q "^persistent_peers =" "$CFG"; then
  sed -i "s#^persistent_peers = \".*\"#persistent_peers = \"${PEERS_VAL}\"#" "$CFG"
else
  echo "persistent_peers = \"${PEERS_VAL}\"" >> "$CFG"
fi
INNERSH

echo "▶️  Launch second node"
docker exec -d "$SECOND_NAME" sh -lc 'wasmd start --minimum-gas-prices 0usltn'

echo "⏳ Wait a few seconds for peering"
sleep 6

echo "🧪 net_info primary"
curl -sf http://127.0.0.1:26657/net_info | jq -r '.result.n_peers as $n | "peers="+$n|tostring, (.result.peers[]?.node_info.id // empty)' || true

echo "🧪 net_info second"
curl -sf http://127.0.0.1:26667/net_info | jq -r '.result.n_peers as $n | "peers="+$n|tostring, (.result.peers[]?.node_info.id // empty)' || true

echo "✅ P2P two-node smoke test attempted. Check peer counts above."
