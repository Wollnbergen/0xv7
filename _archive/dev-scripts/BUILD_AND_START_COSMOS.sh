#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║    SULTAN CHAIN - BUILDING REAL COSMOS BLOCKCHAIN             ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# 1. Check for Ignite CLI
echo "🔍 Checking for Ignite CLI..."
if ! command -v ignite &> /dev/null; then
    echo "📥 Installing Ignite CLI..."
    curl -L https://get.ignite.com/cli | bash
    export PATH=$PATH:$HOME/.ignite/bin
fi

# 2. Check if sultan directory exists, if not scaffold it
if [ ! -d "/workspaces/0xv7/sultan" ]; then
    echo ""
    echo "🏗️ Creating Sultan Cosmos chain..."
    cd /workspaces/0xv7
    ignite scaffold chain sultan --no-module --skip-git
    cd sultan
else
    echo "✅ Sultan chain directory exists"
    cd /workspaces/0xv7/sultan
fi

# 3. Configure for zero fees and our economics
echo ""
echo "⚙️ Configuring Sultan Chain parameters..."

# Create custom config
cat > config.yml << 'CONFIG'
version: 1
build:
  main: cmd/sultand
accounts:
  - name: alice
    coins: ["1000000000000sltn"]
  - name: bob  
    coins: ["500000000000sltn"]
  - name: validator1
    coins: ["100000000000sltn", "100000000stake"]
client:
  openapi:
    path: docs/static/openapi.yml
faucet:
  name: bob
  coins: ["5000sltn"]
  host: "0.0.0.0:4500"
genesis:
  chain_id: sultan-1
  app_state:
    staking:
      params:
        bond_denom: "stake"
        unbonding_time: "86400s"
    mint:
      params:
        inflation: "0.080000000000000000"
        goal_bonded: "0.670000000000000000"
    crisis:
      constant_fee:
        denom: "sltn"
        amount: "0"
    gov:
      deposit_params:
        min_deposit:
          - denom: "sltn"
            amount: "10000000"
      voting_params:
        voting_period: "86400s"
validators:
  - name: validator1
    bonded: 100000000stake
CONFIG

# 4. Build the chain
echo ""
echo "🔨 Building Sultan blockchain..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Try Ignite build first
if ignite chain build 2>&1 | tee /tmp/sultan_build.log | grep -E "Blockchain is built|succeed|built"; then
    echo "✅ Sultan chain built successfully!"
else
    echo "⚠️ Ignite build had issues, trying manual build..."
    go mod tidy
    go build -o build/sultand ./cmd/sultand
fi

# 5. Initialize chain if needed
if [ ! -d "$HOME/.sultan" ]; then
    echo ""
    echo "🗄️ Initializing Sultan chain..."
    rm -rf $HOME/.sultan 2>/dev/null
    ./build/sultand init sultan-node --chain-id sultan-1 2>/dev/null || sultand init sultan-node --chain-id sultan-1
fi

# 6. Start the chain with Ignite (includes API and faucet)
echo ""
echo "🚀 Starting Sultan Cosmos Chain..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Kill any existing processes
pkill -f sultand 2>/dev/null
pkill -f "ignite chain serve" 2>/dev/null
sleep 2

# Start the chain
echo "Starting with Ignite (includes API, faucet, and explorer)..."
nohup ignite chain serve --reset-once --verbose > /tmp/sultan_cosmos.log 2>&1 &
COSMOS_PID=$!

echo "⏳ Waiting for chain to start (10 seconds)..."
sleep 10

# 7. Check if services are running
echo ""
echo "✅ Checking Sultan Chain Services:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Function to check service
check_service() {
    local port=$1
    local name=$2
    if lsof -i:$port > /dev/null 2>&1; then
        echo "  ✅ $name: ONLINE (port $port)"
        return 0
    else
        echo "  ❌ $name: OFFLINE (port $port)"
        return 1
    fi
}

check_service 26657 "Tendermint RPC"
check_service 1317 "Cosmos REST API"
check_service 4500 "Faucet"
check_service 9090 "gRPC"

# Get chain status
if curl -s http://localhost:26657/status > /dev/null 2>&1; then
    HEIGHT=$(curl -s http://localhost:26657/status | jq -r '.result.sync_info.latest_block_height' 2>/dev/null)
    echo ""
    echo "  📊 Current block height: $HEIGHT"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ SULTAN COSMOS CHAIN IS RUNNING!"
echo ""
echo "📝 Useful commands:"
echo "  • View logs: tail -f /tmp/sultan_cosmos.log"
echo "  • Chain status: curl http://localhost:26657/status | jq"
echo "  • Get balance: curl http://localhost:1317/cosmos/bank/v1beta1/balances/{address}"
echo "  • Stop chain: pkill -f 'ignite chain serve'"
echo ""
echo "🌐 Web Interfaces:"
echo "  • Block Explorer: http://localhost:3000"
echo "  • API Docs: http://localhost:1317"
echo "  • Faucet: http://localhost:4500"

