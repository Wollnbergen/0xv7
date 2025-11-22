//! Sultan Core Standalone Node
//!
//! Production-grade blockchain node that runs independently with:
//! - Block production
//! - Transaction processing
//! - Consensus mechanism
//! - P2P networking
//! - RPC server
//! - Persistent storage

use sultan_core::*;
use anyhow::{Result, Context};
use tracing::{info, warn, error, debug};
use tracing_subscriber;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use std::path::PathBuf;
use clap::Parser;

/// Sultan Node CLI Arguments
#[derive(Parser, Debug)]
#[clap(name = "sultan-node")]
#[clap(about = "Sultan Layer 1 Blockchain Node", long_about = None)]
struct Args {
    /// Node name
    #[clap(short, long, default_value = "sultan-node-1")]
    name: String,

    /// Data directory
    #[clap(short, long, default_value = "./data")]
    data_dir: String,

    /// Block time in seconds
    #[clap(short, long, default_value = "5")]
    block_time: u64,

    /// Enable validator mode
    #[clap(short, long)]
    validator: bool,

    /// Validator address (required if --validator)
    #[clap(long)]
    validator_address: Option<String>,

    /// Validator stake (required if --validator)
    #[clap(long)]
    validator_stake: Option<u64>,

    /// P2P listen address
    #[clap(short, long, default_value = "/ip4/0.0.0.0/tcp/26656")]
    p2p_addr: String,

    /// RPC listen address
    #[clap(short, long, default_value = "0.0.0.0:26657")]
    rpc_addr: String,

    /// Genesis accounts (address:balance,address:balance,...)
    #[clap(long)]
    genesis: Option<String>,
}

/// Main node state
struct NodeState {
    blockchain: Arc<RwLock<Blockchain>>,
    consensus: Arc<RwLock<ConsensusEngine>>,
    storage: Arc<RwLock<PersistentStorage>>,
    validator_address: Option<String>,
    block_time: u64,
}

impl NodeState {
    async fn new(args: &Args) -> Result<Self> {
        // Initialize storage
        let storage_path = PathBuf::from(&args.data_dir).join("blocks");
        std::fs::create_dir_all(&storage_path)
            .context("Failed to create data directory")?;
        
        let storage = PersistentStorage::new(storage_path.to_str().unwrap())
            .context("Failed to initialize storage")?;

        // Load or create blockchain
        let mut blockchain = if let Some(latest_block) = storage.get_latest_block()? {
            info!("Loading existing blockchain from height {}", latest_block.index);
            
            // Reconstruct blockchain from storage
            let mut chain = Blockchain::new();
            
            // Load all blocks
            for i in 0..=latest_block.index {
                if let Some(block) = storage.get_block_by_height(i)? {
                    chain.chain.push(block);
                }
            }
            
            // Rebuild state from all transactions
            for block in &chain.chain {
                for tx in &block.transactions {
                    let mut temp_state = chain.state.clone();
                    if chain.apply_transaction(&mut temp_state, tx).is_ok() {
                        chain.state = temp_state;
                    }
                }
            }
            
            info!("Loaded {} blocks, current height: {}", chain.chain.len(), chain.height());
            chain
        } else {
            info!("Creating new blockchain");
            Blockchain::new()
        };
        
        // Parse and add genesis accounts (for both new and loaded blockchains)
        if let Some(genesis_str) = &args.genesis {
            for account in genesis_str.split(',') {
                let parts: Vec<&str> = account.split(':').collect();
                if parts.len() == 2 {
                    let address = parts[0].to_string();
                    let balance: u64 = parts[1].parse()
                        .context("Invalid balance in genesis")?;
                    blockchain.init_account(address.clone(), balance);
                    info!("Genesis account: {} = {}", address, balance);
                }
            }
        } else {
            // Default genesis accounts for testing
            blockchain.init_account("alice".to_string(), 1_000_000);
            blockchain.init_account("bob".to_string(), 500_000);
            blockchain.init_account("charlie".to_string(), 250_000);
            info!("Using default genesis accounts");
        }

        // Initialize consensus
        let mut consensus = ConsensusEngine::new();
        
        // Add this node as validator if specified
        if args.validator {
            let validator_addr = args.validator_address.as_ref()
                .context("--validator-address required when --validator is set")?;
            let validator_stake = args.validator_stake
                .context("--validator-stake required when --validator is set")?;
            
            consensus.add_validator(validator_addr.clone(), validator_stake)
                .context("Failed to add validator")?;
            
            info!("Running as validator: {} (stake: {})", validator_addr, validator_stake);
        }

        Ok(Self {
            blockchain: Arc::new(RwLock::new(blockchain)),
            consensus: Arc::new(RwLock::new(consensus)),
            storage: Arc::new(RwLock::new(storage)),
            validator_address: args.validator_address.clone(),
            block_time: args.block_time,
        })
    }

    /// Block production loop
    async fn run_block_production(&self) -> Result<()> {
        let mut ticker = interval(Duration::from_secs(self.block_time));
        
        loop {
            ticker.tick().await;
            
            if let Err(e) = self.produce_block().await {
                error!("Block production failed: {}", e);
                continue;
            }
        }
    }

    /// Produce a single block
    async fn produce_block(&self) -> Result<()> {
        let mut consensus = self.consensus.write().await;
        let mut blockchain = self.blockchain.write().await;
        let storage = self.storage.read().await;

        // Select proposer
        let proposer = consensus.select_proposer()
            .context("No validator available to propose")?;

        // Check if we are the proposer
        if let Some(our_address) = &self.validator_address {
            if &proposer != our_address {
                debug!("Not our turn to propose (proposer: {})", proposer);
                return Ok(());
            }
        }

        // Create block
        let block = blockchain.create_block(proposer.clone())
            .context("Failed to create block")?;

        // Record proposal
        consensus.record_proposal(&proposer)
            .context("Failed to record proposal")?;

        // Validate block
        if !blockchain.validate_block(&block)? {
            error!("Created invalid block!");
            return Ok(());
        }

        // Add block to chain
        blockchain.chain.push(block.clone());

        // Save to storage
        storage.save_block(&block)
            .context("Failed to save block")?;

        info!(
            "✅ Block {} produced by {} | {} txs | state_root: {}",
            block.index,
            block.validator,
            block.transactions.len(),
            &block.state_root[..16]
        );

        // Log balances for monitoring
        debug!("Current balances:");
        for (addr, account) in &blockchain.state {
            debug!("  {}: {}", addr, account.balance);
        }

        Ok(())
    }

    /// Transaction submission endpoint
    async fn submit_transaction(&self, tx: Transaction) -> Result<String> {
        let mut blockchain = self.blockchain.write().await;
        
        blockchain.add_transaction(tx.clone())
            .context("Transaction validation failed")?;

        let tx_hash = format!("{:?}", tx);
        info!("Transaction accepted: {} -> {} ({})", tx.from, tx.to, tx.amount);
        
        Ok(tx_hash)
    }

    /// Get blockchain status
    async fn get_status(&self) -> Result<NodeStatus> {
        let blockchain = self.blockchain.read().await;
        let consensus = self.consensus.read().await;
        
        let latest_block = blockchain.get_latest_block();
        
        Ok(NodeStatus {
            height: blockchain.height(),
            latest_hash: latest_block.hash.clone(),
            validator_count: consensus.validator_count(),
            pending_txs: blockchain.pending_transactions.len(),
            total_accounts: blockchain.state.len(),
        })
    }
}

#[derive(Debug, serde::Serialize)]
struct NodeStatus {
    height: u64,
    latest_hash: String,
    validator_count: usize,
    pending_txs: usize,
    total_accounts: usize,
}

/// Simple RPC server for the node
mod rpc {
    use super::*;
    use warp::Filter;
    use std::net::SocketAddr;

    pub async fn run_rpc_server(
        addr: SocketAddr,
        state: Arc<NodeState>,
    ) -> Result<()> {
        info!("Starting RPC server on {}", addr);

        // GET /status
        let status_route = warp::path("status")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_status);

        // POST /tx
        let tx_route = warp::path("tx")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_state(state.clone()))
            .and_then(handle_submit_tx);

        // GET /block/:height
        let block_route = warp::path!("block" / u64)
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_get_block);

        // GET /balance/:address
        let balance_route = warp::path!("balance" / String)
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_get_balance);

        let routes = status_route
            .or(tx_route)
            .or(block_route)
            .or(balance_route)
            .with(warp::cors().allow_any_origin());

        warp::serve(routes).run(addr).await;

        Ok(())
    }

    fn with_state(
        state: Arc<NodeState>,
    ) -> impl Filter<Extract = (Arc<NodeState>,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || state.clone())
    }

    async fn handle_status(
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        match state.get_status().await {
            Ok(status) => Ok(warp::reply::json(&status)),
            Err(e) => {
                error!("Status query failed: {}", e);
                Err(warp::reject())
            }
        }
    }

    async fn handle_submit_tx(
        tx: Transaction,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        match state.submit_transaction(tx).await {
            Ok(hash) => Ok(warp::reply::json(&serde_json::json!({ "hash": hash }))),
            Err(e) => {
                warn!("Transaction rejected: {}", e);
                Err(warp::reject())
            }
        }
    }

    async fn handle_get_block(
        height: u64,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let blockchain = state.blockchain.read().await;
        
        match blockchain.get_block(height) {
            Some(block) => Ok(warp::reply::json(block)),
            None => Err(warp::reject()),
        }
    }

    async fn handle_get_balance(
        address: String,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let blockchain = state.blockchain.read().await;
        let balance = blockchain.get_balance(&address);
        
        Ok(warp::reply::json(&serde_json::json!({
            "address": address,
            "balance": balance,
            "nonce": blockchain.get_nonce(&address)
        })))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();

    let args = Args::parse();

    info!("🚀 Starting Sultan Node: {}", args.name);
    info!("📁 Data directory: {}", args.data_dir);
    info!("⏱️  Block time: {}s", args.block_time);
    info!("🌐 P2P address: {}", args.p2p_addr);
    info!("🔌 RPC address: {}", args.rpc_addr);

    // Initialize node state
    let state = Arc::new(NodeState::new(&args).await?);

    // Get initial status
    let status = state.get_status().await?;
    info!("📊 Initial state:");
    info!("   Height: {}", status.height);
    info!("   Validators: {}", status.validator_count);
    info!("   Accounts: {}", status.total_accounts);

    // Start RPC server
    let rpc_addr: std::net::SocketAddr = args.rpc_addr.parse()
        .context("Invalid RPC address")?;
    let rpc_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = rpc::run_rpc_server(rpc_addr, rpc_state).await {
            error!("RPC server error: {}", e);
        }
    });

    // Start P2P network
    let p2p_addr = args.p2p_addr.clone();
    tokio::spawn(async move {
        info!("P2P network would listen on: {} (Phase 2)", p2p_addr);
        // Will be implemented with libp2p in production
        // For now, single node mode
    });

    info!("✅ Node initialized successfully");
    info!("🔗 RPC available at http://{}", args.rpc_addr);
    
    // Start block production if validator
    if args.validator {
        info!("⛏️  Starting block production (validator mode)");
        state.run_block_production().await?;
    } else {
        info!("👁️  Running in observer mode (no block production)");
        // Keep alive
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }

    Ok(())
}
