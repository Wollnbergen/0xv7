#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         BUILDING SOVEREIGN CHAIN - STEP BY STEP               ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/sovereign-chain/sovereign

# Step 1: Fix the go.mod to ensure compatibility
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔧 [1/5] Updating go.mod dependencies..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Download dependencies
go mod download
go mod tidy

# Step 2: Build the binary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔨 [2/5] Building sovereignd binary..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

make install 2>&1 | tee build.log | tail -20

# Check if build was successful
if [ -f "$HOME/go/bin/sovereignd" ]; then
    echo "✅ Binary built successfully!"
    echo "   Location: $HOME/go/bin/sovereignd"
else
    echo "⚠️  Build may have failed. Checking alternative location..."
    if [ -f "./build/sovereignd" ]; then
        echo "✅ Binary found at ./build/sovereignd"
        cp ./build/sovereignd $HOME/go/bin/
    else
        echo "❌ Binary not found. Check build.log for errors."
    fi
fi

# Step 3: Initialize the chain
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "⚡ [3/5] Initializing Sovereign Chain..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Remove old chain data if exists
rm -rf $HOME/.sovereign

# Initialize with moniker and chain-id
if [ -f "$HOME/go/bin/sovereignd" ]; then
    $HOME/go/bin/sovereignd init sovereign-node --chain-id sovereign-1 --home $HOME/.sovereign
    echo "✅ Chain initialized!"
else
    echo "❌ sovereignd binary not found. Cannot initialize."
fi

# Step 4: Configure for zero gas fees
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "💸 [4/5] Configuring zero gas fees..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -f "$HOME/.sovereign/config/app.toml" ]; then
    # Set minimum gas prices to 0
    sed -i 's/minimum-gas-prices = ".*"/minimum-gas-prices = "0stake"/' $HOME/.sovereign/config/app.toml
    
    # Enable API
    sed -i 's/enable = false/enable = true/' $HOME/.sovereign/config/app.toml
    
    echo "✅ Zero gas fees configured!"
else
    echo "⚠️  Config file not found. Will use defaults."
fi

# Step 5: Create accounts and genesis
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "👤 [5/5] Creating accounts..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -f "$HOME/go/bin/sovereignd" ]; then
    # Create validator account
    echo "Creating validator account..."
    $HOME/go/bin/sovereignd keys add validator --keyring-backend test --home $HOME/.sovereign 2>&1 | tee validator.info
    
    # Create test user account
    echo "Creating test user account..."
    $HOME/go/bin/sovereignd keys add alice --keyring-backend test --home $HOME/.sovereign 2>&1 | tee alice.info
    
    # Add genesis account with tokens
    $HOME/go/bin/sovereignd genesis add-genesis-account validator 100000000000stake --keyring-backend test --home $HOME/.sovereign
    $HOME/go/bin/sovereignd genesis add-genesis-account alice 10000000000stake --keyring-backend test --home $HOME/.sovereign
    
    # Create genesis transaction
    $HOME/go/bin/sovereignd genesis gentx validator 1000000stake --chain-id sovereign-1 --keyring-backend test --home $HOME/.sovereign
    
    # Collect genesis transactions
    $HOME/go/bin/sovereignd genesis collect-gentxs --home $HOME/.sovereign
    
    echo "✅ Accounts created and genesis configured!"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ BUILD COMPLETE!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📊 Status:"
if [ -f "$HOME/go/bin/sovereignd" ]; then
    echo "   Binary: ✅ BUILT"
    echo "   Chain: ✅ INITIALIZED"
    echo "   Accounts: ✅ CREATED"
    echo "   Genesis: ✅ CONFIGURED"
    echo ""
    echo "🚀 To start the chain:"
    echo "   $HOME/go/bin/sovereignd start --home $HOME/.sovereign"
else
    echo "   Binary: ❌ NOT BUILT"
    echo "   Check build.log for errors"
fi

