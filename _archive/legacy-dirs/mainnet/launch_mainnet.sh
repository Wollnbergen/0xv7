#!/bin/bash

echo "🚀 Launching Sultan Chain Mainnet..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Initialize chain
sultand init sultan-mainnet --chain-id sultan-mainnet-1

# Copy genesis
cp /workspaces/0xv7/mainnet/config/genesis.json ~/.sultan/config/genesis.json

# Start node
sultand start \
    --minimum-gas-prices="0usltn" \
    --api.enable=true \
    --api.address="tcp://0.0.0.0:1317" \
    --grpc.enable=true \
    --grpc.address="0.0.0.0:9090" \
    --p2p.persistent_peers=""

echo "✅ Mainnet launched!"
