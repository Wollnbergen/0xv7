#!/usr/bin/env bash
set -euo pipefail

# Harden RPC and P2P settings inside the running cosmwasm/wasmd container.
# Usage: CONTAINER=cosmos-sultan ./scripts/harden_sultan_node.sh

CONTAINER=${CONTAINER:-cosmos-sultan}

echo "🔐 Hardening node in container: $CONTAINER"

docker ps --format '{{.Names}}' | grep -qw "$CONTAINER" || {
  echo "❌ Container $CONTAINER not running"; exit 1; }

docker exec "$CONTAINER" sh -lc '
  set -e
  CFG="$HOME/.wasmd/config/config.toml"
  APP="$HOME/.wasmd/config/app.toml"
  # RPC: limit open connections
  sed -i "s/^max_open_connections = .*/max_open_connections = 64/" "$CFG" || true
  # RPC: reduce body/header sizes (defense-in-depth)
  sed -i "s/^max_body_bytes = .*/max_body_bytes = 1048576/" "$CFG" || true
  sed -i "s/^max_header_bytes = .*/max_header_bytes = 1048576/" "$CFG" || true
  # P2P: tighten peer counts and enable strict addrbook
  sed -i "s/^max_num_inbound_peers = .*/max_num_inbound_peers = 10/" "$CFG" || true
  sed -i "s/^max_num_outbound_peers = .*/max_num_outbound_peers = 10/" "$CFG" || true
  sed -i "s/^addr_book_strict = .*/addr_book_strict = true/" "$CFG" || true
  # PEX enabled (peer exchange), seed_mode off
  sed -i "s/^pex = .*/pex = true/" "$CFG" || true
  sed -i "s/^seed_mode = .*/seed_mode = false/" "$CFG" || true
  # Mempool tuning
  sed -i "s/^size = .*/size = 5000/" "$CFG" || true
  sed -i "s/^max_txs_bytes = .*/max_txs_bytes = 1073741824/" "$CFG" || true
  # API/gRPC already enabled in app.toml; ensure unsafe flags not on
  if [ -f "$APP" ]; then
    sed -i "s/^enabled-unsafe-cors = .*/enabled-unsafe-cors = false/" "$APP" || true
  fi
'

echo "♻️ Restarting container to apply config"
docker restart "$CONTAINER" >/dev/null

echo "⏳ Waiting for RPC"
for i in $(seq 1 20); do
  sleep 1
  if curl -sf http://127.0.0.1:26657/status >/dev/null; then
    echo "✅ RPC back online"; exit 0
  fi
done

echo "⚠️ RPC not reachable after restart (check logs)"
exit 1
