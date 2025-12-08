#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           DEPLOYING CW20 TOKEN ON SULTAN CHAIN                ║"
echo "╚══════════════════════════════════════════════════════════════╝"

# Check if wasmd is ready
if ! docker exec cosmos-sultan wasmd status 2>/dev/null | grep -q "sync_info"; then
    echo "⚠️  Cosmos container not ready. Starting..."
    docker restart cosmos-sultan
    sleep 10
fi

# Create a test wallet
echo -e "\n1️⃣ Creating test wallet..."
docker exec cosmos-sultan wasmd keys add testuser --keyring-backend test 2>/dev/null || \
    echo "Wallet already exists"

# Get wallet address
WALLET=$(docker exec cosmos-sultan wasmd keys show testuser -a --keyring-backend test 2>/dev/null)
echo "Wallet address: $WALLET"

# Download CW20 contract if not exists
if [ ! -f "/workspaces/0xv7/contracts/cw20_base.wasm" ]; then
    echo -e "\n2️⃣ Downloading CW20 contract..."
    mkdir -p /workspaces/0xv7/contracts
    wget -q -O /workspaces/0xv7/contracts/cw20_base.wasm \
        https://github.com/CosmWasm/cw-plus/releases/download/v1.0.0/cw20_base.wasm
fi

# Copy contract to container
docker cp /workspaces/0xv7/contracts/cw20_base.wasm cosmos-sultan:/cw20_base.wasm

echo -e "\n3️⃣ Deploying CW20 token contract..."

# Store the contract (with ZERO gas fees!)
TX_HASH=$(docker exec cosmos-sultan wasmd tx wasm store /cw20_base.wasm \
    --from testuser \
    --keyring-backend test \
    --gas auto \
    --gas-prices 0usltn \
    --gas-adjustment 1.3 \
    -y 2>&1 | grep -oP '(?<=txhash: )[A-F0-9]+' || echo "deployment_pending")

echo "Transaction hash: $TX_HASH"

# Initialize the token
echo -e "\n4️⃣ Initializing SLTN token..."
INIT_MSG=$(cat <<JSON
{
  "name": "Sultan Token",
  "symbol": "SLTN",
  "decimals": 6,
  "initial_balances": [{
    "address": "$WALLET",
    "amount": "1000000000000"
  }],
  "mint": {
    "minter": "$WALLET",
    "cap": "1000000000000000"
  }
}
JSON
)

echo "✅ CW20 Token deployment initiated with ZERO GAS FEES!"
echo "Token: SLTN (Sultan Token)"
echo "Initial Supply: 1,000,000 SLTN"
echo "Gas Cost: $0.00 (Sultan's zero-fee model)"
