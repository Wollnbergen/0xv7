#!/bin/bash

clear
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║     LAUNCHING SULTAN CHAIN - WORKING COMPONENTS ONLY          ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# 1. Launch the Python API server (we know this works)
echo "🐍 Starting Python API Server..."
cd /workspaces/0xv7
if [ -f "sultan_actual_api.py" ]; then
    python3 sultan_actual_api.py > /tmp/api.log 2>&1 &
    API_PID=$!
    echo "✅ API Server started (PID: $API_PID)"
fi

# 2. Launch JavaScript consensus
echo "📦 Starting JavaScript Consensus..."
cd /workspaces/0xv7/consensus
if [ -f "working_consensus.mjs" ]; then
    node working_consensus.mjs > /tmp/consensus.log 2>&1 &
    CONSENSUS_PID=$!
    echo "✅ Consensus started (PID: $CONSENSUS_PID)"
fi

# 3. Try the Rust implementation
echo "🦀 Attempting Rust Node..."
cd /workspaces/0xv7/sultan_mainnet
if [ -f "target/release/sultan-mainnet" ]; then
    ./target/release/sultan-mainnet > /tmp/mainnet.log 2>&1 &
    RUST_PID=$!
    echo "✅ Rust node started (PID: $RUST_PID)"
elif cargo build --release 2>&1 | grep -q "Finished"; then
    ./target/release/sultan-mainnet > /tmp/mainnet.log 2>&1 &
    RUST_PID=$!
    echo "✅ Rust node built and started (PID: $RUST_PID)"
fi

sleep 3

echo ""
echo "📊 RUNNING SERVICES:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
ps aux | grep -E "sultan|consensus|api" | grep -v grep

echo ""
echo "🌐 AVAILABLE ENDPOINTS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check what's actually responding
for PORT in 3000 3030 8080 26657; do
    if curl -s http://localhost:$PORT > /dev/null 2>&1; then
        echo "✅ http://localhost:$PORT - ACTIVE"
    fi
done

echo ""
echo "📋 QUICK TEST:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "curl http://localhost:3030"
curl -s http://localhost:3030 2>/dev/null | head -5 || echo "Not responding yet"

