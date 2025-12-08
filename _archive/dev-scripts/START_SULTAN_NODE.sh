#!/bin/bash
echo "🚀 Starting Sultan Chain Node..."
cd /workspaces/0xv7/node
if [ -f target/release/sultan_node ]; then
    ./target/release/sultan_node
else
    echo "Building Sultan node first..."
    cargo build --release --bin sultan_node
    ./target/release/sultan_node
fi
