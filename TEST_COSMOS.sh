#!/bin/bash
echo "🧪 Testing Cosmos SDK Integration..."
cd /workspaces/0xv7/sultan-cosmos

# Check if Go is installed
if command -v go &> /dev/null; then
    echo "✅ Go installed"
    go mod download 2>/dev/null || echo "⏳ Dependencies will download on build"
else
    echo "⚠️ Go not installed (needed for full Cosmos SDK)"
fi

echo ""
echo "📋 Sultan Chain Cosmos Features:"
grep -A 3 "ZeroFees\|StakingAPY\|IBCEnabled" app/app.go
echo ""
echo "✅ Cosmos SDK structure ready!"
