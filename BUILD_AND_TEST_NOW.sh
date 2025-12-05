#!/bin/bash

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║     ATTEMPTING TO BUILD YOUR PRODUCTION CODE NOW              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/node

echo "🔨 1. Checking Cargo.toml:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
head -20 Cargo.toml

echo ""
echo "🔨 2. Building the library:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
cargo build --lib 2>&1 | head -20

echo ""
echo "🔨 3. Building sultan_node binary:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
cargo build --bin sultan_node 2>&1 | head -20

echo ""
echo "🔨 4. Checking what binaries we have:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
ls -la target/debug/ 2>/dev/null | grep -E "sultan|rpc" | head -10
ls -la target/release/ 2>/dev/null | grep -E "sultan|rpc" | head -10

echo ""
echo "🚀 5. Running whatever we have:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -f target/debug/sultan_node ]; then
    echo "Running debug sultan_node..."
    timeout 5 ./target/debug/sultan_node 2>&1 | head -20
elif [ -f target/release/sultan_node ]; then
    echo "Running release sultan_node..."
    timeout 5 ./target/release/sultan_node 2>&1 | head -20
elif [ -f target/debug/rpc_server ]; then
    echo "Running RPC server..."
    timeout 5 ./target/debug/rpc_server 2>&1 | head -20
else
    echo "❌ No compiled binaries found"
fi

