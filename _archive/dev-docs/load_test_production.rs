#!/usr/bin/env rust-script
//! Production-Grade Load Testing for Sultan L1
//! 
//! This script generates real transactions and measures actual TPS.
//! 
//! ```cargo
//! [dependencies]
//! tokio = { version = "1", features = ["full"] }
//! reqwest = { version = "0.11", features = ["json"] }
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! anyhow = "1"
//! colored = "2"
//! indicatif = "0.17"
//! rand = "0.8"
//! chrono = "0.4"
//! ```

use tokio::time::{Duration, Instant};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use rand::Rng;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Transaction {
    from: String,
    to: String,
    amount: u64,
    gas_fee: u64,
    timestamp: u64,
    nonce: u64,
    signature: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NodeStatus {
    height: u64,
    validator_count: usize,
    pending_txs: usize,
    total_accounts: usize,
    sharding_enabled: bool,
    shard_count: usize,
}

#[derive(Debug, Deserialize)]
struct SubmitResponse {
    hash: String,
}

struct LoadTestConfig {
    rpc_url: String,
    total_transactions: u64,
    concurrent_workers: usize,
    accounts: Vec<String>,
    batch_size: usize,
}

struct TestMetrics {
    successful_txs: Arc<AtomicU64>,
    failed_txs: Arc<AtomicU64>,
    start_time: Instant,
    end_time: Option<Instant>,
}

impl TestMetrics {
    fn new() -> Self {
        Self {
            successful_txs: Arc::new(AtomicU64::new(0)),
            failed_txs: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
            end_time: None,
        }
    }

    fn record_success(&self) {
        self.successful_txs.fetch_add(1, Ordering::Relaxed);
    }

    fn record_failure(&self) {
        self.failed_txs.fetch_add(1, Ordering::Relaxed);
    }

    fn get_tps(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let successful = self.successful_txs.load(Ordering::Relaxed) as f64;
        if elapsed > 0.0 {
            successful / elapsed
        } else {
            0.0
        }
    }

    fn finalize(&mut self) {
        self.end_time = Some(Instant::now());
    }

    fn print_summary(&self) {
        let elapsed = if let Some(end) = self.end_time {
            end.duration_since(self.start_time)
        } else {
            self.start_time.elapsed()
        };

        let successful = self.successful_txs.load(Ordering::Relaxed);
        let failed = self.failed_txs.load(Ordering::Relaxed);
        let total = successful + failed;
        let tps = successful as f64 / elapsed.as_secs_f64();

        println!("\n{}", "═".repeat(80).bright_cyan());
        println!("{}", "  LOAD TEST RESULTS".bright_cyan().bold());
        println!("{}", "═".repeat(80).bright_cyan());
        println!();
        println!("  {}  {}", "Total Transactions:".bright_white(), format!("{}", total).bright_yellow().bold());
        println!("  {}     {}", "Successful:".bright_green(), format!("{}", successful).bright_green().bold());
        println!("  {}         {}", "Failed:".bright_red(), format!("{}", failed).bright_red().bold());
        println!("  {}      {}", "Success Rate:".bright_white(), format!("{:.2}%", (successful as f64 / total as f64) * 100.0).bright_cyan().bold());
        println!();
        println!("  {}     {}", "Total Time:".bright_white(), format!("{:.2}s", elapsed.as_secs_f64()).bright_yellow().bold());
        println!("  {}   {}", "Avg Latency:".bright_white(), format!("{:.2}ms", elapsed.as_millis() as f64 / total as f64).bright_cyan());
        println!();
        println!("  {} {}", "ACTUAL TPS:".bright_white().bold(), format!("{:.2}", tps).bright_green().bold().underline());
        println!();
        println!("{}", "═".repeat(80).bright_cyan());
        println!();

        // Compare with other blockchains
        println!("{}", "  COMPARISON WITH OTHER BLOCKCHAINS".bright_cyan().bold());
        println!("{}", "─".repeat(80).bright_black());
        println!("  {:<20} {}", "Ethereum:".bright_white(), format!("~15 TPS").bright_red());
        println!("  {:<20} {}", "Bitcoin:".bright_white(), format!("~7 TPS").bright_red());
        println!("  {:<20} {}", "Solana:".bright_white(), format!("~3,000 TPS").bright_yellow());
        println!("  {:<20} {}", "Avalanche:".bright_white(), format!("~4,500 TPS").bright_yellow());
        println!("  {:<20} {}", "Sultan L1:".bright_white().bold(), format!("{:.2} TPS", tps).bright_green().bold());
        println!();

        if tps > 10000.0 {
            println!("  {} Sultan L1 is {:.0}x faster than Solana!", 
                "🚀".bright_yellow(), 
                tps / 3000.0);
        } else if tps > 1000.0 {
            println!("  {} Excellent performance!", "✅".bright_green());
        } else {
            println!("  {} Performance below expected (target: 200,000 TPS)", "⚠️".bright_yellow());
        }
        println!();
    }
}

async fn check_node_status(client: &Client, rpc_url: &str) -> Result<NodeStatus> {
    let status: NodeStatus = client
        .get(format!("{}/status", rpc_url))
        .send()
        .await
        .context("Failed to fetch node status")?
        .json()
        .await
        .context("Failed to parse status response")?;
    
    Ok(status)
}

async fn submit_transaction(
    client: &Client,
    rpc_url: &str,
    tx: &Transaction,
) -> Result<String> {
    let response = client
        .post(format!("{}/tx", rpc_url))
        .json(tx)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .context("Failed to send transaction")?;

    if !response.status().is_success() {
        anyhow::bail!("Transaction rejected: {}", response.status());
    }

    let submit_resp: SubmitResponse = response
        .json()
        .await
        .context("Failed to parse submit response")?;

    Ok(submit_resp.hash)
}

fn generate_transaction(from: &str, to: &str, amount: u64, nonce: u64) -> Transaction {
    Transaction {
        from: from.to_string(),
        to: to.to_string(),
        amount,
        gas_fee: 0,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        nonce,
        signature: Some(format!("sig_{}", nonce)),
    }
}

async fn worker_task(
    worker_id: usize,
    client: Client,
    config: Arc<LoadTestConfig>,
    metrics: Arc<TestMetrics>,
    transactions_to_send: u64,
    progress: ProgressBar,
) {
    let mut rng = rand::thread_rng();
    let mut local_nonce = 0u64;

    for _ in 0..transactions_to_send {
        // Random sender and receiver
        let from_idx = rng.gen_range(0..config.accounts.len());
        let mut to_idx = rng.gen_range(0..config.accounts.len());
        
        // Ensure different accounts
        while to_idx == from_idx {
            to_idx = rng.gen_range(0..config.accounts.len());
        }

        let from = &config.accounts[from_idx];
        let to = &config.accounts[to_idx];
        let amount = rng.gen_range(1..1000);

        let tx = generate_transaction(from, to, amount, local_nonce);
        local_nonce += 1;

        match submit_transaction(&client, &config.rpc_url, &tx).await {
            Ok(_) => {
                metrics.record_success();
                progress.inc(1);
            }
            Err(e) => {
                metrics.record_failure();
                progress.inc(1);
                eprintln!("Worker {}: Transaction failed: {}", worker_id, e);
            }
        }
    }

    progress.finish_with_message(format!("Worker {} complete", worker_id));
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════════════════════╗".bright_cyan());
    println!("{}", "║                                                                           ║".bright_cyan());
    println!("{}", "║           🚀 SULTAN L1 PRODUCTION LOAD TEST 🚀                           ║".bright_cyan().bold());
    println!("{}", "║                                                                           ║".bright_cyan());
    println!("{}", "╚═══════════════════════════════════════════════════════════════════════════╝".bright_cyan());
    println!();

    // Configuration
    let rpc_url = std::env::var("RPC_URL").unwrap_or_else(|_| "http://localhost:26657".to_string());
    let total_transactions = std::env::var("TOTAL_TXS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10000);
    let concurrent_workers = std::env::var("WORKERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    // Generate test accounts
    let num_accounts = 1000;
    let accounts: Vec<String> = (0..num_accounts)
        .map(|i| format!("account_{}", i))
        .collect();

    println!("  {}  {}", "RPC Endpoint:".bright_white(), rpc_url.bright_cyan());
    println!("  {}    {}", "Total Transactions:".bright_white(), format!("{}", total_transactions).bright_yellow().bold());
    println!("  {} {}", "Concurrent Workers:".bright_white(), format!("{}", concurrent_workers).bright_yellow().bold());
    println!("  {}      {}", "Test Accounts:".bright_white(), format!("{}", num_accounts).bright_cyan());
    println!();

    // Check node status
    println!("  {} Checking node status...", "📡".bright_blue());
    let client = Client::new();
    let status = check_node_status(&client, &rpc_url).await?;

    println!("  {} Node is online!", "✅".bright_green());
    println!("    • Block Height: {}", status.height.to_string().bright_cyan());
    println!("    • Validators: {}", status.validator_count.to_string().bright_cyan());
    println!("    • Sharding: {}", if status.sharding_enabled {
        format!("✅ {} shards", status.shard_count).bright_green()
    } else {
        "❌ Disabled".bright_red().to_string()
    });
    println!();

    // Prepare load test
    let config = Arc::new(LoadTestConfig {
        rpc_url: rpc_url.clone(),
        total_transactions,
        concurrent_workers,
        accounts,
        batch_size: 100,
    });

    let metrics = Arc::new(TestMetrics::new());

    // Calculate transactions per worker
    let txs_per_worker = total_transactions / concurrent_workers as u64;

    println!("  {} Starting load test...", "🔥".bright_yellow().bold());
    println!("    • {} transactions per worker", txs_per_worker.to_string().bright_cyan());
    println!();

    // Create progress bars
    let multi_progress = MultiProgress::new();
    let main_progress = multi_progress.add(ProgressBar::new(total_transactions));
    main_progress.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) | ETA: {eta} | TPS: {per_sec}")
            .unwrap()
            .progress_chars("█▓▒░"),
    );
    main_progress.set_message("Total Progress");

    // Spawn workers
    let mut handles = vec![];
    for worker_id in 0..concurrent_workers {
        let worker_progress = multi_progress.add(ProgressBar::new(txs_per_worker));
        worker_progress.set_style(
            ProgressStyle::default_bar()
                .template(&format!("Worker {:03} [{{bar:20.green/blue}}] {{pos}}/{{len}}", worker_id))
                .unwrap()
                .progress_chars("█▓▒░"),
        );

        let client = client.clone();
        let config = Arc::clone(&config);
        let metrics = Arc::clone(&metrics);

        let handle = tokio::spawn(async move {
            worker_task(worker_id, client, config, metrics, txs_per_worker, worker_progress).await;
        });

        handles.push(handle);
    }

    // Monitor progress
    let metrics_clone = Arc::clone(&metrics);
    let monitor_handle = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let successful = metrics_clone.successful_txs.load(Ordering::Relaxed);
            let failed = metrics_clone.failed_txs.load(Ordering::Relaxed);
            let total = successful + failed;
            
            main_progress.set_position(total);

            if total >= total_transactions {
                break;
            }
        }
    });

    // Wait for all workers
    for handle in handles {
        handle.await?;
    }

    monitor_handle.await?;
    main_progress.finish_with_message("Load test complete!");

    // Finalize and print results
    let mut metrics_mut = Arc::try_unwrap(metrics).unwrap_or_else(|arc| (*arc).clone());
    metrics_mut.finalize();
    metrics_mut.print_summary();

    // Verify node status after test
    println!("{}", "  POST-TEST VERIFICATION".bright_cyan().bold());
    println!("{}", "─".repeat(80).bright_black());
    let final_status = check_node_status(&client, &rpc_url).await?;
    println!("  • Final Block Height: {}", final_status.height.to_string().bright_cyan());
    println!("  • Pending Transactions: {}", final_status.pending_txs.to_string().bright_yellow());
    println!("  • Total Accounts: {}", final_status.total_accounts.to_string().bright_cyan());
    println!();

    println!("{}", "🎉 Load test completed successfully!".bright_green().bold());
    println!();

    Ok(())
}
