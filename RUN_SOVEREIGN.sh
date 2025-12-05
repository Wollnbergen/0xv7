#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║            STARTING SOVEREIGN BLOCKCHAIN                      ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Check if binary exists
if [ ! -f "$HOME/go/bin/sovereignd" ]; then
    echo "❌ sovereignd binary not found!"
    echo "   Run: ./BUILD_SOVEREIGN_CHAIN.sh first"
    exit 1
fi

echo "🚀 Starting Sovereign Chain..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Chain ID: sovereign-1"
echo "RPC: http://localhost:26657"
echo "API: http://localhost:1317"
echo "Gas Fees: ZERO! 🎉"
echo ""
echo "Press Ctrl+C to stop"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Start the chain
$HOME/go/bin/sovereignd start --home $HOME/.sovereign

