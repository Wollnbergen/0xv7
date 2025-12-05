#!/bin/bash

API="http://127.0.0.1:3030"
PASSED=0
FAILED=0

echo "Testing Sultan Chain Endpoints..."
echo "================================="

# Test function
test_endpoint() {
    local method=$1
    local params=$2
    local expected=$3
    
    response=$(curl -s -X POST $API \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":$params,\"id\":1}")
    
    if echo "$response" | grep -q "$expected"; then
        echo "✅ $method: PASSED"
        ((PASSED++))
    else
        echo "❌ $method: FAILED"
        echo "   Response: $response"
        ((FAILED++))
    fi
}

# Run tests
test_endpoint "chain_status" "[]" "sultan"
test_endpoint "wallet_create" "[\"test_wallet\"]" "address"
test_endpoint "wallet_balance" "[\"test_wallet\"]" "balance"
test_endpoint "token_transfer" "[\"alice\",\"bob\",1000]" "tx_hash"
test_endpoint "validator_list" "[]" "validators"
test_endpoint "get_apy" "[]" "base_apy"
test_endpoint "mobile_validator_info" "[]" "bonus"

echo ""
echo "Results: $PASSED passed, $FAILED failed"
