#!/bin/bash

clear
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║      SULTAN CHAIN - ACTIVATING EXISTING COMPONENTS            ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# First, check what we actually have
echo "📁 Checking existing files..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check for consensus
if [ -f "/workspaces/0xv7/consensus/consensus.js" ]; then
    echo "✅ Consensus system found"
    # Start consensus nodes
    cd /workspaces/0xv7/consensus
    npm install express axios crypto 2>/dev/null || true
    
    # Start 3 nodes
    for i in 1 2 3; do
        PORT=$((4000 + i)) node consensus.js > /tmp/consensus_node_$i.log 2>&1 &
        echo "   • Started consensus node $i on port $((4000 + i))"
    done
else
    echo "⚠️  Consensus not found - will create"
fi

# Check for database
if [ -f "/workspaces/0xv7/database/database_manager.js" ]; then
    echo "✅ Database manager found"
    cd /workspaces/0xv7/database
    npm install cassandra-driver 2>/dev/null || true
    node database_manager.js > /tmp/database.log 2>&1 &
    echo "   • Database manager started"
else
    echo "⚠️  Database manager not found"
fi

# Check for P2P network
if [ -f "/workspaces/0xv7/p2p/p2p_network.js" ]; then
    echo "✅ P2P network found"
    cd /workspaces/0xv7/p2p
    node p2p_network.js > /tmp/p2p.log 2>&1 &
    echo "   • P2P network started"
else
    echo "⚠️  P2P network not found"
fi

# Check main API
echo ""
echo "🔌 Starting Main API..."
if [ -f "/workspaces/0xv7/api/server.js" ]; then
    cd /workspaces/0xv7/api
    pkill -f "node.*3030" 2>/dev/null || true
    node server.js > /tmp/api.log 2>&1 &
    echo "✅ API started on port 3030"
elif [ -f "/workspaces/0xv7/api/sultan_api_v2.js" ]; then
    cd /workspaces/0xv7/api
    pkill -f "sultan_api" 2>/dev/null || true
    node sultan_api_v2.js > /tmp/api.log 2>&1 &
    echo "✅ API v2 started"
else
    echo "⚠️  No API server found"
fi

# Check validator portal
echo ""
echo "🌐 Checking Web Portals..."
if [ -f "/workspaces/0xv7/validators/recruitment_portal.html" ]; then
    echo "✅ Validator recruitment portal exists"
    echo "   URL: file:///workspaces/0xv7/validators/recruitment_portal.html"
fi

if [ -f "/workspaces/0xv7/live_network_dashboard.html" ]; then
    echo "✅ Live network dashboard exists"
    echo "   URL: file:///workspaces/0xv7/live_network_dashboard.html"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 NETWORK STATUS CHECK:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

sleep 2

# Test services
echo ""
echo "Testing services..."

# Test API
if curl -s http://localhost:3030 > /dev/null 2>&1; then
    echo "✅ Main API: RESPONDING"
else
    echo "⚠️  Main API: NOT RESPONDING"
fi

# Test consensus nodes
for port in 4001 4002 4003; do
    if curl -s http://localhost:$port/consensus_state > /dev/null 2>&1; then
        echo "✅ Consensus node on $port: RESPONDING"
    else
        echo "⚠️  Consensus node on $port: NOT RESPONDING"
    fi
done

echo ""
echo "🚀 QUICK ACTIONS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1. Open Validator Portal:"
echo "   $BROWSER file:///workspaces/0xv7/validators/recruitment_portal.html"
echo ""
echo "2. Open Network Dashboard:"
echo "   $BROWSER file:///workspaces/0xv7/live_network_dashboard.html"
echo ""
echo "3. Test Zero-Fee Transaction:"
echo "   curl -X POST http://localhost:3030 -H 'Content-Type: application/json' -d '{\"method\":\"send_transaction\",\"params\":[{\"from\":\"test\",\"to\":\"test2\",\"amount\":1000}]}'"
echo ""
echo "4. Check Logs:"
echo "   tail -f /tmp/api.log"
echo "   tail -f /tmp/consensus_node_1.log"
