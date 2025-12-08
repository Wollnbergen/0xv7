#!/bin/bash

echo "🚀 Deploying Sultan Smart Contracts..."

# Store CW20 contract
if [ -f "/workspaces/0xv7/contracts/cw20-sultan/target/wasm32-unknown-unknown/release/cw20_sultan.wasm" ]; then
    docker cp /workspaces/0xv7/contracts/cw20-sultan/target/wasm32-unknown-unknown/release/cw20_sultan.wasm cosmos-sultan:/tmp/
    
    echo "Storing CW20 contract on chain..."
    TX=$(docker exec cosmos-sultan wasmd tx wasm store /tmp/cw20_sultan.wasm \
        --from validator \
        --keyring-backend test \
        --chain-id test-1 \
        --gas auto \
        --fees 0stake \
        -y --output json 2>&1 | jq -r '.txhash // empty')
    
    if [ ! -z "$TX" ]; then
        echo "✅ CW20 stored - TX: ${TX:0:8}..."
        sleep 6
        
        # Instantiate
        INIT='{"name":"Sultan Token","symbol":"SLTN","decimals":6,"initial_balances":[],"mint":{"minter":"wasm1..."}}'
        docker exec cosmos-sultan wasmd tx wasm instantiate 1 "$INIT" \
            --from validator \
            --label "sultan-token" \
            --keyring-backend test \
            --chain-id test-1 \
            --fees 0stake \
            -y 2>/dev/null
        echo "✅ Sultan Token instantiated"
    fi
fi
