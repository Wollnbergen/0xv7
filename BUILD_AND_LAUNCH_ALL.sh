#!/bin/bash

clear
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         BUILDING SULTAN CHAIN - ALL COMPONENTS                ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Build the main node
echo "🔨 Building Sultan Node..."
cd /workspaces/0xv7/node
cargo build --release --bin sultan_node 2>&1 | grep -E "Compiling|Finished" | tail -5

# Build RPC servers
echo "🔨 Building RPC Services..."
cargo build --release --bin rpc_server 2>&1 | grep "Finished"
cargo build --release --bin rpcd 2>&1 | grep "Finished"

# Build interop bridges
echo "🌉 Building Bridge Services..."
cd /workspaces/0xv7/sultan-interop
cargo build --release 2>&1 | grep "Finished"

echo ""
echo "✅ BUILD COMPLETE! Starting services..."
echo ""

# Start everything
cd /workspaces/0xv7

# Kill existing processes
pkill -f sultan 2>/dev/null
pkill -f rpc 2>/dev/null
sleep 2

echo "🚀 LAUNCHING SERVICES:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 1. Sultan Node
if [ -f "node/target/release/sultan_node" ]; then
    node/target/release/sultan_node > /tmp/sultan.log 2>&1 &
    echo "✅ Sultan Node started (PID: $!)"
fi

# 2. RPC Server
if [ -f "node/target/release/rpc_server" ]; then
    node/target/release/rpc_server > /tmp/rpc.log 2>&1 &
    echo "✅ RPC Server started (PID: $!)"
fi

# 3. Bridge Service
if [ -f "sultan-interop/target/release/sultan-interop" ]; then
    sultan-interop/target/release/sultan-interop > /tmp/bridge.log 2>&1 &
    echo "✅ Bridge Service started (PID: $!)"
fi

sleep 3

echo ""
echo "📊 CHECKING SERVICES:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
ps aux | grep -E "sultan|rpc|bridge" | grep -v grep | awk '{print "  • " $11}'

echo ""
echo "🌐 TESTING ENDPOINTS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
for PORT in 3000 3030 8080 26657; do
    if curl -s http://localhost:$PORT > /dev/null 2>&1; then
        echo "✅ Port $PORT: ACTIVE"
    fi
done

echo ""
echo "📈 LIVE METRICS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s -X POST http://localhost:3030 \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"chain_status","id":1}' | jq '.'

