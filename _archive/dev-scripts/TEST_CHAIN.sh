#!/bin/bash

clear
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          SULTAN CHAIN - ENDPOINT TESTING                      ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "🔍 Testing all endpoints..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Test Consensus
echo "1. CONSENSUS NODE (Port 4001):"
RESULT=$(curl -s http://localhost:4001/consensus_state 2>/dev/null)
if [ ! -z "$RESULT" ]; then
    echo "$RESULT" | jq '{blockHeight, status, validators}'
    echo "   ✅ Consensus is working"
else
    echo "   ❌ Not responding"
fi
echo ""

# Test API
echo "2. API SERVER (Port 3000):"
RESULT=$(curl -s http://localhost:3000 2>/dev/null)
if [ ! -z "$RESULT" ]; then
    echo "$RESULT" | jq '{chain, version, gasFees, apy}'
    echo "   ✅ API is working"
else
    echo "   ❌ Not responding"
fi
echo ""

# Test RPC
echo "3. RPC SERVER (Port 3030):"
RESULT=$(curl -s http://localhost:3030 2>/dev/null)
if [ ! -z "$RESULT" ]; then
    echo "$RESULT" | jq '.'
    echo "   ✅ RPC is working"
else
    echo "   ❌ Not responding"
fi
echo ""

# Test P2P
echo "4. P2P NETWORK (Port 5001):"
RESULT=$(curl -s http://localhost:5001/status 2>/dev/null)
if [ ! -z "$RESULT" ]; then
    echo "$RESULT" | jq '.'
    echo "   ✅ P2P is working"
else
    echo "   ❌ Not responding"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Count services
SERVICES_UP=0
for port in 3000 3030 4001 5001 5002 5003; do
    nc -z localhost $port 2>/dev/null && ((SERVICES_UP++))
done

echo "📊 SUMMARY: $SERVICES_UP/6 services responding"
echo ""

if [ $SERVICES_UP -eq 6 ]; then
    echo "✅ SULTAN CHAIN IS FULLY OPERATIONAL!"
    echo ""
    echo "🌐 View the dashboard:"
    echo "   $BROWSER /workspaces/0xv7/dashboard.html"
elif [ $SERVICES_UP -ge 4 ]; then
    echo "⚠️ Sultan Chain is partially operational"
else
    echo "❌ Services need to be started"
    echo "   Run: /workspaces/0xv7/PYTHON_SERVICES.sh"
fi

