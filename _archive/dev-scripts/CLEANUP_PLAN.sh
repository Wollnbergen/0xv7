#!/bin/bash

echo "🧹 SULTAN CHAIN CLEANUP SCRIPT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "This script will organize your project into a single, coherent structure."
echo ""
echo "⚠️  WARNING: This will reorganize your entire project!"
echo "Press Ctrl+C to cancel, or Enter to continue..."
read

# 1. Create new unified structure
echo "📁 Creating unified project structure..."
mkdir -p /workspaces/0xv7/sultan-unified
mkdir -p /workspaces/0xv7/sultan-unified/core
mkdir -p /workspaces/0xv7/sultan-unified/cosmos-modules
mkdir -p /workspaces/0xv7/sultan-unified/legacy
mkdir -p /workspaces/0xv7/sultan-unified/docs
mkdir -p /workspaces/0xv7/sultan-unified/scripts
mkdir -p /workspaces/0xv7/sultan-unified/tests

# 2. Consolidate Rust implementation (keeping the best parts)
echo "🦀 Consolidating Rust components..."
if [ -d "/workspaces/0xv7/sultan-chain-mainnet/core/src" ]; then
    cp -r /workspaces/0xv7/sultan-chain-mainnet/core/* /workspaces/0xv7/sultan-unified/core/
    echo "  ✅ Copied sultan-chain-mainnet/core"
fi

if [ -d "/workspaces/0xv7/node/src" ]; then
    # Merge node/src with core/src (avoiding duplicates)
    for file in /workspaces/0xv7/node/src/*.rs; do
        filename=$(basename "$file")
        if [ ! -f "/workspaces/0xv7/sultan-unified/core/src/$filename" ]; then
            cp "$file" /workspaces/0xv7/sultan-unified/core/src/
            echo "  ✅ Added $filename from node/src"
        fi
    done
fi

# 3. Move Cosmos SDK modules (for potential future use)
echo "📦 Preserving Cosmos SDK modules..."
if [ -d "/workspaces/0xv7/sovereign-chain/sovereign/x" ]; then
    cp -r /workspaces/0xv7/sovereign-chain/sovereign/x/* /workspaces/0xv7/sultan-unified/cosmos-modules/
    echo "  ✅ Preserved Cosmos modules"
fi

# 4. Archive old implementations
echo "📚 Archiving legacy code..."
mv /workspaces/0xv7/working-chain /workspaces/0xv7/sultan-unified/legacy/go-simple-chain 2>/dev/null
mv /workspaces/0xv7/sovereign-chain /workspaces/0xv7/sultan-unified/legacy/cosmos-attempt 2>/dev/null
mv /workspaces/0xv7/sultan-chain-mainnet /workspaces/0xv7/sultan-unified/legacy/original-rust 2>/dev/null
mv /workspaces/0xv7/node /workspaces/0xv7/sultan-unified/legacy/node-attempt 2>/dev/null

# 5. Clean up scattered files
echo "🧹 Cleaning up scattered files..."
find /workspaces/0xv7 -maxdepth 1 -name "*.rs" -exec mv {} /workspaces/0xv7/sultan-unified/legacy/ \;
find /workspaces/0xv7 -maxdepth 1 -name "day*.rs" -exec mv {} /workspaces/0xv7/sultan-unified/legacy/ \;
find /workspaces/0xv7 -maxdepth 1 -name "*_test.rs" -exec mv {} /workspaces/0xv7/sultan-unified/tests/ \;

# 6. Create unified Cargo.toml
echo "📝 Creating unified Cargo.toml..."
cat > /workspaces/0xv7/sultan-unified/Cargo.toml << 'CARGO'
[package]
name = "sultan-chain"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "sultan"
path = "core/src/main.rs"

[dependencies]
# Core dependencies
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Networking
libp2p = { version = "0.53", features = ["tcp", "noise", "yamux", "gossipsub", "kad", "identify", "ping"] }

# Storage
rocksdb = "0.21"

# Cryptography
pqcrypto-dilithium = "0.5"
sha3 = "0.10"

# Utilities
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
uuid = { version = "1.6", features = ["v4", "serde"] }

[dev-dependencies]
criterion = "0.5"
proptest = "1.4"
CARGO

# 7. Create README for the unified structure
echo "📄 Creating documentation..."
cat > /workspaces/0xv7/sultan-unified/README.md << 'README'
# Sultan Chain - Unified Implementation

## Project Structure

```
sultan-unified/
├── core/               # Main Rust blockchain implementation
│   ├── src/
│   │   ├── main.rs     # Entry point
│   │   ├── blockchain.rs
│   │   ├── consensus.rs
│   │   ├── p2p.rs
│   │   └── quantum.rs
│   └── Cargo.toml
├── cosmos-modules/     # Preserved Cosmos SDK modules for future integration
├── legacy/            # Archived previous attempts
├── docs/              # Documentation
├── scripts/           # Build and deployment scripts
└── tests/            # Test suites
```

## Current Status
- **Core Implementation**: ~40% complete
- **Focus**: Rust-based blockchain with quantum resistance
- **Next Steps**: Complete persistence, networking, and consensus

## Build Instructions
```bash
cd /workspaces/0xv7/sultan-unified
cargo build --release
```
README

echo ""
echo "✅ CLEANUP COMPLETE!"
echo ""
echo "📊 Summary:"
echo "  • Unified Rust implementation in: /workspaces/0xv7/sultan-unified/core"
echo "  • Legacy code archived in: /workspaces/0xv7/sultan-unified/legacy"
echo "  • Cosmos modules preserved in: /workspaces/0xv7/sultan-unified/cosmos-modules"
echo ""
echo "🎯 Next Steps:"
echo "  1. cd /workspaces/0xv7/sultan-unified"
echo "  2. cargo build --release"
echo "  3. Focus on completing the Rust implementation"
