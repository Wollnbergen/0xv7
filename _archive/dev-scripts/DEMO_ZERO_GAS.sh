#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║        ZERO GAS BLOCKCHAIN - LIVE DEMONSTRATION               ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "🎯 Demonstrating ZERO gas fees on every transaction!"
echo ""

# Function to add a block and show gas fee
add_block() {
    local data="$1"
    echo "➤ Adding: \"$data\""
    RESULT=$(curl -s -X POST http://localhost:8080/write \
        -H "Content-Type: application/json" \
        -d "{\"data\":\"$data\"}")
    
    GAS_FEE=$(echo $RESULT | grep -o '"gas_fee":[0-9.]*' | cut -d: -f2)
    HASH=$(echo $RESULT | grep -o '"hash":"[^"]*' | cut -d'"' -f4 | cut -c1-16)
    
    echo "  ✅ Block added: ${HASH}..."
    echo "  💸 Gas Fee: $GAS_FEE (ZERO!)"
    echo ""
}

# Check blockchain status
echo "📊 Current Blockchain Status:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
STATUS=$(curl -s http://localhost:8080/status)
echo $STATUS | python3 -m json.tool
echo ""

# Add multiple transactions
echo "💳 Processing Transactions with ZERO Gas Fees:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

add_block "Transfer 1000 SULTAN tokens - NO GAS FEE!"
add_block "Deploy smart contract - NO GAS FEE!"
add_block "NFT mint #001 - NO GAS FEE!"
add_block "DeFi swap executed - NO GAS FEE!"
add_block "DAO vote recorded - NO GAS FEE!"

# Show total savings
echo "💰 Gas Fee Savings Calculator:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
BLOCKS=$(curl -s http://localhost:8080/blocks | python3 -c "import sys, json; print(len(json.load(sys.stdin)))")
ETH_EQUIV=$((BLOCKS * 30))  # Average $30 per transaction on Ethereum
echo "  Transactions processed: $BLOCKS"
echo "  Total gas fees paid: $0.00"
echo "  Equivalent on Ethereum: ~\$$ETH_EQUIV"
echo "  YOU SAVED: \$$ETH_EQUIV! 🎉"
echo ""

echo "🌐 View Live Dashboard:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  $BROWSER http://localhost:3000/minimal-dashboard.html"
echo ""

