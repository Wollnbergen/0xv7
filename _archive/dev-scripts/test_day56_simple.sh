#!/bin/bash

echo "Testing Day 5-6 features..."

export SULTAN_JWT_SECRET='production_secret_32_bytes_minimum_required'
TOKEN=$(cargo run -q -p sultan-coordinator --bin jwt_gen prod 3600 2>/dev/null)

# Test token transfer (will fail if not implemented)
echo "Testing token transfer..."
curl -sS -X POST http://127.0.0.1:3030 \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"token_transfer","params":["sultan1alice","sultan1bob",100,"test transfer"],"id":1}' | jq .

# Test calculate rewards
echo "Testing calculate rewards..."
curl -sS -X POST http://127.0.0.1:3030 \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"calculate_rewards","params":["sultan1alice"],"id":2}' | jq .

# Test claim rewards
echo "Testing claim rewards..."
curl -sS -X POST http://127.0.0.1:3030 \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"claim_rewards","params":["sultan1alice"],"id":3}' | jq .
