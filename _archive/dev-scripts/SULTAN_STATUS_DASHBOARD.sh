#!/bin/bash

clear
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SULTAN CHAIN STATUS DASHBOARD                         ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "📅 $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

cd /workspaces/0xv7

echo "📊 COMPONENT STATUS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check services
echo -n "• ScyllaDB Database: "
docker ps | grep -q scylla && echo "✅ Running" || echo "❌ Stopped"

echo -n "• Testnet API (3030): "
if curl -s http://localhost:3030 > /dev/null 2>&1; then
    echo "✅ Running"
    echo "  └─ Public URL: https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/"
else
    echo "❌ Stopped"
fi

echo -n "• Node Binary: "
[ -f node/target/release/sultan_node ] && echo "✅ Built" || echo "❌ Not built"

echo -n "• RPC Server Binary: "
[ -f node/target/release/rpc_server ] && echo "✅ Built" || echo "❌ Not built"

echo ""
echo "💰 ECONOMICS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "• Inflation Rate: 4% annually"
echo "• Validator APY: 13.33%"
echo "• Mobile Validator APY: 18.66% (with 40% bonus)"
echo "• Gas Fees: $0.00 (subsidized)"

echo ""
echo "📈 NETWORK METRICS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "• Target TPS: 10,000+"
echo "• Block Time: 5 seconds"
echo "• Max Validators: 100"
echo "• Min Stake: 5,000 SLTN"

echo ""
echo "🎯 QUICK COMMANDS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "• Start Node: ./START_SULTAN_MAINNET.sh"
echo "• Test API: curl -X POST http://localhost:3030 -d '{\"jsonrpc\":\"2.0\",\"method\":\"get_apy\",\"id\":1}'"
echo "• View Logs: docker logs scylla"
echo "• Open Dashboard: $BROWSER https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/"

echo ""
echo "📊 MAINNET READINESS: "
READY=0
TOTAL=6
docker ps | grep -q scylla && ((READY++))
curl -s http://localhost:3030 > /dev/null 2>&1 && ((READY++))
[ -f node/target/release/sultan_node ] && ((READY++))
[ -f node/target/release/rpc_server ] && ((READY++))
[ -f node/migrations/init.cql ] && ((READY++))
[ -f docker-compose.yml ] && ((READY++))

PERCENTAGE=$((READY * 100 / TOTAL))
echo -n "["
for i in $(seq 1 10); do
    if [ $((i * 10)) -le $PERCENTAGE ]; then
        echo -n "█"
    else
        echo -n "░"
    fi
done
echo "] $PERCENTAGE% ($READY/$TOTAL components ready)"

