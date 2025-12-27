#!/bin/bash
set -e
echo "=== Starting sultan-node build ==="
cd /workspaces/0xv7
cargo build --release --bin sultan-node
echo "=== Build complete ==="
ls -la /tmp/cargo-target/release/sultan-node
cp /tmp/cargo-target/release/sultan-node /workspaces/0xv7/sultan-node.new
echo "Binary copied to /workspaces/0xv7/sultan-node.new"
