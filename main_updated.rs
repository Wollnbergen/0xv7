use anyhow::Result;
use std::sync::Arc;
mod blockchain; // Core blockchain logic
mod scylla_db; // ScyllaDB integration for sharding/TPS
mod grpc_service; // gRPC for interop
mod quantum; // Quantum-proof signatures
mod load_testing; // TPS simulation
mod lib; // Shared structs like Block/Transaction
mod transaction_validator; // Gas-free subsidies, MEV
use blockchain::{Blockchain as SultanBlockchain, Config, start_block_production};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    // Create config
    let config = Config {
        database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| "scylla://127.0.0.1:9042".to_string()),
        port: 8080,
        enable_p2p: true,
        p2p_port: 9000,
        enable_consensus: true,
        enable_metrics: true,
        enable_rate_limiting: true,
    };
    // Initialize blockchain
    let blockchain = Arc::new(SultanBlockchain::new(config).await?);
    // Handle different commands
    match args.get(1).map(|s| s.as_str()) {
        Some("--migrate") => {
            println!("🔄 Starting ScyllaDB migration...");
            run_migration(blockchain).await?;
        }
        Some("--load-test") => {
            println!("🧪 Starting load test...");
            load_testing::run_load_test_suite(blockchain).await?;
        }
        Some("--grpc-server") => {
            println!("🚀 Starting gRPC server...");
            let addr = "0.0.0.0:50051".to_string();
            grpc_service::start_grpc_server(blockchain, addr).await?;
        }
        _ => {
            println!("🏃 Starting Sultan Blockchain...");
            // Start normal blockchain operation
            start_block_production(blockchain.clone()).await;
        }
    }
    Ok(())
}

async fn run_migration(blockchain: Arc<SultanBlockchain>) -> Result<()> {
    // Initialize ScyllaDB
    let scylla = Arc::new(
        scylla_db::ScyllaCluster::new(
            vec!["127.0.0.1:9042"],
            "sultan_blockchain"
        ).await?
    );
    // Create migrator
    let migrator = scylla_db::DataMigrator::new(
        blockchain.storage.clone(),
        scylla
    );
    // Get current block height
    let height = blockchain.storage.get_latest_block_height()?;
    // Migrate in batches
    migrator.migrate_blocks(0, 0, height).await?;
    println!("✅ Migration complete!");
    Ok(())
}
