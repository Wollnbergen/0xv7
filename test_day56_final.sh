#!/bin/bash

cd /workspaces/0xv7

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           TESTING DAY 5-6 TOKEN ECONOMICS                     ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Get JWT token
TOKEN=$(./auth_client.sh 2>/dev/null | grep "TOKEN=" | cut -d'=' -f2)

if [ -z "$TOKEN" ]; then
    echo "❌ Failed to get auth token"
    exit 1
fi

echo "✅ Got auth token"
echo ""

# Test token transfer
echo "🔄 Testing token transfer..."
curl -s -X POST http://127.0.0.1:3030 \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $TOKEN" \
    -d '{
        "jsonrpc": "2.0",
        "method": "token_transfer",
        "params": ["sultan1test1", "sultan1test2", 100, "Test transfer"],
        "id": 1
    }' | jq '.'

echo ""

# Test reward calculation
echo "📊 Testing reward calculation..."
curl -s -X POST http://127.0.0.1:3030 \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $TOKEN" \
    -d '{
        "jsonrpc": "2.0",
        "method": "calculate_rewards",
        "params": ["sultan1test1"],
        "id": 2
    }' | jq '.'

echo ""

# Test reward claiming
echo "💰 Testing reward claiming..."
curl -s -X POST http://127.0.0.1:3030 \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $TOKEN" \
    -d '{
        "jsonrpc": "2.0",
        "method": "claim_rewards",
        "params": ["sultan1test1"],
        "id": 3
    }' | jq '.'

echo ""
echo "✅ Day 5-6 tests complete!"
