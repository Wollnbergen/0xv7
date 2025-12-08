#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SULTAN CHAIN - COMPLETE SYSTEM VERIFICATION           ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Wait a moment for API to start
sleep 2

echo "🌐 TESTNET API STATUS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if curl -s http://localhost:3030 > /dev/null 2>&1; then
    echo "✅ API is running on port 3030"
    
    # Test the API
    echo ""
    echo "📊 Testing Economics Endpoint:"
    RESPONSE=$(curl -s -X POST http://localhost:3030 \
        -H 'Content-Type: application/json' \
        -d '{"jsonrpc":"2.0","method":"get_economics","id":1}')
    echo "$RESPONSE" | jq '.result' 2>/dev/null || echo "$RESPONSE"
    
    echo ""
    echo "📊 Testing Chain Status:"
    curl -s -X POST http://localhost:3030 \
        -H 'Content-Type: application/json' \
        -d '{"jsonrpc":"2.0","method":"chain_status","id":1}' | jq '.result | {height, validators, apy: .validator_apy, zero_fees}'
else
    echo "❌ API not running. Checking process..."
    ps aux | grep -E "node|npm" | grep -v grep
fi

echo ""
echo "⛓️ MAINNET BINARY STATUS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
MAINNET_BIN="/workspaces/0xv7/sultan_mainnet/target/release/sultan-mainnet"
if [ -f "$MAINNET_BIN" ]; then
    echo "✅ Mainnet binary exists at: $MAINNET_BIN"
    echo "   Size: $(ls -lh $MAINNET_BIN | awk '{print $5}')"
    echo "   Run it with: RUST_LOG=info $MAINNET_BIN"
else
    echo "⚠️ Mainnet binary not found at expected location"
fi

echo ""
echo "🚀 QUICK ACTIONS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1. Open Testnet UI in browser:"
echo "   $BROWSER https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/"
echo ""
echo "2. Run Mainnet Node:"
echo "   RUST_LOG=info /workspaces/0xv7/sultan_mainnet/target/release/sultan-mainnet"
echo ""
echo "3. Test Zero-Fee Transfer:"
echo '   curl -X POST http://localhost:3030 -d '"'"'{"jsonrpc":"2.0","method":"token_transfer","params":["alice","bob",100],"id":1}'"'"' | jq'

