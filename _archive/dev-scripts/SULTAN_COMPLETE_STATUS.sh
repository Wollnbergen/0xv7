#!/bin/bash

clear
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SULTAN CHAIN - COMPLETE SYSTEM STATUS                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "📅 $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# Function to check service
check_service() {
    if $1 2>/dev/null; then
        echo "✅ $2"
    else
        echo "❌ $2"
    fi
}

echo "🌐 NETWORK STATUS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check testnet
echo -n "• Testnet API (Port 3030): "
if curl -s http://localhost:3030 > /dev/null 2>&1; then
    echo "✅ RUNNING"
    echo "  └─ Public: https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/"
else
    echo "❌ Not running"
fi

# Check mainnet binary
echo -n "• Mainnet Binary: "
if [ -f /workspaces/0xv7/target/release/sultan-mainnet ]; then
    echo "✅ BUILT ($(ls -lah /workspaces/0xv7/target/release/sultan-mainnet | awk '{print $5}'))"
elif [ -f /workspaces/0xv7/sultan_minimal ]; then
    echo "✅ MINIMAL VERSION READY"
else
    echo "❌ Not built"
fi

# Check databases
echo -n "• ScyllaDB: "
docker ps | grep -q scylla && echo "✅ Running" || echo "❌ Not running"

echo -n "• Redis: "
docker ps | grep -q redis && echo "✅ Running" || echo "❌ Not running"

echo ""
echo "📊 LIVE METRICS (from Testnet):"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Get live data
RESPONSE=$(curl -s -X POST http://localhost:3030 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"chain_status","id":1}')

if [ $? -eq 0 ]; then
    echo "$RESPONSE" | jq -r '.result | 
    "• Block Height: #\(.height)
• Active Validators: \(.validators) (\(.mobile_validators) mobile)
• TPS Capacity: \(.tps)+
• Zero Fees: \(.zero_fees)
• Validator APY: \(.validator_apy)
• Mobile APY: \(.mobile_validator_apy)"'
fi

echo ""
echo "🚀 QUICK LAUNCH COMMANDS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1. Open Testnet UI (Working):"
echo "   $BROWSER https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/"
echo ""
echo "2. Launch Mainnet:"
echo "   ./LAUNCH_SULTAN_MAINNET_NOW.sh"
echo ""
echo "3. Test Zero-Fee Transfer:"
echo '   curl -X POST http://localhost:3030 -d '"'"'{"jsonrpc":"2.0","method":"token_transfer","params":["alice","bob",100],"id":1}'"'"' | jq'
echo ""
echo "4. Check APY:"
echo '   curl -X POST http://localhost:3030 -d '"'"'{"jsonrpc":"2.0","method":"get_apy","id":1}'"'"' | jq'

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "💎 SULTAN CHAIN: The World's First ZERO Gas Fee Blockchain"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

