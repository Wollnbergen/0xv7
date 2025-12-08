#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║     SULTAN CHAIN - COMPLETE COSMOS SDK INTEGRATION            ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# 1. First, check if we have Ignite installed
echo "🔧 Setting up build tools..."
if ! command -v ignite &> /dev/null; then
    echo "Installing Ignite CLI..."
    curl https://get.ignite.com/cli! | bash 2>/dev/null || {
        echo "Installing via go..."
        go install github.com/ignite/cli/ignite@latest
    }
fi

# 2. Create or rebuild the Sultan Cosmos chain
echo ""
echo "🏗️ Building Sultan Cosmos Chain..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ ! -d "/workspaces/0xv7/sultan" ]; then
    echo "Creating new Sultan chain with Ignite..."
    cd /workspaces/0xv7
    ignite scaffold chain sultan --no-module
    cd sultan
    
    # Add our zero-fee configuration
    cat >> app/app.go << 'APPCODE'
// Sultan Chain - Zero Gas Fees Implementation
func (app *App) SetZeroFees() {
    // Override gas fees to always be zero
    app.SetAnteHandler(NewZeroFeeAnteHandler())
}
APPCODE
else
    echo "Sultan chain directory exists, rebuilding..."
    cd /workspaces/0xv7/sultan
fi

# 3. Configure chain parameters
echo ""
echo "⚙️ Configuring Sultan Chain parameters..."

# Update config for zero fees and fast blocks
if [ -f "config.yml" ]; then
    cat > config.yml << 'CONFIG'
version: 1
validation:
  max-block-bytes: 22020096
  max-gas: -1
build:
  proto:
    path: proto
    third_party_paths: ["third_party/proto", "proto_vendor"]
accounts:
  - name: alice
    coins: ["1000000000sltn"]
  - name: bob
    coins: ["500000000sltn"]
  - name: validator1
    coins: ["100000000sltn"]
client:
  openapi:
    path: docs/static/openapi.yml
  vuex:
    path: vue/src/store
faucet:
  name: bob
  coins: ["5000sltn", "100000000stake"]
genesis:
  chain_id: sultan-1
  app_state:
    staking:
      params:
        unbonding_time: "86400s" # 1 day instead of 21
    mint:
      params:
        inflation: "0.080000000000000000" # 4% inflation
    gov:
      voting_params:
        voting_period: "86400s" # 1 day for testing
CONFIG
fi

# 4. Build the chain
echo ""
echo "🔨 Building Sultan binary..."
ignite chain build 2>&1 | grep -E "Built|Error|Success" || {
    echo "Trying manual Go build..."
    go mod tidy
    go build -o build/sultand ./cmd/sultand
}

# 5. Initialize chain if needed
echo ""
echo "🗄️ Initializing chain data..."
if [ ! -d "$HOME/.sultan" ]; then
    ./build/sultand init sultan-node --chain-id sultan-1
    
    # Add genesis accounts
    ./build/sultand add-genesis-account alice 1000000000sltn
    ./build/sultand add-genesis-account validator1 100000000sltn,100000000stake
    
    # Create validator
    ./build/sultand gentx validator1 100000000stake --chain-id sultan-1
    ./build/sultand collect-gentxs
fi

# 6. Start the chain
echo ""
echo "🚀 Starting Sultan Cosmos Chain..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Kill any existing processes
pkill -f sultand 2>/dev/null
pkill -f "ignite chain serve" 2>/dev/null

# Start with Ignite (includes API and faucet)
nohup ignite chain serve --reset-once -v > /tmp/sultan_cosmos.log 2>&1 &

echo "⏳ Waiting for chain to start..."
sleep 5

# 7. Verify chain is running
echo ""
echo "✅ Verifying Sultan Chain Status:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check if chain is responding
if curl -s http://localhost:26657/status > /dev/null 2>&1; then
    echo "✅ Tendermint RPC: ONLINE (port 26657)"
    LATEST_HEIGHT=$(curl -s http://localhost:26657/status | jq -r '.result.sync_info.latest_block_height' 2>/dev/null)
    echo "   Block Height: $LATEST_HEIGHT"
else
    echo "❌ Tendermint RPC not responding"
fi

if curl -s http://localhost:1317/cosmos/bank/v1beta1/supply > /dev/null 2>&1; then
    echo "✅ Cosmos API: ONLINE (port 1317)"
else
    echo "❌ Cosmos API not responding"
fi

if lsof -i:4500 > /dev/null 2>&1; then
    echo "✅ Faucet: ONLINE (port 4500)"
else
    echo "⚠️ Faucet not running"
fi

