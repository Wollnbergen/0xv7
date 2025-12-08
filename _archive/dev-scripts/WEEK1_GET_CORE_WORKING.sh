#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║     WEEK 1: GETTING SULTAN CHAIN CORE WORKING                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "🎯 Goal: Fix compilation and get basic blockchain running"
echo ""

cd /workspaces/0xv7/node

# Step 1: Fix Cargo.toml dependencies
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔧 [1/5] Fixing Cargo.toml dependencies..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cat > Cargo.toml << 'TOML'
[package]
name = "sultan-node"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core
tokio = { version = "1.34", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"
anyhow = "1.0"
thiserror = "1.0"

# Cryptography
sha2 = "0.10"
ed25519-dalek = "2.0"
rand = "0.8"
hex = "0.4"

# Networking
libp2p = { version = "0.53", features = ["tcp", "noise", "yamux", "gossipsub", "identify", "kad", "mdns"] }
jsonrpc-http-server = "18.0"

# Database
rocksdb = "0.21"

# Utilities
clap = { version = "4.0", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
chrono = "0.4"

[features]
default = []

[[bin]]
name = "sultan_node"
path = "src/bin/sultan_node.rs"
TOML

# Step 2: Create minimal working lib.rs
echo ""
echo "🔧 [2/5] Creating minimal lib.rs..."

cat > src/lib.rs << 'RUST'
// Core modules - only include what exists
pub mod blockchain;
pub mod config;
pub mod consensus;
pub mod types;

// Re-exports
pub use blockchain::Blockchain;
pub use config::ChainConfig;
pub use types::Transaction;
RUST

# Step 3: Fix blockchain.rs to compile
echo ""
echo "🔧 [3/5] Fixing blockchain.rs..."

cat > src/blockchain.rs << 'RUST'
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub prev_hash: String,
    pub hash: String,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub gas_fee: u64, // Always 0 for Sultan Chain
    pub timestamp: u64,
}

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis = Block {
            index: 0,
            timestamp: Self::current_timestamp(),
            transactions: vec![],
            prev_hash: String::from("0"),
            hash: String::from("genesis"),
            nonce: 0,
        };
        
        Blockchain {
            chain: vec![genesis],
            pending_transactions: vec![],
        }
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.pending_transactions.push(tx);
    }

    pub fn create_block(&mut self) -> Block {
        let prev_block = self.chain.last().unwrap();
        let mut block = Block {
            index: prev_block.index + 1,
            timestamp: Self::current_timestamp(),
            transactions: self.pending_transactions.clone(),
            prev_hash: prev_block.hash.clone(),
            hash: String::new(),
            nonce: 0,
        };
        
        block.hash = Self::calculate_hash(&block);
        self.pending_transactions.clear();
        self.chain.push(block.clone());
        block
    }

    fn calculate_hash(block: &Block) -> String {
        let data = format!("{}{}{:?}{}", 
            block.index, 
            block.timestamp, 
            block.transactions,
            block.prev_hash
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub fn get_latest_block(&self) -> &Block {
        self.chain.last().unwrap()
    }
}

impl Transaction {
    pub fn new(from: String, to: String, amount: u64) -> Self {
        Transaction {
            from,
            to,
            amount,
            gas_fee: 0, // Zero gas fees!
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}
RUST

# Step 4: Create simple types.rs
echo ""
echo "🔧 [4/5] Creating types.rs..."

cat > src/types.rs << 'RUST'
pub use crate::blockchain::Transaction;
pub use crate::blockchain::Block;

#[derive(Debug)]
pub struct Address(pub String);

impl Address {
    pub fn new(addr: &str) -> Self {
        Address(addr.to_string())
    }
}
RUST

# Step 5: Create minimal config.rs
echo ""
echo "🔧 [5/5] Creating config.rs..."

cat > src/config.rs << 'RUST'
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: String,
    pub gas_price: u64,
    pub block_time: u64,
    pub max_block_size: usize,
}

impl Default for ChainConfig {
    fn default() -> Self {
        ChainConfig {
            chain_id: String::from("sultan-1"),
            gas_price: 0, // Zero gas fees!
            block_time: 5, // 5 second blocks
            max_block_size: 1000,
        }
    }
}
RUST

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔨 Testing compilation..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cargo build --lib 2>&1 | head -20

