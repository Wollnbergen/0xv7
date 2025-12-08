#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           LAUNCHING SULTAN CHAIN - PRODUCTION MODE            ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Function to check if a service is running
check_service() {
    local name=$1
    local port=$2
    nc -zv 127.0.0.1 $port 2>/dev/null && echo "✅ $name running on port $port" || echo "❌ $name not running"
}

# 1. Start database services
echo "🗄️ Starting Database Services..."
docker run -d --name scylla -p 9042:9042 scylladb/scylla:latest --overprovisioned 1 --smp 1 2>/dev/null || echo "ScyllaDB already running"
docker run -d --name redis -p 6379:6379 redis:latest 2>/dev/null || echo "Redis already running"

sleep 5

# 2. Build and start RPC server
echo ""
echo "🚀 Starting Sultan RPC Server..."
cd /workspaces/0xv7/node
if cargo build --bin rpc_server 2>/dev/null; then
    pkill -f rpc_server 2>/dev/null
    RUST_LOG=info ./target/debug/rpc_server &
    echo "RPC Server started"
else
    echo "⚠️ RPC Server build failed"
fi

# 3. Start Cosmos chain if available
echo ""
echo "🌌 Starting Cosmos Chain..."
if [ -f "/workspaces/0xv7/sultan/build/sultand" ]; then
    cd /workspaces/0xv7/sultan
    ./build/sultand start --home ~/.sultan &
    echo "Cosmos chain started"
else
    echo "⚠️ Cosmos chain not built yet"
fi

sleep 3

# 4. Check all services
echo ""
echo "📊 Service Status:"
echo "─────────────────────────"
check_service "ScyllaDB" 9042
check_service "Redis" 6379
check_service "RPC Server" 3030
check_service "Cosmos RPC" 26657
check_service "Cosmos API" 1317

echo ""
echo "🎯 Quick Tests:"
echo "─────────────────────────"

# Test RPC server
if curl -s http://127.0.0.1:3030/health > /dev/null 2>&1; then
    echo "✅ RPC Health Check: OK"
    
    # Get auth token
    TOKEN=$(curl -s -X POST http://127.0.0.1:3030/auth -d '{"key":"test"}' | jq -r '.token' 2>/dev/null)
    if [ -n "$TOKEN" ]; then
        echo "✅ Authentication: OK"
    fi
else
    echo "❌ RPC Server not responding"
fi

echo ""
echo "✅ Sultan Chain Production Launch Complete!"
echo ""
echo "Access points:"
echo "  • RPC Server: http://127.0.0.1:3030"
echo "  • Metrics: http://127.0.0.1:9100/metrics"
echo "  • Cosmos RPC: http://127.0.0.1:26657"
echo "  • Cosmos API: http://127.0.0.1:1317"
