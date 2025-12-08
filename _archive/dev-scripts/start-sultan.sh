#!/bin/bash
# Sultan L1 - Quick Start Script
# Starts the blockchain node and web dashboard

set -e

echo "════════════════════════════════════════════════════════════"
echo "           🚀 Starting Sultan L1 Blockchain 🚀              "
echo "════════════════════════════════════════════════════════════"
echo ""

# Check if binary exists (workspace builds to /tmp/cargo-target)
BINARY_PATH="/tmp/cargo-target/release/sultan-node"
if [ ! -f "$BINARY_PATH" ]; then
    echo "❌ sultan-node binary not found!"
    echo "   Building now..."
    cd /workspaces/0xv7
    cargo build --release -p sultan-core --bin sultan-node
    echo "✅ Build complete"
fi

# Stop any existing instances
echo "🧹 Cleaning up old processes..."
pkill -9 sultan-node 2>/dev/null || true
pkill -9 -f "python3 -m http.server 8080" 2>/dev/null || true
sleep 2

# Start the node
echo "🚀 Starting Sultan L1 node..."
cd /workspaces/0xv7/sultan-core
$BINARY_PATH \
  --validator \
  --validator-address sultan1validator7xj3k2p8n9m5q4r6t8v0w2y4z6a8c0e2g4 \
  --validator-stake 10000000000000 \
  --enable-sharding \
  --shard-count 100 \
  --tx-per-shard 10000 \
  --block-time 2 \
  --data-dir ../sultan-data \
  --rpc-addr 0.0.0.0:26657 \
  --p2p-addr 0.0.0.0:26656 \
  > ../sultan-node.log 2>&1 &

NODE_PID=$!
echo "✅ Node started (PID: $NODE_PID)"
echo "   Logs: tail -f /workspaces/0xv7/sultan-node.log"

# Wait for node to be ready
echo ""
echo "⏳ Waiting for node to initialize..."
for i in {1..10}; do
    sleep 1
    if curl -s http://localhost:26657/status > /dev/null 2>&1; then
        echo "✅ Node is ready!"
        break
    fi
    echo -n "."
done

# Check if node is running
echo ""
if curl -s http://localhost:26657/status > /dev/null 2>&1; then
    BLOCK_HEIGHT=$(curl -s http://localhost:26657/status | jq -r '.result.sync_info.latest_block_height // "N/A"' 2>/dev/null || echo "N/A")
    echo "✅ Node Status: RUNNING"
    echo "   Block Height: $BLOCK_HEIGHT"
    echo "   RPC: http://localhost:26657"
else
    echo "❌ Node failed to start!"
    echo "   Check logs: tail -f /workspaces/0xv7/sultan-node.log"
    exit 1
fi

# Start website
echo ""
echo "🌐 Starting web dashboard..."
cd /workspaces/0xv7
python3 -m http.server 8080 > website.log 2>&1 &
WEBSITE_PID=$!
echo "✅ Website started (PID: $WEBSITE_PID)"
echo "   URL: http://localhost:8080"

# Save PIDs
echo $NODE_PID > /workspaces/0xv7/.sultan-node.pid
echo $WEBSITE_PID > /workspaces/0xv7/.sultan-website.pid

echo ""
echo "════════════════════════════════════════════════════════════"
echo "           ✅ Sultan L1 is Running! ✅                       "
echo "════════════════════════════════════════════════════════════"
echo ""
echo "📊 Quick Access:"
echo "   • Node RPC:    http://localhost:26657/status"
echo "   • Website:     http://localhost:8080"
echo "   • Keplr Setup: http://localhost:8080/add-to-keplr.html"
echo ""
echo "🔍 Testing:"
echo "   • Staking/Gov: bash test_staking_governance.sh"
echo "   • Bridge Fees: bash test_bridge_fees.sh"
echo ""
echo "📝 Logs:"
echo "   • Node:        tail -f sultan-node.log"
echo "   • Website:     tail -f website.log"
echo ""
echo "🛑 Shutdown:"
echo "   • Clean stop:  bash shutdown-sultan.sh"
echo ""
echo "════════════════════════════════════════════════════════════"
