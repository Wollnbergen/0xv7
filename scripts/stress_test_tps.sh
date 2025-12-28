#!/bin/bash
# Sultan L1 TPS Stress Test
# Tests transaction throughput capacity

set -e

RPC_ENDPOINT="${1:-http://134.122.96.36:8545}"
TX_COUNT="${2:-1000}"
CONCURRENCY="${3:-50}"

echo "=========================================="
echo "Sultan L1 TPS Stress Test"
echo "=========================================="
echo "Endpoint: $RPC_ENDPOINT"
echo "Total Transactions: $TX_COUNT"
echo "Concurrency: $CONCURRENCY"
echo "=========================================="

# Check initial block height
INITIAL_HEIGHT=$(curl -s "$RPC_ENDPOINT/status" | jq -r '.height')
echo "Initial block height: $INITIAL_HEIGHT"
echo ""

# Create temp directory for results
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

# Function to send a batch of transactions
send_transactions() {
    local batch_id=$1
    local count=$2
    local results_file="$TEMP_DIR/batch_$batch_id.txt"
    
    for i in $(seq 1 $count); do
        local nonce=$((batch_id * 10000 + i))
        local timestamp=$(date +%s)
        local from="stress_test_$batch_id"
        local to="receiver_$((i % 100))"
        
        # Send transaction
        start_time=$(date +%s%N)
        result=$(curl -s -X POST "$RPC_ENDPOINT/tx" \
            -H "Content-Type: application/json" \
            -d "{\"from\":\"$from\",\"to\":\"$to\",\"amount\":1,\"gas_fee\":0,\"timestamp\":$timestamp,\"nonce\":$nonce}" \
            2>/dev/null)
        end_time=$(date +%s%N)
        
        if echo "$result" | grep -q "hash"; then
            echo "success,$((($end_time - $start_time) / 1000000))" >> "$results_file"
        else
            echo "fail,$((($end_time - $start_time) / 1000000))" >> "$results_file"
        fi
    done
}

# Calculate transactions per batch
BATCHES=$CONCURRENCY
TXS_PER_BATCH=$((TX_COUNT / BATCHES))

echo "Starting stress test..."
echo "Sending $TX_COUNT transactions in $BATCHES parallel batches..."
echo ""

# Record start time
START_TIME=$(date +%s%N)

# Launch parallel batches
for batch in $(seq 1 $BATCHES); do
    send_transactions $batch $TXS_PER_BATCH &
done

# Wait for all batches to complete
wait

# Record end time
END_TIME=$(date +%s%N)

# Calculate duration in milliseconds
DURATION_MS=$(((END_TIME - START_TIME) / 1000000))
DURATION_SEC=$(echo "scale=2; $DURATION_MS / 1000" | bc)

# Count results
SUCCESS_COUNT=$(cat $TEMP_DIR/batch_*.txt 2>/dev/null | grep "^success" | wc -l)
FAIL_COUNT=$(cat $TEMP_DIR/batch_*.txt 2>/dev/null | grep "^fail" | wc -l)

# Calculate TPS
if [ "$DURATION_SEC" != "0" ]; then
    TPS=$(echo "scale=2; $SUCCESS_COUNT / $DURATION_SEC" | bc)
else
    TPS="N/A"
fi

# Get final block height
sleep 2
FINAL_HEIGHT=$(curl -s "$RPC_ENDPOINT/status" | jq -r '.height')
BLOCKS_PRODUCED=$((FINAL_HEIGHT - INITIAL_HEIGHT))

# Calculate average latency
if [ -f "$TEMP_DIR/batch_1.txt" ]; then
    AVG_LATENCY=$(cat $TEMP_DIR/batch_*.txt | grep "^success" | cut -d',' -f2 | awk '{sum+=$1; count++} END {if(count>0) print sum/count; else print "N/A"}')
else
    AVG_LATENCY="N/A"
fi

echo "=========================================="
echo "RESULTS"
echo "=========================================="
echo "Duration: ${DURATION_SEC}s"
echo "Successful Transactions: $SUCCESS_COUNT"
echo "Failed Transactions: $FAIL_COUNT"
echo "Transactions Per Second (TPS): $TPS"
echo "Average Latency: ${AVG_LATENCY}ms"
echo "Blocks Produced: $BLOCKS_PRODUCED"
echo "Final Block Height: $FINAL_HEIGHT"
echo "=========================================="

# Save results
RESULTS_FILE="/workspaces/0xv7/scripts/tps_results_$(date +%Y%m%d_%H%M%S).json"
cat > "$RESULTS_FILE" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "endpoint": "$RPC_ENDPOINT",
  "tx_count": $TX_COUNT,
  "concurrency": $CONCURRENCY,
  "duration_sec": $DURATION_SEC,
  "success_count": $SUCCESS_COUNT,
  "fail_count": $FAIL_COUNT,
  "tps": $TPS,
  "avg_latency_ms": $AVG_LATENCY,
  "blocks_produced": $BLOCKS_PRODUCED,
  "initial_height": $INITIAL_HEIGHT,
  "final_height": $FINAL_HEIGHT
}
EOF
echo "Results saved to: $RESULTS_FILE"
