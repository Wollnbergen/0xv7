#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          SULTAN CHAIN - TESTING ALL ENDPOINTS                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Test consensus
echo "1️⃣ Testing Consensus (http://localhost:4001):"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s http://localhost:4001/consensus_state 2>/dev/null | jq '.' 2>/dev/null || echo "❌ Not responding"
echo ""

# Test RPC
echo "2️⃣ Testing RPC Server (http://localhost:3030):"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s -X POST http://localhost:3030 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"chain_getInfo","params":[],"id":1}' 2>/dev/null | jq '.' 2>/dev/null || echo "❌ Not responding"
echo ""

# Test API
echo "3️⃣ Testing API Server (http://localhost:3000):"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s http://localhost:3000 2>/dev/null | head -20 || echo "❌ Not responding"
echo ""

# Test P2P
echo "4️⃣ Testing P2P Network (http://localhost:5001):"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s http://localhost:5001/status 2>/dev/null | jq '.' 2>/dev/null || echo "❌ Not responding"

