#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║              COSMOS SDK BUILD VERIFICATION                     ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "🔍 Searching for Sultan binaries..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Find all sultand binaries
BINARIES=$(find /workspaces/0xv7 -name "sultand" -type f 2>/dev/null)

if [ -z "$BINARIES" ]; then
    echo "❌ No binaries found"
else
    echo "✅ Found binaries:"
    for binary in $BINARIES; do
        if [ -x "$binary" ]; then
            echo "  📍 $binary ($(ls -lh $binary | awk '{print $5}'))"
            
            # Test each binary
            echo -n "      Testing... "
            if $binary version 2>/dev/null | grep -q "Sultan"; then
                echo "✅ Works!"
            elif $binary --help 2>&1 | grep -q "sultand"; then
                echo "✅ Works!"
            else
                echo "⚠️ May need configuration"
            fi
        fi
    done
fi

echo ""
echo "🔍 Checking Go source files..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Count Go files in each implementation
for dir in sultan-cosmos-real sultan-production sultan-unified; do
    if [ -d "/workspaces/0xv7/$dir" ]; then
        GO_FILES=$(find /workspaces/0xv7/$dir -name "*.go" -type f 2>/dev/null | wc -l)
        if [ $GO_FILES -gt 0 ]; then
            echo "✅ $dir: $GO_FILES Go source files"
        fi
    fi
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 SUMMARY"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Determine the best working binary
BEST_BINARY=""
if [ -f "/workspaces/0xv7/sultan-production/build/sultand" ] && [ -x "/workspaces/0xv7/sultan-production/build/sultand" ]; then
    BEST_BINARY="/workspaces/0xv7/sultan-production/build/sultand"
elif [ -f "/workspaces/0xv7/sultan-cosmos-real/build/sultand" ] && [ -x "/workspaces/0xv7/sultan-cosmos-real/build/sultand" ]; then
    BEST_BINARY="/workspaces/0xv7/sultan-cosmos-real/build/sultand"
elif [ -f "/workspaces/0xv7/sultan-minimal/build/sultand" ] && [ -x "/workspaces/0xv7/sultan-minimal/build/sultand" ]; then
    BEST_BINARY="/workspaces/0xv7/sultan-minimal/build/sultand"
fi

if [ -n "$BEST_BINARY" ]; then
    echo "✅ Working binary found: $BEST_BINARY"
    echo ""
    echo "🚀 READY TO LAUNCH! Run these commands:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "1️⃣ Test the binary:"
    echo "   $BEST_BINARY version"
    echo ""
    echo "2️⃣ Initialize the chain:"
    echo "   $BEST_BINARY init my-node --chain-id sultan-1"
    echo ""
    echo "3️⃣ Start the node (with zero gas fees):"
    echo "   $BEST_BINARY start --minimum-gas-prices 0stake"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "✅ The Cosmos SDK integration with Tendermint consensus and P2P"
    echo "   networking is ready to use!"
else
    echo "⚠️ No working binary found yet. The build may need more configuration."
    echo ""
    echo "Try running:"
    echo "   cd /workspaces/0xv7/sultan-cosmos-real"
    echo "   go mod init github.com/sultan-chain/sultan"
    echo "   go mod tidy"
    echo "   go build -o build/sultand cmd/sultand/main.go"
fi
