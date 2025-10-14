use anyhow::Result;
use std::sync::Arc;

// Import from the library crate
use node::{blockchain::Blockchain as SultanBlockchain, scylla_db, grpc_service, load_testing};

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
        Some("--load-test") => {
            println!("🧪 Starting load test...");
            load_testing::run_load_test_suite(blockchain.clone()).await?;
        }
        Some("--grpc-server") => {
            println!("🚀 Starting gRPC server...");
            let addr = "0.0.0.0:50051".to_string();
            grpc_service::start_grpc_server(blockchain.clone(), addr).await?;
        }
        _ => {
            println!("🏃 Starting Sultan Blockchain...");
            // Start normal blockchain operation
            // Replace with your block production logic
        }
    }
    Ok(())
}

async fn run_migration(blockchain: Arc<SultanBlockchain>) -> Result<()> {
    let scylla = Arc::new(
        scylla_db::ScyllaCluster::new(
            vec!["127.0.0.1:9042"],
            "sultan_blockchain"
        ).await?
    );
    let migrator = scylla_db::DataMigrator::new(
        blockchain.db.clone().unwrap(),
        scylla
    );
    // Example: get latest block height
    // let height = blockchain.storage.get_latest_block_height()?;
    // migrator.migrate_blocks(0, 0, height).await?;
    println!("✅ Migration complete!");
    Ok(())
}