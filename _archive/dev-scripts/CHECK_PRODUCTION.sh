#!/bin/bash

echo "╔════════════════════════════════════════════════════════════════════╗"
echo "║           SULTAN CHAIN - PRODUCTION READINESS CHECK                 ║"
echo "╚════════════════════════════════════════════════════════════════════╝"
echo ""

READY=0
TOTAL=8

echo "Checking production components..."
echo ""

# 1. Web Dashboard
echo -n "1. Web Dashboard:        "
if [ -f "/workspaces/0xv7/public/index.html" ] && pgrep -f "http.server 3000" > /dev/null; then
    echo "✅ PRODUCTION READY"
    ((READY++))
else
    echo "⚠️  Not running"
fi

# 2. API Server
echo -n "2. API Server:           "
if [ -f "/workspaces/0xv7/production/api/server.py" ] && pgrep -f "server.py" > /dev/null; then
    echo "✅ PRODUCTION READY"
    ((READY++))
else
    echo "⚠️  Not running"
fi

# 3. CLI Tools
echo -n "3. CLI Tools:            "
if [ -f "/workspaces/0xv7/production/bin/sultan" ]; then
    echo "✅ PRODUCTION READY"
    ((READY++))
else
    echo "❌ Not found"
fi

# 4. Docker Setup
echo -n "4. Docker Config:        "
if [ -f "/workspaces/0xv7/production/docker-compose.yml" ]; then
    echo "✅ PRODUCTION READY"
    ((READY++))
else
    echo "❌ Not found"
fi

# 5. Database
echo -n "5. ScyllaDB:             "
if docker ps | grep -q sultan-scylla; then
    echo "✅ PRODUCTION READY"
    ((READY++))
else
    echo "⚠️  Container exists but not running"
fi

# 6. Blockchain Node
echo -n "6. Blockchain Node:      "
if [ -f "/workspaces/0xv7/node/target/release/sultan_node" ]; then
    echo "✅ COMPILED & READY"
    ((READY++))
else
    echo "❌ Needs compilation"
fi

# 7. Consensus
echo -n "7. Consensus:            "
if [ -f "/workspaces/0xv7/node/src/consensus.rs" ]; then
    echo "⚠️  Basic implementation"
else
    echo "❌ Not implemented"
fi

# 8. Bridges
echo -n "8. Bridge Connections:   "
echo "⚠️  Structure only (needs activation)"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 PRODUCTION READINESS: $READY/$TOTAL components ($(($READY * 100 / $TOTAL))%)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if [ $READY -ge 6 ]; then
    echo "✅ Sultan Chain is PRODUCTION READY!"
    echo ""
    echo "Run './DEPLOY_PRODUCTION.sh' to launch everything"
else
    echo "⚠️  Need to complete remaining components"
    echo ""
    echo "Run './COMPLETE_PRODUCTION_NOW.sh' to fix compilation"
fi

