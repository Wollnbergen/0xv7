#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║              SULTAN CHAIN DASHBOARD                           ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Check service status
echo "📊 Service Health Check:"
echo "─────────────────────────"

# Check demo API
if curl -s http://127.0.0.1:3030/health > /dev/null 2>&1; then
    echo "✅ Demo API: Running on port 3030"
else
    echo "❌ Demo API: Not running"
fi

# Check web UI
if curl -s http://127.0.0.1:8080 > /dev/null 2>&1; then
    echo "✅ Web UI: Running on port 8080"
else
    echo "❌ Web UI: Not running"
fi

# Check databases
nc -zv 127.0.0.1 9042 2>/dev/null && echo "✅ ScyllaDB: Running" || echo "❌ ScyllaDB: Not running"
nc -zv 127.0.0.1 6379 2>/dev/null && echo "✅ Redis: Running" || echo "❌ Redis: Not running"

echo ""
echo "🔗 Access Points:"
echo "─────────────────────────"
echo "  • API Endpoint: http://127.0.0.1:3030"
echo "  • Web Dashboard: http://127.0.0.1:8080/sultan_web_ui.html"
echo ""

# Show recent transactions
echo "📈 Recent Activity:"
echo "─────────────────────────"
curl -s -X POST http://127.0.0.1:3030 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"chain_status","id":1}' 2>/dev/null | \
  jq -r '.result | "  • Block Height: \(.height)\n  • Validators: \(.validators) (\(.mobile_validators) mobile)\n  • TPS Capacity: \(.tps)\n  • Zero Fees: \(.zero_fees)"'

echo ""
echo "💰 Economics:"
echo "─────────────────────────"
curl -s -X POST http://127.0.0.1:3030 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"get_apy","id":1}' 2>/dev/null | \
  jq -r '.result | "  • Base APY: \(.base_apy)\n  • Mobile Bonus: \(.mobile_validator_bonus)\n  • Max APY: \(.total_possible)"'
