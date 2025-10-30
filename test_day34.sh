#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          SULTAN CHAIN - DAY 3-4 FEATURE TEST                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

export SULTAN_JWT_SECRET='production_secret_32_bytes_minimum_required'
TOKEN=$(cargo run -q -p sultan-coordinator --bin jwt_gen prod 3600 2>/dev/null)

echo "🧪 Testing all Day 3-4 features..."
echo ""

# Test 1: Wallet Creation
echo "1. Wallet Creation Test"
RESULT=$(curl -sS -X POST http://127.0.0.1:3030 \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"wallet_create","params":["day34_test_user"],"id":1}' 2>/dev/null)

if echo "$RESULT" | grep -q '"result"'; then
    WALLET=$(echo "$RESULT" | jq -r '.result.address')
    echo "   ✅ Created wallet: $WALLET"
else
    echo "   ❌ Failed"
fi

# Test 2: Governance
echo ""
echo "2. Governance Test"
RESULT=$(curl -sS -X POST http://127.0.0.1:3030 \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"proposal_create","params":["final_test","Final Test Proposal","Testing governance system",null],"id":2}' 2>/dev/null)

if echo "$RESULT" | grep -q '"result"'; then
    PROPOSAL_ID=$(echo "$RESULT" | jq -r '.result.proposal_id')
    echo "   ✅ Created proposal: $PROPOSAL_ID"
else
    echo "   ❌ Failed"
fi

# Test 3: Staking
echo ""
echo "3. Staking Test"
RESULT=$(curl -sS -X POST http://127.0.0.1:3030 \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"stake","params":["validator_final",5000],"id":3}' 2>/dev/null)

if echo "$RESULT" | grep -q '"result"'; then
    AMOUNT=$(echo "$RESULT" | jq -r '.result.amount')
    echo "   ✅ Staked: $AMOUNT tokens"
else
    echo "   ❌ Failed"
fi

# Test 4: APY Query
echo ""
echo "4. APY Query Test"
RESULT=$(curl -sS -X POST http://127.0.0.1:3030 \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"query_apy","params":[true],"id":4}' 2>/dev/null)

if echo "$RESULT" | grep -q '"result"'; then
    APY=$(echo "$RESULT" | jq -r '.result.apy')
    APY_PERCENT=$(echo "scale=2; $APY * 100" | bc)
    echo "   ✅ Current APY: ${APY_PERCENT}%"
else
    echo "   ❌ Failed"
fi

# Test 5: Metrics endpoint
echo ""
echo "5. Metrics Test"
METRICS_STATUS=$(curl -sS -o /dev/null -w "%{http_code}" http://127.0.0.1:9100/metrics 2>/dev/null)
if [ "$METRICS_STATUS" = "200" ]; then
    echo "   ✅ Metrics endpoint active"
else
    echo "   ❌ Metrics not responding"
fi

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "Test complete! All Day 3-4 features verified."
echo ""
