#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          SULTAN CHAIN - COMPLETE STATUS CHECK                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "🌐 TESTNET STATUS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if curl -s http://localhost:3030 > /dev/null 2>&1; then
    echo "✅ API Running on port 3030"
    echo "✅ Public URL: https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/"
    
    # Test the API
    echo ""
    echo "📊 API Test:"
    curl -s -X POST http://localhost:3030 \
      -H 'Content-Type: application/json' \
      -d '{"jsonrpc":"2.0","method":"chain_status","id":1}' | \
      jq -r '.result | "   Height: \(.height)\n   Validators: \(.validators)\n   APY: \(.validator_apy)"'
else
    echo "❌ Testnet API not running"
fi

echo ""
echo "⛓️ MAINNET STATUS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check different mainnet locations
if [ -f /workspaces/0xv7/target/release/sultan-mainnet ]; then
    echo "✅ Mainnet binary found: /workspaces/0xv7/target/release/sultan-mainnet"
elif [ -f /workspaces/0xv7/sultan_mainnet/target/release/sultan-mainnet ]; then
    echo "✅ Mainnet binary found: sultan_mainnet/target/release/sultan-mainnet"
elif [ -f /workspaces/0xv7/sultan_simple/target/release/sultan-simple ]; then
    echo "✅ Simple mainnet found: sultan_simple/target/release/sultan-simple"
else
    echo "❌ No mainnet binary found"
fi

echo ""
echo "🎯 QUICK ACTIONS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1. Open Testnet UI:    \"$BROWSER\" https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/"
echo "2. Fix Mainnet Build:  ./FIX_MAINNET_BUILD.sh"
echo "3. Create Simple Node: ./CREATE_SIMPLE_MAINNET.sh"
echo "4. Test API:          curl -X POST http://localhost:3030 -d '{\"jsonrpc\":\"2.0\",\"method\":\"get_apy\",\"id\":1}' | jq"

