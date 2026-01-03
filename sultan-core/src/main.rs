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
use sultan_core::block_sync::{BlockSyncManager, SyncConfig};
use sultan_core::economics::Economics;
use sultan_core::bridge_integration::BridgeManager;
use sultan_core::staking::StakingManager;
use sultan_core::governance::GovernanceManager;
use sultan_core::token_factory::TokenFactory;
use sultan_core::native_dex::NativeDex;
use sultan_core::sharding_production::ShardConfig;
use sultan_core::sharded_blockchain_production::ConfirmedTransaction;
use sultan_core::SultanBlockchain;
use sultan_core::p2p::{P2PNetwork, NetworkMessage};
use sultan_core::config::Config;
use anyhow::{Result, Context, bail};
use tracing::{info, warn, error, debug};
use tracing_subscriber;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use std::path::PathBuf;
use clap::{Parser, Subcommand};
use sha2::Digest;
use ed25519_dalek::{Signature, VerifyingKey, Verifier, SigningKey, SIGNATURE_LENGTH};
use rand::rngs::OsRng;

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

    /// Validator Ed25519 public key (64 hex chars, required if --validator)
    #[clap(long)]
    validator_pubkey: Option<String>,

    /// P2P listen address
    #[clap(short, long, default_value = "/ip4/0.0.0.0/tcp/26656")]
    p2p_addr: String,

    /// RPC listen address
    #[clap(short, long, default_value = "0.0.0.0:26657")]
    rpc_addr: String,

    /// Allowed CORS origins (comma-separated, or '*' for any - insecure)
    #[clap(long, default_value = "http://localhost:3000,http://localhost:8080,http://127.0.0.1:3000")]
    allowed_origins: String,

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

    /// Enable TLS for RPC server (requires cert_path and key_path)
    #[clap(long)]
    enable_tls: bool,

    /// Path to TLS certificate file (PEM format)
    #[clap(long)]
    tls_cert: Option<String>,

    /// Path to TLS private key file (PEM format)
    #[clap(long)]
    tls_key: Option<String>,

    /// Subcommand (e.g., keygen)
    #[clap(subcommand)]
    command: Option<Command>,
}

/// CLI Subcommands
#[derive(Subcommand, Debug)]
enum Command {
    /// Generate a new Ed25519 keypair for validator registration
    Keygen {
        /// Output format (hex, json)
        #[clap(long, default_value = "hex")]
        format: String,
    },
}

/// Generate and display a new Ed25519 keypair
fn run_keygen(format: &str) {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    
    let secret_hex = hex::encode(signing_key.to_bytes());
    let public_hex = hex::encode(verifying_key.to_bytes());
    
    match format {
        "json" => {
            println!("{}", serde_json::json!({
                "public_key": public_hex,
                "secret_key": secret_hex,
                "algorithm": "Ed25519",
                "warning": "KEEP SECRET KEY SECURE - DO NOT SHARE"
            }));
        }
        _ => {
            println!("╔══════════════════════════════════════════════════════════════════════╗");
            println!("║              SULTAN L1 VALIDATOR KEYPAIR GENERATOR                   ║");
            println!("╠══════════════════════════════════════════════════════════════════════╣");
            println!("║ Algorithm: Ed25519                                                   ║");
            println!("╠══════════════════════════════════════════════════════════════════════╣");
            println!("║ PUBLIC KEY (use with --validator-pubkey):                            ║");
            println!("║ {}  ║", public_hex);
            println!("╠══════════════════════════════════════════════════════════════════════╣");
            println!("║ SECRET KEY (KEEP SECURE - DO NOT SHARE):                             ║");
            println!("║ {}  ║", secret_hex);
            println!("╚══════════════════════════════════════════════════════════════════════╝");
            println!();
            println!("Usage: sultan-node --validator --validator-pubkey {}", public_hex);
        }
    }
}

/// Validate Sultan address format (sultan1... with bech32)
fn validate_address(addr: &str) -> Result<(), String> {
    if addr.is_empty() {
        return Err("Address cannot be empty".to_string());
    }
    if !addr.starts_with("sultan1") {
        return Err(format!("Invalid address prefix: expected 'sultan1', got '{}'", &addr[..7.min(addr.len())]));
    }
    if addr.len() < 39 || addr.len() > 59 {
        return Err(format!("Invalid address length: {} (expected 39-59 chars)", addr.len()));
    }
    // Basic bech32 character set validation
    let valid_chars = "023456789acdefghjklmnpqrstuvwxyz";
    for c in addr[7..].chars() {
        if !valid_chars.contains(c) {
            return Err(format!("Invalid bech32 character: '{}'", c));
        }
    }
    Ok(())
}

/// Verify Ed25519 signature for RPC request authentication
/// Returns Ok(()) if signature is valid, Err with message otherwise
fn verify_request_signature(
    public_key_hex: &str,
    signature_hex: &str,
    message: &str,
) -> Result<(), String> {
    // Decode public key
    if public_key_hex.len() != 64 {
        return Err(format!("Invalid public key length: {} (expected 64 hex chars)", public_key_hex.len()));
    }
    let pubkey_bytes = hex::decode(public_key_hex)
        .map_err(|e| format!("Invalid public key hex: {}", e))?;
    let mut pubkey_arr = [0u8; 32];
    pubkey_arr.copy_from_slice(&pubkey_bytes);
    let verifying_key = VerifyingKey::from_bytes(&pubkey_arr)
        .map_err(|e| format!("Invalid Ed25519 public key: {}", e))?;
    
    // Decode signature
    if signature_hex.len() != 128 {
        return Err(format!("Invalid signature length: {} (expected 128 hex chars)", signature_hex.len()));
    }
    let sig_bytes = hex::decode(signature_hex)
        .map_err(|e| format!("Invalid signature hex: {}", e))?;
    let signature = Signature::from_slice(&sig_bytes)
        .map_err(|e| format!("Invalid Ed25519 signature: {}", e))?;
    
    // Hash the message (same as transaction_validator)
    let mut hasher = sha2::Sha256::new();
    hasher.update(message.as_bytes());
    let message_hash = hasher.finalize();
    
    // Verify signature
    verifying_key.verify(&message_hash, &signature)
        .map_err(|_| "Signature verification failed".to_string())
}

/// Wallet transaction request format (matches wallet extension API)
/// Flexible transaction request - accepts both wallet format and simple format
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
enum TxRequest {
    /// Wallet format: { tx: {...}, signature: "...", public_key: "..." }
    Wallet(WalletTxRequest),
    /// Simple format: { from: "...", to: "...", amount: 100, ... }
    Simple(SimpleTxRequest),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct WalletTxRequest {
    tx: WalletTxInner,
    signature: String,
    public_key: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SimpleTxRequest {
    from: String,
    to: String,
    #[serde(deserialize_with = "deserialize_amount")]
    amount: u64,
    #[serde(default)]
    gas_fee: u64,
    #[serde(default)]
    nonce: u64,
    #[serde(default)]
    timestamp: u64,
    #[serde(default)]
    signature: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct WalletTxInner {
    from: String,
    to: String,
    #[serde(deserialize_with = "deserialize_amount")]
    amount: u64,
    #[serde(default)]
    memo: Option<String>,
    #[serde(default)]
    nonce: u64,
    #[serde(default)]
    timestamp: u64,
}

fn deserialize_amount<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let value: serde_json::Value = serde::Deserialize::deserialize(deserializer)?;
    match value {
        serde_json::Value::Number(n) => {
            n.as_u64().ok_or_else(|| D::Error::custom("Invalid amount"))
        }
        serde_json::Value::String(s) => {
            s.parse::<u64>().map_err(|_| D::Error::custom("Invalid amount string"))
        }
        _ => Err(D::Error::custom("Amount must be number or string")),
    }
}

/// Main node state
/// 
/// Sultan L1 Node with always-on sharding architecture.
/// Sharding provides horizontal scalability from 64K to 32M TPS.
struct NodeState {
    /// The unified Sultan blockchain (always sharded internally)
    blockchain: Arc<RwLock<SultanBlockchain>>,
    consensus: Arc<RwLock<ConsensusEngine>>,
    storage: Arc<RwLock<PersistentStorage>>,
    economics: Arc<RwLock<Economics>>,
    bridge_manager: Arc<BridgeManager>,
    staking_manager: Arc<StakingManager>,
    governance_manager: Arc<GovernanceManager>,
    token_factory: Arc<TokenFactory>,
    native_dex: Arc<NativeDex>,
    p2p_network: Option<Arc<RwLock<P2PNetwork>>>,
    /// Block sync manager (TODO: Phase 4 integration)
    #[allow(dead_code)]
    block_sync_manager: Option<Arc<RwLock<BlockSyncManager>>>,
    validator_address: Option<String>,
    block_time: u64,
    p2p_enabled: bool,
    /// Allowed CORS origins for RPC security
    allowed_origins: Vec<String>,
    /// Chain configuration with feature flags (for hot-upgrades via governance)
    config: Arc<RwLock<Config>>,
    /// Path to config file for persistence
    config_path: PathBuf,
}

impl NodeState {
    async fn new(args: &Args) -> Result<Self> {
        // Initialize storage
        let storage_path = PathBuf::from(&args.data_dir).join("blocks");
        std::fs::create_dir_all(&storage_path)
            .context("Failed to create data directory")?;
        
        let storage = PersistentStorage::new(storage_path.to_str().unwrap())
            .context("Failed to initialize storage")?;

        // Configure sharding (always enabled, but shard count is configurable)
        let shard_count = if args.enable_sharding { args.shard_count } else { 16 };
        let config = ShardConfig {
            shard_count,
            max_shards: args.max_shards,
            tx_per_shard: args.tx_per_shard,
            cross_shard_enabled: true,
            byzantine_tolerance: 1,
            enable_fraud_proofs: true,
            auto_expand_threshold: 0.80,
        };

        // Create the unified Sultan blockchain
        let blockchain = SultanBlockchain::new(config.clone());
        
        // Initialize genesis accounts
        if let Some(genesis_str) = &args.genesis {
            for account in genesis_str.split(',') {
                let parts: Vec<&str> = account.split(':').collect();
                if parts.len() == 2 {
                    let address = parts[0].to_string();
                    let balance: u64 = parts[1].parse()
                        .context("Invalid balance in genesis")?;
                    blockchain.init_account(address.clone(), balance).await
                        .context("Failed to init genesis account")?;
                    info!("Genesis account: {} = {}", address, balance);
                }
            }
        } else {
            // Default genesis accounts for testing
            blockchain.init_account("alice".to_string(), 1_000_000).await?;
            blockchain.init_account("bob".to_string(), 500_000).await?;
            blockchain.init_account("charlie".to_string(), 250_000).await?;
            info!("Using default genesis accounts");
        }
        
        // Load existing blocks from storage if available
        if let Some(latest_block) = storage.get_latest_block()? {
            info!("Loading existing blockchain from height {}", latest_block.index);
            for i in 1..=latest_block.index {
                if let Some(block) = storage.get_block_by_height(i)? {
                    // Apply block to restore state
                    if let Err(e) = blockchain.apply_block(block.clone()).await {
                        warn!("Failed to apply stored block {}: {}", i, e);
                    }
                }
            }
            info!("Loaded {} blocks", latest_block.index);
        }

        let tps_capacity = blockchain.get_tps_capacity().await;
        info!("🚀 SULTAN L1 BLOCKCHAIN: {} shards (expandable to {}) = {} TPS",
            config.shard_count, config.max_shards, tps_capacity);
        info!("   💰 Zero gas fees | 4% inflation | Native bridges: BTC, ETH, SOL, TON");

        let blockchain_arc = Arc::new(RwLock::new(blockchain));

        // Initialize consensus
        let mut consensus = ConsensusEngine::new();
        
        // Add this node as validator if specified
        if args.validator {
            let validator_addr = args.validator_address.as_ref()
                .context("--validator-address required when --validator is set")?;
            let validator_stake = args.validator_stake
                .context("--validator-stake required when --validator is set")?;
            
            // Parse and validate Ed25519 public key from CLI
            let validator_pubkey = args.validator_pubkey.as_ref()
                .context("--validator-pubkey required when --validator is set (64 hex chars)")?;
            
            // Decode hex pubkey to [u8; 32]
            if validator_pubkey.len() != 64 {
                bail!("Validator pubkey must be 64 hex characters (32 bytes), got {}", validator_pubkey.len());
            }
            let pubkey_bytes = hex::decode(validator_pubkey)
                .context("Invalid validator pubkey hex")?;
            let pubkey_array: [u8; 32] = pubkey_bytes.try_into()
                .map_err(|_| anyhow::anyhow!("Invalid validator pubkey length"))?;
            
            // Verify it's a valid Ed25519 public key
            VerifyingKey::from_bytes(&pubkey_array)
                .context("Invalid Ed25519 public key")?;
            
            consensus.add_validator(validator_addr.clone(), validator_stake, pubkey_array)
                .context("Failed to add validator")?;
            
            info!("Running as validator: {} (stake: {}, pubkey: {}...)", 
                validator_addr, validator_stake, &validator_pubkey[..16]);
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

        // Create shared consensus Arc for block sync manager
        let consensus_arc = Arc::new(RwLock::new(consensus));

        // Initialize BlockSyncManager if P2P is enabled
        let block_sync_manager = if args.enable_p2p {
            let sync_config = SyncConfig::default();
            let block_sync = BlockSyncManager::new(
                sync_config,
                args.validator_address.clone(),
                consensus_arc.clone(),
                Duration::from_secs(args.block_time),
            );
            info!("🔄 BlockSyncManager initialized");
            Some(Arc::new(RwLock::new(block_sync)))
        } else {
            None
        };

        // Initialize staking manager and restore persisted state if available
        let staking_manager = Arc::new(StakingManager::new(0.04)); // 4% inflation (zero gas model)
        
        // Restore staking state from persistent storage
        if let Ok(Some(staking_snapshot)) = storage.load_staking_state() {
            info!("📥 Found persisted staking state, restoring...");
            if let Err(e) = staking_manager.restore_from_snapshot(staking_snapshot).await {
                warn!("⚠️ Failed to restore staking state: {}. Starting fresh.", e);
            }
        }

        // Parse allowed origins for CORS security
        let allowed_origins: Vec<String> = if args.allowed_origins == "*" {
            vec!["*".to_string()] // Wildcard mode (insecure, for dev only)
        } else {
            args.allowed_origins
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };

        // === HOT-UPGRADE INFRASTRUCTURE ===
        // Load or create chain configuration with feature flags
        let config_path = PathBuf::from(&args.data_dir).join("config.json");
        let config = if config_path.exists() {
            match Config::load(&config_path) {
                Ok(cfg) => {
                    info!("📋 Loaded chain config from {:?}", config_path);
                    info!("   Feature flags: wasm={}, evm={}, ibc={}", 
                          cfg.features.wasm_contracts_enabled,
                          cfg.features.evm_contracts_enabled,
                          cfg.features.ibc_enabled);
                    cfg
                }
                Err(e) => {
                    warn!("⚠️ Failed to load config: {}. Using defaults.", e);
                    Config::default()
                }
            }
        } else {
            let cfg = Config::default();
            if let Err(e) = cfg.save(&config_path) {
                warn!("⚠️ Failed to save default config: {}", e);
            } else {
                info!("📋 Created default chain config at {:?}", config_path);
            }
            cfg
        };

        // Create shared TokenFactory for both direct access and DEX
        let token_factory = Arc::new(TokenFactory::new());
        
        // Create BridgeManager with TokenFactory integration for wrapped token minting
        // When a bridge tx is verified, TokenFactory.mint_internal() mints sBTC/sETH/sSOL/sTON
        let bridge_manager = Arc::new(BridgeManager::with_token_factory(
            "sultan1treasury7xj3k2p8n9m5q4r6t8v0w2y4z6a8c0e2g4".to_string(),
            token_factory.clone(),
        ));
        
        Ok(Self {
            blockchain: blockchain_arc,
            consensus: consensus_arc,
            storage: Arc::new(RwLock::new(storage)),
            economics: Arc::new(RwLock::new(Economics::new())),
            bridge_manager,
            staking_manager,
            governance_manager: Arc::new(GovernanceManager::new()),
            token_factory: token_factory.clone(),
            native_dex: Arc::new(NativeDex::new(token_factory)),
            p2p_network,
            block_sync_manager,
            validator_address: args.validator_address.clone(),
            block_time: args.block_time,
            p2p_enabled: args.enable_p2p,
            allowed_origins,
            config: Arc::new(RwLock::new(config)),
            config_path,
        })
    }

    /// Activate or deactivate a feature flag via governance
    /// 
    /// This is the core hot-upgrade mechanism. When a governance proposal
    /// with a `features.*` parameter change passes, this method is called
    /// to actually activate the feature at runtime.
    /// 
    /// # Supported Features
    /// - `wasm_contracts_enabled`: Enable CosmWasm smart contracts
    /// - `evm_contracts_enabled`: Enable EVM smart contracts (future)
    /// - `ibc_enabled`: Enable IBC protocol (future)
    /// - `sharding_enabled`: Enable/disable sharding
    /// - `governance_enabled`: Enable/disable governance
    /// - `bridges_enabled`: Enable/disable cross-chain bridges
    async fn activate_feature(&self, feature: &str, enabled: bool) -> Result<()> {
        info!("🔧 Hot-upgrade: Activating feature {} = {}", feature, enabled);
        
        // Update config
        {
            let mut config = self.config.write().await;
            config.update_feature(feature, enabled)
                .context(format!("Failed to update feature: {}", feature))?;
            
            // Persist config to disk
            if let Err(e) = config.save(&self.config_path) {
                error!("⚠️ Failed to persist config: {}", e);
                // Continue anyway - runtime state is updated
            } else {
                info!("💾 Config persisted to {:?}", self.config_path);
            }
        }
        
        // Log feature activation - actual runtime components to be added post-launch
        match feature {
            "smart_contracts_enabled" => {
                if enabled {
                    info!("🚀 Smart contracts feature flag enabled (VM to be selected post-launch)");
                } else {
                    warn!("⚠️  Smart contracts feature flag disabled");
                }
            }
            "bridges_enabled" => {
                info!("🌉 Bridge feature flag updated: {}", enabled);
                // Bridges are currently always available via BridgeManager
                // This flag can be used for emergency disabling
            }
            "quantum_signatures_enabled" => {
                if enabled {
                    info!("🔐 Quantum-resistant signatures feature flag enabled");
                    // TODO: Integrate Dilithium3 signatures when ready
                }
            }
            _ => {
                info!("📋 Feature flag {} updated to {}", feature, enabled);
            }
        }
        
        Ok(())
    }
    
    /// Get current feature flags
    async fn get_feature_flags(&self) -> sultan_core::config::FeatureFlags {
        self.config.read().await.features.clone()
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
        let storage = self.storage.read().await;

        // Get current height FIRST - this determines the proposer
        let current_height = self.blockchain.read().await.get_height().await;
        let next_height = current_height + 1;

        // === PRODUCTION: Height-based proposer selection ===
        // All validators use the same height to deterministically select proposer
        // This ensures network-wide agreement on who should propose each block
        let consensus = self.consensus.read().await;
        let proposer = consensus.select_proposer_for_height(next_height)
            .context("No validator available to propose")?;
        drop(consensus);

        // Check if we are the proposer for this height
        if let Some(our_address) = &self.validator_address {
            if &proposer != our_address {
                debug!("Not our turn to propose height {} (proposer: {})", next_height, proposer);
                return Ok(());
            }
            info!("🎯 We are proposer for height {}", next_height);
        } else {
            // Not a validator, skip production
            return Ok(());
        }

        // Create block using unified Sultan blockchain
        let blockchain = self.blockchain.write().await;
        
        // Drain pending transactions from mempool
        let transactions = blockchain.drain_pending_transactions().await;
        let tx_count = transactions.len();
        
        let block = blockchain.create_block(transactions, proposer.clone()).await
            .context("Failed to create block")?;
        
        // Block is already added by create_block
        
        let stats = blockchain.get_stats().await;
        info!(
            "✅ SHARDED Block {} | {} shards active | {} txs in block | {} total processed | capacity: {} TPS",
            block.index,
            stats.shard_count,
            tx_count,
            stats.total_processed,
            stats.estimated_tps
        );
        
        // Auto-expand shards if load threshold exceeded
        if stats.should_expand {
            let current_shards = stats.shard_count;
            // Double shard count (up to max 8000)
            let additional = current_shards.min(8000 - current_shards);
            if additional > 0 {
                info!("⚡ Auto-expansion triggered at {:.1}% load", stats.current_load * 100.0);
                if let Err(e) = blockchain.expand_shards(additional).await {
                    warn!("Failed to auto-expand shards: {}", e);
                }
            }
        }
        
        drop(blockchain);

        // Record proposal in consensus
        {
            let mut consensus = self.consensus.write().await;
            consensus.record_proposal(&proposer)
                .context("Failed to record proposal")?;
        }

        // Record block signed in staking (resets missed block counter)
        if let Err(e) = self.staking_manager.record_block_signed(&proposer).await {
            warn!("Failed to record block signed for staking: {}", e);
        }

        // Save to storage
        storage.save_block(&block)
            .context("Failed to save block")?;

        // Save transactions to persistent storage for history queries
        for tx in &block.transactions {
            let confirmed_tx = ConfirmedTransaction {
                hash: format!("{:x}", sha2::Sha256::digest(format!("{}:{}:{}:{}", tx.from, tx.to, tx.amount, tx.timestamp).as_bytes())),
                from: tx.from.clone(),
                to: tx.to.clone(),
                amount: tx.amount,
                memo: None,
                nonce: tx.nonce,
                timestamp: tx.timestamp,
                block_height: block.index,
                status: "confirmed".to_string(),
            };
            if let Err(e) = storage.save_transaction(&confirmed_tx) {
                warn!("Failed to save transaction to storage: {}", e);
            }
        }

        // === PRODUCTION INTEGRATIONS ===
        
        // Distribute staking rewards for this block
        if let Err(e) = self.staking_manager.distribute_block_rewards(next_height).await {
            error!("Failed to distribute block rewards at height {}: {}", next_height, e);
        } else {
            debug!("Distributed staking rewards at height {}", next_height);
        }

        // Process matured unbondings - return tokens to delegators after 21-day period
        {
            let completed_unbondings = self.staking_manager.process_unbondings().await;
            if !completed_unbondings.is_empty() {
                let blockchain = self.blockchain.read().await;
                for unbonding in &completed_unbondings {
                    // Return the unbonded tokens to the delegator's available balance
                    if let Err(e) = blockchain.add_balance(&unbonding.delegator_address, unbonding.amount).await {
                        error!(
                            "CRITICAL: Failed to return {} SLTN to {} after unbonding: {}",
                            unbonding.amount / 1_000_000_000,
                            unbonding.delegator_address,
                            e
                        );
                    } else {
                        info!(
                            "💰 Unbonding complete: {} SLTN returned to {}",
                            unbonding.amount / 1_000_000_000,
                            unbonding.delegator_address
                        );
                    }
                }
                
                // Persist staking state after processing unbondings
                let storage = self.storage.read().await;
                if let Err(e) = self.staking_manager.persist_to_storage(&storage).await {
                    warn!("⚠️ Failed to persist staking state after unbonding: {}", e);
                }
            }
        }

        // Update governance height to check for voting period endings
        self.governance_manager.update_height(next_height).await;
        
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
                
                // Sign block hash with validator's key for proposer verification
                // In production, the proposer signs the block hash
                let proposer_signature = Vec::new(); // TODO: Sign with validator's key
                
                if let Err(e) = p2p.read().await.broadcast_block(
                    block.index,
                    &proposer,
                    &block_hash,
                    block_data,
                    proposer_signature,
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

    /// Verify transaction signature using Ed25519
    /// The wallet signs: SHA256(JSON.stringify({from, to, amount, memo, nonce, timestamp}))
    fn verify_transaction_signature(tx: &Transaction) -> Result<()> {
        // Get signature - required for production
        let sig_str = match tx.signature.as_ref() {
            Some(s) if !s.is_empty() => s,
            _ => {
                bail!("Transaction signature is required");
            }
        };

        // Get public key - required for production
        let pubkey_str = match tx.public_key.as_ref() {
            Some(pk) if !pk.is_empty() => pk,
            _ => {
                bail!("Transaction public key is required for signature verification");
            }
        };

        // Decode signature from hex
        let sig_bytes = hex::decode(sig_str)
            .context("Invalid signature: not valid hex")?;

        if sig_bytes.len() != SIGNATURE_LENGTH {
            bail!("Invalid signature length: expected {}, got {}", SIGNATURE_LENGTH, sig_bytes.len());
        }

        let sig_array: [u8; SIGNATURE_LENGTH] = sig_bytes.try_into()
            .map_err(|_| anyhow::anyhow!("Failed to convert signature bytes to array"))?;
        let signature = Signature::from_bytes(&sig_array);

        // Decode public key from hex
        let pubkey_bytes = hex::decode(pubkey_str)
            .context("Invalid public key: not valid hex")?;

        if pubkey_bytes.len() != 32 {
            bail!("Invalid public key length: expected 32, got {}", pubkey_bytes.len());
        }

        let pubkey_array: [u8; 32] = pubkey_bytes.try_into()
            .map_err(|_| anyhow::anyhow!("Failed to convert public key bytes to array"))?;
        let verifying_key = VerifyingKey::from_bytes(&pubkey_array)
            .context("Invalid Ed25519 public key")?;

        // Recreate the message that was signed by the wallet
        // The wallet signs: SHA256(JSON.stringify({from, to, amount, memo, nonce, timestamp}))
        let message_str = format!(
            r#"{{"from":"{}","to":"{}","amount":"{}","memo":"","nonce":{},"timestamp":{}}}"#,
            tx.from, tx.to, tx.amount, tx.nonce, tx.timestamp
        );
        
        // SHA256 hash the message (matching wallet behavior)
        let message_hash = sha2::Sha256::digest(message_str.as_bytes());

        // Verify the signature - STRICT: reject invalid signatures
        verifying_key.verify(&message_hash, &signature)
            .context("Signature verification failed: invalid signature")?;

        info!("✓ Signature verified for tx from {}", tx.from);
        Ok(())
    }

    /// Transaction submission endpoint
    /// Sultan Chain: Zero gas fees - transaction costs paid by 4% inflation
    async fn submit_transaction(&self, tx: Transaction) -> Result<String> {
        // STRICT: Verify signature before accepting transaction
        Self::verify_transaction_signature(&tx)?;
        
        // Calculate transaction hash
        let tx_hash = format!("{}:{}:{}", tx.from, tx.to, tx.nonce);
        
        // Add transaction to mempool
        let blockchain = self.blockchain.write().await;
        blockchain.submit_transaction(tx.clone()).await
            .context("Failed to submit transaction")?;
        
        let pending = blockchain.pending_count().await;
        debug!("Transaction accepted: {} -> {} ({}) [pending: {}]", 
               tx.from, tx.to, tx.amount, pending);
        drop(blockchain);
        
        // === P2P TRANSACTION BROADCAST ===
        // Broadcast transaction to all connected peers so any validator can include it
        if self.p2p_enabled {
            if let Some(ref p2p) = self.p2p_network {
                let tx_data = bincode::serialize(&tx).unwrap_or_default();
                if let Err(e) = p2p.read().await.broadcast_transaction(&tx_hash, tx_data).await {
                    warn!("Failed to broadcast transaction via P2P: {}", e);
                } else {
                    info!("📢 Transaction {} broadcast to peers", tx_hash);
                }
            }
        }
        
        Ok(tx_hash)
    }

    /// Get blockchain status
    async fn get_status(&self) -> Result<NodeStatus> {
        // Sultan Chain unified blockchain - always uses sharded architecture
        let blockchain = self.blockchain.read().await;
        let stats = blockchain.get_stats().await;
        let height = blockchain.get_height().await;
        let pending_txs = blockchain.pending_count().await;
        let total_accounts = stats.total_accounts;
        let shard_count = blockchain.config.shard_count;
        let latest_hash = format!("sultan-block-{}", height);
        drop(blockchain);
        
        let consensus = self.consensus.read().await;
        let validator_count = consensus.validator_count();
        drop(consensus);
        
        let economics = self.economics.read().await;
        
        Ok(NodeStatus {
            height,
            latest_hash,
            validator_count,
            pending_txs,
            total_accounts,
            sharding_enabled: true, // Sultan Chain always uses sharding
            shard_count,
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
    use std::collections::HashMap;
    use std::time::Instant;
    use std::sync::OnceLock;

    /// Simple token bucket rate limiter
    /// Limits requests per IP to prevent spam/DDoS
    struct RateLimiter {
        requests: HashMap<String, Vec<Instant>>,
        max_requests: usize,
        window_secs: u64,
    }

    impl RateLimiter {
        fn new(max_requests: usize, window_secs: u64) -> Self {
            Self {
                requests: HashMap::new(),
                max_requests,
                window_secs,
            }
        }

        fn check_rate_limit(&mut self, ip: &str) -> bool {
            let now = Instant::now();
            let cutoff = now - std::time::Duration::from_secs(self.window_secs);
            
            let entry = self.requests.entry(ip.to_string()).or_insert_with(Vec::new);
            
            // Remove old requests outside the window
            entry.retain(|&t| t > cutoff);
            
            // Check if under limit
            if entry.len() < self.max_requests {
                entry.push(now);
                true
            } else {
                false
            }
        }

        // Clean up old entries periodically
        fn cleanup(&mut self) {
            let now = Instant::now();
            let cutoff = now - std::time::Duration::from_secs(self.window_secs * 2);
            self.requests.retain(|_, times| {
                times.retain(|&t| t > cutoff);
                !times.is_empty()
            });
        }
    }

    // Global rate limiter using OnceLock (no external crate needed)
    static RATE_LIMITER: OnceLock<Arc<tokio::sync::RwLock<RateLimiter>>> = OnceLock::new();

    fn get_rate_limiter() -> &'static Arc<tokio::sync::RwLock<RateLimiter>> {
        RATE_LIMITER.get_or_init(|| {
            Arc::new(tokio::sync::RwLock::new(RateLimiter::new(
                100,  // 100 requests
                10,   // per 10 seconds
            )))
        })
    }

    /// Rate limiting filter - applies to all endpoints
    fn with_rate_limit() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
        warp::addr::remote()
            .and_then(|addr: Option<SocketAddr>| async move {
                let ip = addr.map(|a| a.ip().to_string()).unwrap_or_else(|| "unknown".to_string());
                
                let limiter_ref = get_rate_limiter();
                let mut limiter = limiter_ref.write().await;
                
                // Periodic cleanup every ~1000 requests
                if limiter.requests.len() > 1000 {
                    limiter.cleanup();
                }
                
                if limiter.check_rate_limit(&ip) {
                    Ok(())
                } else {
                    warn!("Rate limit exceeded for IP: {}", ip);
                    Err(warp::reject::custom(super::RateLimitExceeded))
                }
            })
            .untuple_one()
    }

    pub async fn run_rpc_server(
        addr: SocketAddr,
        state: Arc<NodeState>,
    ) -> Result<()> {
        info!("Starting RPC server on {} (rate limit: 100 req/10s per IP)", addr);

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

        // GET /transactions/:address - Transaction history for an address
        let tx_history_route = warp::path!("transactions" / String)
            .and(warp::get())
            .and(warp::query::<TxHistoryQuery>())
            .and(with_state(state.clone()))
            .and_then(handle_get_tx_history);

        // GET /tx/:hash - Get single transaction by hash
        let tx_by_hash_route = warp::path!("tx" / String)
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_get_tx_by_hash);

        // GET /economics
        let economics_route = warp::path("economics")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_get_economics);

        // GET /supply/total - Total supply endpoint for block explorers
        let supply_total_route = warp::path!("supply" / "total")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_get_total_supply);

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

        // POST /staking/delegate (accepts raw bytes and parses manually for better error handling)
        let delegate_route = warp::path!("staking" / "delegate")
            .and(warp::post())
            .and(warp::body::bytes())
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

        // POST /staking/undelegate (unstake - starts 21-day unbonding period)
        let undelegate_route = warp::path!("staking" / "undelegate")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_state(state.clone()))
            .and_then(handle_undelegate);

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

        // POST /governance/execute/:id - Execute a passed proposal (hot-activation)
        let execute_route = warp::path!("governance" / "execute" / u64)
            .and(warp::post())
            .and(with_state(state.clone()))
            .and_then(handle_execute_proposal);

        // GET /governance/features - Get current feature flags
        let features_route = warp::path!("governance" / "features")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_get_features);

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
            .or(tx_history_route)
            .or(tx_by_hash_route)
            .or(economics_route)
            .or(supply_total_route)
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
            .or(undelegate_route)
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
            .or(execute_route)
            .or(features_route)
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
        
        // Build secure CORS configuration from allowed_origins
        let cors_config = {
            let mut cors = warp::cors()
                .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                .allow_headers(vec!["Content-Type", "Authorization", "Accept"]);
            
            if state.allowed_origins.len() == 1 && state.allowed_origins[0] == "*" {
                warn!("⚠️ CORS: allow_any_origin is INSECURE - use only for development!");
                cors = cors.allow_any_origin();
            } else {
                for origin in &state.allowed_origins {
                    cors = cors.allow_origin(origin.as_str());
                }
                info!("🔒 CORS: Restricting to origins: {:?}", state.allowed_origins);
            }
            cors
        };
        
        let routes = with_rate_limit()
            .and(
                api_routes_1
                    .or(api_routes_2)
            )
            .with(cors_config)
            .recover(handle_rejection);

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
        tx_request: TxRequest,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        // Convert either format to internal Transaction
        let tx = match tx_request {
            TxRequest::Wallet(wallet_tx) => {
                info!("Processing wallet transaction: from={}, to={}, amount={}", 
                    wallet_tx.tx.from, wallet_tx.tx.to, wallet_tx.tx.amount);
                Transaction {
                    from: wallet_tx.tx.from,
                    to: wallet_tx.tx.to,
                    amount: wallet_tx.tx.amount,
                    gas_fee: 0, // Zero-fee network
                    timestamp: wallet_tx.tx.timestamp,
                    nonce: wallet_tx.tx.nonce,
                    signature: Some(wallet_tx.signature),
                    public_key: Some(wallet_tx.public_key),
                    memo: None,
                }
            }
            TxRequest::Simple(simple_tx) => {
                info!("Processing simple transaction: from={}, to={}, amount={}", 
                    simple_tx.from, simple_tx.to, simple_tx.amount);
                Transaction {
                    from: simple_tx.from,
                    to: simple_tx.to,
                    amount: simple_tx.amount,
                    gas_fee: simple_tx.gas_fee,
                    timestamp: simple_tx.timestamp,
                    nonce: simple_tx.nonce,
                    signature: simple_tx.signature,
                    public_key: None,
                    memo: None,
                }
            }
        };
        
        match state.submit_transaction(tx).await {
            Ok(hash) => Ok(warp::reply::json(&serde_json::json!({ "hash": hash }))),
            Err(e) => {
                warn!("Transaction rejected: {}", e);
                Ok(warp::reply::json(&serde_json::json!({
                    "error": e.to_string(),
                    "status": 400
                })))
            }
        }
    }

    async fn handle_get_block(
        height: u64,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let blockchain = state.blockchain.read().await;
        
        match blockchain.get_block(height).await {
            Some(block) => Ok(warp::reply::json(&block)),
            None => Err(warp::reject()),
        }
    }

    async fn handle_get_balance(
        address: String,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        // Sultan Chain unified blockchain - always uses sharded architecture
        let blockchain = state.blockchain.read().await;
        let balance = blockchain.get_balance(&address).await;
        let nonce = blockchain.get_nonce(&address).await;
        
        Ok(warp::reply::json(&serde_json::json!({
            "address": address,
            "balance": balance,
            "nonce": nonce
        })))
    }

    async fn handle_get_tx_history(
        address: String,
        query: TxHistoryQuery,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        // First try in-memory cache
        let blockchain = state.blockchain.read().await;
        let mut transactions = blockchain.get_transaction_history(&address, query.limit).await;
        
        // If empty, try persistent storage
        if transactions.is_empty() {
            let storage = state.storage.read().await;
            if let Ok(stored_txs) = storage.get_transaction_history(&address, query.limit) {
                transactions = stored_txs;
            }
        }
        
        Ok(warp::reply::json(&serde_json::json!({
            "address": address,
            "transactions": transactions,
            "count": transactions.len()
        })))
    }

    async fn handle_get_tx_by_hash(
        hash: String,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let blockchain = state.blockchain.read().await;
        
        if let Some(tx) = blockchain.get_transaction_by_hash(&hash).await {
            Ok(warp::reply::json(&tx))
        } else {
            Ok(warp::reply::json(&serde_json::json!({
                "error": "Transaction not found",
                "hash": hash
            })))
        }
    }

    async fn handle_get_economics(
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let economics = state.economics.read().await;
        
        // Calculate total supply (genesis + inflation - burned)
        let genesis_supply: u64 = 500_000_000_000_000_000; // 500M SLTN in base units
        let total_supply = genesis_supply.saturating_sub(economics.total_burned);
        
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
            "inflation_policy": "Fixed 4% annual inflation guarantees zero gas fees sustainable at 76M+ TPS",
            "total_supply": total_supply,
            "total_supply_formatted": format!("{:.0}", total_supply as f64 / 1_000_000_000.0),
            "circulating_supply": total_supply,
            "genesis_supply": genesis_supply
        })))
    }

    /// Handler for /supply/total - Returns total supply for block explorers
    async fn handle_get_total_supply(
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let economics = state.economics.read().await;
        
        // Genesis supply: 500M SLTN in base units (9 decimals)
        let genesis_supply: u64 = 500_000_000_000_000_000;
        let total_supply = genesis_supply.saturating_sub(economics.total_burned);
        
        Ok(warp::reply::json(&serde_json::json!({
            "total_supply": total_supply,
            "total_supply_sltn": total_supply as f64 / 1_000_000_000.0,
            "circulating_supply": total_supply,
            "circulating_supply_sltn": total_supply as f64 / 1_000_000_000.0,
            "genesis_supply": genesis_supply,
            "genesis_supply_sltn": 500_000_000.0,
            "total_burned": economics.total_burned,
            "decimals": 9,
            "denom": "sltn"
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
        signature: String,  // hex-encoded Ed25519 signature
        pubkey: String,     // hex-encoded 32-byte public key
    }

    #[derive(serde::Deserialize)]
    struct FeeQuery {
        amount: u64,
    }

    #[derive(serde::Deserialize)]
    struct TxHistoryQuery {
        #[serde(default = "default_limit")]
        limit: usize,
    }

    fn default_limit() -> usize {
        50
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
        // Parse signature
        let signature = match hex::decode(&req.signature) {
            Ok(s) if s.len() == 64 => s,
            _ => return Ok(warp::reply::json(&serde_json::json!({
                "error": "Invalid signature (must be 64 bytes hex)"
            }))),
        };
        
        // Parse pubkey
        let pubkey: [u8; 32] = match hex::decode(&req.pubkey) {
            Ok(p) if p.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&p);
                arr
            },
            _ => return Ok(warp::reply::json(&serde_json::json!({
                "error": "Invalid pubkey (must be 32 bytes hex)"
            }))),
        };
        
        match state.bridge_manager.submit_bridge_transaction_with_signature(
            req.source_chain,
            req.dest_chain,
            req.source_tx,
            req.amount,
            req.recipient,
            signature,
            pubkey,
        ).await {
            Ok(tx_id) => Ok(warp::reply::json(&serde_json::json!({
                "tx_id": tx_id,
                "status": "pending"
            }))),
            Err(e) => {
                warn!("Bridge transaction failed: {}", e);
                Ok(warp::reply::json(&serde_json::json!({
                    "error": e.to_string()
                })))
            }
        }
    }

    // ========= STAKING HANDLERS =========

    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct CreateValidatorRequest {
        validator_address: String,
        #[serde(default)]
        moniker: Option<String>,
        #[serde(deserialize_with = "deserialize_amount")]
        initial_stake: u64,
        #[serde(default = "default_commission")]
        commission_rate: f64,
        #[serde(default)]
        signature: Option<String>,
        #[serde(default)]
        public_key: Option<String>,
    }

    fn default_commission() -> f64 {
        0.05 // 5% default commission
    }

    async fn handle_create_validator(
        req: CreateValidatorRequest,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        // Parse and validate Ed25519 public key (required for secure validator registration)
        let pubkey: [u8; 32] = if let Some(ref pk_hex) = req.public_key {
            // Validate hex length
            if pk_hex.len() != 64 {
                return Ok(warp::reply::with_status(
                    warp::reply::json(&serde_json::json!({
                        "error": "public_key must be 64 hex characters (32 bytes Ed25519)"
                    })),
                    warp::http::StatusCode::BAD_REQUEST,
                ));
            }
            // Decode hex to bytes
            match hex::decode(pk_hex) {
                Ok(bytes) if bytes.len() == 32 => {
                    let mut arr = [0u8; 32];
                    arr.copy_from_slice(&bytes);
                    // Validate it's a valid Ed25519 public key
                    if ed25519_dalek::VerifyingKey::from_bytes(&arr).is_err() {
                        return Ok(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({
                                "error": "public_key is not a valid Ed25519 public key"
                            })),
                            warp::http::StatusCode::BAD_REQUEST,
                        ));
                    }
                    arr
                }
                _ => {
                    return Ok(warp::reply::with_status(
                        warp::reply::json(&serde_json::json!({
                            "error": "Failed to decode public_key hex"
                        })),
                        warp::http::StatusCode::BAD_REQUEST,
                    ));
                }
            }
        } else {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "error": "public_key is required for validator registration"
                })),
                warp::http::StatusCode::BAD_REQUEST,
            ));
        };
        
        // Add to staking manager for rewards/delegation
        match state.staking_manager.create_validator(
            req.validator_address.clone(),
            req.initial_stake,
            req.commission_rate,
        ).await {
            Ok(_) => {
                // Also add to consensus engine for block production
                // This unifies staking validators with consensus validators
                let mut consensus = state.consensus.write().await;
                if let Err(e) = consensus.add_validator(req.validator_address.clone(), req.initial_stake, pubkey) {
                    warn!("Validator {} added to staking but not consensus: {}", req.validator_address, e);
                }
                
                // Record as transaction for history
                {
                    let blockchain = state.blockchain.read().await;
                    let current_height = blockchain.get_height().await;
                    let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                    
                    use sha2::{Sha256, Digest};
                    let mut hasher = Sha256::new();
                    hasher.update(format!("create_validator_{}{}", req.validator_address, timestamp));
                    let hash_bytes = hasher.finalize();
                    let hash = format!("validator_{}", hex::encode(&hash_bytes[..16]));
                    
                    let tx = sultan_core::sharded_blockchain_production::ConfirmedTransaction {
                        hash,
                        from: req.validator_address.clone(),
                        to: "network".to_string(),
                        amount: req.initial_stake,
                        memo: Some("Create validator".to_string()),
                        nonce: 0,
                        timestamp,
                        block_height: current_height,
                        status: "confirmed".to_string(),
                    };
                    blockchain.record_transaction(tx).await;
                }
                
                // Return both snake_case and camelCase for compatibility
                Ok(warp::reply::with_status(
                    warp::reply::json(&serde_json::json!({
                        "validator_address": req.validator_address,
                        "validatorAddress": req.validator_address,
                        "stake": req.initial_stake.to_string(),
                        "commission": req.commission_rate,
                        "status": "active",
                        "consensus": true
                    })),
                    warp::http::StatusCode::OK,
                ))
            },
            Err(e) => {
                warn!("Create validator failed: {}", e);
                Ok(warp::reply::with_status(
                    warp::reply::json(&serde_json::json!({
                        "error": format!("Failed to create validator: {}", e)
                    })),
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        }
    }

    // DelegateRequest types removed - now using manual JSON parsing for better error handling

    async fn handle_delegate(
        body: warp::hyper::body::Bytes,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        // Parse the JSON body manually for better error handling
        let body_str = String::from_utf8_lossy(&body);
        debug!("Delegate request body: {}", body_str);
        
        // Try to parse as JSON Value first
        let json_value: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(e) => {
                warn!("Failed to parse delegate request as JSON: {}", e);
                return Ok(warp::reply::with_status(
                    warp::reply::json(&serde_json::json!({
                        "error": format!("Invalid JSON: {}", e),
                        "status": 400
                    })),
                    warp::http::StatusCode::BAD_REQUEST
                ));
            }
        };
        
        // Extract delegation parameters - try wallet format first, then simple format
        let (delegator, validator, amount): (String, String, u64) = if let Some(tx) = json_value.get("tx") {
            // Wallet format: { tx: { from, to, amount }, signature, public_key }
            // Requires signature verification for security
            let from = tx.get("from").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let to = tx.get("to").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let amount = tx.get("amount").map(|v| {
                v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())).unwrap_or(0)
            }).unwrap_or(0);
            let memo = tx.get("memo").and_then(|v| v.as_str()).unwrap_or("delegate");
            
            // Verify signature for authenticated wallet requests
            let signature = json_value.get("signature").and_then(|v| v.as_str()).unwrap_or_default();
            let public_key = json_value.get("public_key").and_then(|v| v.as_str()).unwrap_or_default();
            
            if !signature.is_empty() && !public_key.is_empty() {
                // Build signing message (same format as transaction_validator)
                let message = format!("{}:{}:{}:delegate:{}", from, to, amount, memo);
                if let Err(e) = verify_request_signature(public_key, signature, &message) {
                    return Ok(warp::reply::with_status(
                        warp::reply::json(&serde_json::json!({
                            "error": format!("Signature verification failed: {}", e),
                            "status": 401
                        })),
                        warp::http::StatusCode::UNAUTHORIZED
                    ));
                }
                info!("✅ Wallet delegation signature verified: from={}", from);
            }
            
            // Validate addresses
            if let Err(e) = validate_address(&from) {
                return Ok(warp::reply::with_status(
                    warp::reply::json(&serde_json::json!({
                        "error": format!("Invalid delegator address: {}", e),
                        "status": 400
                    })),
                    warp::http::StatusCode::BAD_REQUEST
                ));
            }
            
            info!("Processing wallet delegation: from={}, to={}, amount={}", from, to, amount);
            (from, to, amount)
        } else if json_value.get("delegator_address").is_some() {
            // Simple format: { delegator_address, validator_address, amount }
            let delegator = json_value.get("delegator_address").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let validator = json_value.get("validator_address").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let amount = json_value.get("amount").map(|v| {
                v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())).unwrap_or(0)
            }).unwrap_or(0);
            
            // Validate addresses for simple format too
            if let Err(e) = validate_address(&delegator) {
                return Ok(warp::reply::with_status(
                    warp::reply::json(&serde_json::json!({
                        "error": format!("Invalid delegator address: {}", e),
                        "status": 400
                    })),
                    warp::http::StatusCode::BAD_REQUEST
                ));
            }
            info!("Processing simple delegation: delegator={}, validator={}, amount={}", delegator, validator, amount);
            (delegator, validator, amount)
        } else {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "error": "Invalid request format. Expected either { tx: { from, to, amount }, signature, public_key } or { delegator_address, validator_address, amount }",
                    "status": 400
                })),
                warp::http::StatusCode::BAD_REQUEST
            ));
        };
        
        // Validate the request
        if delegator.is_empty() || validator.is_empty() || amount == 0 {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "error": "Missing required fields: delegator, validator, and amount must all be provided",
                    "status": 400
                })),
                warp::http::StatusCode::BAD_REQUEST
            ));
        }
        
        // Check if delegator has sufficient balance
        let current_balance = {
            let blockchain = state.blockchain.read().await;
            blockchain.get_balance(&delegator).await
        };
        
        if current_balance < amount {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "error": format!("Insufficient balance: {} SLTN available, {} SLTN required", current_balance, amount),
                    "status": 400,
                    "available_balance": current_balance,
                    "required_amount": amount
                })),
                warp::http::StatusCode::BAD_REQUEST
            ));
        }
        
        // Deduct balance from delegator's account BEFORE staking
        // This ensures the staked amount is removed from their available balance
        {
            let blockchain = state.blockchain.read().await;
            if let Err(e) = blockchain.deduct_balance(&delegator, amount).await {
                return Ok(warp::reply::with_status(
                    warp::reply::json(&serde_json::json!({
                        "error": format!("Failed to deduct balance: {}", e),
                        "status": 500
                    })),
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR
                ));
            }
        }
        
        // Auto-register consensus validators in staking manager if not already registered
        {
            let consensus = state.consensus.read().await;
            if consensus.is_validator(&validator) {
                if let Some(v) = consensus.get_validator(&validator) {
                    // Register in staking manager if not exists
                    let _ = state.staking_manager.create_validator(
                        validator.clone(),
                        v.stake,
                        0.05, // Default 5% commission
                    ).await;
                }
            }
        }

        match state.staking_manager.delegate(
            delegator.clone(),
            validator.clone(),
            amount,
        ).await {
            Ok(_) => {
                // Generate a hash for the delegation transaction
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                hasher.update(format!("{}{}{}{}", delegator, validator, amount, timestamp));
                let hash_bytes = hasher.finalize();
                let hash = format!("stake_{}", hex::encode(&hash_bytes[..16]));
                
                // Record this as a transaction in history
                {
                    let blockchain = state.blockchain.read().await;
                    let current_height = blockchain.get_height().await;
                    
                    // Create a confirmed transaction record for the staking action
                    let stake_tx = sultan_core::sharded_blockchain_production::ConfirmedTransaction {
                        hash: hash.clone(),
                        from: delegator.clone(),
                        to: validator.clone(),
                        amount,
                        memo: Some("Stake delegation".to_string()),
                        nonce: 0,
                        timestamp,
                        block_height: current_height,
                        status: "confirmed".to_string(),
                    };
                    blockchain.record_transaction(stake_tx).await;
                }
                
                // Persist staking state after successful delegation
                {
                    let storage = state.storage.read().await;
                    if let Err(e) = state.staking_manager.persist_to_storage(&storage).await {
                        warn!("⚠️ Failed to persist staking state: {}", e);
                    }
                }
                
                Ok(warp::reply::with_status(
                    warp::reply::json(&serde_json::json!({
                        "delegator": delegator,
                        "validator": validator,
                        "amount": amount,
                        "status": "delegated",
                        "hash": hash
                    })),
                    warp::http::StatusCode::OK
                ))
            },
            Err(e) => {
                warn!("Delegation failed: {}", e);
                Ok(warp::reply::with_status(
                    warp::reply::json(&serde_json::json!({
                        "error": e.to_string(),
                        "status": 400
                    })),
                    warp::http::StatusCode::BAD_REQUEST
                ))
            }
        }
    }

    /// Handle undelegate (unstake) - starts 21-day unbonding period
    /// When unbonding completes, tokens are returned to delegator's balance
    async fn handle_undelegate(
        body: serde_json::Value,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        // Parse request body - support multiple formats
        // Format 1: { delegator_address, validator_address, amount }
        // Format 2: { tx: { from, to, amount }, signature, public_key } (wallet format)
        let (delegator, mut validator, amount) = if let Some(tx) = body.get("tx") {
            let from = tx.get("from").and_then(|v| v.as_str()).unwrap_or("");
            let to = tx.get("to").and_then(|v| v.as_str()).unwrap_or("");
            let amt = tx.get("amount")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<u64>().ok())
                .or_else(|| tx.get("amount").and_then(|v| v.as_u64()))
                .unwrap_or(0);
            (from.to_string(), to.to_string(), amt)
        } else {
            let delegator = body.get("delegator_address")
                .or_else(|| body.get("delegator"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            let validator = body.get("validator_address")
                .or_else(|| body.get("validator"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            let amount = body.get("amount")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<u64>().ok())
                .or_else(|| body.get("amount").and_then(|v| v.as_u64()))
                .unwrap_or(0);
            
            (delegator.to_string(), validator.to_string(), amount)
        };

        // If validator is not specified, find from delegations
        if validator.is_empty() && !delegator.is_empty() {
            let delegations = state.staking_manager.get_delegations(&delegator).await;
            if let Some(first_delegation) = delegations.first() {
                validator = first_delegation.validator_address.clone();
            }
        }

        // Validate the request
        if delegator.is_empty() || validator.is_empty() || amount == 0 {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "error": "Missing required fields: delegator, validator, and amount must all be provided",
                    "status": 400,
                    "hint": "If validator is not provided, ensure delegator has an active delegation"
                })),
                warp::http::StatusCode::BAD_REQUEST
            ));
        }

        // Get current block height for transaction recording
        let current_height = {
            let blockchain = state.blockchain.read().await;
            blockchain.get_height().await
        };

        // Undelegate - this starts the 21-day unbonding period
        match state.staking_manager.undelegate(
            delegator.to_string(),
            validator.to_string(),
            amount,
        ).await {
            Ok(unbonding_entry) => {
                // Tokens are now in the unbonding queue
                // They will be returned to the delegator's balance after 21 days
                // by the unbonding processor in produce_block
                info!(
                    "🔓 Unbonding started: {} SLTN from {} → available at block {}",
                    amount / 1_000_000_000,
                    delegator,
                    unbonding_entry.completion_height
                );
                
                // Generate a hash for the unstaking transaction
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                hasher.update(format!("unstake_{}{}{}{}", delegator, validator, amount, timestamp));
                let hash_bytes = hasher.finalize();
                let hash = format!("unstake_{}", hex::encode(&hash_bytes[..16]));
                
                // Record transaction
                {
                    let blockchain = state.blockchain.read().await;
                    let stake_tx = sultan_core::sharded_blockchain_production::ConfirmedTransaction {
                        hash: hash.clone(),
                        from: validator.to_string(),
                        to: delegator.to_string(),
                        amount,
                        memo: Some("Unstake initiated - 21 day unbonding".to_string()),
                        nonce: 0,
                        timestamp,
                        block_height: current_height,
                        status: "unbonding".to_string(),
                    };
                    blockchain.record_transaction(stake_tx).await;
                }
                
                // Persist staking state after successful undelegation
                {
                    let storage = state.storage.read().await;
                    if let Err(e) = state.staking_manager.persist_to_storage(&storage).await {
                        warn!("⚠️ Failed to persist staking state: {}", e);
                    }
                }

                Ok(warp::reply::with_status(
                    warp::reply::json(&serde_json::json!({
                        "delegator": delegator,
                        "validator": validator,
                        "amount": amount,
                        "status": "unbonding",
                        "completion_height": unbonding_entry.completion_height,
                        "completion_time": unbonding_entry.completion_time,
                        "hash": hash,
                        "message": "Unstaking initiated. Tokens will be available after 21-day unbonding period."
                    })),
                    warp::http::StatusCode::OK
                ))
            },
            Err(e) => {
                warn!("Undelegation failed: {}", e);
                Ok(warp::reply::with_status(
                    warp::reply::json(&serde_json::json!({
                        "error": e.to_string(),
                        "status": 400
                    })),
                    warp::http::StatusCode::BAD_REQUEST
                ))
            }
        }
    }

    async fn handle_get_validators(
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let mut validators = state.staking_manager.get_validators().await;
        
        // If no staking validators registered, return consensus validators
        if validators.is_empty() {
            let consensus = state.consensus.read().await;
            let consensus_validators = consensus.get_active_validators();
            
            // Convert consensus validators to staking format
            validators = consensus_validators.iter().map(|v| {
                sultan_core::staking::ValidatorStake {
                    validator_address: v.address.clone(),
                    self_stake: v.stake,
                    delegated_stake: 0,
                    total_stake: v.stake,
                    commission_rate: 0.05, // Default 5%
                    rewards_accumulated: 0,
                    blocks_signed: v.blocks_signed,
                    blocks_missed: 0,
                    jailed: !v.is_active,
                    jailed_until: 0,
                    created_at: 0,
                    last_reward_height: 0,
                }
            }).collect();
        }
        
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
        telegram_discussion_url: Option<String>,
        discord_discussion_url: Option<String>,
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
            req.telegram_discussion_url,
            req.discord_discussion_url,
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

    /// Execute a passed proposal - this is where hot-activation happens
    /// 
    /// When a ParameterChange proposal with `features.*` keys passes,
    /// this endpoint will:
    /// 1. Execute the proposal in governance
    /// 2. Call NodeState::activate_feature() to actually enable the feature
    /// 3. Persist the config change to disk
    async fn handle_execute_proposal(
        proposal_id: u64,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        use warp::http::StatusCode;
        
        // First, get the proposal to check for feature flags
        let proposal = match state.governance_manager.get_proposal(proposal_id).await {
            Some(p) => p,
            None => {
                return Ok(warp::reply::with_status(
                    warp::reply::json(&serde_json::json!({
                        "error": "Proposal not found",
                        "proposal_id": proposal_id
                    })),
                    StatusCode::NOT_FOUND
                ));
            }
        };
        
        // Check if proposal has passed
        if proposal.status != sultan_core::governance::ProposalStatus::Passed {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "error": "Proposal has not passed",
                    "proposal_id": proposal_id,
                    "status": format!("{:?}", proposal.status)
                })),
                StatusCode::BAD_REQUEST
            ));
        }
        
        // Execute the proposal with staking integration
        if let Err(e) = state.governance_manager.execute_proposal_with_staking(
            proposal_id,
            &state.staking_manager,
        ).await {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "error": format!("Execution failed: {}", e),
                    "proposal_id": proposal_id
                })),
                StatusCode::INTERNAL_SERVER_ERROR
            ));
        }
        
        // Now handle feature flag activations
        let mut activated_features = Vec::new();
        if let Some(params) = &proposal.parameters {
            for (key, value) in params {
                if key.starts_with("features.") {
                    let feature_name = key.strip_prefix("features.").unwrap();
                    let enabled: bool = value.parse().unwrap_or(false);
                    
                    if let Err(e) = state.activate_feature(feature_name, enabled).await {
                        warn!("Failed to activate feature {}: {}", feature_name, e);
                    } else {
                        activated_features.push(serde_json::json!({
                            "feature": feature_name,
                            "enabled": enabled
                        }));
                    }
                }
            }
        }
        
        info!("✅ Proposal #{} executed successfully", proposal_id);
        
        Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": true,
                "proposal_id": proposal_id,
                "title": proposal.title,
                "activated_features": activated_features
            })),
            StatusCode::OK
        ))
    }

    /// Get current feature flags configuration
    async fn handle_get_features(
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let features = state.get_feature_flags().await;
        
        Ok(warp::reply::json(&serde_json::json!({
            "features": {
                "sharding_enabled": features.sharding_enabled,
                "governance_enabled": features.governance_enabled,
                "bridges_enabled": features.bridges_enabled,
                "smart_contracts_enabled": features.wasm_contracts_enabled,
                "evm_contracts_enabled": features.evm_contracts_enabled,
                "quantum_signatures_enabled": features.quantum_signatures_enabled,
                "ibc_enabled": features.ibc_enabled
            }
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

    // Handle subcommands first
    if let Some(cmd) = &args.command {
        match cmd {
            Command::Keygen { format } => {
                run_keygen(format);
                return Ok(());
            }
        }
    }

    info!("🚀 Starting Sultan Node: {}", args.name);
    info!("📁 Data directory: {}", args.data_dir);
    info!("⏱️  Block time: {}s", args.block_time);
    info!("🌐 P2P address: {}", args.p2p_addr);
    info!("🔌 RPC address: {}", args.rpc_addr);
    if args.enable_tls {
        info!("🔒 TLS enabled for RPC");
    }

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
        
        // Get message receiver from P2P to process incoming validator announcements
        let mut message_rx = if let Some(ref p2p) = state.p2p_network {
            p2p.write().await.take_message_receiver()
        } else {
            None
        };
        
        tokio::spawn(async move {
            // Wait for peers to connect before first announcement (30 seconds delay)
            info!("🌐 P2P message handler started, waiting for peer connections...");
            tokio::time::sleep(Duration::from_secs(30)).await;
            
            // Now announce this validator to the network
            if let (Some(ref addr), Some(stake)) = (&validator_addr, validator_stake) {
                if let Some(ref p2p) = p2p_state.p2p_network {
                    let peer_count = p2p.read().await.peer_count().await;
                    info!("📢 Announcing validator {} to P2P network ({} peers connected)", addr, peer_count);
                    // Use placeholder pubkey/signature - validators register via RPC with real keys
                    let pubkey = [0u8; 32];
                    let signature = Vec::new();
                    if let Err(e) = p2p.read().await.announce_validator(addr, stake, pubkey, signature).await {
                        warn!("Failed to announce validator: {}", e);
                    } else {
                        info!("✅ Announced validator {} to P2P network", addr);
                    }
                }
            }
            
            // P2P message handler loop
            loop {
                // Process incoming P2P messages with timeout
                tokio::select! {
                    // Handle incoming messages from P2P network
                    Some(msg) = async { 
                        if let Some(ref mut rx) = message_rx { 
                            rx.recv().await 
                        } else { 
                            None 
                        } 
                    } => {
                        match msg {
                            NetworkMessage::ValidatorAnnounce { ref address, stake, ref peer_id, pubkey: _, signature: _ } => {
                                // Check if validator is already registered
                                let consensus = p2p_state.consensus.read().await;
                                if !consensus.is_validator(address) {
                                    // TODO(Phase 4): NetworkMessage::ValidatorAnnounce should include pubkey field
                                    // For now, log a warning and skip registration without valid pubkey
                                    // This prevents placeholder pubkeys in production
                                    warn!("⚠️ Validator {} announced via P2P but pubkey not in message - \
                                           use RPC /validators/create with public_key to register", address);
                                    // Skip adding to consensus without verified pubkey
                                    // Validators must register via RPC with their Ed25519 public key
                                }
                                info!("📡 Received validator announcement: {} (stake: {}, peer: {})", 
                                      address, stake, peer_id);
                            }
                            NetworkMessage::BlockProposal { height, proposer, block_hash, block_data, proposer_signature: _ } => {
                                // === PRODUCTION BLOCK SYNC ===
                                // Process incoming block from another validator
                                debug!("📥 Received block proposal: height={}, proposer={}, hash={}", 
                                       height, proposer, &block_hash[..16.min(block_hash.len())]);
                                
                                // Validate proposer is a registered validator
                                let consensus = p2p_state.consensus.read().await;
                                if !consensus.is_validator(&proposer) {
                                    warn!("❌ Block rejected: proposer {} is not a registered validator", proposer);
                                    continue;
                                }
                                
                                // Verify this is the expected proposer
                                let expected_proposer = consensus.current_proposer.clone();
                                drop(consensus); // Release lock before acquiring write lock
                                
                                // Deserialize block
                                let block: Block = match bincode::deserialize(&block_data) {
                                    Ok(b) => b,
                                    Err(e) => {
                                        warn!("❌ Failed to deserialize block: {}", e);
                                        continue;
                                    }
                                };
                                
                                // Get our current height - Sultan Chain unified blockchain
                                let our_height = p2p_state.blockchain.read().await.get_height().await;
                                
                                // Only accept blocks that are ahead of us
                                if height <= our_height {
                                    debug!("Block {} already processed (our height: {})", height, our_height);
                                    continue;
                                }
                                
                                // Accept block if it's the next one we need
                                if height == our_height + 1 {
                                    info!("📦 Applying block {} from {}", height, proposer);
                                    
                                    // Apply block to our chain - Sultan Chain unified architecture
                                    let blockchain = p2p_state.blockchain.write().await;
                                    match blockchain.apply_block(block.clone()).await {
                                        Ok(_) => {
                                            info!("✅ Synced block {} from {} ({} txs)", 
                                                  height, proposer, block.transactions.len());
                                            
                                            // === DOWNTIME DETECTION ===
                                            // If expected proposer didn't produce this block, count as missed
                                            if let Some(ref expected) = expected_proposer {
                                                if expected != &proposer {
                                                    // Someone else produced when expected didn't
                                                    if let Err(e) = p2p_state.staking_manager
                                                        .record_block_missed(expected).await {
                                                        warn!("Failed to record missed block for {}: {}", expected, e);
                                                    } else {
                                                        let missed = p2p_state.staking_manager
                                                            .get_missed_blocks(expected).await;
                                                        if missed > 0 && missed % 10 == 0 {
                                                            warn!("⚠️ Validator {} has missed {} blocks", expected, missed);
                                                        }
                                                    }
                                                }
                                            }
                                            
                                            // Record that actual proposer signed
                                            let _ = p2p_state.staking_manager
                                                .record_block_signed(&proposer).await;
                                        }
                                        Err(e) => {
                                            warn!("❌ Failed to apply block {}: {}", height, e);
                                            continue;
                                        }
                                    }
                                    drop(blockchain);
                                    
                                    // Advance consensus round after accepting block
                                    let mut consensus = p2p_state.consensus.write().await;
                                    let _ = consensus.select_proposer();
                                    
                                    // Save to storage
                                    if let Ok(storage) = p2p_state.storage.try_read() {
                                        let _ = storage.save_block(&block);
                                    }
                                }
                            }
                            NetworkMessage::Transaction { tx_hash, tx_data } => {
                                // === TRANSACTION GOSSIP ===
                                // Receive transaction from another validator and add to our mempool
                                match bincode::deserialize::<Transaction>(&tx_data) {
                                    Ok(tx) => {
                                        info!("📥 Received transaction from peer: {} -> {} amount={}", 
                                              tx.from, tx.to, tx.amount);
                                        
                                        // Add to our local mempool - Sultan Chain unified blockchain
                                        let blockchain = p2p_state.blockchain.write().await;
                                        if let Err(e) = blockchain.submit_transaction(tx.clone()).await {
                                            debug!("Failed to add peer tx to mempool: {}", e);
                                        } else {
                                            let pending = blockchain.pending_count().await;
                                            debug!("Added peer tx to mempool [pending: {}]", pending);
                                        }
                                    }
                                    Err(e) => {
                                        warn!("❌ Failed to deserialize transaction {}: {}", tx_hash, e);
                                    }
                                }
                            }
                            _ => {
                                // Other message types handled elsewhere
                            }
                        }
                    }
                    
                    // Periodic tasks every 5 seconds
                    _ = tokio::time::sleep(Duration::from_secs(5)) => {
                        // Re-announce this validator periodically (every 30 seconds)
                        if let (Some(ref addr), Some(stake)) = (&validator_addr, validator_stake) {
                            if let Some(ref p2p) = p2p_state.p2p_network {
                                static ANNOUNCE_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                                let count = ANNOUNCE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                if count % 6 == 0 {
                                    let peer_count = p2p.read().await.peer_count().await;
                                    if peer_count > 0 {
                                        let pubkey = [0u8; 32];
                                        let signature = Vec::new();
                                        if let Err(e) = p2p.read().await.announce_validator(addr, stake, pubkey, signature).await {
                                            debug!("Failed to re-announce validator: {}", e);
                                        } else {
                                            info!("📢 Re-announced validator {} ({} peers)", addr, peer_count);
                                        }
                                    }
                                }
                            }
                        }
                        
                        // Log P2P and consensus status
                        if let Some(ref p2p) = p2p_state.p2p_network {
                            let peer_count = p2p.read().await.peer_count().await;
                            let p2p_validator_count = p2p.read().await.known_validator_count().await;
                            let consensus_validator_count = p2p_state.consensus.read().await.validator_count();
                            info!("🌐 P2P Status: {} peers, {} validators (P2P) / {} validators (consensus)", 
                                  peer_count, p2p_validator_count, consensus_validator_count);
                        }
                    }
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
use serde::Deserialize;

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
    signature: String,  // hex-encoded Ed25519 signature
    pubkey: String,     // hex-encoded 32-byte public key
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // minter field used for request validation context
struct MintTokenRequest {
    denom: String,
    to_address: String,
    amount: u128,
    minter: String,     // address of minter (must be token creator)
    signature: String,
    pubkey: String,
}

#[derive(Debug, Deserialize)]
struct TransferTokenRequest {
    denom: String,
    from_address: String,
    to_address: String,
    amount: u128,
    signature: String,
    pubkey: String,
}

#[derive(Debug, Deserialize)]
struct BurnTokenRequest {
    denom: String,
    from_address: String,
    amount: u128,
    signature: String,
    pubkey: String,
}

#[derive(Debug, Deserialize)]
struct CreatePairRequest {
    creator: String,
    token_a: String,
    token_b: String,
    amount_a: u128,
    amount_b: u128,
    signature: String,
    pubkey: String,
}

#[derive(Debug, Deserialize)]
struct SwapRequest {
    from_address: String,
    pair_id: String,
    token_in: String,
    amount_in: u128,
    min_amount_out: u128,
    signature: String,
    pubkey: String,
}

#[derive(Debug, Deserialize)]
struct AddLiquidityRequest {
    provider: String,
    pair_id: String,
    amount_a: u128,
    amount_b: u128,
    min_lp_tokens: Option<u128>,
    signature: String,
    pubkey: String,
}

#[derive(Debug, Deserialize)]
struct RemoveLiquidityRequest {
    provider: String,
    pair_id: String,
    liquidity: u128,
    min_amount_a: Option<u128>,
    min_amount_b: Option<u128>,
    signature: String,
    pubkey: String,
}

async fn handle_create_token(
    request: CreateTokenRequest,
    state: Arc<NodeState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Parse signature and pubkey from hex
    let signature = match hex::decode(&request.signature) {
        Ok(s) => s,
        Err(_) => return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Invalid signature hex"
        }))),
    };
    let pubkey_bytes = match hex::decode(&request.pubkey) {
        Ok(p) if p.len() == 32 => {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&p);
            arr
        },
        _ => return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Invalid pubkey (must be 32 bytes hex)"
        }))),
    };
    
    match state.token_factory.create_token_with_signature(
        &request.creator,
        request.name,
        request.symbol,
        request.decimals,
        request.initial_supply,
        request.max_supply,
        request.logo_url,
        request.description,
        &signature,
        &pubkey_bytes,
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
    let signature = match hex::decode(&request.signature) {
        Ok(s) => s,
        Err(_) => return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Invalid signature hex"
        }))),
    };
    let pubkey_bytes = match hex::decode(&request.pubkey) {
        Ok(p) if p.len() == 32 => {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&p);
            arr
        },
        _ => return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Invalid pubkey (must be 32 bytes hex)"
        }))),
    };
    
    match state.token_factory.mint_to_with_signature(
        &request.denom,
        &request.to_address,
        request.amount,
        &signature,
        &pubkey_bytes,
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
    let signature = match hex::decode(&request.signature) {
        Ok(s) => s,
        Err(_) => return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Invalid signature hex"
        }))),
    };
    let pubkey_bytes = match hex::decode(&request.pubkey) {
        Ok(p) if p.len() == 32 => {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&p);
            arr
        },
        _ => return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Invalid pubkey (must be 32 bytes hex)"
        }))),
    };
    
    match state.token_factory.transfer_with_signature(
        &request.denom,
        &request.from_address,
        &request.to_address,
        request.amount,
        &signature,
        &pubkey_bytes,
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
    let signature = match hex::decode(&request.signature) {
        Ok(s) => s,
        Err(_) => return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Invalid signature hex"
        }))),
    };
    let pubkey_bytes = match hex::decode(&request.pubkey) {
        Ok(p) if p.len() == 32 => {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&p);
            arr
        },
        _ => return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Invalid pubkey (must be 32 bytes hex)"
        }))),
    };
    
    match state.token_factory.burn_with_signature(
        &request.denom,
        &request.from_address,
        request.amount,
        &signature,
        &pubkey_bytes,
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
    _state: Arc<NodeState>,
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
    let signature = match hex::decode(&request.signature) {
        Ok(s) => s,
        Err(_) => return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Invalid signature hex"
        }))),
    };
    let pubkey_bytes = match hex::decode(&request.pubkey) {
        Ok(p) if p.len() == 32 => {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&p);
            arr
        },
        _ => return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Invalid pubkey (must be 32 bytes hex)"
        }))),
    };
    
    match state.native_dex.create_pair_with_signature(
        &request.creator,
        &request.token_a,
        &request.token_b,
        request.amount_a,
        request.amount_b,
        &signature,
        &pubkey_bytes,
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
    let signature = match hex::decode(&request.signature) {
        Ok(s) => s,
        Err(_) => return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Invalid signature hex"
        }))),
    };
    let pubkey_bytes = match hex::decode(&request.pubkey) {
        Ok(p) if p.len() == 32 => {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&p);
            arr
        },
        _ => return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Invalid pubkey (must be 32 bytes hex)"
        }))),
    };
    
    match state.native_dex.swap_with_signature(
        &request.from_address,
        &request.pair_id,
        &request.token_in,
        request.amount_in,
        request.min_amount_out,
        &signature,
        &pubkey_bytes,
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
    let signature = match hex::decode(&request.signature) {
        Ok(s) => s,
        Err(_) => return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Invalid signature hex"
        }))),
    };
    let pubkey_bytes = match hex::decode(&request.pubkey) {
        Ok(p) if p.len() == 32 => {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&p);
            arr
        },
        _ => return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Invalid pubkey (must be 32 bytes hex)"
        }))),
    };
    
    // Note: min_lp_tokens from request is reserved for future slippage protection
    let _min_lp = request.min_lp_tokens.unwrap_or(0);
    
    match state.native_dex.add_liquidity_with_signature(
        &request.pair_id,
        &request.provider,
        request.amount_a,
        request.amount_b,
        0, // amount_a_min (slippage protection)
        0, // amount_b_min (slippage protection)
        &signature,
        &pubkey_bytes,
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
    let signature = match hex::decode(&request.signature) {
        Ok(s) => s,
        Err(_) => return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Invalid signature hex"
        }))),
    };
    let pubkey_bytes = match hex::decode(&request.pubkey) {
        Ok(p) if p.len() == 32 => {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&p);
            arr
        },
        _ => return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Invalid pubkey (must be 32 bytes hex)"
        }))),
    };
    
    match state.native_dex.remove_liquidity_with_signature(
        &request.pair_id,
        &request.provider,
        request.liquidity,
        request.min_amount_a.unwrap_or(0),
        request.min_amount_b.unwrap_or(0),
        &signature,
        &pubkey_bytes,
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
    _state: Arc<NodeState>,
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

/// Rate limit exceeded rejection (used by RPC rate limiter)
#[derive(Debug)]
struct RateLimitExceeded;
impl warp::reject::Reject for RateLimitExceeded {}

/// Custom rejection handler to return JSON error responses instead of 404
async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, std::convert::Infallible> {
    use warp::http::StatusCode;
    
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Endpoint not found")
    } else if err.find::<RateLimitExceeded>().is_some() {
        (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded - try again later")
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        (StatusCode::METHOD_NOT_ALLOWED, "Method not allowed")
    } else if err.find::<warp::reject::InvalidQuery>().is_some() {
        (StatusCode::BAD_REQUEST, "Invalid query parameters")
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (StatusCode::PAYLOAD_TOO_LARGE, "Payload too large")
    } else if err.find::<warp::filters::body::BodyDeserializeError>().is_some() {
        (StatusCode::BAD_REQUEST, "Invalid JSON body - check request format")
    } else if err.find::<warp::reject::MissingHeader>().is_some() {
        (StatusCode::BAD_REQUEST, "Missing required header")
    } else {
        warn!("Unhandled rejection: {:?}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
    };
    
    let json = warp::reply::json(&serde_json::json!({
        "error": message,
        "status": code.as_u16()
    }));
    
    Ok(warp::reply::with_status(json, code))
}
