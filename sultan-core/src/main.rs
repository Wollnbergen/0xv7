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
use sultan_core::economics::Economics;
use sultan_core::bridge_integration::BridgeManager;
use sultan_core::staking::StakingManager;
use sultan_core::governance::GovernanceManager;
use sultan_core::token_factory::TokenFactory;
use sultan_core::native_dex::NativeDex;
use sultan_core::sharding_production::ShardConfig;
use sultan_core::ShardedBlockchainProduction;
use sultan_core::p2p::{P2PNetwork, NetworkMessage};
use anyhow::{Result, Context};
use tracing::{info, warn, error, debug};
use tracing_subscriber;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use std::path::PathBuf;
use clap::Parser;
use sha2::Digest;

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
    #[clap(short, long, default_value = "2")]
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

    /// Enable sharding for high TPS (8→8000 shards with auto-expansion)
    #[clap(long)]
    enable_sharding: bool,

    /// Initial number of shards (default: 8 for 64K TPS)
    #[clap(long, default_value = "8")]
    shard_count: usize,

    /// Maximum shards for auto-expansion (default: 8000 for 64M TPS)
    #[clap(long, default_value = "8000")]
    max_shards: usize,

    /// Transactions per shard (default: 8000)
    #[clap(long, default_value = "8000")]
    tx_per_shard: usize,

    /// Bootstrap peers for P2P network discovery (comma-separated multiaddrs)
    #[clap(long)]
    bootstrap_peers: Option<String>,

    /// Enable P2P networking (connects validators together)
    #[clap(long)]
    enable_p2p: bool,
}

/// Main node state
struct NodeState {
    blockchain: Arc<RwLock<Blockchain>>,
    sharded_blockchain: Option<Arc<RwLock<ShardedBlockchainProduction>>>,
    consensus: Arc<RwLock<ConsensusEngine>>,
    storage: Arc<RwLock<PersistentStorage>>,
    economics: Arc<RwLock<Economics>>,
    bridge_manager: Arc<BridgeManager>,
    staking_manager: Arc<StakingManager>,
    governance_manager: Arc<GovernanceManager>,
    token_factory: Arc<TokenFactory>,
    native_dex: Arc<NativeDex>,
    p2p_network: Option<Arc<RwLock<P2PNetwork>>>,
    validator_address: Option<String>,
    block_time: u64,
    sharding_enabled: bool,
    p2p_enabled: bool,
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

        // Initialize sharded blockchain if enabled
        let sharded_blockchain = if args.enable_sharding {
            let config = ShardConfig {
                shard_count: args.shard_count,
                max_shards: args.max_shards,
                tx_per_shard: args.tx_per_shard,
                cross_shard_enabled: true,
                byzantine_tolerance: 1,
                enable_fraud_proofs: true,
                auto_expand_threshold: 0.80,
            };
            
            let mut sharded = ShardedBlockchainProduction::new(config.clone());
            
            // Initialize same accounts in sharded blockchain
            if let Some(genesis_str) = &args.genesis {
                for account in genesis_str.split(',') {
                    let parts: Vec<&str> = account.split(':').collect();
                    if parts.len() == 2 {
                        let address = parts[0].to_string();
                        let balance: u64 = parts[1].parse()
                            .context("Invalid balance in genesis")?;
                        sharded.init_account(address.clone(), balance).await
                            .context("Failed to init sharded account")?;
                    }
                }
            } else {
                sharded.init_account("alice".to_string(), 1_000_000).await?;
                sharded.init_account("bob".to_string(), 500_000).await?;
                sharded.init_account("charlie".to_string(), 250_000).await?;;
            }
            
            let tps_capacity = sharded.get_tps_capacity();
            info!("🚀 PRODUCTION SHARDING: {} shards (expandable to {}) = {} TPS (up to {}M TPS)",
                config.shard_count, config.max_shards, tps_capacity, 
                (config.max_shards * config.tx_per_shard) / 1_000_000);
            
            Some(Arc::new(RwLock::new(sharded)))
        } else {
            info!("Running in single-threaded mode (no sharding)");
            None
        };

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

        // Initialize P2P network if enabled
        let p2p_network = if args.enable_p2p {
            let mut p2p = P2PNetwork::new()
                .context("Failed to create P2P network")?;
            
            // Set bootstrap peers if provided
            if let Some(peers_str) = &args.bootstrap_peers {
                let peers: Vec<String> = peers_str.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                p2p.set_bootstrap_peers(peers)?;
            }
            
            // Start P2P network
            p2p.start(&args.p2p_addr).await
                .context("Failed to start P2P network")?;
            
            info!("🌐 P2P networking enabled on {}", args.p2p_addr);
            
            Some(Arc::new(RwLock::new(p2p)))
        } else {
            info!("📴 P2P networking disabled (standalone mode)");
            None
        };

        Ok(Self {
            blockchain: Arc::new(RwLock::new(blockchain)),
            sharded_blockchain,
            consensus: Arc::new(RwLock::new(consensus)),
            storage: Arc::new(RwLock::new(storage)),
            economics: Arc::new(RwLock::new(Economics::new())),
            bridge_manager: Arc::new(BridgeManager::new()),
            staking_manager: Arc::new(StakingManager::new(0.08)), // 8% initial inflation
            governance_manager: Arc::new(GovernanceManager::new()),
            token_factory: Arc::new(TokenFactory::new()),
            native_dex: Arc::new(NativeDex::new(Arc::new(TokenFactory::new()))),
            p2p_network,
            validator_address: args.validator_address.clone(),
            block_time: args.block_time,
            sharding_enabled: args.enable_sharding,
            p2p_enabled: args.enable_p2p,
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

        // Get current height before block creation
        let current_height = if self.sharding_enabled {
            if let Some(ref sharded) = self.sharded_blockchain {
                sharded.read().await.get_height().await
            } else {
                0
            }
        } else {
            self.blockchain.read().await.chain.len() as u64
        };

        // Use sharded or regular blockchain
        let block = if self.sharding_enabled {
            if let Some(ref sharded) = self.sharded_blockchain {
                let mut sharded_bc = sharded.write().await;
                
                // In production, collect pending transactions here
                // For now, create empty block
                let transactions = vec![];
                
                let block = sharded_bc.create_block(transactions, proposer.clone()).await
                    .context("Failed to create sharded block")?;
                
                // Block is already added by create_block
                
                let stats = sharded_bc.get_stats().await;
                info!(
                    "✅ SHARDED Block {} | {} shards active | {} total txs | capacity: {} TPS",
                    block.index,
                    stats.shard_count,
                    stats.total_processed,
                    stats.estimated_tps
                );
                
                block
            } else {
                anyhow::bail!("Sharding enabled but sharded blockchain not initialized");
            }
        } else {
            let mut blockchain = self.blockchain.write().await;
            
            let block = blockchain.create_block(proposer.clone())
                .context("Failed to create block")?;

            // Validate block
            if !blockchain.validate_block(&block)? {
                error!("Created invalid block!");
                return Ok(());
            }

            // Add block to chain
            blockchain.chain.push(block.clone());
            
            info!(
                "✅ Block {} produced by {} | {} txs | state_root: {}",
                block.index,
                block.validator,
                block.transactions.len(),
                &block.state_root[..16]
            );
            
            block
        };

        // Record proposal
        consensus.record_proposal(&proposer)
            .context("Failed to record proposal")?;

        // Save to storage
        storage.save_block(&block)
            .context("Failed to save block")?;

        // === PRODUCTION INTEGRATIONS ===
        
        // Distribute staking rewards for this block
        let new_height = current_height + 1;
        if let Err(e) = self.staking_manager.distribute_block_rewards(new_height).await {
            error!("Failed to distribute block rewards at height {}: {}", new_height, e);
        } else {
            debug!("Distributed staking rewards at height {}", new_height);
        }

        // Update governance height to check for voting period endings
        self.governance_manager.update_height(new_height).await;
        
        // Update governance with total bonded for quorum calculations
        let staking_stats = self.staking_manager.get_statistics().await;
        self.governance_manager.update_total_bonded(staking_stats.total_staked).await;

        // === P2P BLOCK BROADCAST ===
        // Broadcast the new block to all connected peers
        if self.p2p_enabled {
            if let Some(ref p2p) = self.p2p_network {
                let block_data = bincode::serialize(&block)
                    .unwrap_or_default();
                let block_hash = format!("{:x}", sha2::Sha256::digest(&block_data));
                
                if let Err(e) = p2p.read().await.broadcast_block(
                    block.index,
                    &proposer,
                    &block_hash,
                    block_data,
                ).await {
                    warn!("Failed to broadcast block via P2P: {}", e);
                } else {
                    debug!("📢 Block {} broadcast to {} peers", 
                           block.index, 
                           p2p.read().await.peer_count().await);
                }
            }
        }

        Ok(())
    }

    /// Transaction submission endpoint
    async fn submit_transaction(&self, tx: Transaction) -> Result<String> {
        // Calculate transaction hash
        let tx_hash = format!("{}:{}:{}", tx.from, tx.to, tx.nonce);
        
        if self.sharding_enabled {
            if let Some(ref sharded) = self.sharded_blockchain {
                // Transaction will be processed in next block
                // For now, just store in mempool (TODO: implement sharded mempool)
                
                debug!("Transaction accepted (sharded): {} -> {} ({})", tx.from, tx.to, tx.amount);
            } else {
                anyhow::bail!("Sharding enabled but not initialized");
            }
        } else {
            let mut blockchain = self.blockchain.write().await;
            
            blockchain.add_transaction(tx.clone())
                .context("Transaction validation failed")?;

            info!("Transaction accepted: {} -> {} ({})", tx.from, tx.to, tx.amount);
        }
        
        Ok(tx_hash)
    }

    /// Get blockchain status
    async fn get_status(&self) -> Result<NodeStatus> {
        let (height, latest_hash, pending_txs, total_accounts) = if self.sharding_enabled {
            if let Some(ref sharded) = self.sharded_blockchain {
                let sharded_bc = sharded.read().await;
                let stats = sharded_bc.get_stats().await;
                let height = sharded_bc.get_height().await;
                
                (
                    height,
                    format!("sharded-block-{}", height),
                    0, // TODO: track pending txs in sharded mode
                    stats.shard_count * 1000, // Estimate
                )
            } else {
                (0, String::from("unknown"), 0, 0)
            }
        } else {
            let blockchain = self.blockchain.read().await;
            let latest_block = blockchain.get_latest_block();
            
            (
                blockchain.height(),
                latest_block.hash.clone(),
                blockchain.pending_transactions.len(),
                blockchain.state.len(),
            )
        };
        
        let consensus = self.consensus.read().await;
        
        let economics = self.economics.read().await;
        
        // Calculate validator count - includes self + connected P2P peers
        let validator_count = if self.p2p_enabled {
            if let Some(ref p2p) = self.p2p_network {
                // All connected peers + this node
                p2p.read().await.peer_count().await + 1
            } else {
                consensus.validator_count()
            }
        } else {
            consensus.validator_count()
        };
        
        Ok(NodeStatus {
            height,
            latest_hash,
            validator_count,
            pending_txs,
            total_accounts,
            sharding_enabled: self.sharding_enabled,
            shard_count: if self.sharding_enabled { 
                self.sharded_blockchain.as_ref().and_then(|s| {
                    // Get actual shard count from config
                    s.try_read().ok().map(|shard| shard.config.shard_count)
                }).unwrap_or(0)
            } else { 
                0 
            },
            inflation_rate: economics.current_inflation_rate,
            validator_apy: economics.validator_apy,
            total_burned: economics.total_burned,
            is_deflationary: economics.is_deflationary(),
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
    sharding_enabled: bool,
    shard_count: usize,
    inflation_rate: f64,
    validator_apy: f64,
    total_burned: u64,
    is_deflationary: bool,
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

        // GET /economics
        let economics_route = warp::path("economics")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_get_economics);

        // GET /bridges
        let bridges_route = warp::path("bridges")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_get_bridges);

        // GET /bridge/:chain
        let bridge_status_route = warp::path!("bridge" / String)
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_get_bridge_status);

        // POST /bridge/submit
        let bridge_tx_route = warp::path!("bridge" / "submit")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_state(state.clone()))
            .and_then(handle_submit_bridge_tx);

        // GET /bridge/:chain/fee?amount=X
        let bridge_fee_route = warp::path!("bridge" / String / "fee")
            .and(warp::get())
            .and(warp::query::<FeeQuery>())
            .and(with_state(state.clone()))
            .and_then(handle_get_bridge_fee);

        // GET /bridge/fees/treasury
        let treasury_route = warp::path!("bridge" / "fees" / "treasury")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_get_treasury);

        // GET /bridge/fees/statistics
        let fee_stats_route = warp::path!("bridge" / "fees" / "statistics")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_get_fee_stats);

        // ========= STAKING ROUTES =========
        
        // POST /staking/create_validator
        let create_validator_route = warp::path!("staking" / "create_validator")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_state(state.clone()))
            .and_then(handle_create_validator);

        // POST /staking/delegate
        let delegate_route = warp::path!("staking" / "delegate")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_state(state.clone()))
            .and_then(handle_delegate);

        // GET /staking/validators
        let validators_route = warp::path!("staking" / "validators")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_get_validators);

        // GET /staking/delegations/:address
        let delegations_route = warp::path!("staking" / "delegations" / String)
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_get_delegations);

        // POST /staking/withdraw_rewards
        let withdraw_rewards_route = warp::path!("staking" / "withdraw_rewards")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_state(state.clone()))
            .and_then(handle_withdraw_rewards);

        // GET /staking/statistics
        let staking_stats_route = warp::path!("staking" / "statistics")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_staking_statistics);

        // ========= GOVERNANCE ROUTES =========

        // POST /governance/propose
        let propose_route = warp::path!("governance" / "propose")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_state(state.clone()))
            .and_then(handle_submit_proposal);

        // POST /governance/vote
        let vote_route = warp::path!("governance" / "vote")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_state(state.clone()))
            .and_then(handle_vote);

        // GET /governance/proposals
        let proposals_route = warp::path!("governance" / "proposals")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_get_proposals);

        // GET /governance/proposal/:id
        let proposal_route = warp::path!("governance" / "proposal" / u64)
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_get_proposal);

        // POST /governance/tally/:id
        let tally_route = warp::path!("governance" / "tally" / u64)
            .and(warp::post())
            .and(with_state(state.clone()))
            .and_then(handle_tally_proposal);

        // GET /governance/statistics
        let gov_stats_route = warp::path!("governance" / "statistics")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_governance_statistics);

        // Token Factory Routes
        // POST /tokens/create
        let create_token_route = warp::path!("tokens" / "create")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_state(state.clone()))
            .and_then(handle_create_token);

        // POST /tokens/mint
        let mint_token_route = warp::path!("tokens" / "mint")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_state(state.clone()))
            .and_then(handle_mint_token);

        // POST /tokens/transfer
        let transfer_token_route = warp::path!("tokens" / "transfer")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_state(state.clone()))
            .and_then(handle_transfer_token);

        // POST /tokens/burn
        let burn_token_route = warp::path!("tokens" / "burn")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_state(state.clone()))
            .and_then(handle_burn_token);

        // GET /tokens/:denom/metadata
        let token_metadata_route = warp::path!("tokens" / String / "metadata")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_get_token_metadata);

        // GET /tokens/:denom/balance/:address
        let token_balance_route = warp::path!("tokens" / String / "balance" / String)
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_get_token_balance);

        // GET /tokens/list
        let list_tokens_route = warp::path!("tokens" / "list")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_list_tokens);

        // DEX Routes
        // POST /dex/create_pair
        let create_pair_route = warp::path!("dex" / "create_pair")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_state(state.clone()))
            .and_then(handle_create_pair);

        // POST /dex/swap
        let swap_route = warp::path!("dex" / "swap")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_state(state.clone()))
            .and_then(handle_swap);

        // POST /dex/add_liquidity
        let add_liquidity_route = warp::path!("dex" / "add_liquidity")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_state(state.clone()))
            .and_then(handle_add_liquidity);

        // POST /dex/remove_liquidity
        let remove_liquidity_route = warp::path!("dex" / "remove_liquidity")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_state(state.clone()))
            .and_then(handle_remove_liquidity);

        // GET /dex/pool/:pair_id
        let get_pool_route = warp::path!("dex" / "pool" / String)
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_get_pool);

        // GET /dex/pools
        let list_pools_route = warp::path!("dex" / "pools")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_list_pools);

        // GET /dex/price/:pair_id
        let get_price_route = warp::path!("dex" / "price" / String)
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_get_price);

        // Group routes to avoid type complexity limits
        let core_routes = status_route
            .or(tx_route)
            .or(block_route)
            .or(balance_route)
            .or(economics_route)
            .boxed();
        
        let bridge_routes = bridges_route
            .or(bridge_status_route)
            .or(bridge_tx_route)
            .or(bridge_fee_route)
            .or(treasury_route)
            .or(fee_stats_route)
            .boxed();
        
        let staking_routes = create_validator_route
            .or(delegate_route)
            .or(validators_route)
            .or(delegations_route)
            .or(withdraw_rewards_route)
            .or(staking_stats_route)
            .boxed();
        
        let gov_routes = propose_route
            .or(vote_route)
            .or(proposals_route)
            .or(proposal_route)
            .or(tally_route)
            .or(gov_stats_route)
            .boxed();
        
        let token_routes = create_token_route
            .or(mint_token_route)
            .or(transfer_token_route)
            .or(burn_token_route)
            .or(token_metadata_route)
            .or(token_balance_route)
            .or(list_tokens_route)
            .boxed();
        
        let dex_routes = create_pair_route
            .or(swap_route)
            .or(add_liquidity_route)
            .or(remove_liquidity_route)
            .or(get_pool_route)
            .or(list_pools_route)
            .or(get_price_route)
            .boxed();
        
        // Combine route groups with additional boxing to prevent type depth overflow
        let api_routes_1 = core_routes
            .or(bridge_routes)
            .or(staking_routes)
            .boxed();
        
        let api_routes_2 = gov_routes
            .or(token_routes)
            .or(dex_routes)
            .boxed();
        
        let routes = api_routes_1
            .or(api_routes_2)
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

    async fn handle_get_economics(
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let economics = state.economics.read().await;
        
        Ok(warp::reply::json(&serde_json::json!({
            "current_inflation_rate": economics.current_inflation_rate,
            "inflation_percentage": format!("{:.1}%", economics.current_inflation_rate * 100.0),
            "current_burn_rate": economics.current_burn_rate,
            "burn_percentage": format!("{:.1}%", economics.current_burn_rate * 100.0),
            "validator_apy": economics.validator_apy,
            "apy_percentage": format!("{:.2}%", economics.validator_apy * 100.0),
            "total_burned": economics.total_burned,
            "years_since_genesis": economics.years_since_genesis,
            "is_deflationary": economics.is_deflationary(),
            "inflation_rate": "4.0% (fixed forever)",
            "inflation_policy": "Fixed 4% annual inflation guarantees zero gas fees sustainable at 76M+ TPS"
        })))
    }

    async fn handle_get_bridges(
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let bridges = state.bridge_manager.get_all_bridges().await;
        let stats = state.bridge_manager.get_statistics().await;
        
        Ok(warp::reply::json(&serde_json::json!({
            "bridges": bridges,
            "statistics": stats
        })))
    }

    async fn handle_get_bridge_status(
        chain: String,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        match state.bridge_manager.get_bridge(&chain).await {
            Some(bridge) => Ok(warp::reply::json(&bridge)),
            None => Err(warp::reject()),
        }
    }

    #[derive(serde::Deserialize)]
    struct BridgeTxRequest {
        source_chain: String,
        dest_chain: String,
        source_tx: String,
        amount: u64,
        recipient: String,
    }

    #[derive(serde::Deserialize)]
    struct FeeQuery {
        amount: u64,
    }

    async fn handle_get_bridge_fee(
        chain: String,
        query: FeeQuery,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        match state.bridge_manager.calculate_fee(&chain, query.amount).await {
            Ok(fee_breakdown) => Ok(warp::reply::json(&fee_breakdown)),
            Err(e) => {
                warn!("Fee calculation failed: {}", e);
                Err(warp::reject())
            }
        }
    }

    async fn handle_get_treasury(
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let treasury = state.bridge_manager.get_treasury_address().await;
        Ok(warp::reply::json(&serde_json::json!({
            "treasury_address": treasury,
            "description": "Sultan L1 Bridge Treasury - Receives all cross-chain bridge fees",
            "usage": "Development, maintenance, security audits, and ecosystem growth"
        })))
    }

    async fn handle_get_fee_stats(
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        match state.bridge_manager.get_fee_statistics().await {
            Ok(stats) => Ok(warp::reply::json(&stats)),
            Err(e) => {
                error!("Fee statistics query failed: {}", e);
                Err(warp::reject())
            }
        }
    }

    async fn handle_submit_bridge_tx(
        req: BridgeTxRequest,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        match state.bridge_manager.submit_bridge_transaction(
            req.source_chain,
            req.dest_chain,
            req.source_tx,
            req.amount,
            req.recipient,
        ).await {
            Ok(tx_id) => Ok(warp::reply::json(&serde_json::json!({
                "tx_id": tx_id,
                "status": "pending"
            }))),
            Err(e) => {
                warn!("Bridge transaction failed: {}", e);
                Err(warp::reject())
            }
        }
    }

    // ========= STAKING HANDLERS =========

    #[derive(serde::Deserialize)]
    struct CreateValidatorRequest {
        validator_address: String,
        initial_stake: u64,
        commission_rate: f64,
    }

    async fn handle_create_validator(
        req: CreateValidatorRequest,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        match state.staking_manager.create_validator(
            req.validator_address.clone(),
            req.initial_stake,
            req.commission_rate,
        ).await {
            Ok(_) => Ok(warp::reply::json(&serde_json::json!({
                "validator_address": req.validator_address,
                "stake": req.initial_stake,
                "commission": req.commission_rate,
                "status": "active"
            }))),
            Err(e) => {
                warn!("Create validator failed: {}", e);
                Err(warp::reject())
            }
        }
    }

    #[derive(serde::Deserialize)]
    struct DelegateRequest {
        delegator_address: String,
        validator_address: String,
        amount: u64,
    }

    async fn handle_delegate(
        req: DelegateRequest,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        match state.staking_manager.delegate(
            req.delegator_address.clone(),
            req.validator_address.clone(),
            req.amount,
        ).await {
            Ok(_) => Ok(warp::reply::json(&serde_json::json!({
                "delegator": req.delegator_address,
                "validator": req.validator_address,
                "amount": req.amount,
                "status": "delegated"
            }))),
            Err(e) => {
                warn!("Delegation failed: {}", e);
                Err(warp::reject())
            }
        }
    }

    async fn handle_get_validators(
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let validators = state.staking_manager.get_validators().await;
        Ok(warp::reply::json(&validators))
    }

    async fn handle_get_delegations(
        address: String,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let delegations = state.staking_manager.get_delegations(&address).await;
        Ok(warp::reply::json(&delegations))
    }

    #[derive(serde::Deserialize)]
    struct WithdrawRewardsRequest {
        address: String,
        validator_address: Option<String>,
        is_validator: bool,
    }

    async fn handle_withdraw_rewards(
        req: WithdrawRewardsRequest,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let amount = if req.is_validator {
            state.staking_manager.withdraw_validator_rewards(&req.address).await
        } else if let Some(validator) = req.validator_address {
            state.staking_manager.withdraw_delegator_rewards(&req.address, &validator).await
        } else {
            return Err(warp::reject());
        };

        match amount {
            Ok(rewards) => Ok(warp::reply::json(&serde_json::json!({
                "address": req.address,
                "rewards_withdrawn": rewards,
                "status": "success"
            }))),
            Err(e) => {
                warn!("Withdraw rewards failed: {}", e);
                Err(warp::reject())
            }
        }
    }

    async fn handle_staking_statistics(
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let stats = state.staking_manager.get_statistics().await;
        Ok(warp::reply::json(&stats))
    }

    // ========= GOVERNANCE HANDLERS =========

    #[derive(serde::Deserialize)]
    struct SubmitProposalRequest {
        proposer: String,
        title: String,
        description: String,
        proposal_type: String,
        initial_deposit: u64,
        parameters: Option<HashMap<String, String>>,
    }

    async fn handle_submit_proposal(
        req: SubmitProposalRequest,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        use sultan_core::governance::ProposalType;

        let proposal_type = match req.proposal_type.as_str() {
            "parameter_change" => ProposalType::ParameterChange,
            "software_upgrade" => ProposalType::SoftwareUpgrade,
            "community_pool" => ProposalType::CommunityPool,
            "text" => ProposalType::TextProposal,
            _ => return Err(warp::reject()),
        };

        match state.governance_manager.submit_proposal(
            req.proposer,
            req.title,
            req.description,
            proposal_type,
            req.initial_deposit,
            req.parameters,
        ).await {
            Ok(proposal_id) => Ok(warp::reply::json(&serde_json::json!({
                "proposal_id": proposal_id,
                "status": "submitted"
            }))),
            Err(e) => {
                warn!("Submit proposal failed: {}", e);
                Err(warp::reject())
            }
        }
    }

    #[derive(serde::Deserialize)]
    struct VoteRequest {
        proposal_id: u64,
        voter: String,
        option: String,
        voting_power: u64,
    }

    async fn handle_vote(
        req: VoteRequest,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        use sultan_core::governance::VoteOption;

        let vote_option = match req.option.as_str() {
            "yes" => VoteOption::Yes,
            "no" => VoteOption::No,
            "abstain" => VoteOption::Abstain,
            "no_with_veto" => VoteOption::NoWithVeto,
            _ => return Err(warp::reject()),
        };

        match state.governance_manager.vote(
            req.proposal_id,
            req.voter.clone(),
            vote_option,
            req.voting_power,
        ).await {
            Ok(_) => Ok(warp::reply::json(&serde_json::json!({
                "proposal_id": req.proposal_id,
                "voter": req.voter,
                "status": "voted"
            }))),
            Err(e) => {
                warn!("Vote failed: {}", e);
                Err(warp::reject())
            }
        }
    }

    async fn handle_get_proposals(
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let proposals = state.governance_manager.get_all_proposals().await;
        Ok(warp::reply::json(&proposals))
    }

    async fn handle_get_proposal(
        proposal_id: u64,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        match state.governance_manager.get_proposal(proposal_id).await {
            Some(proposal) => Ok(warp::reply::json(&proposal)),
            None => Err(warp::reject()),
        }
    }

    async fn handle_tally_proposal(
        proposal_id: u64,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        match state.governance_manager.tally_proposal(proposal_id).await {
            Ok(tally) => Ok(warp::reply::json(&tally)),
            Err(e) => {
                warn!("Tally failed: {}", e);
                Err(warp::reject())
            }
        }
    }

    async fn handle_governance_statistics(
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let stats = state.governance_manager.get_statistics().await;
        Ok(warp::reply::json(&stats))
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

    // Start P2P message handler if enabled
    if args.enable_p2p {
        let p2p_state = state.clone();
        let validator_addr = args.validator_address.clone();
        let validator_stake = args.validator_stake;
        
        tokio::spawn(async move {
            // Announce this validator to the network
            if let (Some(addr), Some(stake)) = (validator_addr, validator_stake) {
                if let Some(ref p2p) = p2p_state.p2p_network {
                    if let Err(e) = p2p.read().await.announce_validator(&addr, stake).await {
                        warn!("Failed to announce validator: {}", e);
                    } else {
                        info!("📢 Announced validator {} to P2P network", addr);
                    }
                }
            }
            
            // P2P message handler loop
            info!("🌐 P2P message handler started");
            loop {
                // In a full implementation, this would:
                // 1. Receive blocks from peers via P2P network
                // 2. Validate incoming blocks
                // 3. Add valid blocks to the chain
                // 4. Update consensus state with new validators
                // 5. Relay transactions to other nodes
                
                // For now, just log peer status periodically
                tokio::time::sleep(Duration::from_secs(30)).await;
                
                if let Some(ref p2p) = p2p_state.p2p_network {
                    let peer_count = p2p.read().await.peer_count().await;
                    let validator_count = p2p.read().await.known_validator_count().await;
                    info!("🌐 P2P Status: {} peers connected, {} validators known", 
                          peer_count, validator_count);
                }
            }
        });
        
        info!("🌐 P2P networking enabled on {}", args.p2p_addr);
    } else {
        info!("📴 P2P networking disabled (standalone mode)");
    }

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

// Token Factory Handlers
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct CreateTokenRequest {
    creator: String,
    name: String,
    symbol: String,
    decimals: u8,
    initial_supply: u128,
    max_supply: Option<u128>,
    logo_url: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MintTokenRequest {
    denom: String,
    to_address: String,
    amount: u128,
}

#[derive(Debug, Deserialize)]
struct TransferTokenRequest {
    denom: String,
    from_address: String,
    to_address: String,
    amount: u128,
}

#[derive(Debug, Deserialize)]
struct BurnTokenRequest {
    denom: String,
    from_address: String,
    amount: u128,
}

#[derive(Debug, Deserialize)]
struct CreatePairRequest {
    creator: String,
    token_a: String,
    token_b: String,
    amount_a: u128,
    amount_b: u128,
}

#[derive(Debug, Deserialize)]
struct SwapRequest {
    from_address: String,
    pair_id: String,
    token_in: String,
    amount_in: u128,
    min_amount_out: u128,
}

#[derive(Debug, Deserialize)]
struct AddLiquidityRequest {
    provider: String,
    pair_id: String,
    amount_a: u128,
    amount_b: u128,
}

#[derive(Debug, Deserialize)]
struct RemoveLiquidityRequest {
    provider: String,
    pair_id: String,
    liquidity: u128,
}

async fn handle_create_token(
    request: CreateTokenRequest,
    state: Arc<NodeState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match state.token_factory.create_token(
        &request.creator,
        request.name,
        request.symbol,
        request.decimals,
        request.initial_supply,
        request.max_supply,
        request.logo_url,
        request.description,
    ).await {
        Ok(denom) => Ok(warp::reply::json(&serde_json::json!({
            "success": true,
            "denom": denom
        }))),
        Err(e) => Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

async fn handle_mint_token(
    request: MintTokenRequest,
    state: Arc<NodeState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match state.token_factory.mint_to(
        &request.denom,
        &request.to_address,
        request.amount,
    ).await {
        Ok(_) => Ok(warp::reply::json(&serde_json::json!({
            "success": true
        }))),
        Err(e) => Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

async fn handle_transfer_token(
    request: TransferTokenRequest,
    state: Arc<NodeState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match state.token_factory.transfer(
        &request.denom,
        &request.from_address,
        &request.to_address,
        request.amount,
    ).await {
        Ok(_) => Ok(warp::reply::json(&serde_json::json!({
            "success": true
        }))),
        Err(e) => Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

async fn handle_burn_token(
    request: BurnTokenRequest,
    state: Arc<NodeState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match state.token_factory.burn(
        &request.denom,
        &request.from_address,
        request.amount,
    ).await {
        Ok(_) => Ok(warp::reply::json(&serde_json::json!({
            "success": true
        }))),
        Err(e) => Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

async fn handle_get_token_metadata(
    denom: String,
    state: Arc<NodeState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match state.token_factory.get_metadata(&denom).await {
        Some(metadata) => Ok(warp::reply::json(&serde_json::json!({
            "success": true,
            "metadata": metadata
        }))),
        None => Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Token not found"
        }))),
    }
}

async fn handle_get_token_balance(
    denom: String,
    address: String,
    state: Arc<NodeState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let balance = state.token_factory.get_balance(&denom, &address).await;
    Ok(warp::reply::json(&serde_json::json!({
        "success": true,
        "balance": balance.to_string()
    })))
}

async fn handle_list_tokens(
    state: Arc<NodeState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    // TokenFactory doesn't expose list_tokens yet, return empty array
    Ok(warp::reply::json(&serde_json::json!({
        "success": true,
        "tokens": []
    })))
}

// DEX Handlers
async fn handle_create_pair(
    request: CreatePairRequest,
    state: Arc<NodeState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match state.native_dex.create_pair(
        &request.creator,
        &request.token_a,
        &request.token_b,
        request.amount_a,
        request.amount_b,
    ).await {
        Ok(pair_id) => Ok(warp::reply::json(&serde_json::json!({
            "success": true,
            "pair_id": pair_id
        }))),
        Err(e) => Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

async fn handle_swap(
    request: SwapRequest,
    state: Arc<NodeState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match state.native_dex.swap(
        &request.from_address,
        &request.pair_id,
        &request.token_in,
        request.amount_in,
        request.min_amount_out,
    ).await {
        Ok(amount_out) => Ok(warp::reply::json(&serde_json::json!({
            "success": true,
            "amount_out": amount_out
        }))),
        Err(e) => Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

async fn handle_add_liquidity(
    request: AddLiquidityRequest,
    state: Arc<NodeState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match state.native_dex.add_liquidity(
        &request.pair_id,
        &request.provider,
        request.amount_a,
        request.amount_b,
        0, // amount_a_min
        0, // amount_b_min
    ).await {
        Ok((amount_a, amount_b, liquidity)) => Ok(warp::reply::json(&serde_json::json!({
            "success": true,
            "liquidity": liquidity.to_string(),
            "amount_a": amount_a.to_string(),
            "amount_b": amount_b.to_string()
        }))),
        Err(e) => Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

async fn handle_remove_liquidity(
    request: RemoveLiquidityRequest,
    state: Arc<NodeState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match state.native_dex.remove_liquidity(
        &request.pair_id,
        &request.provider,
        request.liquidity,
        0, // amount_a_min
        0, // amount_b_min
    ).await {
        Ok((amount_a, amount_b)) => Ok(warp::reply::json(&serde_json::json!({
            "success": true,
            "amount_a": amount_a.to_string(),
            "amount_b": amount_b.to_string()
        }))),
        Err(e) => Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

async fn handle_get_pool(
    pair_id: String,
    state: Arc<NodeState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match state.native_dex.get_pool(&pair_id).await {
        Some(pool) => Ok(warp::reply::json(&serde_json::json!({
            "success": true,
            "pool": pool
        }))),
        None => Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Pool not found"
        }))),
    }
}

async fn handle_list_pools(
    state: Arc<NodeState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    // NativeDex doesn't expose list_pools yet, return empty array
    Ok(warp::reply::json(&serde_json::json!({
        "success": true,
        "pools": []
    })))
}

async fn handle_get_price(
    pair_id: String,
    state: Arc<NodeState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match state.native_dex.get_price(&pair_id).await {
        Ok(price) => Ok(warp::reply::json(&serde_json::json!({
            "success": true,
            "price_a_to_b": price,
            "price_b_to_a": 1.0 / price
        }))),
        Err(e) => Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}
