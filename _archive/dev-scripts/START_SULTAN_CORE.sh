#!/bin/bash
# Sultan Core Production Startup Script
# Starts Sultan-first Rust blockchain (Layer 1)

set -e

echo "🏰 Sultan Core - Layer 1 Blockchain"
echo "===================================="
echo ""

# Configuration
DATA_DIR="$HOME/.sultan-core"
VALIDATOR_ADDRESS="genesis-validator"
VALIDATOR_STAKE="10000000000"  # 10,000 SLTN (with 6 decimals = 10,000,000,000 usltn)
GENESIS_ACCOUNTS="genesis:500000000000000"  # 500M SLTN total supply
BLOCK_TIME="6"  # 6 seconds to match Cosmos version

# Create data directory
mkdir -p "$DATA_DIR"

echo "📊 Configuration:"
echo "  Data Directory: $DATA_DIR"
echo "  Validator: $VALIDATOR_ADDRESS"
echo "  Stake: $VALIDATOR_STAKE usltn (10,000 SLTN)"
echo "  Genesis Supply: 500,000,000 SLTN"
echo "  Block Time: ${BLOCK_TIME}s"
echo ""

# Check if binary exists
if [ ! -f "/workspaces/0xv7/sultan-core/target/release/sultan-node" ]; then
    echo "❌ Binary not found. Building..."
    cd /workspaces/0xv7/sultan-core
    cargo build --release --bin sultan-node
fi

echo "✅ Binary ready"
echo ""

# Start the node
echo "🚀 Starting Sultan Core node..."
echo ""

cd /workspaces/0xv7/sultan-core

./target/release/sultan-node \
    --name "sultan-core-validator-1" \
    --data-dir "$DATA_DIR" \
    --block-time "$BLOCK_TIME" \
    --validator \
    --validator-address "$VALIDATOR_ADDRESS" \
    --validator-stake "$VALIDATOR_STAKE" \
    --rpc-addr "0.0.0.0:26657" \
    --p2p-addr "/ip4/0.0.0.0/tcp/26656" \
    --genesis "$GENESIS_ACCOUNTS"
