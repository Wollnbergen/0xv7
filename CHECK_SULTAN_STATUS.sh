#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          SULTAN CHAIN - COMPREHENSIVE STATUS CHECK            ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Function to check if a service is running
check_service() {
    local name=$1
    local port=$2
    if lsof -i:$port > /dev/null 2>&1; then
        echo "✅ $name: RUNNING on port $port"
        return 0
    else
        echo "❌ $name: NOT RUNNING on port $port"
        return 1
    fi
}

echo "🔍 Checking Running Services..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
check_service "Web Dashboard" 3000
check_service "API Server" 1317
check_service "RPC Server" 8545

echo ""
echo "📂 Checking Project Structure..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Count files in major directories
for dir in node sultan-sdk sultan-interop sultan-cosmos sultan-chain-mainnet; do
    if [ -d "/workspaces/0xv7/$dir" ]; then
        file_count=$(find /workspaces/0xv7/$dir -type f 2>/dev/null | wc -l)
        echo "✅ /$dir: $file_count files"
    else
        echo "❌ /$dir: Directory not found"
    fi
done

echo ""
echo "🔧 Checking Compilation Status..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check if binaries exist
if [ -f "/tmp/sultan-blockchain-standalone/target/release/sultan-blockchain" ]; then
    echo "✅ Standalone blockchain binary: EXISTS"
else
    echo "❌ Standalone blockchain binary: NOT FOUND"
fi

if [ -d "/workspaces/0xv7/node/target" ]; then
    echo "✅ Node target directory: EXISTS"
    ls -la /workspaces/0xv7/node/target/*/sultan* 2>/dev/null | head -5
else
    echo "❌ Node target directory: NOT FOUND"
fi

echo ""
echo "🌐 Web Interface Access..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Local: http://localhost:3000"
echo "GitHub Codespace: https://orange-telegram-pj6qgwgv59jjfrj9j-3000.app.github.dev"

echo ""
echo "📊 Quick Metrics..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "• Gas Fees: $0.00 (hardcoded)"
echo "• Target TPS: 1,230,992"
echo "• Staking APY: 26.67%"
echo "• Completion: ~70%"

