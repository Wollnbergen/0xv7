// load_testing.rs - TPS simulation and load testing

// Add load testing logic here
use anyhow::Result;
use rand::Rng;
use std::time::Instant;
use ed25519_dalek::{Keypair, Signer}; // Keypair at root with rand_core
use tokio::time::sleep;
use std::time::Duration;
use crate::blockchain::Blockchain;
use std::sync::Arc;
use crate::grpc_service::sultan::SubmitTransactionRequest; // Singular, proto gen

pub async fn run_load_test_suite(blockchain: Arc<Blockchain>) -> Result<()> {
    let start = Instant::now();
    let mut rng = rand::thread_rng();
    let keypair = Keypair::generate(&mut rng);
    let shards = 8;
    let tps_target = 2_000_000;
    for i in 0..tps_target {
        let shard_id = (i % shards) as u32;
        let message = format!("Production tx {} on shard {}", i, shard_id);
        let signature = keypair.sign(message.as_bytes()).to_bytes().to_vec();
        let tx = SubmitTransactionRequest {
            from: "prod_from".to_string(),
            to: "prod_to".to_string(),
            amount: 1,
            nonce: i as u64,
            signature,
            // Gas-free via subsidies
        };
        blockchain.batch_execute(&tx).await?;
        // No sim delay for production benchmark
    }
    let duration = start.elapsed();
    println!("Real TPS: {}", tps_target as f64 / duration.as_secs_f64());
    Ok(())
}