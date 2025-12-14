#!/bin/bash
# Sultan Node Launcher - Production Ready
# Usage: ./start-sultan-node.sh [validator|observer]

set -e

MODE=${1:-validator}
DATA_DIR="./sultan-data"
NODE_NAME="sultan-prod-node-1"

echo "🚀 Sultan Chain Node Launcher"
echo "================================"
echo "Mode: $MODE"
echo "Data Directory: $DATA_DIR"
echo ""

# Build if needed
if [ ! -f "/tmp/cargo-target/release/sultan-node" ]; then
    echo "📦 Building Sultan node..."
    cd /workspaces/0xv7/sultan-core
    cargo build --release --bin sultan-node
    cd -
fi

# Clean data for fresh start (comment out to persist)
# rm -rf "$DATA_DIR"

# Create data directory
mkdir -p "$DATA_DIR"

echo "✅ Binary ready"
echo ""

if [ "$MODE" == "validator" ]; then
    echo "⛏️  Starting VALIDATOR node..."
    echo "   Address: validator1"
    echo "   Stake: 100000"
    echo "   Block time: 5s"
    echo ""
    
    /tmp/cargo-target/release/sultan-node \
        --name "$NODE_NAME" \
        --data-dir "$DATA_DIR" \
        --block-time 5 \
        --validator \
        --validator-address "validator1" \
        --validator-stake 100000 \
        --p2p-addr "/ip4/0.0.0.0/tcp/26656" \
        --rpc-addr "0.0.0.0:26657" \
        --genesis "alice:1000000,bob:500000,charlie:250000,validator1:100000"
else
    echo "👁️  Starting OBSERVER node..."
    echo "   (No block production)"
    echo ""
    
    /tmp/cargo-target/release/sultan-node \
        --name "$NODE_NAME" \
        --data-dir "$DATA_DIR" \
        --block-time 5 \
        --p2p-addr "/ip4/0.0.0.0/tcp/26656" \
        --rpc-addr "0.0.0.0:26657" \
        --genesis "alice:1000000,bob:500000,charlie:250000"
fi
