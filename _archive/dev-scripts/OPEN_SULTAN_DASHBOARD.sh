#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           SULTAN CHAIN - COMPLETE DASHBOARD                   ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Open testnet UI
echo "🌐 Opening Sultan Chain Testnet UI..."
"$BROWSER" https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/ &

# Show live API data
echo ""
echo "📊 LIVE CHAIN ECONOMICS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s -X POST http://localhost:3030 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"get_economics","id":1}' | jq -r '.result | 
  "💰 Validator APY: \(.validator_apy)\n📱 Mobile APY: \(.mobile_validator_total_apy)\n⛽ Gas Fees: $\(.user_gas_fees)\n📈 Inflation: \(.inflation_rate)"'

echo ""
echo "📊 CHAIN STATUS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s -X POST http://localhost:3030 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"chain_status","id":1}' | jq -r '.result |
  "⛓️  Height: #\(.height)\n👥 Validators: \(.validators) (\(.mobile_validators) mobile)\n⚡ TPS Capacity: \(.tps)\n🚀 IBC: \(.ibc_enabled)"'

echo ""
echo "📊 STAKING REWARDS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s -X POST http://localhost:3030 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"calculate_rewards","params":{"amount":10000,"is_mobile":false},"id":1}' | jq -r '.result |
  "💰 10,000 SLTN staked:\n   • Annual rewards: \(.annual_rewards) SLTN\n   • Daily rewards: \(.daily_rewards) SLTN"'

echo ""
echo "📱 MOBILE STAKING REWARDS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s -X POST http://localhost:3030 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"calculate_rewards","params":{"amount":10000,"is_mobile":true},"id":1}' | jq -r '.result |
  "📱 10,000 SLTN mobile staked:\n   • Annual rewards: \(.annual_rewards) SLTN (+40% bonus)\n   • Daily rewards: \(.daily_rewards) SLTN"'

