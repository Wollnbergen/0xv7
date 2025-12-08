#!/bin/bash

echo ""
echo "🧪 TESTING WHAT ACTUALLY WORKS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 1. Check if sultand binary exists and works
if [ -f "/workspaces/0xv7/sultan/build/sultand" ]; then
    echo "Testing sultand binary:"
    /workspaces/0xv7/sultan/build/sultand version 2>/dev/null || echo "   ❌ Binary not working"
else
    echo "❌ No sultand binary found"
fi

# 2. Check if Cosmos chain is initialized
if [ -d "$HOME/.sultan" ]; then
    echo "✅ Sultan chain data directory exists"
else
    echo "⚠️ Chain not initialized"
fi

# 3. Try to start the chain
echo ""
echo "Attempting to start Sultan Cosmos chain..."
cd /workspaces/0xv7/sultan 2>/dev/null || cd /workspaces/0xv7

if [ -f "sultan/build/sultand" ]; then
    echo "Starting sultan daemon..."
    timeout 5 ./sultan/build/sultand start 2>&1 | grep -E "starting|listening|error" | head -5
elif command -v ignite &> /dev/null; then
    echo "Using Ignite to build and serve..."
    cd sultan 2>/dev/null && timeout 10 ignite chain serve --reset-once 2>&1 | grep -E "Blockchain is running|error" | head -5
fi

# 4. Check our custom RPC
echo ""
echo "Checking Sultan RPC (port 3030):"
curl -s http://localhost:3030 -X POST -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"chain_getInfo","params":[],"id":1}' 2>/dev/null | jq . 2>/dev/null || echo "   ⚠️ RPC not responding"

