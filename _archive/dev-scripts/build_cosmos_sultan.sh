#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           BUILDING SULTAN WITH COSMOS SDK                     ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/sultan

# 1. Initialize go modules properly
echo "🔧 Initializing Go modules..."
go mod tidy 2>&1 | head -5

# 2. Build the chain
echo ""
echo "🔨 Building Sultan chain..."
make build 2>&1 | tail -10

# 3. If make build fails, try direct go build
if [ ! -f "build/sultand" ]; then
    echo ""
    echo "🔨 Trying direct go build..."
    go build -o build/sultand ./cmd/sultand 2>&1 | tail -10
fi

# 4. Check if build succeeded
if [ -f "build/sultand" ]; then
    echo ""
    echo "✅ Sultan chain built successfully!"
    
    # Initialize the chain
    echo ""
    echo "🚀 Initializing Sultan testnet..."
    ./build/sultand init sultan-test --chain-id sultan-1 2>&1 | tail -5
else
    echo "❌ Build failed. Checking for missing dependencies..."
    go list -m all | grep -E "cosmos-sdk|tendermint" | head -5
fi

echo ""
echo "📊 Build Status:"
ls -la build/ 2>/dev/null || echo "No build directory yet"
