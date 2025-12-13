#!/bin/bash

clear
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║     CHECKING ACTUAL COSMOS SDK IMPLEMENTATION STATUS          ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# 1. Check for Cosmos SDK integration
echo "1️⃣ COSMOS SDK INTEGRATION:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check for sultan-cosmos directory
if [ -d "/workspaces/0xv7/sultan-cosmos" ]; then
    echo "✅ sultan-cosmos directory EXISTS"
    echo "   Contents:"
    ls -la /workspaces/0xv7/sultan-cosmos/ | head -5
fi

# Check for Cosmos wrapper
if [ -f "/workspaces/0xv7/sdk_cosmos_wrapper.rs" ]; then
    echo "✅ Cosmos SDK wrapper EXISTS"
    grep -E "impl SultanCosmosSDK|transfer|create_wallet" /workspaces/0xv7/sdk_cosmos_wrapper.rs | head -5
fi

# Check for sultan directory (Ignite scaffold)
if [ -d "/workspaces/0xv7/sultan" ]; then
    echo "✅ sultan (Ignite) directory EXISTS"
    if [ -f "/workspaces/0xv7/sultan/go.mod" ]; then
        echo "   • go.mod present"
        grep "cosmos-sdk" /workspaces/0xv7/sultan/go.mod | head -2
    fi
    if [ -f "/workspaces/0xv7/sultan/build/sultand" ]; then
        echo "   • ✅ sultand binary BUILT"
    fi
fi

echo ""
echo "2️⃣ CHECKING BUILD SCRIPTS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
ls -la /workspaces/0xv7/*cosmos*.sh 2>/dev/null | awk '{print $9}' | while read file; do
    basename "$file"
done

echo ""
echo "3️⃣ CHECKING ACTUAL IMPLEMENTATION:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check if we have actual Tendermint consensus
if grep -r "tendermint" /workspaces/0xv7 --include="*.rs" --include="*.go" 2>/dev/null | head -1 > /dev/null; then
    echo "✅ Tendermint references found"
fi

# Check app.go
if [ -f "/workspaces/0xv7/sultan-cosmos/app.go" ]; then
    echo "✅ app.go EXISTS (Cosmos app definition)"
    grep -E "SultanApp|NewSultanApp" /workspaces/0xv7/sultan-cosmos/app.go | head -3
fi

# Check for actual modules
if [ -d "/workspaces/0xv7/sultan/x" ]; then
    echo "✅ Custom modules directory exists:"
    ls -la /workspaces/0xv7/sultan/x/ 2>/dev/null | grep "^d" | awk '{print "   • " $NF}'
fi

echo ""
echo "4️⃣ WHAT'S ACTUALLY RUNNING:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
ps aux | grep -E "sultand|cosmos" | grep -v grep | head -3

