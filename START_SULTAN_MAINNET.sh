#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         STARTING SULTAN CHAIN MAINNET NODE                    ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/node

# Check compilation status
if [ -f target/release/sultan_node ]; then
    echo "✅ Node binary found!"
    
    # Set up environment
    export RUST_LOG=info
    
    echo ""
    echo "🚀 Starting Sultan Chain Node..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    ./target/release/sultan_node
else
    echo "❌ Node not compiled. Building now..."
    cargo build --release --bin sultan_node
    
    if [ -f target/release/sultan_node ]; then
        echo "✅ Build successful! Starting node..."
        ./target/release/sultan_node
    else
        echo "❌ Build failed. Check errors above."
    fi
fi

