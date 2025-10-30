#!/bin/bash
# Day 1-2 Comprehensive Test Suite

echo "=== Sultan Chain Day 1-2 Test Suite ==="

# Generate token
export SULTAN_JWT_SECRET='production_secret_32_bytes_minimum_required'
TOKEN=$(cargo run -q -p sultan-coordinator --bin jwt_gen prod 3600 2>/dev/null)

# Test counters
PASSED=0
FAILED=0

# Function to test endpoint
test_endpoint() {
    local name=$1
    local method=$2
    local params=$3
    local expected=$4
    
    echo -n "Testing $name... "
    result=$(curl -sS -H "Authorization: Bearer $TOKEN" \
                  -H "Content-Type: application/json" \
                  -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$method\",\"params\":$params}" \
                  http://127.0.0.1:3030 2>/dev/null)
    
    if echo "$result" | grep -q "$expected"; then
        echo "✅ PASSED"
        ((PASSED++))
        return 0
    else
        echo "❌ FAILED"
        echo "  Got: $result"
        ((FAILED++))
        return 1
    fi
}

# Run tests
test_endpoint "Authentication" "auth_ping" "[]" "\"ok\":true"
test_endpoint "Wallet Creation" "wallet_create" "[\"testuser\"]" "sultan1"
test_endpoint "Proposal Creation" "proposal_create" "[\"test_prop\",\"Test\",\"Desc\",\"active\"]" "created"
test_endpoint "Vote Submission" "vote_on_proposal" "{\"proposal_id\":\"test_prop\",\"validator_id\":\"val1\",\"vote\":true}" "signed"
test_endpoint "Vote Tally" "votes_tally" "[\"test_prop\"]" "yes_votes"
test_endpoint "APY Query" "query_apy" "[true]" "apy"

echo ""
echo "=== Test Results ==="
echo "Passed: $PASSED"
echo "Failed: $FAILED"
if [ $PASSED -gt 0 ]; then
    echo "Success Rate: $(( PASSED * 100 / (PASSED + FAILED) ))%"
else
    echo "Success Rate: 0%"
fi
