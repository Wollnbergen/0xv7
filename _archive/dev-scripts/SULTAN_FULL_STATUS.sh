#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          SULTAN CHAIN - FULL SYSTEM STATUS                    ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "📅 $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# Check for all binaries
echo "🔍 SEARCHING FOR BINARIES:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
find /workspaces/0xv7 -name "sultan*" -type f -executable 2>/dev/null | while read -r binary; do
    echo "✅ Found: $binary ($(stat -c%s "$binary" | numfmt --to=iec-i --suffix=B))"
done

echo ""
echo "🌐 TESTNET API:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if curl -s http://localhost:3030 > /dev/null 2>&1; then
    echo "✅ Running on http://localhost:3030"
    echo "✅ Public: https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/"
    
    # Get current stats
    RESPONSE=$(curl -s -X POST http://localhost:3030 \
      -H 'Content-Type: application/json' \
      -d '{"jsonrpc":"2.0","method":"chain_status","id":1}')
    echo "$RESPONSE" | jq -r '.result | "   • Height: \(.height)\n   • Validators: \(.validators) (\(.mobile_validators) mobile)\n   • Zero Fees: \(.zero_fees)\n   • APY: \(.validator_apy) (mobile: \(.mobile_validator_apy))"' 2>/dev/null || echo "   $RESPONSE"
else
    echo "❌ Not running"
fi

echo ""
echo "🐳 DOCKER STATUS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" | head -5

echo ""
echo "🎯 QUICK COMMANDS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "• Open Testnet UI:     \"$BROWSER\" https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/"
echo "• Find & Run Mainnet:  ./FIND_AND_RUN_SULTAN.sh"
echo "• Create Minimal Node: ./CREATE_MINIMAL_SULTAN.sh"
echo "• Test API:           curl -X POST http://localhost:3030 -d '{\"jsonrpc\":\"2.0\",\"method\":\"get_apy\",\"id\":1}' | jq"

