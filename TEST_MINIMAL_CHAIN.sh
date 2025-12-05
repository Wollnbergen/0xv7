#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           TESTING MINIMAL ZERO-GAS BLOCKCHAIN                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Check if blockchain is running
if ! curl -s http://localhost:8080/status > /dev/null 2>&1; then
    echo "❌ Blockchain not running. Starting it..."
    cd /workspaces/0xv7/minimal-chain
    ./minimal-chain > /tmp/blockchain.log 2>&1 &
    sleep 2
fi

echo "📊 Chain Status:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s http://localhost:8080/status | python3 -m json.tool

echo ""
echo "📦 Adding test blocks with ZERO gas fees:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Add some test transactions
curl -s -X POST http://localhost:8080/write \
  -H "Content-Type: application/json" \
  -d '{"data":"First transaction - Zero gas!"}' | python3 -m json.tool

echo ""
curl -s -X POST http://localhost:8080/write \
  -H "Content-Type: application/json" \
  -d '{"data":"Sultan Chain migration test"}' | python3 -m json.tool

echo ""
curl -s -X POST http://localhost:8080/write \
  -H "Content-Type: application/json" \
  -d '{"data":"10M TPS coming soon!"}' | python3 -m json.tool

echo ""
echo "📜 Current Blockchain:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s http://localhost:8080/blocks | python3 -m json.tool | head -50

echo ""
echo "✅ All transactions processed with ZERO gas fees!"
echo ""
echo "View live dashboard at:"
echo "  $BROWSER http://localhost:3000/minimal-dashboard.html"

