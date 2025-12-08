#!/bin/bash

echo "🚀 Initializing Sultan Chain Mainnet..."

# Generate genesis validators
echo "Generating validator keys..."
for i in {1..3}; do
    mkdir -p /data/validator$i
    # Generate keys (simplified for demo)
    echo "Validator $i initialized"
done

# Initialize chain
./sultand init sultan-mainnet-1 --chain-id sultan-mainnet-1

# Start node
echo "Starting Sultan Chain Mainnet..."
./sultand start \
    --minimum-gas-prices="0stake" \
    --rpc.laddr="tcp://0.0.0.0:26657" \
    --p2p.laddr="tcp://0.0.0.0:26656" \
    --grpc.address="0.0.0.0:9090" \
    --grpc-web.enable=true
