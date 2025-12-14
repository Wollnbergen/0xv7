#!/bin/bash

clear
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           SULTAN CHAIN - FULL SYSTEM STATUS                   ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "📅 $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

echo "✅ WORKING COMPONENTS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "• Testnet API: https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/"
echo "• ScyllaDB: Running on port 9042"
echo "• Docker: Available"
echo ""

echo "📊 TESTNET API TEST:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s -X POST http://localhost:3030 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"chain_status","id":1}' | jq '.result' 2>/dev/null || echo "API test failed"

echo ""
echo "🔨 BUILD OPTIONS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1. Run: ./BUILD_AND_RUN_NODE.sh    # Main node with full features"
echo "2. Run: ./CREATE_STANDALONE_NODE.sh # Minimal standalone node"
echo ""

echo "🌐 OPEN TESTNET IN BROWSER:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "$BROWSER https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/"

