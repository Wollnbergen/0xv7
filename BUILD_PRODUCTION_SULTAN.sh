#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║      BUILDING YOUR ACTUAL PRODUCTION SULTAN CHAIN             ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/node

# First, let's see what we're working with
echo "📦 Your actual Cargo.toml binaries:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
grep -A2 "\[\[bin\]\]" Cargo.toml

echo ""
echo "🔨 Building ALL your binaries:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Build everything
cargo build --all 2>&1 | tail -20

echo ""
echo "✅ Built binaries:"
ls -la target/debug/ | grep -E "sultan|rpc|wallet|production" | grep -v ".d$"

echo ""
echo "🚀 Starting Sultan Chain services:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Start RPC server if it exists
if [ -f "target/debug/rpc_server" ]; then
    echo "Starting RPC server..."
    ./target/debug/rpc_server &
    RPC_PID=$!
    echo "✅ RPC Server started (PID: $RPC_PID)"
fi

# Start sultan_node if it exists
if [ -f "target/debug/sultan_node" ]; then
    echo "Starting Sultan node..."
    ./target/debug/sultan_node &
    NODE_PID=$!
    echo "✅ Sultan Node started (PID: $NODE_PID)"
fi

# Start production_test if it exists
if [ -f "target/debug/production_test" ]; then
    echo "Running production test..."
    timeout 10 ./target/debug/production_test 2>&1 | head -20
fi

sleep 3

echo ""
echo "�� CHECKING SERVICES:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
ps aux | grep -E "sultan|rpc" | grep -v grep

echo ""
echo "🧪 TESTING ENDPOINTS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Test RPC
curl -s -X POST http://localhost:3030 \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"chain_getInfo","id":1}' | jq . 2>/dev/null || echo "RPC not responding on 3030"

# Test alternate port
curl -s -X POST http://localhost:26657 \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"status","id":1}' | jq . 2>/dev/null || echo "RPC not responding on 26657"

