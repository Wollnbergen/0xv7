#!/bin/bash

echo "Testing Day 5-6 Token Economics..."
echo ""

# Get auth token
export SULTAN_JWT_SECRET='production_secret_32_bytes_minimum_required'
TOKEN=$(cargo run -q -p sultan-coordinator --bin jwt_gen prod 3600 2>/dev/null)

if [ -z "$TOKEN" ]; then
    echo "❌ Failed to get auth token"
    exit 1
fi

echo "1️⃣ Testing token_transfer..."
curl -sS -X POST http://127.0.0.1:3030 \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"token_transfer","params":["sultan1alice","sultan1bob",100],"id":1}' | jq . || echo "❌ Not implemented"

echo ""
echo "2️⃣ Testing calculate_rewards..."
curl -sS -X POST http://127.0.0.1:3030 \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"calculate_rewards","params":["sultan1alice"],"id":2}' | jq . || echo "❌ Not implemented"

echo ""
echo "3️⃣ Testing claim_rewards..."
curl -sS -X POST http://127.0.0.1:3030 \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"claim_rewards","params":["sultan1alice"],"id":3}' | jq . || echo "❌ Not implemented"

echo ""
echo "✅ Day 5-6 tests complete!"
