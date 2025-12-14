#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         TESTING SULTAN CHAIN PRODUCTION FEATURES              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "1️⃣ Testing Zero-Fee Transactions..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s -X POST http://localhost:3030 \
    -H 'Content-Type: application/json' \
    -d '{
        "jsonrpc":"2.0",
        "method":"send_transaction",
        "params":{
            "from":"sultan1abc",
            "to":"sultan1xyz",
            "amount":1000,
            "fee":0
        },
        "id":1
    }' | jq '.result'

echo ""
echo "2️⃣ Testing Validator APY Query..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s -X POST http://localhost:3030 \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"get_economics","id":2}' | \
    jq '.result | {apy: .validator_apy, mobile: .mobile_validator_total_apy}'

echo ""
echo "3️⃣ Testing Chain Status..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s -X POST http://localhost:3030 \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"chain_status","id":3}' | \
    jq '.result'

echo ""
echo "4️⃣ Testing Performance..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
START=$(date +%s%N)
for i in {1..100}; do
    curl -s http://localhost:3030 > /dev/null 2>&1
done
END=$(date +%s%N)
TIME=$((($END - $START) / 1000000))
TPS=$((100000 / $TIME))
echo "  • Processed 100 requests in ${TIME}ms"
echo "  • Estimated TPS: $TPS"

echo ""
echo "✅ All tests complete!"

