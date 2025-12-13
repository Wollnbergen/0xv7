#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           SULTAN CHAIN - COMPREHENSIVE TEST SUITE             ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

TESTS_PASSED=0
TESTS_FAILED=0

# Test function
run_test() {
    local test_name=$1
    local command=$2
    
    echo -n "Testing $test_name... "
    if eval "$command" > /dev/null 2>&1; then
        echo "✅ PASSED"
        ((TESTS_PASSED++))
    else
        echo "❌ FAILED"
        ((TESTS_FAILED++))
    fi
}

echo "🧪 RUNNING TESTS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 1. Test Consensus
run_test "Consensus API" "curl -s http://localhost:4001/consensus_state"

# 2. Test RPC
run_test "RPC Server" "curl -s http://localhost:3030"

# 3. Test API
run_test "Main API" "curl -s http://localhost:3000"

# 4. Test block production
if curl -s http://localhost:4001/consensus_state > /dev/null 2>&1; then
    BLOCK1=$(curl -s http://localhost:4001/consensus_state | jq -r '.current_block')
    sleep 2
    BLOCK2=$(curl -s http://localhost:4001/consensus_state | jq -r '.current_block')
    
    if [ "$BLOCK2" -gt "$BLOCK1" ] 2>/dev/null; then
        echo "Testing Block Production... ✅ PASSED"
        ((TESTS_PASSED++))
    else
        echo "Testing Block Production... ❌ FAILED"
        ((TESTS_FAILED++))
    fi
fi

echo ""
echo "📊 TEST RESULTS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  ✅ Passed: $TESTS_PASSED"
echo "  ❌ Failed: $TESTS_FAILED"
echo ""

if [ $TESTS_FAILED -eq 0 ] && [ $TESTS_PASSED -gt 0 ]; then
    echo "🎉 ALL TESTS PASSED! Sultan Chain is operational!"
elif [ $TESTS_PASSED -gt 0 ]; then
    echo "⚠️ Some tests passed. Partial functionality available."
else
    echo "❌ All tests failed. Run: /workspaces/0xv7/START_SULTAN_BLOCKCHAIN.sh"
fi

