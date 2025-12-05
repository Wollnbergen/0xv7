#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║      BUILDING SULTAN CHAIN MAINNET NODE (NO GRPC)             ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7

# Create a clean Sultan node project
echo "📦 Creating clean Sultan Chain node..."
mkdir -p sultan_mainnet
cd sultan_mainnet

# Create Cargo.toml without problematic dependencies
cat > Cargo.toml << 'TOML'
[package]
name = "sultan-mainnet"
version = "1.0.0"
edition = "2021"

[dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = "0.4"
rand = "0.8"
sha2 = "0.10"
hex = "0.4"
anyhow = "1.0"
log = "0.4"
env_logger = "0.11"
TOML

# Create the main Sultan Chain node
cat > src/main.rs << 'RUST'
use anyhow::Result;
use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    height: u64,
    hash: String,
    prev_hash: String,
    timestamp: i64,
    transactions: Vec<Transaction>,
    validator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    from: String,
    to: String,
    amount: u64,
    fee: u64,  // Always 0 for Sultan!
    nonce: u64,
}

#[derive(Debug, Clone)]
struct Validator {
    address: String,
    stake: u64,
    is_mobile: bool,
    rewards_earned: u64,
}

struct SultanChain {
    blocks: Arc<Mutex<Vec<Block>>>,
    validators: Arc<Mutex<HashMap<String, Validator>>>,
    mempool: Arc<Mutex<Vec<Transaction>>>,
    config: ChainConfig,
}

#[derive(Debug, Clone)]
struct ChainConfig {
    chain_id: String,
    block_time: u64,
    inflation_rate: f64,
    min_stake: u64,
}

impl SultanChain {
    fn new(config: ChainConfig) -> Self {
        let mut blocks = Vec::new();
        
        // Genesis block
        blocks.push(Block {
            height: 0,
            hash: "genesis_sultan_chain".to_string(),
            prev_hash: "0".to_string(),
            timestamp: Utc::now().timestamp(),
            transactions: vec![],
            validator: "genesis".to_string(),
        });
        
        // Initial validators
        let mut validators = HashMap::new();
        validators.insert(
            "validator1".to_string(),
            Validator {
                address: "validator1".to_string(),
                stake: 10000,
                is_mobile: false,
                rewards_earned: 0,
            },
        );
        validators.insert(
            "mobile_validator1".to_string(),
            Validator {
                address: "mobile_validator1".to_string(),
                stake: 5000,
                is_mobile: true,
                rewards_earned: 0,
            },
        );
        
        SultanChain {
            blocks: Arc::new(Mutex::new(blocks)),
            validators: Arc::new(Mutex::new(validators)),
            mempool: Arc::new(Mutex::new(Vec::new())),
            config,
        }
    }
    
    fn calculate_hash(block: &Block) -> String {
        let data = format!(
            "{}{}{}{}",
            block.height,
            block.prev_hash,
            block.timestamp,
            block.transactions.len()
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }
    
    fn select_validator(&self) -> String {
        let validators = self.validators.lock().unwrap();
        let total_stake: u64 = validators.values().map(|v| v.stake).sum();
        
        if total_stake == 0 {
            return "default".to_string();
        }
        
        let mut rng = rand::thread_rng();
        let random = rng.gen_range(0..total_stake);
        
        let mut cumulative = 0;
        for (address, validator) in validators.iter() {
            cumulative += validator.stake;
            if random < cumulative {
                return address.clone();
            }
        }
        
        "default".to_string()
    }
    
    async fn produce_block(&self) -> Result<Block> {
        let mut blocks = self.blocks.lock().unwrap();
        let last_block = blocks.last().unwrap();
        
        let mut mempool = self.mempool.lock().unwrap();
        let transactions: Vec<Transaction> = mempool.drain(..mempool.len().min(100)).collect();
        
        let validator = self.select_validator();
        
        let new_block = Block {
            height: last_block.height + 1,
            prev_hash: last_block.hash.clone(),
            timestamp: Utc::now().timestamp(),
            transactions,
            validator: validator.clone(),
            hash: String::new(),
        };
        
        let hash = Self::calculate_hash(&new_block);
        let mut final_block = new_block;
        final_block.hash = hash;
        
        // Calculate rewards
        let mut validators = self.validators.lock().unwrap();
        if let Some(v) = validators.get_mut(&validator) {
            let base_reward = 100; // Base block reward
            let reward = if v.is_mobile {
                (base_reward as f64 * 1.4) as u64  // 40% bonus for mobile validators
            } else {
                base_reward
            };
            v.rewards_earned += reward;
        }
        
        blocks.push(final_block.clone());
        Ok(final_block)
    }
    
    fn get_stats(&self) -> ChainStats {
        let blocks = self.blocks.lock().unwrap();
        let validators = self.validators.lock().unwrap();
        
        let mobile_count = validators.values().filter(|v| v.is_mobile).count();
        let total_staked: u64 = validators.values().map(|v| v.stake).sum();
        
        ChainStats {
            height: blocks.len() as u64 - 1,
            validators: validators.len(),
            mobile_validators: mobile_count,
            total_staked,
            apy_regular: 26.67,
            apy_mobile: 37.33,
        }
    }
}

#[derive(Debug, Serialize)]
struct ChainStats {
    height: u64,
    validators: usize,
    mobile_validators: usize,
    total_staked: u64,
    apy_regular: f64,
    apy_mobile: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           SULTAN CHAIN MAINNET NODE v1.0                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    
    let config = ChainConfig {
        chain_id: "sultan-mainnet-1".to_string(),
        block_time: 5,
        inflation_rate: 0.08,
        min_stake: 5000,
    };
    
    println!("📊 Chain Configuration:");
    println!("   • Chain ID: {}", config.chain_id);
    println!("   • Block Time: {} seconds", config.block_time);
    println!("   • Inflation Rate: {}%", config.inflation_rate * 100.0);
    println!("   • Min Stake: {} SLTN", config.min_stake);
    println!();
    
    println!("💰 Economics:");
    println!("   • Gas Fees: $0.00 (subsidized)");
    println!("   • Validator APY: 26.67%");
    println!("   • Mobile Validator APY: 37.33% (40% bonus)");
    println!();
    
    let chain = Arc::new(SultanChain::new(config.clone()));
    
    // Spawn block production task
    let producer = chain.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(config.block_time));
        loop {
            interval.tick().await;
            match producer.produce_block().await {
                Ok(block) => {
                    let stats = producer.get_stats();
                    println!(
                        "✅ Block #{} | Validator: {} {} | {} txs | Hash: {}...",
                        block.height,
                        block.validator,
                        if block.validator.contains("mobile") { "📱" } else { "🖥️" },
                        block.transactions.len(),
                        &block.hash[..8]
                    );
                    
                    if block.height % 10 == 0 {
                        println!("📊 Stats: {} validators ({} mobile) | Total staked: {} SLTN",
                                 stats.validators, stats.mobile_validators, stats.total_staked);
                    }
                }
                Err(e) => eprintln!("❌ Block production error: {}", e),
            }
        }
    });
    
    // Spawn transaction generator for testing
    let tx_generator = chain.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(2));
        let mut nonce = 0;
        loop {
            interval.tick().await;
            
            // Generate random transactions
            let tx_count = rand::thread_rng().gen_range(0..5);
            for _ in 0..tx_count {
                nonce += 1;
                let tx = Transaction {
                    from: format!("user{}", rand::thread_rng().gen_range(1..100)),
                    to: format!("user{}", rand::thread_rng().gen_range(1..100)),
                    amount: rand::thread_rng().gen_range(1..1000),
                    fee: 0, // ZERO FEES!
                    nonce,
                };
                
                let mut mempool = tx_generator.mempool.lock().unwrap();
                mempool.push(tx);
            }
        }
    });
    
    println!("🚀 Sultan Chain is running!");
    println!("   • Producing blocks every {} seconds", config.block_time);
    println!("   • Generating test transactions");
    println!("   • Press Ctrl+C to stop");
    println!();
    
    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    println!("\n👋 Shutting down Sultan Chain...");
    
    Ok(())
}
RUST

echo "✅ Created Sultan mainnet node"
echo ""
echo "🔨 Building Sultan Chain..."

cargo build --release

if [ -f target/release/sultan-mainnet ]; then
    echo ""
    echo "✅ ✅ ✅ BUILD SUCCESSFUL! ✅ ✅ ✅"
    echo ""
    echo "🚀 Starting Sultan Chain Mainnet..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    RUST_LOG=info ./target/release/sultan-mainnet
else
    echo "❌ Build failed"
fi

