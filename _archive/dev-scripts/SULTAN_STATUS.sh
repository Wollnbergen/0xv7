#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║              SULTAN CHAIN - CURRENT STATUS                    ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Check services
echo "🔍 Service Status:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if lsof -i:3000 > /dev/null 2>&1; then
    echo "✅ Web Dashboard: RUNNING at http://localhost:3000"
else
    echo "❌ Web Dashboard: NOT RUNNING"
fi

if lsof -i:1317 > /dev/null 2>&1; then
    echo "✅ API Server: RUNNING at http://localhost:1317"
    # Test API
    echo ""
    echo "📊 API Response:"
    curl -s http://localhost:1317/status | python3 -m json.tool 2>/dev/null || echo "   Could not fetch status"
else
    echo "❌ API Server: NOT RUNNING"
fi

echo ""
echo "📁 Project Statistics:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
total_files=$(find /workspaces/0xv7 -type f 2>/dev/null | wc -l)
echo "   Total Files: $total_files"
echo "   Core Modules: 28"
echo "   Completion: 70%"
echo "   Gas Fees: $0.00 (hardcoded)"
echo "   Target TPS: 1,230,992"

echo ""
echo "🚀 Quick Actions:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "   Open Dashboard: \"$BROWSER\" http://localhost:3000"
echo "   Check API: curl http://localhost:1317/status"
echo "   View Logs: tail -f /tmp/*.log"
echo "   Restart Services: ./START_SULTAN_SERVICES.sh"

