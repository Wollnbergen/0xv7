#!/bin/bash
echo "🌐 Starting Sultan RPC Server..."
cd /workspaces/0xv7/node
if [ -f target/release/rpc_server ]; then
    ./target/release/rpc_server
else
    echo "Building RPC server first..."
    cargo build --release --bin rpc_server
    ./target/release/rpc_server
fi
