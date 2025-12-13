#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           SULTAN CHAIN - COMPLETE TEST SUITE                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

TESTS_PASSED=0
TESTS_FAILED=0

# Test 1: API Health
echo "🧪 Test 1: API Health Check"
if curl -s http://localhost:3030 > /dev/null 2>&1; then
    echo "✅ PASSED: API is healthy"
    ((TESTS_PASSED++))
else
    echo "❌ FAILED: API not responding"
    ((TESTS_FAILED++))
fi
echo ""

# Test 2: Zero Gas Fees
echo "🧪 Test 2: Zero Gas Fee Transfer"
TRANSFER=$(curl -s -X POST http://localhost:3030 \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"transfer","params":{"from":"sultan1abc","to":"sultan1xyz","amount":1000000},"id":1}')

GAS_FEE=$(echo "$TRANSFER" | python3 -c "import sys, json; print(json.load(sys.stdin).get('result', {}).get('gas_fee', -1))")
if [ "$GAS_FEE" == "0" ]; then
    echo "✅ PASSED: Gas fee is $0.00"
    ((TESTS_PASSED++))
else
    echo "❌ FAILED: Gas fee is not zero"
    ((TESTS_FAILED++))
fi
echo ""

# Test 3: Validator APY
echo "🧪 Test 3: Validator APY Check"
APY=$(curl -s -X POST http://localhost:3030 \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"get_apy","id":1}')

if echo "$APY" | grep -q "13.33"; then
    echo "✅ PASSED: Validator APY is 13.33%"
    ((TESTS_PASSED++))
else
    echo "❌ FAILED: Validator APY incorrect"
    ((TESTS_FAILED++))
fi
echo ""

# Test 4: Economics Model
echo "🧪 Test 4: Economics Model"
ECON=$(curl -s -X POST http://localhost:3030 \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"get_economics","id":1}')

if echo "$ECON" | grep -q "8%"; then
    echo "✅ PASSED: Inflation schedule correct"
    ((TESTS_PASSED++))
else
    echo "❌ FAILED: Inflation schedule incorrect"
    ((TESTS_FAILED++))
fi
echo ""

# Test 5: Block Production
echo "🧪 Test 5: Block Production"
HEIGHT1=$(curl -s -X POST http://localhost:3030 \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"get_status","id":1}' | python3 -c "import sys, json; print(json.load(sys.stdin).get('result', {}).get('block_height', 0))")

sleep 2

HEIGHT2=$(curl -s -X POST http://localhost:3030 \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"get_status","id":1}' | python3 -c "import sys, json; print(json.load(sys.stdin).get('result', {}).get('block_height', 0))")

if [ "$HEIGHT2" -gt "$HEIGHT1" ]; then
    echo "✅ PASSED: Blocks are being produced"
    ((TESTS_PASSED++))
else
    echo "❌ FAILED: Block production issue"
    ((TESTS_FAILED++))
fi
echo ""

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 TEST RESULTS:"
echo "   ✅ Passed: $TESTS_PASSED"
echo "   ❌ Failed: $TESTS_FAILED"

if [ "$TESTS_FAILED" -eq 0 ]; then
    echo ""
    echo "🎉 ALL TESTS PASSED! Sultan Chain is fully operational!"
else
    echo ""
    echo "⚠️  Some tests failed. Please review the output above."
fi
