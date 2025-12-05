#!/bin/bash
# Sultan L1 Blockchain - End-to-End Test Script
# Production-grade testing of complete stack

set -e

echo "=================================================="
echo "Sultan L1 Blockchain - E2E Production Test"
echo "=================================================="
echo ""

# Configuration
export CHAIN_ID="sultan-1"
export SULTAND_HOME="$HOME/.sultand"
export LD_LIBRARY_PATH="/tmp/cargo-target/release:/workspaces/0xv7/target/release:$LD_LIBRARY_PATH"

echo "Step 1: Verify FFI Library"
echo "----------------------------"
# Check both possible library locations
if [ -f "/tmp/cargo-target/release/libsultan_cosmos_bridge.so" ]; then
    ls -lh /tmp/cargo-target/release/libsultan_cosmos_bridge.* 2>/dev/null
    echo "✅ FFI library found at /tmp/cargo-target/release/"
elif [ -f "/workspaces/0xv7/target/release/libsultan_cosmos_bridge.so" ]; then
    ls -lh /workspaces/0xv7/target/release/libsultan_cosmos_bridge.* 2>/dev/null
    echo "✅ FFI library found at /workspaces/0xv7/target/release/"
else
    echo "Building FFI library..."
    cd /workspaces/0xv7/sultan-cosmos-bridge
    cargo build --release
    if [ -f "/tmp/cargo-target/release/libsultan_cosmos_bridge.so" ]; then
        ls -lh /tmp/cargo-target/release/libsultan_cosmos_bridge.* 2>/dev/null
        echo "✅ FFI library built at /tmp/cargo-target/release/"
    elif [ -f "/workspaces/0xv7/target/release/libsultan_cosmos_bridge.so" ]; then
        ls -lh /workspaces/0xv7/target/release/libsultan_cosmos_bridge.* 2>/dev/null
        echo "✅ FFI library built at /workspaces/0xv7/target/release/"
    else
        echo "ERROR: FFI library not found!"
        exit 1
    fi
fi
echo ""

echo "Step 2: Initialize Blockchain"
echo "----------------------------"
cd /workspaces/0xv7/sultand

# Clean previous data
rm -rf "$SULTAND_HOME"

# Initialize
./sultand init testnode --chain-id $CHAIN_ID --home "$SULTAND_HOME"
echo "✅ Chain initialized"
echo ""

echo "Step 3: Create Test Accounts"
echo "----------------------------"
# Create alice
echo "Creating alice..."
./sultand keys add alice --home "$SULTAND_HOME" --keyring-backend test 2>&1 | grep -E "(address|mnemonic)" || true

# Create bob  
echo "Creating bob..."
./sultand keys add bob --home "$SULTAND_HOME" --keyring-backend test 2>&1 | grep -E "(address|mnemonic)" || true

# Get addresses
ALICE=$(./sultand keys show alice -a --home "$SULTAND_HOME" --keyring-backend test)
BOB=$(./sultand keys show bob -a --home "$SULTAND_HOME" --keyring-backend test)

echo "Alice address: $ALICE"
echo "Bob address: $BOB"
echo "✅ Accounts created"
echo ""

echo "Step 4: Configure Genesis"
echo "----------------------------"
# Add genesis accounts
./sultand add-genesis-account $ALICE 1000000000sultan --home "$SULTAND_HOME"
./sultand add-genesis-account $BOB 500000000sultan --home "$SULTAND_HOME"

echo "✅ Genesis accounts added"
echo ""

echo "Step 5: Validate Genesis"
echo "----------------------------"
./sultand validate --home "$SULTAND_HOME"
echo "✅ Genesis valid"
echo ""

echo "=================================================="
echo "Setup Complete! Ready to start Sultan node"
echo "=================================================="
echo ""
echo "To start the node:"
echo "  cd /workspaces/0xv7/sultand"
echo "  export LD_LIBRARY_PATH=/tmp/cargo-target/release:\$LD_LIBRARY_PATH"
echo "  ./sultand start --home ~/.sultand"
echo ""
echo "To query balances:"
echo "  ./sultand query bank balances $ALICE"
echo "  ./sultand query bank balances $BOB"
echo ""

echo "=================================================="
echo "Setup Complete! Ready to start node"
echo "=================================================="
echo ""
echo "To start the node:"
echo "  cd /workspaces/0xv7/sultand"
echo "  export LD_LIBRARY_PATH=/workspaces/0xv7/target/release:\$LD_LIBRARY_PATH"
echo "  ./sultand start --home $SULTAND_HOME"
echo ""
echo "In another terminal, test transactions:"
echo "  export LD_LIBRARY_PATH=/workspaces/0xv7/target/release:\$LD_LIBRARY_PATH"
echo "  cd /workspaces/0xv7/sultand"
echo "  ./sultand tx send $ALICE $BOB 1000sultan \\"
echo "    --from alice --chain-id $CHAIN_ID \\"
echo "    --home $SULTAND_HOME --keyring-backend test --yes"
echo ""
echo "Query balance (via FFI to Sultan core):"
echo "  ./sultand query balance $BOB --home $SULTAND_HOME"
echo ""
echo "Expected flow:"
echo "  User → Cosmos SDK → Go Keeper → CGo Bridge → FFI → Sultan Core (Rust)"
echo "  Sultan Core processes TX → FFI → Go → Cosmos SDK → User"
echo ""
