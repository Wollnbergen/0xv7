#!/bin/bash
set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║     🚀 STARTING REAL COSMOS SULTAN BLOCKCHAIN 🚀             ║"
echo "╚══════════════════════════════════════════════════════════════╝"

# Try sultan-cosmos-real first (newest)
if [ -d "/workspaces/0xv7/sultan-cosmos-real" ]; then
    echo "Found sultan-cosmos-real, building..."
    cd /workspaces/0xv7/sultan-cosmos-real
    
    # Build if not built
    if [ ! -f "build/sultand" ]; then
        go mod tidy
        go build -o build/sultand ./cmd/sultand
    fi
    
    # Initialize if not initialized
    if [ ! -d "$HOME/.sultan" ]; then
        ./build/sultand init test-node --chain-id sultan-1
        ./build/sultand keys add validator --keyring-backend test
        ./build/sultand add-genesis-account validator 1000000000stake --keyring-backend test
        ./build/sultand gentx validator 1000000stake --keyring-backend test --chain-id sultan-1
        ./build/sultand collect-gentxs
    fi
    
    # Start the chain
    echo "Starting Sultan Cosmos Chain..."
    ./build/sultand start --minimum-gas-prices 0stake
else
    echo "❌ No Cosmos implementation found ready to run"
    echo "Run: cd /workspaces/0xv7 && ignite scaffold chain sultan"
fi
