#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║     CREATING MINIMAL WORKING SULTAN NODE                      ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7

# Create a new minimal node directory
rm -rf sultan_node_minimal
mkdir -p sultan_node_minimal
cd sultan_node_minimal

# Create minimal Cargo.toml
cat > Cargo.toml << 'TOML'
[package]
name = "sultan-node"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = "0.4"
rand = "0.8"
TOML

# Create minimal main.rs
cat > src/main.rs << 'RUST'
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    height: u64,
    timestamp: i64,
    transactions: Vec<Transaction>,
    hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    from: String,
    to: String,
    amount: u64,
    fee: u64, // Always 0 for Sultan!
}

struct SultanNode {
    chain: Arc<Mutex<Vec<Block>>>,
    mempool: Arc<Mutex<Vec<Transaction>>>,
}

impl SultanNode {
    fn new() -> Self {
        let mut chain = Vec::new();
        // Genesis block
        chain.push(Block {
            height: 0,
            timestamp: chrono::Utc::now().timestamp(),
            transactions: vec![],
            hash: "genesis".to_string(),
        });
        
        SultanNode {
            chain: Arc::new(Mutex::new(chain)),
            mempool: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    async fn produce_blocks(self: Arc<Self>) {
        let mut interval = interval(Duration::from_secs(5));
        
        loop {
            interval.tick().await;
            
            let mut chain = self.chain.lock().unwrap();
            let height = chain.len() as u64;
            
            let mut mempool = self.mempool.lock().unwrap();
            let transactions = mempool.drain(..).collect();
            
            let block = Block {
                height,
                timestamp: chrono::Utc::now().timestamp(),
                transactions,
                hash: format!("{:x}", rand::random::<u64>()),
            };
            
            println!("✅ Block {} produced at {}", block.height, block.timestamp);
            chain.push(block);
        }
    }
}

#[tokio::main]
async fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         SULTAN CHAIN NODE - MINIMAL v1.0                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("💰 Zero Gas Fees");
    println!("📈 26.67% Validator APY");
    println!("⏱️  5 second blocks");
    println!();
    
    let node = Arc::new(SultanNode::new());
    
    // Start block production
    let producer = node.clone();
    tokio::spawn(async move {
        producer.produce_blocks().await;
    });
    
    println!("🚀 Node started! Producing blocks every 5 seconds...");
    println!("   Press Ctrl+C to stop");
    
    // Keep running
    tokio::signal::ctrl_c().await.unwrap();
    println!("\nShutting down...");
}
RUST

echo "🔨 Building minimal node..."
cargo build --release

if [ -f target/release/sultan-node ]; then
    echo ""
    echo "✅ ✅ ✅ MINIMAL NODE BUILT SUCCESSFULLY! ✅ ✅ ✅"
    echo ""
    echo "🚀 Starting the node..."
    ./target/release/sultan-node
else
    echo "❌ Even minimal build failed. Check Rust installation."
fi

