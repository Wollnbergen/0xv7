#!/bin/bash

echo "🧪 Sultan Chain API Quick Test"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

API_URL="https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/"

# Test 1: Get Economics
echo ""
echo "📊 Economics Check:"
curl -s -X POST "$API_URL" \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"get_economics","id":1}' | \
  jq '.result | {validator_apy, mobile_validator_total_apy, user_gas_fees}'

# Test 2: Chain Status
echo ""
echo "⛓️ Chain Status:"
curl -s -X POST "$API_URL" \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"chain_status","id":2}' | \
  jq '.result | {name, validator_apy, mobile_validator_apy}'

# Test 3: Zero Fee Transfer
echo ""
echo "💸 Zero-Fee Transfer Test:"
curl -s -X POST "$API_URL" \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"token_transfer","params":["alice","bob",1000],"id":3}' | \
  jq '.result | {amount, fee, message}'

echo ""
echo "✅ All tests complete!"
