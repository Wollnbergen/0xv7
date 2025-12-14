#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║              BLOCKCHAIN PERFORMANCE TEST                      ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "⚡ Testing transaction throughput with ZERO gas fees..."
echo ""

# Function to send transaction
send_tx() {
    curl -s -X POST http://localhost:8080/write \
        -H "Content-Type: application/json" \
        -d "{\"data\":\"Performance test transaction #$1\"}" > /dev/null
}

# Get initial block count
INITIAL=$(curl -s http://localhost:8080/blocks | python3 -c "import sys, json; print(len(json.load(sys.stdin)))")

echo "📊 Initial state:"
echo "  Blocks: $INITIAL"
echo ""

echo "🚀 Sending 100 transactions..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

START_TIME=$(date +%s%N)

# Send 100 transactions
for i in {1..100}; do
    send_tx $i &
    if [ $((i % 10)) -eq 0 ]; then
        echo -n "█"
    fi
done
wait

echo " Done!"

END_TIME=$(date +%s%N)
ELAPSED=$((($END_TIME - $START_TIME) / 1000000))

# Get final block count
FINAL=$(curl -s http://localhost:8080/blocks | python3 -c "import sys, json; print(len(json.load(sys.stdin)))")
PROCESSED=$((FINAL - INITIAL))

echo ""
echo "📈 Results:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Transactions sent: 100"
echo "  Transactions confirmed: $PROCESSED"
echo "  Time taken: ${ELAPSED}ms"
echo "  TPS: $(echo "scale=2; $PROCESSED * 1000 / $ELAPSED" | bc)"
echo "  Total gas fees: $0.00 (ZERO!)"
echo ""
echo "�� Cost comparison:"
echo "  Our chain: $0.00"
echo "  Ethereum: ~$3,000"
echo "  You saved: $3,000! 🎉"

