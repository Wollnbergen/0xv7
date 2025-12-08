#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║              FIXING CARGO.TOML CORRUPTION                     ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7

# 1. Show the problematic line
echo "🔍 Step 1: Finding the corrupted line..."
sed -n '65,67p' node/Cargo.toml

# 2. Remove the corrupted line
echo ""
echo "🔧 Step 2: Removing corrupted line..."
sed -i '66d' node/Cargo.toml

# 3. Clean up any other libp2p remnants
echo ""
echo "🧹 Step 3: Cleaning up libp2p remnants..."
sed -i '/^\s*"tcp",/d' node/Cargo.toml
sed -i '/^\s*"dns",/d' node/Cargo.toml
sed -i '/^\s*"noise",/d' node/Cargo.toml
sed -i '/^\s*"yamux",/d' node/Cargo.toml
sed -i '/^\s*"async-std",/d' node/Cargo.toml
sed -i '/libp2p/d' node/Cargo.toml

echo "✅ Cleaned Cargo.toml"

# 4. Verify Cargo.toml is valid
echo ""
echo "🔍 Step 4: Verifying Cargo.toml..."
if cargo metadata --no-deps --format-version 1 > /dev/null 2>&1; then
    echo "✅ Cargo.toml is valid!"
else
    echo "⚠️ Cargo.toml still has issues. Checking..."
    cargo metadata --no-deps 2>&1 | head -5
fi

# 5. Build the RPC server
echo ""
echo "🔨 Step 5: Building RPC server..."
cargo build -p sultan-coordinator --bin rpc_server 2>&1 | tail -3

# 6. Build wallet CLI
echo ""
echo "🔨 Step 6: Building wallet CLI..."
cargo build -p sultan-coordinator --bin wallet_cli 2>&1 | tail -3

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                    CARGO.TOML FIXED!                          ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "📋 Next steps:"
echo "  1. Run RPC server: cargo run -p sultan-coordinator --bin rpc_server"
echo "  2. Test wallet CLI: cargo run -p sultan-coordinator --bin wallet_cli"
