#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║            SULTAN CHAIN - SERVICE CHECK                       ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Check running processes
echo "📋 Running Node processes:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
ps aux | grep "node /tmp" | grep -v grep | wc -l | read COUNT
echo "Found $COUNT Node.js services"
echo ""

# Check each port
echo "🔌 Port Status:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
for port in 3000 3030 4001 4002 4003 5001 5002 5003; do
    printf "Port %s: " "$port"
    if nc -z localhost $port 2>/dev/null; then
        echo "✅ OPEN"
    else
        echo "❌ CLOSED"
    fi
done

echo ""
echo "📡 Quick Tests:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Test Consensus
echo "Testing Consensus (4001):"
curl -s http://localhost:4001/consensus_state | jq -c '{blockHeight, status}' 2>/dev/null || echo "Not responding"
echo ""

# Test API
echo "Testing API (3000):"
curl -s http://localhost:3000 | jq -c '.chain' 2>/dev/null || echo "Not responding"
echo ""

# Test RPC
echo "Testing RPC (3030):"
curl -s http://localhost:3030 | jq -c '.service' 2>/dev/null || echo "Not responding"

