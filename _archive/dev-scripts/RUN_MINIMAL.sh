#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║            STARTING MINIMAL ZERO-GAS BLOCKCHAIN               ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Kill any existing instance
pkill -f minimal-chain 2>/dev/null

# Start the blockchain
cd /workspaces/0xv7/minimal-chain
if [ -f "./minimal-chain" ]; then
    ./minimal-chain &
    echo ""
    echo "✅ Blockchain started!"
    echo ""
    echo "Test it:"
    echo "  curl http://localhost:8080/status"
    echo ""
    echo "Dashboard:"
    echo "  $BROWSER http://localhost:3000/minimal-dashboard.html"
else
    echo "❌ Blockchain not built. Run:"
    echo "  ./CREATE_MINIMAL_CHAIN.sh"
fi

