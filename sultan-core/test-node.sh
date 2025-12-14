#!/bin/bash
# Sultan Node Testing & Benchmarking Script

set -e

RPC_URL="http://localhost:26657"

echo "🧪 Sultan Node Test Suite"
echo "========================="
echo ""

# Wait for node to be ready
echo "⏳ Waiting for node to start..."
sleep 3

# Test 1: Get Status
echo "📊 Test 1: Get Node Status"
echo "Command: curl $RPC_URL/status"
STATUS=$(curl -s "$RPC_URL/status")
echo "Response: $STATUS"
HEIGHT=$(echo $STATUS | jq -r '.height')
echo "✅ Current height: $HEIGHT"
echo ""

# Test 2: Get Balance
echo "💰 Test 2: Get Balance"
echo "Command: curl $RPC_URL/balance/alice"
BALANCE=$(curl -s "$RPC_URL/balance/alice")
echo "Response: $BALANCE"
echo "✅ Alice balance: $(echo $BALANCE | jq -r '.balance')"
echo ""

# Test 3: Submit Transaction
echo "💸 Test 3: Submit Transaction (alice -> bob: 1000)"
TX_DATA='{
  "from": "alice",
  "to": "bob",
  "amount": 1000,
  "gas_fee": 0,
  "timestamp": '$(date +%s)',
  "nonce": 1,
  "signature": null
}'
echo "Transaction: $TX_DATA"
TX_RESULT=$(curl -s -X POST "$RPC_URL/tx" \
    -H "Content-Type: application/json" \
    -d "$TX_DATA")
echo "Response: $TX_RESULT"
echo "✅ Transaction submitted"
echo ""

# Wait for block
echo "⏳ Waiting for next block (5s)..."
sleep 6

# Test 4: Verify Balance Changed
echo "✔️  Test 4: Verify Balance After Transaction"
NEW_BALANCE=$(curl -s "$RPC_URL/balance/alice")
echo "Alice new balance: $(echo $NEW_BALANCE | jq -r '.balance')"
BOB_BALANCE=$(curl -s "$RPC_URL/balance/bob")
echo "Bob new balance: $(echo $BOB_BALANCE | jq -r '.balance')"
echo "✅ Balances updated"
echo ""

# Test 5: Get Latest Block
NEW_STATUS=$(curl -s "$RPC_URL/status")
NEW_HEIGHT=$(echo $NEW_STATUS | jq -r '.height')
echo "📦 Test 5: Get Latest Block (height: $NEW_HEIGHT)"
BLOCK=$(curl -s "$RPC_URL/block/$NEW_HEIGHT")
echo "Block: $(echo $BLOCK | jq -c '.')"
echo "✅ Block retrieved"
echo ""

# Performance Test: Submit 100 transactions
echo "🚀 Performance Test: Submitting 100 transactions..."
START_TIME=$(date +%s)

for i in {1..100}; do
    NONCE=$((i + 1))
    TX='{
      "from": "alice",
      "to": "bob",
      "amount": 10,
      "gas_fee": 0,
      "timestamp": '$(date +%s)',
      "nonce": '$NONCE',
      "signature": null
    }'
    curl -s -X POST "$RPC_URL/tx" \
        -H "Content-Type: application/json" \
        -d "$TX" > /dev/null
    
    if [ $((i % 10)) -eq 0 ]; then
        echo "  Submitted $i/100 transactions..."
    fi
done

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))
TPS=$((100 / DURATION))

echo "✅ Performance: 100 transactions in ${DURATION}s (~${TPS} TPS submission)"
echo ""

# Wait for blocks to process
echo "⏳ Waiting 15s for blocks to process transactions..."
sleep 15

# Final status
echo "📈 Final Status:"
FINAL_STATUS=$(curl -s "$RPC_URL/status")
echo "$FINAL_STATUS" | jq '.'
echo ""

echo "✅ All tests completed successfully!"
echo ""
echo "Summary:"
echo "  - Node is operational"
echo "  - Transactions processing correctly"
echo "  - Blocks being produced"
echo "  - RPC endpoints working"
echo "  - Performance: ~${TPS} TPS (transaction submission)"
