#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║      SULTAN CHAIN - TESTING MULTI-NODE CONSENSUS              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "🔗 Proposing a block through node 1..."
curl -s -X POST http://localhost:4001/propose_block \
    -H "Content-Type: application/json" \
    -d '{
        "block": {
            "height": 13300,
            "timestamp": "'$(date +%s)'",
            "transactions": 5,
            "gasFeesCollected": 0
        },
        "proposer": "validator-1"
    }' | jq '.'

echo ""
echo "📊 Checking consensus state across all nodes:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

for port in 4001 4002 4003; do
    echo ""
    echo "Node $((port-4000)) (port $port):"
    curl -s http://localhost:$port/consensus_state | jq '{blockHeight, consensusRounds, lastConsensus}'
done

echo ""
echo "✅ Multi-node consensus test complete!"
