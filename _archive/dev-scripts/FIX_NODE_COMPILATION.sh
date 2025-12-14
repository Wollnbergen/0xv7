#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║            FIXING NODE COMPILATION ERRORS                     ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/node

# Fix 1: Remove duplicate modules from lib.rs
echo "🔧 [1/4] Cleaning up lib.rs..."
cat > src/lib.rs << 'RUST'
// Core modules
pub mod blockchain;
pub mod config;
pub mod consensus;
pub mod types;

// Network & Storage
pub mod p2p;
pub mod scylla_db;
pub mod persistence;

// Features
pub mod rewards;
pub mod sdk;
pub mod transaction_validator;

// Services
pub mod rpc_server;
pub mod multi_consensus;
pub mod state_sync;

// Re-exports
pub use blockchain::Blockchain;
pub use config::ChainConfig;
pub use types::Transaction;
RUST

# Fix 2: Update Cargo.toml to remove invalid features
echo "🔧 [2/4] Fixing Cargo.toml..."
sed -i '/^\[features\]/,/^$/d' Cargo.toml
echo "" >> Cargo.toml
echo "[features]" >> Cargo.toml
echo "default = []" >> Cargo.toml

# Fix 3: Ensure all module files exist
echo "🔧 [3/4] Ensuring all module files exist..."
touch src/state_sync.rs 2>/dev/null

# Fix 4: Try to build
echo "🔧 [4/4] Testing compilation..."
cargo check --quiet 2>&1 | head -10

echo ""
echo "✅ Basic fixes applied. Run 'cargo build' to see remaining issues."

