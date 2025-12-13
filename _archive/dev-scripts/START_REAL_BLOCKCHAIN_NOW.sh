#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          STARTING REAL SULTAN BLOCKCHAIN NOW                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"

# Use the Cosmos implementation that's already built
cd /workspaces/0xv7/sultan

# Build if needed
if [ ! -f "build/sultand" ]; then
    echo "Building Sultan Chain with Cosmos SDK..."
    go build -o build/sultand ./cmd/sultand
fi

# Initialize chain if needed
if [ ! -d "$HOME/.sultan" ]; then
    ./build/sultand init test-node --chain-id sultan-1
    ./build/sultand genesis add-account sultan1test 100000000stake
    ./build/sultand genesis gentx sultan1test 1000000stake
    ./build/sultand genesis collect-gentxs
fi

# Start the chain
echo "🚀 Starting Sultan Chain (Cosmos SDK)..."
./build/sultand start &

sleep 5

# Start the custom API on different port
cd /workspaces/0xv7
python3 sultan_actual_api.py &

echo ""
echo "✅ SULTAN CHAIN IS NOW RUNNING!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Cosmos RPC: http://localhost:26657"
echo "Cosmos API: http://localhost:1317"
echo "Sultan Custom API: http://localhost:3030"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
