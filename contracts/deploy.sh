#!/bin/bash

CHAIN_ID="test-1"
NODE="http://localhost:26657"

echo "🚀 Deploying Sultan Token (CW20) to blockchain..."

# Store the contract
if [ -f "./cw20-token/target/wasm32-unknown-unknown/release/sultan_token.wasm" ]; then
    TX_HASH=$(docker exec cosmos-node wasmd tx wasm store \
        /contracts/sultan_token.wasm \
        --from validator \
        --keyring-backend test \
        --chain-id $CHAIN_ID \
        --gas auto \
        --gas-adjustment 1.3 \
        --fees 0stake \
        -y \
        --output json | jq -r '.txhash')
    
    echo "Contract stored with tx: $TX_HASH"
    sleep 6
    
    # Get code ID
    CODE_ID=$(docker exec cosmos-node wasmd query tx $TX_HASH --output json | \
        jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
    
    echo "Code ID: $CODE_ID"
    
    # Instantiate the contract
    INIT='{
        "name": "Sultan Token",
        "symbol": "SLTN",
        "decimals": 6,
        "initial_balances": [
            {
                "address": "wasm1kkcdw94sdfal63elmjezlu3hx4lexqupkufx7v",
                "amount": "1000000000000"
            }
        ],
        "mint": {
            "minter": "wasm1kkcdw94sdfal63elmjezlu3hx4lexqupkufx7v"
        }
    }'
    
    docker exec cosmos-node wasmd tx wasm instantiate $CODE_ID "$INIT" \
        --from validator \
        --keyring-backend test \
        --label "sultan-token" \
        --chain-id $CHAIN_ID \
        --fees 0stake \
        -y
    
    echo "✅ Sultan Token deployed successfully!"
else
    echo "⚠️  Contract not built yet. Building now..."
fi
