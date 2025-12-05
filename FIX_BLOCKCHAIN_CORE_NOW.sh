#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║    FIXING SULTAN BLOCKCHAIN CORE - TARGETING RIGHT FILES     ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# First, let's find all lib.rs files and identify the main one
echo "🔍 Finding all lib.rs files..."
find /workspaces/0xv7 -name "lib.rs" -type f 2>/dev/null | head -10

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Focusing on sultan-chain-mainnet core"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cd /workspaces/0xv7/sultan-chain-mainnet

# Fix the main lib.rs with duplicates removed
echo "🔧 [1/6] Fixing core/src/lib.rs (removing duplicates)..."
cat > core/src/lib.rs << 'RUST'
// Sultan Chain Core Library
pub mod blockchain;
pub mod config;
pub mod consensus;
pub mod rewards;
pub mod rpc_server;
pub mod scylla_db;
pub mod sdk;
pub mod transaction_validator;
pub mod types;
pub mod persistence;
pub mod p2p;
pub mod multi_consensus;
pub mod state_sync;

// Re-export main types
pub use blockchain::{Blockchain, Block};
pub use config::ChainConfig;
pub use sdk::SultanSDK;
// Note: SultanToken should be defined in types.rs
RUST

# Create/fix the types.rs file
echo "🔧 [2/6] Creating core/src/types.rs..."
cat > core/src/types.rs << 'RUST'
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SultanToken {
    pub symbol: String,
    pub total_supply: u128,
    pub decimals: u8,
}

impl Default for SultanToken {
    fn default() -> Self {
        Self {
            symbol: "SLTN".to_string(),
            total_supply: 1_000_000_000_000_000_000, // 1 billion with 18 decimals
            decimals: 18,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub gas_fee: u64, // Always 0
    pub timestamp: u64,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
}
RUST

# Fix the blockchain.rs imports
echo "🔧 [3/6] Fixing core/src/blockchain.rs..."
if [ -f "core/src/blockchain.rs" ]; then
    # Add missing imports at the top of blockchain.rs
    sed -i '1i\
use serde::{Deserialize, Serialize};\
use sha2::{Sha256, Digest};\
use std::time::{SystemTime, UNIX_EPOCH};\
use crate::types::Transaction;\
use crate::config::ChainConfig;' core/src/blockchain.rs 2>/dev/null || \
    cat > core/src/blockchain.rs << 'RUST'
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::types::Transaction;
use crate::config::ChainConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub validator: String,
}

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub config: ChainConfig,
}

impl Blockchain {
    pub fn new(config: ChainConfig) -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            pending_transactions: Vec::new(),
            config,
        };
        blockchain.create_genesis_block();
        blockchain
    }

    pub fn create_genesis_block(&mut self) {
        let genesis = Block {
            index: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            transactions: vec![],
            previous_hash: "0".to_string(),
            hash: "genesis".to_string(),
            validator: "sultan".to_string(),
        };
        self.chain.push(genesis);
    }

    pub fn create_block(&mut self, transactions: Vec<Transaction>) -> Block {
        let previous_block = self.chain.last().unwrap();
        let block = Block {
            index: self.chain.len() as u64,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            transactions,
            previous_hash: previous_block.hash.clone(),
            hash: String::new(),
            validator: "validator1".to_string(),
        };
        
        let mut block = block;
        block.hash = self.calculate_hash(&block);
        self.chain.push(block.clone());
        block
    }

    fn calculate_hash(&self, block: &Block) -> String {
        let data = format!(
            "{}{}{}{}",
            block.index, block.timestamp, block.transactions.len(), block.previous_hash
        );
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
}
RUST
fi

# Create config.rs if it doesn't exist
echo "�� [4/6] Ensuring core/src/config.rs exists..."
if [ ! -f "core/src/config.rs" ]; then
    cat > core/src/config.rs << 'RUST'
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: String,
    pub block_time: u64,
    pub gas_price: u64,  // Always 0
    pub staking_apy: f64, // 26.67%
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self {
            chain_id: "sultan-1".to_string(),
            block_time: 5,
            gas_price: 0,
            staking_apy: 0.2667,
        }
    }
}
RUST
fi

# Fix the Cargo.toml for the core
echo "🔧 [5/6] Fixing core/Cargo.toml..."
cat > core/Cargo.toml << 'TOML'
[package]
name = "sultan-core"
version = "1.0.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sha2 = "0.10"
tokio = { version = "1.35", features = ["full"] }
anyhow = "1.0"
chrono = "0.4"

# Optional for features
scylla = { version = "0.12", optional = true }
libp2p = { version = "0.53", optional = true }

[features]
default = []
with-scylla = ["scylla"]
with-p2p = ["libp2p"]
TOML

# Create a simple binary to test
echo "🔧 [6/6] Creating test binary..."
mkdir -p core/src/bin
cat > core/src/bin/test_node.rs << 'RUST'
use sultan_core::{Blockchain, ChainConfig};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║            SULTAN CHAIN - BLOCKCHAIN CORE TEST                ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    
    let config = ChainConfig::default();
    println!("\n✅ Configuration:");
    println!("   Chain ID: {}", config.chain_id);
    println!("   Gas Price: ${}", config.gas_price);
    println!("   Staking APY: {:.2}%", config.staking_apy * 100.0);
    
    let mut blockchain = Blockchain::new(config);
    println!("\n✅ Blockchain initialized with genesis block");
    println!("   Genesis Hash: {}", blockchain.chain[0].hash);
    
    // Create a test block
    let block = blockchain.create_block(vec![]);
    println!("\n✅ New block created:");
    println!("   Block #: {}", block.index);
    println!("   Hash: {}", block.hash);
    println!("   Gas Fees: $0.00 (Zero forever!)");
    
    println!("\n🚀 Blockchain core is working!");
}
RUST

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔨 Building Sultan Blockchain Core..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

cd /workspaces/0xv7/sultan-chain-mainnet/core

# Build with cleaner output
cargo build --bin test_node 2>&1 | grep -E "Compiling|Finished|error\[" | tail -20

if [ -f "target/debug/test_node" ]; then
    echo ""
    echo "✅ ✅ ✅ BUILD SUCCESSFUL! ✅ ✅ ✅"
    echo ""
    echo "🚀 Running blockchain core test..."
    echo ""
    ./target/debug/test_node
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "✅ SULTAN BLOCKCHAIN CORE IS WORKING!"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
else
    echo "Checking for compilation errors..."
    cargo build --bin test_node 2>&1 | grep "error\[" | head -10
fi

