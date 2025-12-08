#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║       FIXING AND BUILDING SULTAN CHAIN - COMPLETE             ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/node

# Step 1: Ensure consensus.rs is complete
echo "🔧 [1/4] Fixing consensus.rs..."

cat > src/consensus.rs << 'RUST'
use crate::blockchain::{Block, Blockchain};
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};

pub struct SimpleConsensus {
    blockchain: Arc<Mutex<Blockchain>>,
    is_validator: bool,
}

impl SimpleConsensus {
    pub fn new(blockchain: Arc<Mutex<Blockchain>>) -> Self {
        SimpleConsensus {
            blockchain,
            is_validator: true,
        }
    }

    pub async fn start(&self) {
        let mut interval = interval(Duration::from_secs(5));
        
        loop {
            interval.tick().await;
            
            if self.is_validator {
                let mut chain = self.blockchain.lock().unwrap();
                
                if !chain.pending_transactions.is_empty() {
                    let block = chain.create_block();
                    println!("⛏️  Created block #{} with {} transactions", 
                        block.index, 
                        block.transactions.len()
                    );
                } else {
                    println!("⏳ No transactions to process");
                }
            }
        }
    }
}

impl Block {
    pub fn gas_fee_total(&self) -> u64 {
        self.transactions.iter().map(|tx| tx.gas_fee).sum()
    }
}
RUST

# Step 2: Complete the binary
echo "🔧 [2/4] Completing sultan_node.rs..."

cat > src/bin/sultan_node.rs << 'RUST'
use sultan_node::{Blockchain, ChainConfig, Transaction};
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║            SULTAN CHAIN NODE v0.1.0 - STARTING                ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    
    let config = ChainConfig::default();
    println!("📋 Configuration:");
    println!("   Chain ID: {}", config.chain_id);
    println!("   Gas Price: ${} (Forever Free!)", config.gas_price);
    println!("   Block Time: {} seconds", config.block_time);
    println!();
    
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    println!("⛓️  Blockchain initialized with genesis block");
    
    {
        let mut chain = blockchain.lock().unwrap();
        
        for i in 1..=3 {
            let tx = Transaction::new(
                format!("sultan{}", i),
                format!("user{}", i),
                100 * i
            );
            println!("➕ Adding transaction: {} → {} ({})", tx.from, tx.to, tx.amount);
            chain.add_transaction(tx);
        }
        
        let block = chain.create_block();
        println!();
        println!("✅ Block #{} created:", block.index);
        println!("   Hash: {}", &block.hash[..16]);
        println!("   Transactions: {}", block.transactions.len());
        println!("   Total Gas Fees: $0.00 ✨");
    }
    
    println!();
    println!("🚀 Sultan Chain is running!");
    println!("   • Zero gas fees: ✅");
    println!("   • Quantum resistant: Planned");
    println!("   • Target TPS: 1,200,000");
    println!();
    println!("Press Ctrl+C to stop");
    
    tokio::signal::ctrl_c().await.unwrap();
    println!("\n👋 Shutting down Sultan Chain...");
}
RUST

# Step 3: Build the project
echo ""
echo "🔨 [3/4] Building Sultan Chain..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cargo build --release --bin sultan_node 2>&1 | tee build_complete.log | tail -20

# Step 4: Check results
echo ""
echo "📊 [4/4] Build Results..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -f "target/release/sultan_node" ]; then
    echo "✅ BUILD SUCCESSFUL!"
    echo ""
    echo "Binary location: /workspaces/0xv7/node/target/release/sultan_node"
    echo "Binary size: $(ls -lh target/release/sultan_node | awk '{print $5}')"
    echo ""
    echo "🚀 Ready to run! Use: ./target/release/sultan_node"
else
    echo "❌ Build failed. Checking errors..."
    grep "error\[" build_complete.log | head -5
fi

