#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║              SULTAN CHAIN - QUICK STATUS                      ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Check if mainnet is built
if [ -f /workspaces/0xv7/sultan_mainnet/target/release/sultan-mainnet ]; then
    echo "✅ Mainnet binary: READY"
    echo "   Run: cd /workspaces/0xv7/sultan_mainnet && ./target/release/sultan-mainnet"
else
    echo "❌ Mainnet binary: NOT BUILT"
    echo "   Build: ./FIX_AND_BUILD_SULTAN.sh"
fi

# Check if testnet API is running
if curl -s http://localhost:3030 > /dev/null 2>&1; then
    echo "✅ Testnet API: RUNNING"
    echo "   URL: https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/"
else
    echo "❌ Testnet API: NOT RUNNING"
fi

# Check ScyllaDB
if docker ps | grep -q scylla; then
    echo "✅ ScyllaDB: RUNNING"
else
    echo "❌ ScyllaDB: NOT RUNNING"
fi

echo ""
echo "💡 Quick Commands:"
echo "   • Open UI: \"$BROWSER\" https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/"
echo "   • Test API: curl -X POST http://localhost:3030 -d '{\"jsonrpc\":\"2.0\",\"method\":\"get_apy\",\"id\":1}' | jq"

