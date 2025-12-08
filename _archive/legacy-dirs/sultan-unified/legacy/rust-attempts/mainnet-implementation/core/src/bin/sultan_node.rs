use anyhow::Result;
use std::sync::Arc;

// Import from the library crate using correct module names
use sultan_coordinator::{
    blockchain::Blockchain as SultanBlockchain,
    grpc_service,
    blockchain::ChainConfig,
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           SULTAN CHAIN NODE - MAINNET v1.0                    ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Sultan Chain config
    let config = ChainConfig {
        chain_id: "sultan-mainnet-1".to_string(),
        block_time_ms: 5000,
        max_validators: 100,
        min_stake: 5000,
        inflation_rate: 0.08,  // 8% annual
        total_supply: 1_000_000_000,
        shards: 8,
    };

    println!("📊 Configuration:");
    println!("   • Chain ID: {}", config.chain_id);
    println!("   • Block Time: {}ms", config.block_time_ms);
    println!("   • Inflation: {}%", config.inflation_rate * 100.0);
    println!("   • Validator APY: {:.2}%", (config.inflation_rate / 0.3) * 100.0);
    println!("   • Mobile APY: {:.2}%", (config.inflation_rate / 0.3 * 1.4) * 100.0);
    println!("   • Gas Fees: $0.00");
    println!();

    let blockchain = Arc::new(SultanBlockchain::new(config.clone()));

    match args.get(1).map(|s| s.as_str()) {
        Some("--migrate") => {
            println!("🔄 Starting ScyllaDB migration...");
            run_migration(blockchain.clone()).await?;
        }
        Some("--grpc-server") => {
            println!("🚀 Starting gRPC server...");
            let addr = args
                .get(2)
                .cloned()
                .or_else(|| std::env::var("SULTAN_GRPC_ADDR").ok())
                .unwrap_or_else(|| "0.0.0.0:50051".to_string());
            grpc_service::start_grpc_server(blockchain.clone(), addr).await?;
        }
        _ => {
            println!("�� Starting Sultan Blockchain...");
            
            // Start block production
            let producer = blockchain.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
                loop {
                    interval.tick().await;
                    match producer.produce_block(vec![]) {
                        Ok(block) => {
                            println!("✅ Block #{} produced", block.height);
                        }
                        Err(e) => {
                            eprintln!("❌ Block production error: {}", e);
                        }
                    }
                }
            });
            
            println!("🚀 Node is running! Producing blocks every 5 seconds...");
            println!("   Press Ctrl+C to stop");
            println!();
            
            // Keep running
            tokio::signal::ctrl_c().await?;
            println!("\n👋 Shutting down...");
        }
    }
    Ok(())
}

async fn run_migration(_blockchain: Arc<SultanBlockchain>) -> Result<()> {
    println!("⚠️ Migration disabled in this build.");
    Ok(())
}
