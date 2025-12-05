#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           STARTING SULTAN CHAIN NODE                          ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/node

# Check if binary exists
if [ ! -f target/release/sultan_node ]; then
    echo "⚠️ Node binary not found. Building..."
    cargo build --release --bin sultan_node 2>&1 | tail -5
fi

if [ -f target/release/sultan_node ]; then
    echo "🚀 Starting Sultan Chain Node..."
    echo ""
    
    # Set environment
    export RUST_LOG=info
    
    # Run the node
    ./target/release/sultan_node
else
    echo "❌ Failed to build node. Check errors above."
fi

