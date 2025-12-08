#!/bin/bash

clear
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SULTAN CHAIN - COMPLETE SYSTEM STATUS                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "📅 $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# Check if API is running locally
echo "🔍 Checking System Status..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Test local API
if curl -s http://localhost:3030 > /dev/null 2>&1; then
    echo "✅ Local API: Running"
    LOCAL_API=true
else
    echo "❌ Local API: Not running (Starting...)"
    LOCAL_API=false
    # Start the API
    cd /workspaces/0xv7/api && node simple_server.js > /tmp/api.log 2>&1 &
    sleep 2
fi

# Test public endpoint
if curl -s https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/ > /dev/null 2>&1; then
    echo "✅ Public URL: Accessible"
else
    echo "⚠️  Public URL: May need port forwarding"
fi

echo ""
echo "�� SULTAN CHAIN FEATURES:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  💰 Gas Fees: $0.00 (Forever Free)"
echo "  📈 Validator APY: 13.33%"
echo "  📱 Mobile Bonus: +40% (Total 18.66%)"
echo "  ⏱️  Block Time: 5 seconds"
echo "  💵 Inflation: 4% annually"
echo ""

echo "🧪 TESTING OPTIONS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1) Test Local API"
echo "2) Open Browser UI"
echo "3) Run Full Test Suite"
echo "4) Check Mainnet Readiness"
echo "5) View Documentation"
echo "6) Exit"
echo ""

read -p "Select option (1-6): " choice

case $choice in
    1)
        echo ""
        echo "Testing Local API..."
        curl -s -X POST http://localhost:3030 \
            -H 'Content-Type: application/json' \
            -d '{"jsonrpc":"2.0","method":"chain_status","id":1}' | jq
        ;;
    2)
        "$BROWSER" https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/ &
        echo "Opening browser..."
        ;;
    3)
        /workspaces/0xv7/TEST_SULTAN_CHAIN.sh
        ;;
    4)
        /workspaces/0xv7/MAINNET_READINESS.sh
        ;;
    5)
        cat /workspaces/0xv7/SULTAN_CHAIN_CERTIFICATE.md
        ;;
    6)
        exit 0
        ;;
esac

