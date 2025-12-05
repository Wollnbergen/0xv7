#!/bin/bash
# Deploy a contract to Sultan Chain

CHAIN_ID="test-1"
NODE="http://localhost:26657"

echo "Deploying contract to Sultan Chain..."

# Store the contract code
TX_HASH=$(docker exec cosmos-node wasmd tx wasm store /path/to/contract.wasm \
    --from validator \
    --keyring-backend test \
    --chain-id $CHAIN_ID \
    --gas auto \
    --gas-adjustment 1.3 \
    --fees 0stake \
    -y \
    --output json | jq -r '.txhash')

echo "Contract stored with tx: $TX_HASH"

# Wait for transaction
sleep 6

# Get code ID
CODE_ID=$(docker exec cosmos-node wasmd query tx $TX_HASH --output json | jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')

echo "Contract Code ID: $CODE_ID"

# Instantiate the contract
INIT='{"count":100}'
docker exec cosmos-node wasmd tx wasm instantiate $CODE_ID "$INIT" \
    --from validator \
    --keyring-backend test \
    --label "counter" \
    --chain-id $CHAIN_ID \
    --fees 0stake \
    -y

echo "Contract instantiated!"
