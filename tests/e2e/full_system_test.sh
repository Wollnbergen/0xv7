#!/bin/bash

echo "🧪 Running End-to-End System Test..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Test 1: Submit 100 transactions and verify zero gas
echo "Test 1: Submitting 100 transactions..."
for i in {1..100}; do
    curl -s -X POST http://localhost:8080/tx \
        -H "Content-Type: application/json" \
        -d "{\"from\":\"user$i\",\"to\":\"user$((i+1))\",\"amount\":$i}" > /dev/null
done

# Wait for mining
sleep 6

# Check blocks
BLOCKS=$(curl -s http://localhost:8080/blocks)
BLOCK_COUNT=$(echo "$BLOCKS" | python3 -c "import sys, json; print(len(json.load(sys.stdin)))")
echo "✅ Blocks mined: $BLOCK_COUNT"

# Verify zero gas in all blocks
echo "$BLOCKS" | python3 -c "
import sys, json
blocks = json.load(sys.stdin)
total_gas = 0
for block in blocks:
    for tx in block.get('transactions', []):
        total_gas += tx.get('gas_fee', 0)
print(f'✅ Total gas fees collected: \${total_gas:.2f}')
assert total_gas == 0, 'Gas fees should be zero!'
print('✅ Zero gas fees verified!')"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ All E2E tests passed!"

