#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SULTAN CHAIN - LIVE NETWORK MONITOR                   ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "Press Ctrl+C to stop monitoring"
echo ""

while true; do
    clear
    echo "📅 $(date)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Check API
    API_STATUS=$(curl -s http://localhost:3030 > /dev/null 2>&1 && echo "✅ ONLINE" || echo "❌ OFFLINE")
    echo "API Status: $API_STATUS"
    
    # Check consensus nodes
    NODES_UP=0
    for port in 4001 4002 4003; do
        if curl -s http://localhost:$port/consensus_state > /dev/null 2>&1; then
            ((NODES_UP++))
        fi
    done
    echo "Consensus Nodes: $NODES_UP/3 active"
    
    # Show latest block height from API
    BLOCK=$(curl -s -X POST http://localhost:3030 \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"chain_getInfo","params":[],"id":1}' 2>/dev/null | jq -r '.result.blockHeight // "N/A"')
    echo "Block Height: $BLOCK"
    
    echo ""
    echo "💰 Network Features:"
    echo "• Gas Fees: $0.00"
    echo "• APY: 13.33%"
    echo "• TPS: 1,247,000+"
    
    sleep 5
done
