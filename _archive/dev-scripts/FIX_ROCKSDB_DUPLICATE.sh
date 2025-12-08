#!/bin/bash

echo "🔧 Fixing duplicate RocksDB dependency..."

# Remove duplicate rocksdb entries from Cargo.toml
cd /workspaces/0xv7/node
sed -i '/^rocksdb = "0.21"$/d' Cargo.toml
sed -i '/^libp2p = /d' Cargo.toml

# Add dependencies properly in the [dependencies] section
cat >> Cargo.toml << 'TOML'

[dependencies.rocksdb]
version = "0.21"

[dependencies.libp2p]
version = "0.53"
features = ["tcp", "noise", "yamux", "gossipsub", "identify", "kad"]
TOML

echo "✅ Dependencies fixed!"

# Test compilation
echo "🔨 Testing compilation..."
if cargo check 2>/dev/null; then
    echo "✅ Compilation successful!"
else
    echo "⚠️  Still has issues, checking details..."
    cargo check
fi

