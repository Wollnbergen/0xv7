#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         COMPLETE FIX - REMOVE STRAY BRACKETS                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7

# 1. Show the problem area
echo "🔍 Step 1: Checking line 65 area..."
sed -n '63,67p' node/Cargo.toml

# 2. Remove the stray ] }
echo ""
echo "🔧 Step 2: Removing stray bracket line..."
sed -i '/^\] }$/d' node/Cargo.toml

echo "✅ Removed stray bracket"

# 3. Show the fixed area
echo ""
echo "📝 Step 3: Verifying fix..."
sed -n '63,67p' node/Cargo.toml

# 4. Test if Cargo.toml is now valid
echo ""
echo "🔍 Step 4: Testing Cargo.toml validity..."
if cargo metadata --no-deps --format-version 1 > /dev/null 2>&1; then
    echo "✅ Cargo.toml is now VALID!"
    VALID=true
else
    echo "⚠️ Checking for any remaining issues..."
    cargo check 2>&1 | grep "error" | head -3
    VALID=false
fi

# 5. If valid, try building
if [ "$VALID" = true ]; then
    echo ""
    echo "🔨 Step 5: Building RPC server..."
    if cargo build -p sultan-coordinator --bin rpc_server 2>&1 | tail -1 | grep -q "Finished"; then
        echo "✅ RPC Server built successfully!"
        echo ""
        echo "🚀 Ready to run: cargo run -p sultan-coordinator --bin rpc_server"
    else
        echo "⚠️ Build has some issues. Checking..."
        cargo build -p sultan-coordinator --bin rpc_server 2>&1 | grep "error" | head -5
    fi
fi

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                       FIX COMPLETE                            ║"
echo "╚══════════════════════════════════════════════════════════════╝"
