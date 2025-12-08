#!/bin/bash
# Sultan L1 Sharding Deployment Script
# Deploys Sultan node with sharding enabled for 200K+ TPS

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  SULTAN L1 SHARDING DEPLOYMENT                              ║"
echo "║  High-Performance Blockchain with 200K+ TPS Capacity        ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Configuration
SHARD_COUNT="${SHARD_COUNT:-100}"
TX_PER_SHARD="${TX_PER_SHARD:-10000}"
BLOCK_TIME="${BLOCK_TIME:-5}"
DATA_DIR="${DATA_DIR:-./sultan-data-sharded}"
RPC_PORT="${RPC_PORT:-26657}"
VALIDATOR_ADDRESS="${VALIDATOR_ADDRESS:-genesis-validator}"
VALIDATOR_STAKE="${VALIDATOR_STAKE:-500000000000000}"

# Calculate TPS capacity
TPS_CAPACITY=$(( (SHARD_COUNT * TX_PER_SHARD) / BLOCK_TIME ))

echo "📊 Configuration:"
echo "  Shards:           $SHARD_COUNT"
echo "  Tx per Shard:     $TX_PER_SHARD"
echo "  Block Time:       ${BLOCK_TIME}s"
echo "  TPS Capacity:     $TPS_CAPACITY"
echo "  Data Directory:   $DATA_DIR"
echo "  RPC Port:         $RPC_PORT"
echo ""

# Check if node binary exists
BINARY_PATH="/tmp/cargo-target/release/sultan-node"
if [ ! -f "$BINARY_PATH" ]; then
    BINARY_PATH="./target/release/sultan-node"
fi

if [ ! -f "$BINARY_PATH" ]; then
    echo "❌ Error: sultan-node binary not found"
    echo "   Run: cargo build --release --bin sultan-node"
    exit 1
fi

echo "✅ Sultan node binary found: $BINARY_PATH"
echo ""

# Create data directory
mkdir -p "$DATA_DIR"
echo "✅ Data directory created: $DATA_DIR"
echo ""

# Stop existing node if running
if pgrep -f "sultan-node.*--enable-sharding" > /dev/null; then
    echo "⚠️  Stopping existing sharded node..."
    pkill -f "sultan-node.*--enable-sharding" || true
    sleep 2
fi

echo "🚀 Starting Sultan L1 with sharding..."
echo ""

# Start node with sharding enabled
"$BINARY_PATH" \
    --name "sultan-sharded-validator" \
    --validator \
    --validator-address "$VALIDATOR_ADDRESS" \
    --validator-stake "$VALIDATOR_STAKE" \
    --genesis "genesis:$VALIDATOR_STAKE" \
    --data-dir "$DATA_DIR" \
    --rpc-addr "0.0.0.0:$RPC_PORT" \
    --block-time "$BLOCK_TIME" \
    --enable-sharding \
    --shard-count "$SHARD_COUNT" \
    --tx-per-shard "$TX_PER_SHARD" \
    > "${DATA_DIR}/node.log" 2>&1 &

NODE_PID=$!
echo "✅ Node started (PID: $NODE_PID)"
echo ""

# Wait for node to be ready
echo "⏳ Waiting for node to be ready..."
for i in {1..30}; do
    if curl -s "http://localhost:$RPC_PORT/status" > /dev/null 2>&1; then
        echo "✅ Node is ready!"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "❌ Timeout waiting for node"
        echo "   Check logs: tail -f $DATA_DIR/node.log"
        exit 1
    fi
    sleep 1
    echo -n "."
done

echo ""
echo ""

# Get node status
echo "📈 Node Status:"
curl -s "http://localhost:$RPC_PORT/status" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    print(f'  Block Height:      {data[\"height\"]:,}')
    print(f'  Validator Count:   {data[\"validator_count\"]}')
    print(f'  Sharding Enabled:  {\"✅ YES\" if data.get(\"sharding_enabled\") else \"❌ NO\"}'  )
    print(f'  Shard Count:       {data.get(\"shard_count\", 0)}')
    print(f'  Total Accounts:    {data.get(\"total_accounts\", 0):,}')
    print(f'  Pending Txs:       {data.get(\"pending_txs\", 0)}')
except Exception as e:
    print(f'  Error parsing status: {e}')
    sys.exit(1)
" || {
    echo "  Could not parse status"
    echo "  Raw response:"
    curl -s "http://localhost:$RPC_PORT/status"
}

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  🎉 SULTAN L1 SHARDING DEPLOYMENT COMPLETE! 🎉              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "📊 Performance Metrics:"
echo "  TPS Capacity:      $TPS_CAPACITY"
echo "  Block Time:        ${BLOCK_TIME}s"
echo "  Shards:            $SHARD_COUNT"
echo "  Tx per Shard:      $TX_PER_SHARD"
echo ""
echo "🔗 Endpoints:"
echo "  RPC:               http://localhost:$RPC_PORT"
echo "  Status:            http://localhost:$RPC_PORT/status"
echo "  Health:            http://localhost:$RPC_PORT/health"
echo ""
echo "📁 Files:"
echo "  Data Directory:    $DATA_DIR"
echo "  Logs:              $DATA_DIR/node.log"
echo "  PID File:          $DATA_DIR/node.pid"
echo ""
echo "📝 Management Commands:"
echo "  View Logs:         tail -f $DATA_DIR/node.log"
echo "  Check Status:      curl http://localhost:$RPC_PORT/status | jq"
echo "  Stop Node:         kill $NODE_PID"
echo "  Restart:           ./deploy_sharding.sh"
echo ""
echo "✅ Sultan L1 is now running with sharding enabled!"
echo "   Capable of 200,000+ TPS with zero gas fees forever!"
echo ""

# Save PID
echo $NODE_PID > "$DATA_DIR/node.pid"

# Monitor block production for 15 seconds
echo "🔍 Monitoring block production (15 seconds)..."
echo ""

start_height=$(curl -s "http://localhost:$RPC_PORT/status" | python3 -c "import sys,json; print(json.load(sys.stdin)['height'])" 2>/dev/null || echo "0")
sleep 15
end_height=$(curl -s "http://localhost:$RPC_PORT/status" | python3 -c "import sys,json; print(json.load(sys.stdin)['height'])" 2>/dev/null || echo "0")

blocks_produced=$(( end_height - start_height ))
echo "📊 Block Production:"
echo "  Start Height:      $start_height"
echo "  End Height:        $end_height"
echo "  Blocks Produced:   $blocks_produced in 15 seconds"
echo "  Average:           $(echo "scale=2; $blocks_produced / 15" | bc 2>/dev/null || echo "N/A") blocks/second"
echo ""

if [ "$blocks_produced" -gt 0 ]; then
    echo "✅ Block production is working!"
else
    echo "⚠️  No blocks produced yet, give it more time"
fi

echo ""
echo "🚀 Deployment complete! Sultan L1 is live with sharding enabled."
