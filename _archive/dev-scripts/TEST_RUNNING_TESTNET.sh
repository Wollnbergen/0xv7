#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           TESTING SULTAN CHAIN TESTNET                        ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Test 1: Status check
echo "🧪 Test 1: Chain Status"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s -X POST http://localhost:3030 \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"get_status","id":1}' | python3 -m json.tool
echo ""

# Test 2: Economics
echo "🧪 Test 2: Economics Model"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s -X POST http://localhost:3030 \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"get_economics","id":1}' | python3 -m json.tool
echo ""

# Test 3: Transfer (Zero Fee)
echo "🧪 Test 3: Zero-Fee Transfer"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s -X POST http://localhost:3030 \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"transfer","params":{"from":"sultan1abc","to":"sultan1xyz","amount":100},"id":1}' | python3 -m json.tool
echo ""

# Test 4: APY Query
echo "🧪 Test 4: Validator APY"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s -X POST http://localhost:3030 \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"get_apy","id":1}' | python3 -m json.tool
echo ""

echo "✅ All tests completed!"

