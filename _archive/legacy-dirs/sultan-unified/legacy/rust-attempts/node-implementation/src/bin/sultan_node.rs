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
