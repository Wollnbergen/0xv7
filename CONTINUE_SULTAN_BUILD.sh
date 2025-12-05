#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║        SULTAN CHAIN - CONTINUING TO PRODUCTION                ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "📅 $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# Quick status check
echo "🔍 Checking current setup..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check if API is running
if curl -s http://localhost:3030 > /dev/null 2>&1; then
    echo "✅ Testnet API: RUNNING"
else
    echo "⚠️  Testnet API: NOT RUNNING (starting it...)"
    cd /workspaces/0xv7/api && node simple_server.js > /tmp/api.log 2>&1 &
    sleep 2
fi

# Check if mainnet binary exists
if [ -f /workspaces/0xv7/sultan_mainnet/target/release/sultan-mainnet ]; then
    echo "✅ Mainnet Binary: EXISTS"
else
    echo "⚠️  Mainnet Binary: NEEDS BUILDING"
fi

# Check Docker services
SCYLLA_RUNNING=$(docker ps | grep -c scylla || echo 0)
REDIS_RUNNING=$(docker ps | grep -c redis || echo 0)

if [ "$SCYLLA_RUNNING" -eq 0 ]; then
    echo "⚠️  ScyllaDB: NOT RUNNING (starting...)"
    docker start scylla 2>/dev/null || docker run --name scylla -d -p 9042:9042 scylladb/scylla
else
    echo "✅ ScyllaDB: RUNNING"
fi

if [ "$REDIS_RUNNING" -eq 0 ]; then
    echo "⚠️  Redis: NOT RUNNING (starting...)"
    docker start redis 2>/dev/null || docker run --name redis -d -p 6379:6379 redis:alpine
else
    echo "✅ Redis: RUNNING"
fi

echo ""
echo "📊 NEXT DEVELOPMENT PHASE:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "We need to:"
echo "1. ✨ Add database persistence to mainnet"
echo "2. 🔗 Implement real P2P networking"
echo "3. 🏗️ Create genesis block"
echo "4. 🔐 Add wallet functionality"
echo ""
echo "Starting Phase 1: Database Persistence..."

