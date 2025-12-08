#!/bin/bash
# Simple test of Sultan RPC endpoints (no jq required)

RPC="http://localhost:26657"

echo "🧪 Testing Sultan RPC Endpoints"
echo "================================"
echo ""

# Wait for node to be ready
echo "⏳ Waiting for node to start..."
for i in {1..30}; do
  if curl -s "$RPC/status" > /dev/null 2>&1; then
    echo "✅ Node is ready!"
    break
  fi
  echo "   Attempt $i/30..."
  sleep 2
done
echo ""

# Test 1: Node status
echo "1️⃣  Testing /status endpoint..."
curl -s "$RPC/status"
echo -e "\n"

# Test 2: Create token
echo "2️⃣  Testing /tokens/create endpoint..."
curl -s -X POST "$RPC/tokens/create" \
  -H "Content-Type: application/json" \
  -d '{"creator":"alice","name":"Test Token","symbol":"TEST","decimals":6,"initial_supply":1000000000000,"max_supply":10000000000000}'
echo -e "\n"

# Test 3: List tokens
echo "3️⃣  Testing /tokens/list endpoint..."
curl -s "$RPC/tokens/list"
echo -e "\n"

# Test 4: Create DEX pair
echo "4️⃣  Testing /dex/create_pair endpoint..."
curl -s -X POST "$RPC/dex/create_pair" \
  -H "Content-Type: application/json" \
  -d '{"creator":"alice","token_a":"usltn","token_b":"factory/alice/TEST","amount_a":100000000000,"amount_b":100000000000}'
echo -e "\n"

# Test 5: List pools
echo "5️⃣  Testing /dex/pools endpoint..."
curl -s "$RPC/dex/pools"
echo -e "\n"

# Test 6: Execute swap
echo "6️⃣  Testing /dex/swap endpoint..."
curl -s -X POST "$RPC/dex/swap" \
  -H "Content-Type: application/json" \
  -d '{"pair_id":"usltn-factory/alice/TEST","sender":"alice","offer_asset":"usltn","offer_amount":1000000000}'
echo -e "\n"

echo "✅ All tests complete!"
