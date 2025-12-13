#!/usr/bin/env bash
set -euo pipefail

# Additional DDoS / resource hardening for wasmd container.
# Idempotent config edits + optional host iptables rate limiting.

CONTAINER=${CONTAINER:-cosmos-sultan}
INBOUND_PEERS=${INBOUND_PEERS:-8}
OUTBOUND_PEERS=${OUTBOUND_PEERS:-8}
MAX_OPEN_CONNS=${MAX_OPEN_CONNS:-48}
MEMPOOL_SIZE=${MEMPOOL_SIZE:-3000}
MAX_TXS_BYTES=${MAX_TXS_BYTES:-536870912} # 512MB
SEND_RATE=${SEND_RATE:-512000}
RECV_RATE=${RECV_RATE:-512000}
RPC_PORT=${RPC_PORT:-26657}
RATE_LIMIT_CONN_PER_MIN=${RATE_LIMIT_CONN_PER_MIN:-120}

echo "🛡️  DDoS hardening for container $CONTAINER"
docker ps --format '{{.Names}}' | grep -qw "$CONTAINER" || { echo "❌ Container not running"; exit 1; }

echo "🔧 Applying config.toml tweaks"
docker exec "$CONTAINER" sh -lc "\
  set -e; \
  CFG=\"\$HOME/.wasmd/config/config.toml\"; \
  sed -i 's/^max_open_connections = .*/max_open_connections = ${MAX_OPEN_CONNS}/' \"\$CFG\" || true; \
  sed -i 's/^max_num_inbound_peers = .*/max_num_inbound_peers = ${INBOUND_PEERS}/' \"\$CFG\" || true; \
  sed -i 's/^max_num_outbound_peers = .*/max_num_outbound_peers = ${OUTBOUND_PEERS}/' \"\$CFG\" || true; \
  sed -i 's/^addr_book_strict = .*/addr_book_strict = true/' \"\$CFG\" || true; \
  sed -i 's/^pex = .*/pex = true/' \"\$CFG\" || true; \
  sed -i 's/^seed_mode = .*/seed_mode = false/' \"\$CFG\" || true; \
  sed -i 's/^size = .*/size = ${MEMPOOL_SIZE}/' \"\$CFG\" || true; \
  sed -i 's/^max_txs_bytes = .*/max_txs_bytes = ${MAX_TXS_BYTES}/' \"\$CFG\" || true; \
  sed -i 's/^send_rate = .*/send_rate = ${SEND_RATE}/' \"\$CFG\" || true; \
  sed -i 's/^recv_rate = .*/recv_rate = ${RECV_RATE}/' \"\$CFG\" || true; \
  true"

echo "♻️  Restarting container to apply changes"
docker restart "$CONTAINER" >/dev/null

echo "⏳ Waiting for RPC $RPC_PORT"
for i in $(seq 1 20); do 
  sleep 1; 
  if curl -sf "http://127.0.0.1:${RPC_PORT}/status" >/dev/null; then echo "✅ RPC online"; break; fi 
done

if command -v sudo >/dev/null 2>&1 && command -v iptables >/dev/null 2>&1; then
  echo "🔒 Optional host iptables rate-limit for new RPC connections"
  set +e
  sudo iptables -C INPUT -p tcp --dport ${RPC_PORT} -m state --state NEW -m limit --limit ${RATE_LIMIT_CONN_PER_MIN}/minute -j ACCEPT 2>/dev/null || \
    sudo iptables -A INPUT -p tcp --dport ${RPC_PORT} -m state --state NEW -m limit --limit ${RATE_LIMIT_CONN_PER_MIN}/minute -j ACCEPT
  sudo iptables -C INPUT -p tcp --dport ${RPC_PORT} -m state --state NEW -j DROP 2>/dev/null || \
    sudo iptables -A INPUT -p tcp --dport ${RPC_PORT} -m state --state NEW -j DROP
  set -e
  echo "✅ Basic RPC connection rate limiting applied"
else
  echo "ℹ️  iptables/sudo unavailable; skipping host-level limit"
fi

echo "📝 peers(in/out)=${INBOUND_PEERS}/${OUTBOUND_PEERS} open_conns=${MAX_OPEN_CONNS} mempool=${MEMPOOL_SIZE}"
echo "✅ DDoS hardening complete"
