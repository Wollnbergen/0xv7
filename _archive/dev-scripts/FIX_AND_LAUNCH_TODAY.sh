#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║       SULTAN CHAIN - FIX BUILD & LAUNCH TODAY                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/node

# 1. IMMEDIATE FIX: ChainConfig issue (5 minutes)
echo "🔧 Step 1: Fixing ChainConfig conflicts..."

# Create the missing ChainConfig struct
cat > src/config.rs << 'RUST'
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: String,
    pub min_validators: usize,
    pub max_validators: usize,
    pub block_time_ms: u64,
    pub max_block_size: usize,
    pub genesis_validators: Vec<String>,
    
    // Economics
    pub inflation_rate: f64,
    pub total_supply: u64,
    pub shards: usize,
    
    // Network
    pub rpc_port: u16,
    pub p2p_port: u16,
}

impl Default for ChainConfig {
    fn default() -> Self {
        ChainConfig {
            chain_id: "sultan-1".to_string(),
            min_validators: 4,
            max_validators: 100,
            block_time_ms: 5000,
            max_block_size: 1_000_000,
            genesis_validators: vec![],
            inflation_rate: 0.1333,  // 13.33% APY
            total_supply: 1_000_000_000,
            shards: 4,
            rpc_port: 3030,
            p2p_port: 26656,
        }
    }
}
RUST

# 2. Fix lib.rs to properly export modules
echo ""
echo "🔧 Step 2: Fixing module exports..."
cat > src/lib.rs << 'RUST'
pub mod config;
pub mod types;
pub mod sdk;
pub mod scylla_db;
pub mod blockchain;
pub mod consensus;
pub mod transaction_validator;

// Re-export commonly used types
pub use config::ChainConfig;
pub use types::{SultanToken, Validator, Transaction};
pub use sdk::SDK;
RUST

# 3. Remove problematic imports from other files
echo ""
echo "🔧 Step 3: Cleaning up imports..."
sed -i 's/use crate::ChainConfig/use crate::config::ChainConfig/g' src/blockchain.rs
sed -i 's/use crate::quantum/\/\/ Quantum module temporarily disabled/g' src/consensus.rs
sed -i '/proto::consensus/d' src/consensus.rs

# 4. Try to build
echo ""
echo "🔨 Step 4: Building Sultan..."
cargo build --lib 2>&1 | tail -20

# 5. If lib builds, try RPC server
echo ""
echo "🔨 Step 5: Building RPC Server..."
cargo build --bin rpc_server 2>&1 | tail -10

echo ""
echo "✅ Build fixes applied!"
