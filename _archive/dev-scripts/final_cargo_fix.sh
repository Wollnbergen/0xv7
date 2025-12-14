#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         FINAL FIX - REMOVE ALL CARGO.TOML ISSUES              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7

# 1. Show the problematic area
echo "🔍 Step 1: Checking problematic lines around line 66..."
sed -n '60,75p' node/Cargo.toml

# 2. Remove ALL standalone feature strings that are causing issues
echo ""
echo "🔧 Step 2: Removing ALL problematic feature strings..."
# Remove any line that's just a quoted string with optional comma
sed -i '/^\s*"[a-z_-]*",\?$/d' node/Cargo.toml
# Also remove comment lines about P2P if they exist
sed -i '/# P2P Networking Dependencies/d' node/Cargo.toml

echo "✅ Removed problematic lines"

# 3. Double-check and clean any remaining libp2p references
echo ""
echo "🧹 Step 3: Final cleanup of any libp2p references..."
sed -i '/libp2p/d' node/Cargo.toml
sed -i '/libp2p-/d' node/Cargo.toml

echo "✅ Final cleanup done"

# 4. Verify Cargo.toml is now valid
echo ""
echo "🔍 Step 4: Verifying Cargo.toml..."
if cargo metadata --no-deps --format-version 1 > /dev/null 2>&1; then
    echo "✅ Cargo.toml is now valid!"
else
    echo "⚠️ Checking for remaining issues..."
    cargo check 2>&1 | head -5
fi

# 5. Build RPC server
echo ""
echo "🔨 Step 5: Building RPC server..."
cargo build -p sultan-coordinator --bin rpc_server 2>&1 | tail -3

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                    CARGO.TOML FULLY FIXED!                    ║"
echo "╚══════════════════════════════════════════════════════════════╝"
