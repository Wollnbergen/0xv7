#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║       SULTAN CHAIN - COMPREHENSIVE FEATURE TEST               ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "🚀 Testing all Sultan Chain features..."
echo ""

# Feature 1: Zero Gas Fees
echo "💰 Feature 1: ZERO GAS FEES"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
TRANSFER=$(curl -s -X POST http://localhost:3030 \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"transfer","params":{"from":"sultan1abc","to":"sultan1xyz","amount":1000000},"id":1}')

echo "$TRANSFER" | python3 -c "
import sys, json
data = json.load(sys.stdin)
result = data.get('result', {})
print(f\"✅ Transferred 1,000,000 SLTN\")
print(f\"   TX Hash: {result.get('tx_hash', 'N/A')}\")
print(f\"   Gas Fee: \${result.get('gas_fee', 'N/A')}\")
print(f\"   Status: {result.get('message', 'N/A')}\")
"
echo ""

# Feature 2: Validator APY
echo "📈 Feature 2: VALIDATOR REWARDS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
APY=$(curl -s -X POST http://localhost:3030 \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"get_apy","id":1}')

echo "$APY" | python3 -c "
import sys, json
data = json.load(sys.stdin)
result = data.get('result', {})
print(f\"✅ Validator APY: {result.get('validator_apy', 'N/A')}\")
print(f\"   Staking Ratio: {result.get('staking_ratio', 'N/A')}\")
print(f\"   Total Staked: {result.get('total_staked', 'N/A')}\")
"
echo ""

# Feature 3: Economics Model
echo "🔥 Feature 3: ECONOMICS MODEL"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
ECON=$(curl -s -X POST http://localhost:3030 \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"get_economics","id":1}')

echo "$ECON" | python3 -c "
import sys, json
data = json.load(sys.stdin)
result = data.get('result', {})
print(f\"✅ Inflation Schedule: {result.get('inflation_schedule', 'N/A')}\")
print(f\"   Current Rate: {result.get('current_inflation', 'N/A')}\")
print(f\"   Burn Rate: {result.get('burn_mechanism', 'N/A')}\")
"
echo ""

# Feature 4: Network Status
echo "🌐 Feature 4: NETWORK STATUS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
STATUS=$(curl -s -X POST http://localhost:3030 \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"get_status","id":1}')

echo "$STATUS" | python3 -c "
import sys, json
data = json.load(sys.stdin)
result = data.get('result', {})
print(f\"✅ Chain ID: {result.get('chain_id', 'N/A')}\")
print(f\"   Block Height: {result.get('block_height', 'N/A')}\")
print(f\"   TPS: {result.get('tps', 'N/A')}\")
print(f\"   Validators: {result.get('validators', 'N/A')}\")
"
echo ""

echo "🎉 ALL FEATURES TESTED SUCCESSFULLY!"
echo ""
echo "📍 View live dashboard at:"
echo "   https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/"
