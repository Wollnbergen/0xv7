#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SULTAN CHAIN - ORGANIZED PROJECT STATUS               ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

if [ -d "/workspaces/0xv7/sultan-chain-mainnet" ]; then
    echo "✅ Project structure exists"
    echo ""
    echo "📁 Directory tree:"
    tree -L 2 /workspaces/0xv7/sultan-chain-mainnet 2>/dev/null || ls -la /workspaces/0xv7/sultan-chain-mainnet/
else
    echo "❌ Project not organized yet"
    echo "   Run: ./ORGANIZE_SULTAN_PROJECT.sh"
fi

echo ""
echo "🌐 Current testnet status:"
if curl -s http://localhost:3030 > /dev/null 2>&1; then
    echo "✅ API is running at http://localhost:3030"
else
    echo "❌ API not running"
fi
