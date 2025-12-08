#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         COMPREHENSIVE DAY 5-6 FEATURE TEST                    ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Generate JWT token
export SULTAN_JWT_SECRET='production_secret_32_bytes_minimum_required'
TOKEN=$(cargo run -q -p sultan-coordinator --bin jwt_gen test 3600 2>/dev/null)

# Check server status
echo "🔍 Checking server status..."
curl -sS -X POST http://127.0.0.1:3030 \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"system_health","id":0}' | jq .

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Test existing Day 3-4 features still work
echo "📊 Day 3-4 Features Check:"
echo ""

echo "1. Creating wallet..."
curl -sS -X POST http://127.0.0.1:3030 \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"wallet_create","params":["test_day56"],"id":1}' | jq .

echo ""
echo "2. Minting tokens..."
curl -sS -X POST http://127.0.0.1:3030 \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"token_mint","params":["sultan1test_day56",50000],"id":2}' | jq .

echo ""
echo "3. Checking balance..."
curl -sS -X POST http://127.0.0.1:3030 \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"wallet_balance","params":["sultan1test_day56"],"id":3}' | jq .

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Test new Day 5-6 features
echo "🚀 Day 5-6 New Features Test:"
echo ""

echo "4. Testing token transfer (if implemented)..."
curl -sS -X POST http://127.0.0.1:3030 \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"token_transfer","params":["sultan1test_day56","sultan1recipient",1000,"Test transfer"],"id":4}' | jq .

echo ""
echo "5. Testing staking..."
curl -sS -X POST http://127.0.0.1:3030 \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"stake","params":["validator_test",5000],"id":5}' | jq .

echo ""
echo "6. Calculating rewards (if implemented)..."
curl -sS -X POST http://127.0.0.1:3030 \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"calculate_rewards","params":["validator_test"],"id":6}' | jq .

echo ""
echo "7. Claiming rewards (if implemented)..."
curl -sS -X POST http://127.0.0.1:3030 \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"claim_rewards","params":["validator_test"],"id":7}' | jq .

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Test suite complete!"
echo ""
echo "Summary:"
echo "  Day 3-4 features: Should all work ✅"
echo "  Day 5-6 features: Will show errors if not yet integrated ⚠️"
echo ""
echo "Next steps:"
echo "  1. Add missing RPC methods to rpc_server.rs"
echo "  2. Rebuild: cargo build -p sultan-coordinator"
echo "  3. Restart: ./server_control.sh restart"
echo "  4. Re-run this test"
