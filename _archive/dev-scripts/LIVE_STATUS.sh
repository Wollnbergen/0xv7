#!/bin/bash

clear
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║            SULTAN CHAIN - LIVE STATUS CHECK                   ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "📊 Checking all services..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Test each endpoint
echo "1. CONSENSUS NODE 1 (Port 4001):"
curl -s http://localhost:4001/consensus_state | jq '.' 2>/dev/null || echo "   ❌ Not responding"
echo ""

echo "2. API SERVER (Port 3000):"
curl -s http://localhost:3000 | jq '.' 2>/dev/null || echo "   ❌ Not responding"
echo ""

echo "3. RPC SERVER (Port 3030):"
curl -s http://localhost:3030 | jq '.' 2>/dev/null || echo "   ❌ Not responding"
echo ""

echo "4. P2P NODE 1 (Port 5001):"
curl -s http://localhost:5001/status | jq '.' 2>/dev/null || echo "   ❌ Not responding"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Count running services
RUNNING=0
for port in 4001 4002 4003 3000 3030 5001 5002 5003; do
    if lsof -i:$port > /dev/null 2>&1; then
        ((RUNNING++))
    fi
done

echo "📈 Services Running: $RUNNING/8"
echo ""

if [ $RUNNING -ge 6 ]; then
    echo "✅ Sultan Chain is OPERATIONAL"
    echo ""
    echo "🌐 Open Dashboard:"
    echo "   $BROWSER /workspaces/0xv7/dashboard.html"
else
    echo "⚠️ Some services are down. Run:"
    echo "   /workspaces/0xv7/ROBUST_FIX.sh"
fi

