#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║      SULTAN CHAIN - LIVE CONSENSUS MONITOR                    ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "Press Ctrl+C to stop monitoring"
echo ""

while true; do
    clear
    echo "SULTAN CHAIN - MULTI-NODE CONSENSUS STATUS"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Time: $(date)"
    echo ""
    
    # Get status from each node
    for port in 4001 4002 4003; do
        if curl -s http://localhost:$port/health > /dev/null 2>&1; then
            STATE=$(curl -s http://localhost:$port/consensus_state)
            BLOCK=$(echo $STATE | jq -r '.blockHeight')
            VALIDATORS=$(echo $STATE | jq -r '.validators')
            ROUNDS=$(echo $STATE | jq -r '.consensusRounds')
            
            echo "Node $((port-4000)): Block #$BLOCK | Validators: $VALIDATORS | Consensus Rounds: $ROUNDS"
        else
            echo "Node $((port-4000)): OFFLINE"
        fi
    done
    
    echo ""
    echo "Network Features:"
    echo "• Gas Fees: $0.00 (ALWAYS FREE)"
    echo "• APY: 26.67%"
    echo "• Min Stake: 5,000 SLTN"
    
    sleep 3
done
