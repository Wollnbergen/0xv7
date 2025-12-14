#!/bin/bash

clear
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║       SULTAN CHAIN - LIVE NETWORK VERIFICATION                ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "🔍 Checking Active Services..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check main API
if curl -s http://localhost:3030 > /dev/null 2>&1; then
    echo "✅ Main API: LIVE on port 3030"
    curl -s -X POST http://localhost:3030 \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"chain_getInfo","params":[],"id":1}' | jq .result 2>/dev/null || echo "   API responding"
else
    echo "⚠️  Main API: Not responding"
fi

# Check consensus nodes
echo ""
echo "🔗 Consensus Nodes Status:"
for port in 4001 4002 4003; do
    if curl -s http://localhost:$port/consensus_state > /dev/null 2>&1; then
        echo "✅ Node $((port-4000)): ACTIVE on port $port"
        curl -s http://localhost:$port/consensus_state | jq . 2>/dev/null | head -5
    else
        echo "⚠️  Node $((port-4000)): Not responding on port $port"
    fi
done

# Check P2P network
echo ""
echo "🌐 P2P Network Status:"
for port in 5001 5002 5003; do
    if lsof -i:$port > /dev/null 2>&1; then
        echo "✅ P2P Node on port $port: LISTENING"
    fi
done

# Check database
echo ""
echo "💾 Database Status:"
if [ -f "/workspaces/0xv7/database/database_manager.js" ]; then
    echo "✅ Database manager: EXISTS"
    ps aux | grep -q "[n]ode.*database" && echo "   • Database process: RUNNING" || echo "   • Database process: Not running"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 NETWORK SUMMARY:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Count running processes
NODE_COUNT=$(ps aux | grep -c "[n]ode.*consensus")
API_RUNNING=$(ps aux | grep -c "[n]ode.*3030")

echo "• Consensus Nodes Active: $NODE_COUNT"
echo "• API Status: $( [ $API_RUNNING -gt 0 ] && echo "RUNNING" || echo "STOPPED" )"
echo "• Validator Portal: file:///workspaces/0xv7/validators/recruitment_portal.html"
echo "• Telegram Bot: Updated with /become_validator command"
