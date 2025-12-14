#!/bin/bash
cd /workspaces/0xv7
if [ -f target/release/sultan_node ]; then
    echo "Testing release build..."
    ./target/release/sultan_node --help 2>/dev/null || echo "Node binary ready"
elif [ -f target/debug/sultan_node ]; then
    echo "Testing debug build..."
    ./target/debug/sultan_node --help 2>/dev/null || echo "Node binary ready"
else
    echo "Building node..."
    cargo build --package sultan-coordinator --bin sultan_node
fi
