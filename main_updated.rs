use anyhow::Result;
use std::sync::Arc;

// Import from the library crate
use node::{blockchain::Blockchain as SultanBlockchain, grpc_service};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args: Vec<String> = std::env::args().collect();

    // Example config struct, adjust as needed
    let config = node::ChainConfig {
        inflation_rate: 8.0,
        total_supply: 0,
        min_stake: 5000,
        shards: 8,
    };

    let blockchain = Arc::new(SultanBlockchain::new(config.clone(), None).await?);

    match args.get(1).map(|s| s.as_str()) {
        Some("--migrate") => {
            println!("🔄 Starting ScyllaDB migration...");
            run_migration(blockchain.clone()).await?;
        }
        Some("--grpc-server") => {
            println!("🚀 Starting gRPC server...");
            // Allow overriding via CLI arg or env var
            let addr = args
                .get(2)
                .cloned()
                .or_else(|| std::env::var("SULTAN_GRPC_ADDR").ok())
                .unwrap_or_else(|| "0.0.0.0:50051".to_string());
            grpc_service::start_grpc_server(blockchain.clone(), addr).await?;
        }
        _ => {
            println!("🏃 Starting Sultan Blockchain...");
            // Start normal blockchain operation
        }
    }
    Ok(())
}

async fn run_migration(_blockchain: Arc<SultanBlockchain>) -> Result<()> {
    println!("⚠️ Migration disabled in this build.");
    Ok(())
}
