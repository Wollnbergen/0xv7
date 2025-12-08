#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SULTAN CHAIN - PRODUCTION ENVIRONMENT                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "🚀 Launching complete Sultan Chain ecosystem..."
echo ""

# 1. Start Docker services
echo "1️⃣ Starting infrastructure..."
docker start scylla redis 2>/dev/null || {
    docker run --name scylla -d -p 9042:9042 scylladb/scylla
    docker run --name redis -d -p 6379:6379 redis:alpine
}

# 2. Keep testnet API running
echo "2️⃣ Ensuring testnet API is running..."
if ! curl -s http://localhost:3030 > /dev/null 2>&1; then
    cd /workspaces/0xv7/api && node simple_server.js > /tmp/api.log 2>&1 &
    echo "   Started testnet API"
else
    echo "   Testnet API already running ✅"
fi

# 3. Display running services
echo ""
echo "3️⃣ Active Services:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "• Testnet API: http://localhost:3030"
echo "• Public URL: https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/"
echo "• Mainnet Binary: /workspaces/0xv7/sultan_mainnet/target/release/sultan-mainnet"
echo ""

# 4. Quick test
echo "4️⃣ Testing Sultan Chain..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s -X POST http://localhost:3030 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"get_economics","id":1}' | jq '.result | {fees: .user_gas_fees, apy: .validator_apy, mobile: .mobile_validator_total_apy}'

echo ""
echo "5️⃣ To run mainnet node:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "cd /workspaces/0xv7/sultan_mainnet && ./target/release/sultan-mainnet"
echo ""
echo "💎 Sultan Chain is ready for production!"

