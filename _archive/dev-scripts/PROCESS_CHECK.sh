#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           SULTAN CHAIN - PROCESS & PORT CHECK                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "📋 Node.js Processes:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
ps aux | grep -E "node.*\.(js|mjs)" | grep -v grep | head -10 || echo "No Node.js processes found"

echo ""
echo "🔌 Listening Ports (Sultan Chain):"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

for port in 3000 3030 4001 4002 4003 5001 5002 5003; do
    printf "Port %s: " "$port"
    if lsof -i:$port > /dev/null 2>&1; then
        echo "✅ LISTENING"
    else
        echo "❌ NOT LISTENING"
    fi
done

echo ""
echo "💡 Quick Actions:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "• Start all services: /workspaces/0xv7/ROBUST_FIX.sh"
echo "• Check status: /workspaces/0xv7/LIVE_STATUS.sh"
echo "• View logs: tail -f /tmp/*.log"
echo "• Kill all: pkill -f node"

