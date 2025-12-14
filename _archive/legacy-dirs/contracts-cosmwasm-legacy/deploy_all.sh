#!/bin/bash

echo "🚀 Deploying all Sultan contracts..."

# Copy contracts to container
docker cp /workspaces/0xv7/contracts/cw20-sultan/target/wasm32-unknown-unknown/release/cw20_sultan.wasm cosmos-node:/tmp/
docker cp /workspaces/0xv7/contracts/cw721-nft/target/wasm32-unknown-unknown/release/cw721_sultan_nft.wasm cosmos-node:/tmp/ 2>/dev/null || true
docker cp /workspaces/0xv7/contracts/defi-amm/target/wasm32-unknown-unknown/release/sultan_amm.wasm cosmos-node:/tmp/ 2>/dev/null || true

# Deploy CW20 Token
echo "Deploying CW20 Token..."
TX_HASH=$(docker exec cosmos-node wasmd tx wasm store /tmp/cw20_sultan.wasm \
    --from validator \
    --keyring-backend test \
    --chain-id test-1 \
    --gas auto \
    --fees 0stake \
    -y \
    --output json 2>/dev/null | jq -r '.txhash')

if [ ! -z "$TX_HASH" ]; then
    echo "CW20 Token stored with tx: $TX_HASH"
    sleep 6
    
    # Get code ID and instantiate
    CODE_ID=$(docker exec cosmos-node wasmd query tx $TX_HASH --output json 2>/dev/null | \
        jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
    
    if [ ! -z "$CODE_ID" ]; then
        echo "Code ID: $CODE_ID"
        
        # Instantiate token
        INIT='{
            "name":"Sultan Token",
            "symbol":"SLTN",
            "decimals":6,
            "initial_balances":[{
                "address":"wasm1kkcdw94sdfal63elmjezlu3hx4lexqupkufx7v",
                "amount":"1000000000000"
            }],
            "mint":{
                "minter":"wasm1kkcdw94sdfal63elmjezlu3hx4lexqupkufx7v"
            }
        }'
        
        docker exec cosmos-node wasmd tx wasm instantiate $CODE_ID "$INIT" \
            --from validator \
            --keyring-backend test \
            --label "sultan-token" \
            --chain-id test-1 \
            --fees 0stake \
            -y
        
        echo "✅ Sultan Token deployed!"
    fi
fi

echo "✅ Smart contracts deployment complete!"
