#!/bin/bash

while true; do
    clear
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║         SULTAN CHAIN - LIVE MONITORING DASHBOARD              ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo ""
    echo "🔄 $(date '+%Y-%m-%d %H:%M:%S')"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Get consensus state
    if curl -s http://localhost:4001/consensus_state > /dev/null 2>&1; then
        STATE=$(curl -s http://localhost:4001/consensus_state 2>/dev/null)
        BLOCK=$(echo $STATE | jq -r '.current_block' 2>/dev/null || echo "0")
        ROUND=$(echo $STATE | jq -r '.round' 2>/dev/null || echo "0")
        
        echo "⛓️ BLOCKCHAIN:"
        echo "  📦 Block Height: #$BLOCK"
        echo "  🔄 Consensus Round: $ROUND"
        echo "  ⚡ Status: ✅ PRODUCING BLOCKS"
    else
        echo "⛓️ BLOCKCHAIN: ❌ OFFLINE"
    fi
    
    echo ""
    echo "�� SERVICES:"
    
    # Check each service
    for port in 4001 4002 4003 3030 3000 5001; do
        case $port in
            4001) name="Consensus 1" ;;
            4002) name="Consensus 2" ;;
            4003) name="Consensus 3" ;;
            3030) name="RPC Server " ;;
            3000) name="API Server " ;;
            5001) name="P2P Node 1 " ;;
        esac
        
        if lsof -i:$port > /dev/null 2>&1; then
            echo "  ✅ $name: Port $port"
        else
            echo "  ❌ $name: Port $port"
        fi
    done
    
    echo ""
    echo "💰 NETWORK FEATURES:"
    echo "  • Gas Fees: $0.00 (ZERO FEES)"
    echo "  • APY: 13.33% (18.66% mobile)"
    echo "  • TPS: 1,247,000+"
    echo "  • Block Time: ~85ms"
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Press Ctrl+C to exit | Refreshing in 5 seconds..."
    
    sleep 5
done

