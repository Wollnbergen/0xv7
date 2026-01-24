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
use sultan_core::p2p::{P2PNetwork, NetworkMessage, load_or_generate_keypair};
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
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::Aead;

/// Sultan Node CLI Arguments
#[derive(Parser, Debug)]
#[clap(name = "sultan-node")]
#[clap(version = "0.1.0")]
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

    /// Validator Ed25519 secret key (64 hex chars, required if --validator for block signing)
    /// SECURITY: Pass via environment variable SULTAN_VALIDATOR_SECRET instead of CLI for production
    #[clap(long, env = "SULTAN_VALIDATOR_SECRET")]
    validator_secret: Option<String>,

    /// Reward wallet address (where validator APY rewards are sent)
    /// Defaults to genesis wallet if not specified
    #[clap(long)]
    reward_wallet: Option<String>,

    /// Genesis validators (comma-separated list of validator addresses)
    /// All specified validators will be pre-registered at startup
    /// Example: --genesis-validators addr1,addr2,addr3
    #[clap(long)]
    genesis_validators: Option<String>,

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

    /// Protocol fee wallet address (receives 0.1% of DEX swap fees)
    /// Default: genesis treasury wallet
    #[clap(long, default_value = "sultan15g5nwnlemn7zt6rtl7ch46ssvx2ym2v2umm07g")]
    protocol_fee_address: String,

    /// Token creation fee in SLTN (default: 3 SLTN = ~$0.90)
    #[clap(long, default_value = "3")]
    token_creation_fee: u64,

    /// Enable faucet for Phase 1 (disable when DEX/CEX live)
    #[clap(long, default_value = "true")]
    faucet_enabled: bool,

    /// Faucet amount in SLTN per claim (default: 10 SLTN)
    #[clap(long, default_value = "10")]
    faucet_amount: u64,

    /// Subcommand (e.g., keygen)
    #[clap(subcommand)]
    command: Option<Command>,

    /// Path to encrypted validator key file (alternative to --validator-secret)
    #[clap(long)]
    validator_keyfile: Option<String>,

    /// Password for encrypted key file (use env SULTAN_KEY_PASSWORD for security)
    #[clap(long, env = "SULTAN_KEY_PASSWORD")]
    key_password: Option<String>,
}

/// CLI Subcommands
#[derive(Subcommand, Debug)]
enum Command {
    /// Generate a new Ed25519 keypair for validator registration
    Keygen {
        /// Output format (hex, json, encrypted)
        #[clap(long, default_value = "hex")]
        format: String,
        
        /// Output file path (required for encrypted format)
        #[clap(long, short)]
        output: Option<String>,
        
        /// Password for encrypted output (use env for security)
        #[clap(long, env = "SULTAN_KEY_PASSWORD")]
        password: Option<String>,
    },
}

/// Generate and display a new Ed25519 keypair
fn run_keygen(format: &str, output: Option<&str>, password: Option<&str>) {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    
    let secret_hex = hex::encode(signing_key.to_bytes());
    let public_hex = hex::encode(verifying_key.to_bytes());
    
    match format {
        "encrypted" => {
            let output_path = match output {
                Some(p) => p,
                None => {
                    eprintln!("Error: --output required for encrypted format");
                    std::process::exit(1);
                }
            };
            let pwd = match password {
                Some(p) if !p.is_empty() => p,
                _ => {
                    eprintln!("Error: --password or SULTAN_KEY_PASSWORD required for encrypted format");
                    std::process::exit(1);
                }
            };
            
            match NodeState::save_encrypted_key(&signing_key, std::path::Path::new(output_path), pwd) {
                Ok(_) => {
                    println!("╔══════════════════════════════════════════════════════════════════════╗");
                    println!("║              SULTAN L1 ENCRYPTED VALIDATOR KEY                       ║");
                    println!("╠══════════════════════════════════════════════════════════════════════╣");
                    println!("║ PUBLIC KEY (use with --validator-pubkey):                            ║");
                    println!("║ {}  ║", public_hex);
                    println!("╠══════════════════════════════════════════════════════════════════════╣");
                    println!("║ Encrypted key saved to: {:46}   ║", output_path);
                    println!("╚══════════════════════════════════════════════════════════════════════╝");
                    println!();
                    println!("Usage: sultan-node --validator --validator-keyfile {} --key-password <PASSWORD>", output_path);
                }
                Err(e) => {
                    eprintln!("Error saving encrypted key: {}", e);
                    std::process::exit(1);
                }
            }
        }
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
            println!();
            println!("For encrypted key storage, use: sultan-node keygen --format encrypted --output key.enc --password <PASSWORD>");
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
    /// Validator's Ed25519 signing key for block proposals
    validator_signing_key: Option<SigningKey>,
    block_time: u64,
    p2p_enabled: bool,
    /// Allowed CORS origins for RPC security
    allowed_origins: Vec<String>,
    /// Chain configuration with feature flags (for hot-upgrades via governance)
    config: Arc<RwLock<Config>>,
    /// Path to config file for persistence
    config_path: PathBuf,
    /// TLS configuration for secure RPC
    tls_config: Option<TlsConfig>,
}

/// TLS configuration for secure RPC server
#[derive(Clone)]
struct TlsConfig {
    cert_path: String,
    key_path: String,
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
        
        // Track the first genesis wallet (for validator rewards)
        let mut genesis_wallet: Option<String> = None;
        
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
                    
                    // First genesis account becomes the default reward wallet for genesis validators
                    if genesis_wallet.is_none() {
                        genesis_wallet = Some(address.clone());
                        info!("🏦 Genesis treasury wallet: {} (will receive genesis validator APY)", address);
                    }
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
            
            // Parse and validate Ed25519 public key from CLI (or generate ephemeral key for testing)
            let pubkey_array: [u8; 32] = if let Some(validator_pubkey) = args.validator_pubkey.as_ref() {
                // Decode hex pubkey to [u8; 32]
                if validator_pubkey.len() != 64 {
                    bail!("Validator pubkey must be 64 hex characters (32 bytes), got {}", validator_pubkey.len());
                }
                let pubkey_bytes = hex::decode(validator_pubkey)
                    .context("Invalid validator pubkey hex")?;
                let arr: [u8; 32] = pubkey_bytes.try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid validator pubkey length"))?;
                
                // Verify it's a valid Ed25519 public key
                VerifyingKey::from_bytes(&arr)
                    .context("Invalid Ed25519 public key")?;
                
                info!("Running as validator: {} (stake: {}, pubkey: {}...)", 
                    validator_addr, validator_stake, &validator_pubkey[..16]);
                arr
            } else {
                // Generate ephemeral signing key for testing (NOT for production!)
                warn!("⚠️  No --validator-pubkey provided, generating EPHEMERAL key (testing only!)");
                let signing_key = SigningKey::generate(&mut rand::thread_rng());
                let verifying_key = signing_key.verifying_key();
                let pubkey_hex = hex::encode(verifying_key.as_bytes());
                info!("Running as validator: {} (stake: {}, EPHEMERAL pubkey: {}...)", 
                    validator_addr, validator_stake, &pubkey_hex[..16]);
                *verifying_key.as_bytes()
            };
            
            consensus.add_validator(validator_addr.clone(), validator_stake, pubkey_array)
                .context("Failed to add validator")?;
        }

        // Add genesis validators to consensus (allows accepting blocks from all genesis validators)
        if let Some(genesis_vals_str) = &args.genesis_validators {
            let genesis_validators: Vec<String> = genesis_vals_str.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            
            let our_addr = args.validator_address.as_ref().map(|s| s.as_str()).unwrap_or("");
            
            for gv_addr in &genesis_validators {
                // Skip if it's our own address (already added above)
                if gv_addr == our_addr {
                    continue;
                }
                
                // Add with default stake and zero pubkey (will be updated via sync)
                let default_stake = 10_000_000_000_000u64; // 10M SLTN
                let zero_pubkey = [0u8; 32]; // Will be updated when validator syncs
                
                if let Err(e) = consensus.add_validator(gv_addr.clone(), default_stake, zero_pubkey) {
                    debug!("Genesis validator {} already exists or failed: {}", gv_addr, e);
                } else {
                    info!("📋 Pre-registered genesis validator: {}", gv_addr);
                }
            }
            info!("🏛️ {} genesis validators pre-registered", genesis_validators.len());
        }

        // Initialize P2P network if enabled
        let p2p_network = if args.enable_p2p {
            // Load or generate persistent keypair for stable PeerId across restarts
            let data_dir_path = std::path::Path::new(&args.data_dir);
            let keypair = load_or_generate_keypair(data_dir_path)
                .context("Failed to load/generate node keypair")?;
            
            let mut p2p = P2PNetwork::with_keypair(keypair)
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
        
        // If running as validator, register in staking manager with genesis wallet as reward destination
        if args.validator {
            if let Some(validator_addr) = args.validator_address.as_ref() {
                let stake = args.validator_stake.unwrap_or(10_000_000_000_000); // Default 10M SLTN
                
                // Create validator in staking manager (if not already exists from snapshot)
                if staking_manager.get_validator(validator_addr).await.is_err() {
                    if let Err(e) = staking_manager.create_validator(
                        validator_addr.clone(),
                        stake,
                        0.05, // 5% commission
                    ).await {
                        warn!("⚠️ Failed to register validator in staking: {}", e);
                    } else {
                        info!("✅ Validator {} registered in staking system (stake: {} SLTN)", 
                              validator_addr, stake / 1_000_000_000);
                    }
                }
                
                // Set reward wallet for this validator (CLI flag > genesis wallet > validator address)
                let reward_wallet = args.reward_wallet.clone()
                    .or_else(|| genesis_wallet.clone())
                    .unwrap_or_else(|| validator_addr.clone());
                
                if let Err(e) = staking_manager.set_reward_wallet(validator_addr, reward_wallet.clone()).await {
                    warn!("⚠️ Failed to set reward wallet: {}", e);
                } else {
                    info!("💰 Validator {} APY rewards → {}", validator_addr, reward_wallet);
                }
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

        // Create shared TokenFactory with configurable fee and faucet settings
        let data_path = std::path::PathBuf::from(&args.data_dir);
        let token_factory = Arc::new(TokenFactory::with_config(
            Some(data_path.join("tokens")),
            args.token_creation_fee as u128 * 1_000_000, // Convert SLTN to usltn
            args.faucet_enabled,
            args.faucet_amount as u128 * 1_000_000,      // Convert SLTN to usltn
        ));
        
        // Load persisted token state if exists
        if let Err(e) = token_factory.load_from_storage().await {
            warn!("⚠️ Failed to load token state: {}", e);
        }
        
        info!("💰 Token creation fee: {} SLTN", args.token_creation_fee);
        info!("🚰 Faucet: {} (amount: {} SLTN per claim)", 
            if args.faucet_enabled { "ENABLED (Phase 1)" } else { "DISABLED (Phase 2)" },
            args.faucet_amount
        );
        
        // Create BridgeManager with TokenFactory integration for wrapped token minting
        // When a bridge tx is verified, TokenFactory.mint_internal() mints sBTC/sETH/sSOL/sTON
        let bridge_manager = Arc::new(BridgeManager::with_token_factory(
            "sultan1treasury7xj3k2p8n9m5q4r6t8v0w2y4z6a8c0e2g4".to_string(),
            token_factory.clone(),
        ));
        
        // Create NativeDex with protocol fee address and persistence
        let native_dex = Arc::new(NativeDex::with_config(
            token_factory.clone(),
            args.protocol_fee_address.clone(),
            Some(data_path.join("dex")),
        ));
        
        // Load persisted DEX state if exists
        if let Err(e) = native_dex.load_from_storage().await {
            warn!("⚠️ Failed to load DEX state: {}", e);
        }
        
        info!("💰 Protocol fee address: {}", args.protocol_fee_address);
        
        Ok(Self {
            blockchain: blockchain_arc,
            consensus: consensus_arc,
            storage: Arc::new(RwLock::new(storage)),
            economics: Arc::new(RwLock::new(Economics::new())),
            bridge_manager,
            staking_manager,
            governance_manager: Arc::new(GovernanceManager::new()),
            token_factory: token_factory.clone(),
            native_dex,
            p2p_network,
            block_sync_manager,
            validator_address: args.validator_address.clone(),
            validator_signing_key: Self::load_signing_key(&args)?,
            block_time: args.block_time,
            p2p_enabled: args.enable_p2p,
            allowed_origins,
            config: Arc::new(RwLock::new(config)),
            config_path,
            tls_config: if args.enable_tls {
                match (&args.tls_cert, &args.tls_key) {
                    (Some(cert), Some(key)) => Some(TlsConfig {
                        cert_path: cert.clone(),
                        key_path: key.clone(),
                    }),
                    _ => {
                        warn!("⚠️  TLS enabled but cert/key paths not provided");
                        None
                    }
                }
            } else {
                None
            },
        })
    }

    /// Load validator signing key from CLI arg or environment variable
    fn load_signing_key(args: &Args) -> Result<Option<SigningKey>> {
        if !args.validator {
            return Ok(None);
        }

        // Try loading from encrypted keyfile first
        if let Some(ref keyfile_path) = args.validator_keyfile {
            let password = match &args.key_password {
                Some(p) if !p.is_empty() => p.as_str(),
                _ => {
                    bail!("--key-password or SULTAN_KEY_PASSWORD required when using --validator-keyfile");
                }
            };
            
            let signing_key = Self::load_encrypted_key(std::path::Path::new(keyfile_path), password)?;
            
            // Verify public key matches if provided
            if let Some(ref pubkey_hex) = args.validator_pubkey {
                let expected_pubkey = signing_key.verifying_key();
                let expected_hex = hex::encode(expected_pubkey.to_bytes());
                if expected_hex != *pubkey_hex {
                    bail!("Encrypted key pubkey does not match --validator-pubkey!\n  Expected: {}\n  Got: {}", pubkey_hex, expected_hex);
                }
            }
            
            return Ok(Some(signing_key));
        }

        // Fall back to hex secret from CLI/env
        let secret_hex = match &args.validator_secret {
            Some(s) if !s.is_empty() => s.clone(),
            _ => {
                warn!("⚠️  Validator mode enabled but no signing key provided");
                warn!("   Blocks will be proposed but NOT signed (insecure)");
                warn!("   Use --validator-secret, SULTAN_VALIDATOR_SECRET, or --validator-keyfile");
                return Ok(None);
            }
        };

        let secret_bytes = hex::decode(&secret_hex)
            .context("Invalid validator secret: not valid hex")?;
        
        if secret_bytes.len() != 32 {
            bail!("Invalid validator secret: expected 32 bytes (64 hex chars), got {}", secret_bytes.len());
        }

        let secret_array: [u8; 32] = secret_bytes.try_into()
            .map_err(|_| anyhow::anyhow!("Failed to convert secret to array"))?;
        
        let signing_key = SigningKey::from_bytes(&secret_array);
        
        // Verify public key matches if provided
        if let Some(ref pubkey_hex) = args.validator_pubkey {
            let expected_pubkey = signing_key.verifying_key();
            let expected_hex = hex::encode(expected_pubkey.to_bytes());
            if expected_hex != *pubkey_hex {
                bail!("Validator secret key does not match public key!\n  Expected pubkey: {}\n  Got pubkey: {}", pubkey_hex, expected_hex);
            }
            info!("✅ Validator signing key loaded and verified");
        } else {
            info!("✅ Validator signing key loaded (pubkey: {})", 
                  hex::encode(signing_key.verifying_key().to_bytes()));
        }

        Ok(Some(signing_key))
    }

    /// Save validator signing key encrypted with password
    /// 
    /// Uses AES-256-GCM authenticated encryption with HKDF key derivation
    /// Key file format: [12-byte nonce][32-byte salt][encrypted data][16-byte auth tag]
    fn save_encrypted_key(key: &SigningKey, path: &std::path::Path, password: &str) -> Result<()> {
        use rand::RngCore;
        use hkdf::Hkdf;
        use sha2::Sha256;
        
        // Generate random salt and nonce
        let mut salt = [0u8; 32];
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut nonce_bytes);
        
        // Derive encryption key using HKDF
        let hk = Hkdf::<Sha256>::new(Some(&salt), password.as_bytes());
        let mut encryption_key = [0u8; 32];
        hk.expand(b"sultan-validator-key", &mut encryption_key)
            .map_err(|_| anyhow::anyhow!("HKDF expansion failed"))?;
        
        // Encrypt the signing key
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&encryption_key));
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, key.to_bytes().as_slice())
            .map_err(|_| anyhow::anyhow!("Encryption failed"))?;
        
        // Write: nonce || salt || ciphertext (includes auth tag)
        let mut output = Vec::with_capacity(12 + 32 + ciphertext.len());
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&salt);
        output.extend_from_slice(&ciphertext);
        
        std::fs::write(path, &output)
            .context("Failed to write encrypted key file")?;
        
        info!("🔐 Validator key encrypted and saved to {:?}", path);
        Ok(())
    }

    /// Load validator signing key from encrypted file
    fn load_encrypted_key(path: &std::path::Path, password: &str) -> Result<SigningKey> {
        use hkdf::Hkdf;
        use sha2::Sha256;
        
        let data = std::fs::read(path)
            .context("Failed to read encrypted key file")?;
        
        if data.len() < 12 + 32 + 32 + 16 {
            bail!("Invalid encrypted key file: too short");
        }
        
        // Parse: nonce || salt || ciphertext
        let nonce_bytes: [u8; 12] = data[0..12].try_into()?;
        let salt: [u8; 32] = data[12..44].try_into()?;
        let ciphertext = &data[44..];
        
        // Derive decryption key using HKDF
        let hk = Hkdf::<Sha256>::new(Some(&salt), password.as_bytes());
        let mut decryption_key = [0u8; 32];
        hk.expand(b"sultan-validator-key", &mut decryption_key)
            .map_err(|_| anyhow::anyhow!("HKDF expansion failed"))?;
        
        // Decrypt the signing key
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&decryption_key));
        let nonce = Nonce::from_slice(&nonce_bytes);
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|_| anyhow::anyhow!("Decryption failed - wrong password?"))?;
        
        if plaintext.len() != 32 {
            bail!("Invalid decrypted key length");
        }
        
        let key_bytes: [u8; 32] = plaintext.try_into()
            .map_err(|_| anyhow::anyhow!("Failed to convert decrypted key"))?;
        
        let signing_key = SigningKey::from_bytes(&key_bytes);
        info!("🔓 Validator key decrypted from {:?}", path);
        info!("   Pubkey: {}", hex::encode(signing_key.verifying_key().to_bytes()));
        
        Ok(signing_key)
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
        info!("🔄 Block production loop started, block_time={}s", self.block_time);
        
        // === GOSSIPSUB MESH WARMUP ===
        // Wait for gossipsub mesh to establish before producing first block.
        // This is critical for multi-validator networks - messages won't propagate
        // until the mesh is formed (typically 5-15 seconds after peer connections).
        // IMPORTANT: Wait BEFORE creating the ticker to avoid accumulated missed ticks.
        info!("⏳ Waiting 15s for P2P mesh to establish before block production...");
        tokio::time::sleep(Duration::from_secs(15)).await;
        info!("✅ P2P mesh warmup complete, starting block production");
        
        // Create ticker AFTER the warmup sleep to avoid accumulated ticks
        let mut ticker = interval(Duration::from_secs(self.block_time));
        
        // Track ticks for periodic sync check
        let mut tick_count: u64 = 0;
        let sync_check_interval = 5; // Check every 5 ticks (10s with 2s block time)
        
        loop {
            info!("⏳ Block production: waiting for next tick...");
            ticker.tick().await;
            tick_count += 1;
            info!("⏰ Block production: tick received, starting produce_block()");
            
            // === PERIODIC BLOCK SYNC CHECK ===
            // Every N ticks, check if we're behind and request sync if needed.
            // This handles cases where gossipsub misses blocks.
            if tick_count % sync_check_interval == 0 {
                if let Err(e) = self.check_and_request_sync().await {
                    debug!("Sync check failed: {}", e);
                }
            }
            
            if let Err(e) = self.produce_block().await {
                error!("Block production failed: {}", e);
                continue;
            }
        }
    }
    
    /// Check if we're behind and request sync from peers
    async fn check_and_request_sync(&self) -> Result<()> {
        // Get our current height
        let our_height = self.blockchain.read().await.get_height().await;
        
        // Check if we need to sync using block_sync_manager
        if let Some(ref block_sync) = self.block_sync_manager {
            let sync_manager = block_sync.read().await;
            
            if sync_manager.needs_sync().await {
                let max_peer_height = sync_manager.max_peer_height().await;
                let behind_by = max_peer_height.saturating_sub(our_height);
                
                if behind_by > 0 {
                    // Request blocks we're missing
                    let from_height = our_height + 1;
                    let to_height = std::cmp::min(max_peer_height, from_height + 99); // Max 100 blocks
                    
                    drop(sync_manager); // Release lock before P2P operation
                    
                    if let Some(ref p2p) = self.p2p_network {
                        info!("🔄 Behind by {} blocks (ours: {}, peers: {}), requesting sync for blocks {}-{}", 
                              behind_by, our_height, max_peer_height, from_height, to_height);
                        if let Err(e) = p2p.read().await.request_sync(from_height, to_height).await {
                            warn!("Failed to request sync: {}", e);
                        }
                    }
                    return Ok(());
                }
            }
        }
        
        // Fallback: if we're at genesis and have peers, try to sync
        if our_height == 0 {
            if let Some(ref p2p) = self.p2p_network {
                let peer_count = p2p.read().await.peer_count().await;
                if peer_count > 0 {
                    info!("🔄 Still at genesis height with {} peers, requesting sync for blocks 1-10", peer_count);
                    if let Err(e) = p2p.read().await.request_sync(1, 10).await {
                        warn!("Failed to request sync: {}", e);
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Produce a single block
    async fn produce_block(&self) -> Result<()> {
        info!("🔨 produce_block: entering function");
        
        // Yield to allow P2P handler to complete any pending operations
        // This helps prevent lock starvation
        tokio::task::yield_now().await;
        info!("🔨 produce_block: after yield_now");
        
        // DEADLOCK PREVENTION: Use try_get_height with timeout on inner lock
        // The blockchain read lock is briefly held, but the inner blocks lock
        // uses a timeout to prevent indefinite waiting
        let current_height = {
            let mut retries = 0;
            loop {
                // First try to get outer blockchain lock
                let guard = match self.blockchain.try_read() {
                    Ok(g) => g,
                    Err(_) => {
                        retries += 1;
                        if retries > 50 {
                            anyhow::bail!("Failed to acquire blockchain read lock after 50 retries");
                        }
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        continue;
                    }
                };
                
                // Use try_get_height which has internal timeout
                match guard.try_get_height().await {
                    Some(h) => {
                        drop(guard);
                        break h;
                    }
                    None => {
                        drop(guard);
                        retries += 1;
                        if retries > 50 {
                            anyhow::bail!("Failed to get height after 50 retries (inner lock busy)");
                        }
                        info!("🔨 produce_block: height lock busy, retry {}/50", retries);
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        continue;
                    }
                }
            }
        };
        info!("🔨 produce_block: got height {}", current_height);
        
        let next_height = current_height + 1;

        // === PROPER PoS: Height-based proposer selection with timeout fallback ===
        // Each block height has a designated proposer. If they don't produce within
        // the slot time, fallback validators can step in after a delay.
        // 
        // Slot timing (2 second block time):
        // - 0.0-1.0s: Primary proposer window
        // - 1.0-1.5s: First fallback window  
        // - 1.5-1.8s: Second fallback window
        // - 1.8-2.0s: Third fallback window
        //
        // This ensures liveness even if the primary proposer is offline.
        
        let our_address = match &self.validator_address {
            Some(addr) => addr.clone(),
            None => {
                // Not a validator, skip production
                return Ok(());
            }
        };

        // Get proposer order and check our position
        let (proposer_order, validator_count) = {
            let mut retries = 0;
            loop {
                match self.consensus.try_read() {
                    Ok(guard) => {
                        let order = guard.get_proposer_order_for_height(next_height);
                        let count = guard.validator_count();
                        drop(guard);
                        break (order, count);
                    }
                    Err(_) => {
                        retries += 1;
                        if retries > 50 {
                            anyhow::bail!("Failed to acquire consensus read lock after 50 retries");
                        }
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                }
            }
        };

        if proposer_order.is_empty() {
            anyhow::bail!("No active validators to propose block");
        }

        let primary_proposer = &proposer_order[0];
        let our_position = proposer_order.iter().position(|a| a == &our_address);
        
        info!("🔨 produce_block: primary proposer: {} (validators: {})", primary_proposer, validator_count);
        info!("🔨 produce_block: our_address: {}, our_position: {:?}", our_address, our_position);

        // === GENESIS BOOTSTRAP SAFETY NET ===
        // On a fresh network with only one registered validator (the genesis node),
        // always allow production. This is NOT Bootstrap Mode workaround - this is
        // standard behavior for network genesis. With proper on-chain registration,
        // all validators will have the same view and normal proposer selection works.
        let is_genesis_only = validator_count == 1;
        
        if is_genesis_only && our_position == Some(0) {
            info!("🌱 Genesis mode: single validator producing block {}", next_height);
        }

        // Determine if we should produce based on our position and timing
        let is_our_turn = match our_position {
            None => {
                // We're not in the proposer list (shouldn't happen if we're a validator)
                debug!("Not in proposer order for height {}", next_height);
                false
            }
            Some(0) => {
                // We're the primary proposer - always produce
                info!("🎯 We are PRIMARY proposer for height {}", next_height);
                true
            }
            Some(pos) => {
                // We're a fallback proposer - check if we should step in
                // 
                // ENTERPRISE-GRADE FALLBACK LOGIC:
                // 1. Only step in if primary has missed FALLBACK_THRESHOLD_MISSED_BLOCKS consecutive blocks
                // 2. Only top MAX_FALLBACK_POSITIONS validators can be fallbacks
                // 3. Uses missed_blocks counter which is reset when primary signs
                
                // Import the constants (they're in consensus.rs)
                use crate::consensus::{FALLBACK_THRESHOLD_MISSED_BLOCKS, MAX_FALLBACK_POSITIONS};
                
                // Check if we're eligible to be a fallback (position must be within limit)
                if pos > MAX_FALLBACK_POSITIONS {
                    debug!("Position {} exceeds MAX_FALLBACK_POSITIONS ({}), not stepping in", 
                           pos, MAX_FALLBACK_POSITIONS);
                    false
                } else {
                    // Check if primary proposer seems offline
                    let primary_seems_offline = {
                        if let Ok(guard) = self.consensus.try_read() {
                            if let Some(validator) = guard.get_validator(primary_proposer) {
                                // Primary is "offline" if they've missed enough consecutive blocks
                                let is_offline = validator.missed_blocks >= FALLBACK_THRESHOLD_MISSED_BLOCKS;
                                if is_offline {
                                    debug!("Primary {} has missed {} blocks (threshold: {})", 
                                           primary_proposer, validator.missed_blocks, FALLBACK_THRESHOLD_MISSED_BLOCKS);
                                }
                                is_offline
                            } else {
                                warn!("Primary proposer {} not found in validators!", primary_proposer);
                                true // Validator not found, assume offline
                            }
                        } else {
                            // Can't acquire lock - be conservative, don't step in
                            debug!("Cannot check primary status - lock unavailable");
                            false
                        }
                    };

                    if primary_seems_offline {
                        info!("🔄 Fallback position {}: primary {} missed {} blocks, stepping in for height {}", 
                              pos, primary_proposer, FALLBACK_THRESHOLD_MISSED_BLOCKS, next_height);
                        true
                    } else {
                        debug!("Not our turn: position {} in fallback order for height {}", pos, next_height);
                        false
                    }
                }
            }
        };

        if !is_our_turn {
            // Record that primary proposer missed this block (if they're supposed to produce)
            if our_position == Some(0) {
                // This shouldn't happen - we're primary but not producing?
                warn!("We're primary but not producing for height {} - logic error", next_height);
            }
            return Ok(());
        }
        info!("🔨 produce_block: IT IS OUR TURN for height {}", next_height);

        // === SYNC-CHECK (non-blocking): Log if behind but DON'T skip if we're proposer ===
        // The proposer must always produce - skipping causes chain stalls
        // Other validators will catch up via sync, but the proposer must lead
        // DEADLOCK PREVENTION: Use timeout on sync manager lock
        if let Some(block_sync) = &self.block_sync_manager {
            match tokio::time::timeout(Duration::from_millis(100), block_sync.read()).await {
                Ok(sync_manager) => {
                    // Use timeout on inner async call too
                    if let Ok(needs) = tokio::time::timeout(Duration::from_millis(50), sync_manager.needs_sync()).await {
                        if needs {
                            if let Ok(max_peer) = tokio::time::timeout(Duration::from_millis(50), sync_manager.max_peer_height()).await {
                                if max_peer > next_height + 10 {
                                    warn!("⚠️ We're behind by {} blocks but still producing as proposer", 
                                        max_peer.saturating_sub(next_height));
                                }
                            }
                        }
                    }
                    drop(sync_manager);
                }
                Err(_) => {
                    warn!("⚠️ Sync manager lock busy, skipping sync check");
                }
            }
        }
        info!("🔨 produce_block: passed sync check");
        
        // === PEER-GATE: Disabled for now - just log peer count ===
        // The peer gate was causing the chain to stall because the bootstrap
        // validator couldn't produce without peers, but peers can't connect
        // without blocks to sync. Chicken-and-egg problem.
        // 
        // For now, just log peer status and produce regardless.
        if self.p2p_enabled {
            if let Some(p2p) = &self.p2p_network {
                match tokio::time::timeout(Duration::from_millis(100), p2p.read()).await {
                    Ok(p2p_guard) => {
                        if let Ok(peer_count) = tokio::time::timeout(Duration::from_millis(50), p2p_guard.peer_count()).await {
                            if peer_count < 1 {
                                debug!("📡 No peers connected, but producing block {} anyway", next_height);
                            } else {
                                debug!("📡 {} peer(s) connected, producing block {}", peer_count, next_height);
                            }
                        }
                        drop(p2p_guard);
                    }
                    Err(_) => {
                        warn!("⚠️ P2P lock busy, skipping peer check");
                    }
                }
            }
        }
        info!("🔨 produce_block: passed peer gate");

        info!("🎯 We are proposer for height {}", next_height);

        // Create block using unified Sultan blockchain
        // DEADLOCK PREVENTION: Use try_read with retry instead of blocking read
        let (block, tx_count, stats) = {
            let mut retries = 0;
            let blockchain = loop {
                match self.blockchain.try_read() {
                    Ok(guard) => break guard,
                    Err(_) => {
                        retries += 1;
                        if retries > 100 {
                            anyhow::bail!("Failed to acquire blockchain read lock after 100 retries");
                        }
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                }
            };
            
            // Drain pending transactions from mempool
            let transactions = blockchain.drain_pending_transactions().await;
            let tx_count = transactions.len();
            
            let block = blockchain.create_block(transactions, our_address.clone()).await
                .context("Failed to create block")?;
            
            let stats = blockchain.get_stats().await;
            
            // Auto-expand shards if load threshold exceeded
            if stats.should_expand {
                let current_shards = stats.shard_count;
                let additional = current_shards.min(8000 - current_shards);
                if additional > 0 {
                    info!("⚡ Auto-expansion triggered at {:.1}% load", stats.current_load * 100.0);
                    if let Err(e) = blockchain.expand_shards(additional).await {
                        warn!("Failed to auto-expand shards: {}", e);
                    }
                }
            }
            
            drop(blockchain); // Explicit drop before returning from block
            (block, tx_count, stats)
        }; // blockchain lock guaranteed dropped here
        
        info!(
            "✅ SHARDED Block {} | {} shards active | {} txs in block | {} total processed | capacity: {} TPS",
            block.index,
            stats.shard_count,
            tx_count,
            stats.total_processed,
            stats.estimated_tps
        );

        // Record proposal in consensus - DEADLOCK PREVENTION: Use try_write with retry
        {
            let mut retries = 0;
            loop {
                match self.consensus.try_write() {
                    Ok(mut guard) => {
                        guard.record_proposal(&our_address)
                            .context("Failed to record proposal")?;
                        drop(guard); // Explicit drop
                        break;
                    }
                    Err(_) => {
                        retries += 1;
                        if retries > 100 {
                            anyhow::bail!("Failed to acquire consensus write lock after 100 retries");
                        }
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                }
            }
        }
        
        // Update block sync manager height after producing block
        if let Some(ref block_sync) = self.block_sync_manager {
            block_sync.write().await.set_height(block.index).await;
        }

        // Record block signed in staking (resets missed block counter)
        if let Err(e) = self.staking_manager.record_block_signed(&our_address).await {
            warn!("Failed to record block signed for staking: {}", e);
        }

        // Save to storage - ACQUIRE LOCK ONLY WHEN NEEDED (after all other locks released)
        // This prevents deadlock with P2P handler which acquires locks in different order
        let storage = self.storage.read().await;
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
                // NOTE: Sign the raw block_hash bytes directly - p2p.rs verify_vote_signature verifies raw bytes
                let proposer_signature = if let Some(ref signing_key) = self.validator_signing_key {
                    use ed25519_dalek::Signer;
                    let signature = signing_key.sign(block_hash.as_bytes());
                    signature.to_bytes().to_vec()
                } else {
                    warn!("⚠️  Block {} proposed without signature (no signing key)", block.index);
                    Vec::new()
                };
                
                if let Err(e) = p2p.read().await.broadcast_block(
                    block.index,
                    &our_address,
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

    /// Sign a validator announcement for P2P broadcast
    /// Signs the message: address || stake || peer_id
    fn sign_validator_announcement(&self, address: &str, stake: u64, peer_id: &str) -> Vec<u8> {
        if let Some(ref signing_key) = self.validator_signing_key {
            use ed25519_dalek::Signer;
            let message = format!("{}{}{}", address, stake, peer_id);
            let signature = signing_key.sign(message.as_bytes());
            signature.to_bytes().to_vec()
        } else {
            warn!("⚠️ Cannot sign validator announcement (no signing key)");
            Vec::new()
        }
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

        // GET /block/latest - Latest block for explorers
        let block_latest_route = warp::path!("block" / "latest")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_get_latest_block);

        // GET /blocks?limit=N&offset=M - List recent blocks (paginated)
        let blocks_list_route = warp::path!("blocks")
            .and(warp::get())
            .and(warp::query::<BlocksListQuery>())
            .and(with_state(state.clone()))
            .and_then(handle_get_blocks_list);

        // GET /stats - Network statistics for dashboards
        let stats_route = warp::path!("stats")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_get_stats);

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

        // POST /staking/set_reward_wallet - Set wallet address for validator rewards
        let set_reward_wallet_route = warp::path!("staking" / "set_reward_wallet")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_state(state.clone()))
            .and_then(handle_set_reward_wallet);

        // GET /staking/reward_wallet/:validator_address - Get reward wallet for a validator
        let get_reward_wallet_route = warp::path!("staking" / "reward_wallet" / String)
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_get_reward_wallet);

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

        // === Faucet Routes (Challenge-Response Anti-Sybil) ===
        
        // GET /faucet/challenge/:address - Get challenge nonce to sign (step 1)
        let faucet_challenge_route = warp::path!("faucet" / "challenge" / String)
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_faucet_challenge);
        
        // POST /faucet/claim - Claim SLTN with signed challenge (step 2)
        let faucet_claim_route = warp::path!("faucet" / "claim")
            .and(warp::post())
            .and(warp::body::json::<FaucetClaimRequest>())
            .and(with_state(state.clone()))
            .and_then(handle_faucet_claim);
        
        // GET /faucet/status - Get faucet status and stats
        let faucet_status_route = warp::path!("faucet" / "status")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_faucet_status);
        
        // POST /faucet/toggle - Admin toggle faucet on/off (requires SULTAN_ADMIN_KEY)
        let faucet_toggle_route = warp::path!("faucet" / "toggle")
            .and(warp::post())
            .and(warp::body::json::<FaucetToggleRequest>())
            .and(with_state(state.clone()))
            .and_then(|req: FaucetToggleRequest, state: Arc<NodeState>| async move {
                handle_faucet_toggle(req.enabled, req.admin_key, state).await
            });

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
            .or(block_latest_route)
            .or(block_route)
            .or(blocks_list_route)
            .or(stats_route)
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
            .or(set_reward_wallet_route)
            .or(get_reward_wallet_route)
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
        
        let faucet_routes = faucet_challenge_route
            .or(faucet_claim_route)
            .or(faucet_status_route)
            .or(faucet_toggle_route)
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
            .or(faucet_routes)
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

        // Start server with or without TLS
        if let Some(ref tls_config) = state.tls_config {
            info!("🔒 Starting RPC server with TLS on {}", addr);
            warp::serve(routes)
                .tls()
                .cert_path(&tls_config.cert_path)
                .key_path(&tls_config.key_path)
                .run(addr)
                .await;
        } else {
            info!("⚠️  Starting RPC server WITHOUT TLS on {} (use --enable-tls for production)", addr);
            warp::serve(routes).run(addr).await;
        }

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

    async fn handle_get_latest_block(
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let blockchain = state.blockchain.read().await;
        let height = blockchain.get_height().await;
        
        match blockchain.get_block(height).await {
            Some(block) => Ok(warp::reply::json(&serde_json::json!({
                "block": block,
                "height": height
            }))),
            None => Ok(warp::reply::json(&serde_json::json!({
                "height": height,
                "message": "Genesis block"
            }))),
        }
    }

    async fn handle_get_blocks_list(
        query: BlocksListQuery,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let blockchain = state.blockchain.read().await;
        let current_height = blockchain.get_height().await;
        
        // Calculate range (most recent first)
        let start = if query.offset >= current_height {
            0
        } else {
            current_height - query.offset
        };
        let end = if start >= query.limit as u64 {
            start - query.limit as u64 + 1
        } else {
            1
        };
        
        let mut blocks = Vec::new();
        for height in (end..=start).rev() {
            if let Some(block) = blockchain.get_block(height).await {
                blocks.push(serde_json::json!({
                    "height": block.index,
                    "hash": block.hash,
                    "timestamp": block.timestamp,
                    "tx_count": block.transactions.len(),
                    "validator": block.validator
                }));
            }
            if blocks.len() >= query.limit {
                break;
            }
        }
        
        Ok(warp::reply::json(&serde_json::json!({
            "blocks": blocks,
            "total_height": current_height,
            "count": blocks.len(),
            "limit": query.limit,
            "offset": query.offset
        })))
    }

    async fn handle_get_stats(
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let blockchain = state.blockchain.read().await;
        let height = blockchain.get_height().await;
        let stats = blockchain.get_stats().await;
        
        // Validator count
        let validators = state.staking_manager.get_validators().await;
        let validator_count = validators.len();
        
        // Get config for shard info
        let config = state.config.read().await;
        
        Ok(warp::reply::json(&serde_json::json!({
            "height": height,
            "total_transactions": stats.total_transactions,
            "total_processed": stats.total_processed,
            "estimated_tps": stats.estimated_tps,
            "current_load": format!("{:.1}%", stats.current_load * 100.0),
            "validator_count": validator_count,
            "shard_count": stats.shard_count,
            "healthy_shards": stats.healthy_shards,
            "max_shards": stats.max_shards,
            "pending_cross_shard": stats.pending_cross_shard,
            "total_accounts": stats.total_accounts,
            "should_expand": stats.should_expand,
            "sharding_enabled": config.features.sharding_enabled,
            "block_time_seconds": 2,
            "gas_fees": "zero",
            "network": "Sultan L1 Mainnet"
        })))
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

    #[derive(serde::Deserialize)]
    struct BlocksListQuery {
        #[serde(default = "default_blocks_limit")]
        limit: usize,
        #[serde(default)]
        offset: u64,
    }

    fn default_limit() -> usize {
        50
    }

    fn default_blocks_limit() -> usize {
        20
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
        /// Optional: wallet address for receiving APY rewards
        /// If not provided, defaults to validator_address
        #[serde(default)]
        reward_wallet: Option<String>,
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
                // Set reward wallet: use provided wallet or default to validator's own address
                let reward_wallet = req.reward_wallet.clone().unwrap_or_else(|| req.validator_address.clone());
                if let Err(e) = state.staking_manager.set_reward_wallet(&req.validator_address, reward_wallet.clone()).await {
                    warn!("Failed to set reward wallet for {}: {}", req.validator_address, e);
                } else {
                    info!("💰 New validator {} rewards → {}", req.validator_address, reward_wallet);
                }

                // Also add to consensus engine for block production
                // This unifies staking validators with consensus validators
                // CRITICAL: Drop consensus lock BEFORE acquiring blockchain lock to prevent deadlock
                // Lock ordering: always release consensus before acquiring blockchain
                {
                    let mut consensus = state.consensus.write().await;
                    if let Err(e) = consensus.add_validator(req.validator_address.clone(), req.initial_stake, pubkey) {
                        warn!("Validator {} added to staking but not consensus: {}", req.validator_address, e);
                    }
                } // consensus lock dropped here
                
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

    /// Enhanced validator info with computed fields for explorer
    #[derive(serde::Serialize)]
    struct ValidatorInfo {
        validator_address: String,
        reward_wallet: Option<String>,
        self_stake: u64,
        delegated_stake: u64,
        total_stake: u64,
        commission_rate: f64,
        rewards_accumulated: u64,
        blocks_signed: u64,
        blocks_missed: u64,
        jailed: bool,
        jailed_until: u64,
        created_at: u64,
        last_reward_height: u64,
        // Computed fields for explorer
        uptime_percent: f64,
        voting_power_percent: f64,
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
                    reward_wallet: None, // Genesis validators have no wallet until set
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
        
        // Calculate total stake for voting power percentages
        let total_network_stake: u64 = validators.iter().map(|v| v.total_stake).sum();
        
        // Convert to enhanced info with computed fields
        let enhanced: Vec<ValidatorInfo> = validators.iter().map(|v| {
            // Uptime: blocks_signed / (blocks_signed + blocks_missed) * 100
            // If no blocks yet, show 100% (new validator)
            let total_blocks = v.blocks_signed + v.blocks_missed;
            let uptime_percent = if total_blocks > 0 {
                (v.blocks_signed as f64 / total_blocks as f64) * 100.0
            } else {
                100.0 // New validator, no blocks yet
            };
            
            // Voting power: this validator's stake / total network stake * 100
            let voting_power_percent = if total_network_stake > 0 {
                (v.total_stake as f64 / total_network_stake as f64) * 100.0
            } else {
                0.0
            };
            
            ValidatorInfo {
                validator_address: v.validator_address.clone(),
                reward_wallet: v.reward_wallet.clone(),
                self_stake: v.self_stake,
                delegated_stake: v.delegated_stake,
                total_stake: v.total_stake,
                commission_rate: v.commission_rate,
                rewards_accumulated: v.rewards_accumulated,
                blocks_signed: v.blocks_signed,
                blocks_missed: v.blocks_missed,
                jailed: v.jailed,
                jailed_until: v.jailed_until,
                created_at: v.created_at,
                last_reward_height: v.last_reward_height,
                uptime_percent,
                voting_power_percent,
            }
        }).collect();
        
        Ok(warp::reply::json(&enhanced))
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
        if req.is_validator {
            match state.staking_manager.withdraw_validator_rewards(&req.address).await {
                Ok((rewards, wallet)) => {
                    // Credit the rewards to the reward_wallet (minted via inflation)
                    let blockchain = state.blockchain.read().await;
                    if let Err(e) = blockchain.add_balance(&wallet, rewards).await {
                        warn!("Failed to credit rewards to wallet {}: {}", wallet, e);
                        return Ok(warp::reply::json(&serde_json::json!({
                            "error": format!("Failed to credit rewards: {}", e),
                            "status": "error"
                        })));
                    }
                    info!("Validator {} rewards ({} SLTN) credited to wallet {}", 
                          req.address, rewards / 1_000_000_000, wallet);
                    Ok(warp::reply::json(&serde_json::json!({
                        "validator_address": req.address,
                        "reward_wallet": wallet,
                        "rewards_withdrawn": rewards,
                        "status": "success"
                    })))
                }
                Err(e) => {
                    warn!("Withdraw validator rewards failed: {}", e);
                    Ok(warp::reply::json(&serde_json::json!({
                        "error": e.to_string(),
                        "status": "error"
                    })))
                }
            }
        } else if let Some(validator) = req.validator_address {
            match state.staking_manager.withdraw_delegator_rewards(&req.address, &validator).await {
                Ok(rewards) => {
                    // Credit delegator rewards to their address
                    let blockchain = state.blockchain.read().await;
                    if let Err(e) = blockchain.add_balance(&req.address, rewards).await {
                        warn!("Failed to credit delegator rewards: {}", e);
                        return Ok(warp::reply::json(&serde_json::json!({
                            "error": format!("Failed to credit rewards: {}", e),
                            "status": "error"
                        })));
                    }
                    Ok(warp::reply::json(&serde_json::json!({
                        "address": req.address,
                        "rewards_withdrawn": rewards,
                        "status": "success"
                    })))
                }
                Err(e) => {
                    warn!("Withdraw delegator rewards failed: {}", e);
                    Ok(warp::reply::json(&serde_json::json!({
                        "error": e.to_string(),
                        "status": "error"
                    })))
                }
            }
        } else {
            Ok(warp::reply::json(&serde_json::json!({
                "error": "validator_address required for delegator withdrawals",
                "status": "error"
            })))
        }
    }

    #[derive(serde::Deserialize)]
    struct SetRewardWalletRequest {
        validator_address: String,
        reward_wallet: String,
        /// Signature proving ownership of validator (required)
        /// Signs: "set_reward_wallet:{validator_address}:{reward_wallet}:{timestamp}"
        signature: String,
        /// Public key of the validator (hex encoded, 64 chars)
        public_key: String,
        /// Timestamp to prevent replay attacks (must be within 5 minutes)
        timestamp: u64,
    }

    /// Handle setting a validator's reward wallet
    /// 
    /// This allows validators to designate a wallet where their accumulated 
    /// APY rewards should be sent when withdrawn.
    /// 
    /// SECURITY: Requires signature verification to prevent unauthorized changes.
    async fn handle_set_reward_wallet(
        req: SetRewardWalletRequest,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        // Verify timestamp is within 5 minutes to prevent replay attacks
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        if req.timestamp > now + 300 || req.timestamp < now.saturating_sub(300) {
            return Ok(warp::reply::json(&serde_json::json!({
                "error": "Timestamp must be within 5 minutes of current time",
                "status": "error"
            })));
        }

        // Validate and decode public key
        if req.public_key.len() != 64 {
            return Ok(warp::reply::json(&serde_json::json!({
                "error": "public_key must be 64 hex characters",
                "status": "error"
            })));
        }
        
        let pubkey_bytes = match hex::decode(&req.public_key) {
            Ok(b) if b.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&b);
                arr
            }
            _ => {
                return Ok(warp::reply::json(&serde_json::json!({
                    "error": "Invalid public_key hex",
                    "status": "error"
                })));
            }
        };

        let verifying_key = match ed25519_dalek::VerifyingKey::from_bytes(&pubkey_bytes) {
            Ok(k) => k,
            Err(_) => {
                return Ok(warp::reply::json(&serde_json::json!({
                    "error": "Invalid Ed25519 public key",
                    "status": "error"
                })));
            }
        };

        // Verify the public key matches the validator address (or is registered for this validator)
        // For genesis validators, we check if the pubkey is registered in consensus
        let consensus = state.consensus.read().await;
        let validator_pubkey = consensus.get_validator_pubkey(&req.validator_address);
        drop(consensus);
        
        if let Some(registered_pubkey) = validator_pubkey {
            if registered_pubkey != pubkey_bytes {
                return Ok(warp::reply::json(&serde_json::json!({
                    "error": "Public key does not match registered validator key",
                    "status": "error"
                })));
            }
        }
        // Note: For new validators without registered pubkeys, we allow the operation
        // since they just registered with this pubkey

        // Verify signature
        let message = format!("set_reward_wallet:{}:{}:{}", 
            req.validator_address, req.reward_wallet, req.timestamp);
        
        use sha2::{Sha256, Digest};
        let message_hash = Sha256::digest(message.as_bytes());
        
        let sig_bytes = match hex::decode(&req.signature) {
            Ok(b) if b.len() == 64 => {
                let mut arr = [0u8; 64];
                arr.copy_from_slice(&b);
                arr
            }
            _ => {
                return Ok(warp::reply::json(&serde_json::json!({
                    "error": "Invalid signature hex (must be 128 chars)",
                    "status": "error"
                })));
            }
        };

        let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);

        if verifying_key.verify_strict(&message_hash, &signature).is_err() {
            return Ok(warp::reply::json(&serde_json::json!({
                "error": "Invalid signature - must sign: set_reward_wallet:{validator}:{wallet}:{timestamp}",
                "status": "error"
            })));
        }

        // Signature verified - update reward wallet
        match state.staking_manager.set_reward_wallet(&req.validator_address, req.reward_wallet.clone()).await {
            Ok(()) => {
                info!("Validator {} set reward wallet to {}", req.validator_address, req.reward_wallet);
                Ok(warp::reply::json(&serde_json::json!({
                    "validator_address": req.validator_address,
                    "reward_wallet": req.reward_wallet,
                    "status": "success"
                })))
            }
            Err(e) => {
                warn!("Set reward wallet failed: {}", e);
                Ok(warp::reply::json(&serde_json::json!({
                    "error": e.to_string(),
                    "status": "error"
                })))
            }
        }
    }

    /// Handle getting a validator's reward wallet
    async fn handle_get_reward_wallet(
        validator_address: String,
        state: Arc<NodeState>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        match state.staking_manager.get_reward_wallet(&validator_address).await {
            Ok(wallet) => {
                Ok(warp::reply::json(&serde_json::json!({
                    "validator_address": validator_address,
                    "reward_wallet": wallet,
                    "status": "success"
                })))
            }
            Err(e) => {
                warn!("Get reward wallet failed: {}", e);
                Ok(warp::reply::json(&serde_json::json!({
                    "error": e.to_string(),
                    "status": "error"
                })))
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
            Command::Keygen { format, output, password } => {
                run_keygen(format, output.as_deref(), password.as_deref());
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
                    
                    // Get pubkey from our actual signing key (NOT from consensus, which may have placeholder)
                    // This ensures the pubkey matches the signature we create
                    let (pubkey, signature) = if let Some(ref signing_key) = p2p_state.validator_signing_key {
                        use ed25519_dalek::Signer;
                        let peer_id = p2p.read().await.peer_id().to_string();
                        let message = format!("{}{}{}", addr, stake, peer_id);
                        let sig = signing_key.sign(message.as_bytes());
                        let pk = signing_key.verifying_key().to_bytes();
                        (pk, sig.to_bytes().to_vec())
                    } else {
                        warn!("⚠️ No signing key available for validator announcement");
                        ([0u8; 32], Vec::new())
                    };
                    
                    if pubkey == [0u8; 32] {
                        warn!("⚠️ Skipping validator announcement - no valid pubkey");
                    } else if signature.is_empty() {
                        warn!("⚠️ Skipping validator announcement - no signing key");
                    } else {
                        info!("📢 Announcing with pubkey: {}...", hex::encode(&pubkey[..8]));
                        if let Err(e) = p2p.read().await.announce_validator(addr, stake, pubkey, signature).await {
                            warn!("Failed to announce validator: {}", e);
                        } else {
                            info!("✅ Announced validator {} to P2P network", addr);
                        }
                    }
                }
            }
            
            // Request validator set from peers on startup to sync existing validators
            if let Some(ref p2p) = p2p_state.p2p_network {
                let known_count = {
                    let consensus = p2p_state.consensus.read().await;
                    consensus.validator_count() as u32
                };
                info!("📡 Requesting validator set from peers (we know {} validators)", known_count);
                if let Err(e) = p2p.read().await.request_validator_set(known_count).await {
                    warn!("Failed to request validator set: {}", e);
                }
            }
            
            // Create timer for periodic re-announcement (every 60 seconds)
            let mut reannounce_interval = tokio::time::interval(Duration::from_secs(60));
            reannounce_interval.tick().await; // Skip immediate first tick
            
            // P2P message handler loop
            loop {
                // Process incoming P2P messages with timeout
                tokio::select! {
                    // Periodic validator re-announcement for network churn
                    _ = reannounce_interval.tick() => {
                        if let (Some(ref addr), Some(stake)) = (&validator_addr, validator_stake) {
                            if let Some(ref p2p) = p2p_state.p2p_network {
                                // Get actual validator pubkey from consensus engine
                                // Get pubkey and signature from actual signing key
                                if let Some(ref signing_key) = p2p_state.validator_signing_key {
                                    use ed25519_dalek::Signer;
                                    let peer_id = p2p.read().await.peer_id().to_string();
                                    let message = format!("{}{}{}", addr, stake, peer_id);
                                    let signature = signing_key.sign(message.as_bytes()).to_bytes().to_vec();
                                    let pubkey = signing_key.verifying_key().to_bytes();
                                    
                                    if let Err(e) = p2p.read().await.announce_validator(addr, stake, pubkey, signature).await {
                                        warn!("Failed to re-announce validator: {}", e);
                                    } else {
                                        debug!("🔄 Re-announced validator {} to P2P network", addr);
                                    }
                                }
                            }
                        }
                    }
                    // Handle incoming messages from P2P network
                    Some(msg) = async { 
                        if let Some(ref mut rx) = message_rx { 
                            rx.recv().await 
                        } else { 
                            None 
                        } 
                    } => {
                        match msg {
                            NetworkMessage::ValidatorAnnounce { ref address, stake, ref peer_id, pubkey, signature: _ } => {
                                // ====================================================================
                                // ENTERPRISE-GRADE FIX: P2P announcements are for DISCOVERY ONLY
                                // ====================================================================
                                // Validators MUST register on-chain via /staking/create_validator
                                // P2P announcements only register pubkey for signature verification
                                // This ensures all validators have the SAME view of the validator set
                                // (derived from blockchain state, not P2P message order)
                                // ====================================================================
                                
                                // Register pubkey for signature verification (discovery only)
                                if pubkey != [0u8; 32] {
                                    if let Some(ref p2p) = p2p_state.p2p_network {
                                        p2p.read().await.register_validator_pubkey(address.clone(), pubkey).await;
                                    }
                                }
                                
                                // Check if already in consensus (via on-chain registration)
                                let is_registered = {
                                    let consensus = p2p_state.consensus.read().await;
                                    consensus.is_validator(address)
                                };
                                
                                if is_registered {
                                    info!("📡 Validator {} online (stake: {}, peer: {}) - already in consensus", 
                                          address, stake, peer_id);
                                } else {
                                    // NOT adding to consensus - validator must register on-chain first
                                    info!("📡 Validator {} discovered via P2P (stake: {}, peer: {}) - awaiting on-chain registration", 
                                          address, stake, peer_id);
                                }
                            }
                            NetworkMessage::ValidatorSetRequest { known_count } => {
                                // Another node is requesting our validator set
                                info!("📥 Validator set request received (they know {} validators)", known_count);
                                
                                // Get our validators from consensus
                                let consensus = p2p_state.consensus.read().await;
                                let our_validators: Vec<sultan_core::p2p::ValidatorInfo> = consensus
                                    .get_active_validators()
                                    .iter()
                                    .map(|v| sultan_core::p2p::ValidatorInfo {
                                        address: v.address.clone(),
                                        stake: v.stake,
                                        pubkey: v.pubkey,
                                    })
                                    .collect();
                                
                                // Only send if we have more validators than they know about
                                if our_validators.len() as u32 > known_count {
                                    if let Some(ref p2p) = p2p_state.p2p_network {
                                        if let Err(e) = p2p.read().await.send_validator_set(our_validators).await {
                                            warn!("Failed to send validator set: {}", e);
                                        }
                                    }
                                }
                            }
                            NetworkMessage::ValidatorSetResponse { validators } => {
                                // ====================================================================
                                // ENTERPRISE-GRADE: Validator set response is for DISCOVERY ONLY
                                // ====================================================================
                                // Do NOT add validators to consensus from P2P sync
                                // Validators must register on-chain for consensus participation
                                // We only register pubkeys for signature verification
                                // ====================================================================
                                info!("📥 Received validator set with {} validators (for discovery)", validators.len());
                                
                                let mut discovered_count = 0;
                                for validator in validators {
                                    if validator.pubkey == [0u8; 32] {
                                        continue;
                                    }
                                    
                                    // Register pubkey for signature verification only
                                    if let Some(ref p2p) = p2p_state.p2p_network {
                                        p2p.read().await.register_validator_pubkey(validator.address.clone(), validator.pubkey).await;
                                        discovered_count += 1;
                                    }
                                }
                                
                                if discovered_count > 0 {
                                    info!("📊 Discovered {} validator pubkeys for signature verification", discovered_count);
                                }
                            }
                            NetworkMessage::BlockProposal { height, proposer, block_hash, block_data, proposer_signature: _ } => {
                                // === PRODUCTION BLOCK SYNC ===
                                // Process incoming block from another validator
                                info!("📥 HANDLER: BlockProposal received - height={}, proposer={}, hash={}", 
                                       height, proposer, &block_hash[..16.min(block_hash.len())]);
                                
                                // Validate proposer is a registered validator FIRST
                                // (before updating peer height to avoid sync deadlock)
                                // CRITICAL: Validate proposer and extract expected_proposer, then DROP
                                // consensus lock BEFORE acquiring block_sync lock to prevent deadlock.
                                // Lock ordering: always acquire block_sync before consensus, or ensure
                                // they are never held simultaneously.
                                let (is_valid_proposer, expected_primary_proposer) = {
                                    let consensus = p2p_state.consensus.read().await;
                                    let valid = consensus.is_validator(&proposer);
                                    // Get the expected PRIMARY proposer for this height
                                    let expected = consensus.select_proposer_for_height(height);
                                    (valid, expected)
                                }; // consensus lock dropped here
                                
                                if !is_valid_proposer {
                                    warn!("❌ Block rejected: proposer {} is not a registered validator", proposer);
                                    continue;
                                }
                                
                                // Track missed blocks: if block came from a fallback proposer,
                                // the primary proposer missed their slot
                                // 
                                // ENTERPRISE-GRADE PROTECTION:
                                // 1. Verify the actual proposer is in the valid proposer order
                                // 2. Use deduplication in consensus to prevent double-counting
                                // 3. Only valid fallbacks can trigger missed block tracking
                                if let Some(ref primary) = expected_primary_proposer {
                                    if primary != &proposer {
                                        // Verify the actual proposer is a valid fallback for this height
                                        let is_valid_fallback = {
                                            if let Ok(guard) = p2p_state.consensus.try_read() {
                                                let order = guard.get_proposer_order_for_height(height);
                                                // Check proposer is in the order AND not in position 0 (that's primary)
                                                order.iter().skip(1).any(|addr| addr == &proposer)
                                            } else {
                                                false // Can't verify, don't track
                                            }
                                        };
                                        
                                        if is_valid_fallback {
                                            // Primary proposer missed their slot, block produced by valid fallback
                                            info!("⚠️ Primary proposer {} missed slot for height {}, block from fallback {}", 
                                                  primary, height, proposer);
                                            // Record missed block for the primary (may trigger slashing)
                                            // Note: record_missed_block has built-in deduplication
                                            if let Ok(mut consensus) = p2p_state.consensus.try_write() {
                                                if let Ok(Some(slash_amount)) = consensus.record_missed_block(primary, height) {
                                                    warn!("🔪 Validator {} slashed {} for missing block {}", 
                                                          primary, slash_amount, height);
                                                }
                                            }
                                        } else {
                                            warn!("⚠️ Block from {} is neither primary nor valid fallback for height {}", 
                                                  proposer, height);
                                        }
                                    }
                                }
                                
                                // Only update peer height for valid proposers
                                // This prevents sync deadlock from unknown validators
                                // Now safe: consensus lock is not held
                                if let Some(ref block_sync) = p2p_state.block_sync_manager {
                                    block_sync.write().await.update_peer_height(proposer.clone(), height).await;
                                }
                                
                                // Deserialize block
                                let block: Block = match bincode::deserialize(&block_data) {
                                    Ok(b) => b,
                                    Err(e) => {
                                        warn!("❌ Failed to deserialize block: {}", e);
                                        continue;
                                    }
                                };
                                
                                // Get our current height - DEADLOCK PREVENTION: use try_read
                                let our_height = match p2p_state.blockchain.try_read() {
                                    Ok(guard) => {
                                        let h = guard.get_height().await;
                                        drop(guard); // Explicit drop
                                        h
                                    }
                                    Err(_) => {
                                        debug!("Skipping block proposal - blockchain lock busy");
                                        continue;
                                    }
                                };
                                
                                // Only accept blocks that are ahead of us
                                if height <= our_height {
                                    info!("⏭️ Block {} already processed (our height: {})", height, our_height);
                                    continue;
                                }
                                
                                // If we're far behind (> 10 blocks), request sync instead of waiting
                                if height > our_height + 10 {
                                    info!("🔄 We're behind: our height {} vs incoming block {}. Requesting sync...", 
                                          our_height, height);
                                    
                                    // Send sync request via P2P
                                    if let Some(ref p2p) = p2p_state.p2p_network {
                                        let from = our_height + 1;
                                        let to = std::cmp::min(our_height + 100, height);
                                        if let Err(e) = p2p.read().await.request_sync(from, to).await {
                                            warn!("Failed to send sync request: {}", e);
                                        }
                                    }
                                    continue;
                                }
                                
                                // Accept block if it's the next one we need
                                if height == our_height + 1 {
                                    info!("📦 Applying block {} from {} (our height was {})", height, proposer, our_height);
                                    
                                    // Apply block - DEADLOCK PREVENTION: use try_read to avoid blocking
                                    let apply_result = match p2p_state.blockchain.try_read() {
                                        Ok(blockchain) => {
                                            let result = blockchain.apply_block(block.clone()).await;
                                            drop(blockchain); // Explicit drop
                                            Some(result)
                                        }
                                        Err(_) => {
                                            debug!("Skipping block apply - blockchain lock busy");
                                            None
                                        }
                                    };
                                    
                                    if let Some(result) = apply_result {
                                        match result {
                                            Ok(_) => {
                                                info!("✅ Synced block {} from {} ({} txs)", 
                                                      height, proposer, block.transactions.len());
                                            
                                                // Update block sync manager height - now safe, blockchain lock is released
                                                if let Some(ref block_sync) = p2p_state.block_sync_manager {
                                                    if let Ok(mut sync) = block_sync.try_write() {
                                                        sync.set_height(height).await;
                                                    }
                                                }
                                            
                                                // Note: Missed block tracking is already handled above when we detect
                                                // primary != proposer (around line 3573). No duplicate tracking needed here.
                                            
                                                // Record that actual proposer signed
                                                let _ = p2p_state.staking_manager
                                                    .record_block_signed(&proposer).await;
                                            
                                                // Advance consensus round after accepting block
                                                // DEADLOCK PREVENTION: Use try_write to avoid blocking
                                                if let Ok(mut consensus) = p2p_state.consensus.try_write() {
                                                    let _ = consensus.select_proposer();
                                                    drop(consensus); // Explicit drop
                                                }
                                            
                                                // Save to storage
                                                if let Ok(storage) = p2p_state.storage.try_read() {
                                                    let _ = storage.save_block(&block);
                                                }
                                            }
                                            Err(e) => {
                                                warn!("❌ Failed to apply block {}: {}", height, e);
                                            }
                                        }
                                    }
                                } else {
                                    // Block is ahead but not the immediate next one - gap in sequence
                                    info!("⏳ Block {} received but we need {} first (gap of {} blocks)", 
                                          height, our_height + 1, height - our_height - 1);
                                }
                            }
                            NetworkMessage::Transaction { tx_hash, tx_data } => {
                                // === TRANSACTION GOSSIP ===
                                // Receive transaction from another validator and add to our mempool
                                match bincode::deserialize::<Transaction>(&tx_data) {
                                    Ok(tx) => {
                                        info!("📥 Received transaction from peer: {} -> {} amount={}", 
                                              tx.from, tx.to, tx.amount);
                                        
                                        // DEADLOCK PREVENTION: Use try_write to avoid blocking
                                        if let Ok(blockchain) = p2p_state.blockchain.try_write() {
                                            if let Err(e) = blockchain.submit_transaction(tx.clone()).await {
                                                debug!("Failed to add peer tx to mempool: {}", e);
                                            } else {
                                                let pending = blockchain.pending_count().await;
                                                debug!("Added peer tx to mempool [pending: {}]", pending);
                                            }
                                            drop(blockchain); // Explicit drop
                                        } else {
                                            debug!("Skipping tx add - blockchain lock busy");
                                        }
                                    }
                                    Err(e) => {
                                        warn!("❌ Failed to deserialize transaction {}: {}", tx_hash, e);
                                    }
                                }
                            }
                            NetworkMessage::SyncRequest { from_height, to_height } => {
                                // === BLOCK SYNC REQUEST HANDLER ===
                                // Another node is behind and requesting blocks from us
                                info!("📥 Received sync request for blocks {}-{}", from_height, to_height);
                                
                                // Limit response size to prevent DoS
                                let max_blocks: u64 = 100;
                                let to_height = std::cmp::min(to_height, from_height + max_blocks - 1);
                                
                                // Get our current height
                                let our_height = p2p_state.blockchain.read().await.get_height().await;
                                
                                // Only respond if we have the requested blocks
                                if from_height > our_height {
                                    debug!("Cannot serve sync request: requested {} but our height is {}", 
                                           from_height, our_height);
                                    continue;
                                }
                                
                                // Collect blocks from storage
                                let mut blocks_data: Vec<Vec<u8>> = Vec::new();
                                let storage = p2p_state.storage.read().await;
                                let actual_to = std::cmp::min(to_height, our_height);
                                for h in from_height..=actual_to {
                                    if let Ok(Some(block)) = storage.get_block_by_height(h) {
                                        if let Ok(data) = bincode::serialize(&block) {
                                            blocks_data.push(data);
                                        }
                                    }
                                }
                                drop(storage);
                                
                                if !blocks_data.is_empty() {
                                    // Send sync response using P2P helper
                                    if let Some(ref p2p) = p2p_state.p2p_network {
                                        if let Err(e) = p2p.read().await.send_sync_response(blocks_data).await {
                                            warn!("Failed to send sync response: {}", e);
                                        }
                                    }
                                }
                            }
                            NetworkMessage::SyncResponse { blocks } => {
                                // === BLOCK SYNC RESPONSE HANDLER ===
                                // We requested blocks and received them
                                info!("📥 Received sync response with {} blocks", blocks.len());
                                
                                let mut applied_count = 0;
                                for block_data in blocks {
                                    // Deserialize block
                                    let block: Block = match bincode::deserialize(&block_data) {
                                        Ok(b) => b,
                                        Err(e) => {
                                            warn!("Failed to deserialize sync block: {}", e);
                                            continue;
                                        }
                                    };
                                    
                                    // Get current height - DEADLOCK PREVENTION: use try_read
                                    let our_height = match p2p_state.blockchain.try_read() {
                                        Ok(guard) => {
                                            let h = guard.get_height().await;
                                            drop(guard); // Explicit drop
                                            h
                                        }
                                        Err(_) => {
                                            debug!("Skipping sync block - blockchain lock busy");
                                            continue;
                                        }
                                    };
                                    
                                    if block.index == our_height + 1 {
                                        // Apply the block - DEADLOCK PREVENTION: use try_write
                                        let blockchain = match p2p_state.blockchain.try_write() {
                                            Ok(guard) => guard,
                                            Err(_) => {
                                                debug!("Skipping sync block {} - blockchain write lock busy", block.index);
                                                continue;
                                            }
                                        };
                                        match blockchain.apply_block(block.clone()).await {
                                            Ok(_) => {
                                                applied_count += 1;
                                                drop(blockchain); // Explicit drop before other operations
                                                
                                                // Update block sync manager height
                                                if let Some(ref block_sync) = p2p_state.block_sync_manager {
                                                    if let Ok(mut sync) = block_sync.try_write() {
                                                        sync.set_height(block.index).await;
                                                    }
                                                }
                                                // Save to storage
                                                if let Ok(storage) = p2p_state.storage.try_read() {
                                                    let _ = storage.save_block(&block);
                                                }
                                            }
                                            Err(e) => {
                                                drop(blockchain);
                                                warn!("Failed to apply sync block {}: {}", block.index, e);
                                                break; // Stop on first error to maintain sequence
                                            }
                                        }
                                        
                                        // Advance consensus - DEADLOCK PREVENTION: use try_write
                                        if let Ok(mut consensus) = p2p_state.consensus.try_write() {
                                            let _ = consensus.select_proposer();
                                            drop(consensus); // Explicit drop
                                        }
                                    } else if block.index <= our_height {
                                        debug!("Sync block {} already applied", block.index);
                                    } else {
                                        debug!("Sync block {} out of sequence (our height: {})", block.index, our_height);
                                    }
                                }
                                
                                if applied_count > 0 {
                                    if let Ok(blockchain) = p2p_state.blockchain.try_read() {
                                        let new_height = blockchain.get_height().await;
                                        info!("✅ Synced {} blocks. New height: {}", applied_count, new_height);
                                    }
                                }
                            }
                            _ => {
                                // Other message types handled elsewhere (BlockVote, etc.)
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
                                        // Get actual pubkey from consensus
                                        // Get pubkey and signature from actual signing key
                                        if let Some(ref signing_key) = p2p_state.validator_signing_key {
                                            use ed25519_dalek::Signer;
                                            let peer_id = p2p.read().await.peer_id().to_string();
                                            let message = format!("{}{}{}", addr, stake, peer_id);
                                            let signature = signing_key.sign(message.as_bytes()).to_bytes().to_vec();
                                            let pubkey = signing_key.verifying_key().to_bytes();
                                            
                                            if let Err(e) = p2p.read().await.announce_validator(addr, stake, pubkey, signature).await {
                                                debug!("Failed to re-announce validator: {}", e);
                                            } else {
                                                info!("📢 Re-announced validator {} ({} peers)", addr, peer_count);
                                            }
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
    
    // Setup graceful shutdown handler
    let shutdown_state = state.clone();
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("🛑 Shutdown signal received, initiating graceful shutdown...");
                
                // Persist staking state
                info!("💾 Persisting staking state...");
                let storage = shutdown_state.storage.read().await;
                if let Err(e) = shutdown_state.staking_manager.persist_to_storage(&storage).await {
                    error!("Failed to persist staking state: {}", e);
                }
                
                // Log final status
                if let Ok(status) = shutdown_state.get_status().await {
                    info!("📊 Final state: height={}, validators={}, accounts={}", 
                          status.height, status.validator_count, status.total_accounts);
                }
                
                info!("✅ Graceful shutdown complete");
                std::process::exit(0);
            }
            Err(e) => {
                error!("Failed to listen for shutdown signal: {}", e);
            }
        }
    });
    
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
struct FaucetToggleRequest {
    enabled: bool,
    admin_key: String,
}

#[derive(Debug, Deserialize)]
struct FaucetClaimRequest {
    address: String,
    nonce: String,
    signature: String,  // hex-encoded Ed25519 signature over the nonce
    pubkey: String,     // hex-encoded 32-byte Ed25519 public key
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
    // Check if token_factory feature is enabled
    if !state.config.read().await.features.token_factory_enabled {
        return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Token Factory not enabled. Enable via governance proposal."
        })));
    }
    
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
    // Check if token_factory feature is enabled
    if !state.config.read().await.features.token_factory_enabled {
        return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Token Factory not enabled. Enable via governance proposal."
        })));
    }
    
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
    // Check if token_factory feature is enabled
    if !state.config.read().await.features.token_factory_enabled {
        return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Token Factory not enabled. Enable via governance proposal."
        })));
    }
    
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
    // Check if token_factory feature is enabled
    if !state.config.read().await.features.token_factory_enabled {
        return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Token Factory not enabled. Enable via governance proposal."
        })));
    }
    
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

// Faucet challenge handler (Step 1: Get nonce to sign)
async fn handle_faucet_challenge(
    address: String,
    state: Arc<NodeState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match state.token_factory.generate_faucet_challenge(&address).await {
        Ok(nonce) => Ok(warp::reply::json(&serde_json::json!({
            "success": true,
            "address": address,
            "nonce": nonce,
            "message": "Sign this nonce with your wallet and POST to /faucet/claim",
            "expires_in_seconds": 300
        }))),
        Err(e) => Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

// Faucet claim handler (Step 2: Submit signed challenge)
async fn handle_faucet_claim(
    request: FaucetClaimRequest,
    state: Arc<NodeState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Parse signature
    let signature = match hex::decode(&request.signature) {
        Ok(s) => s,
        Err(_) => return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Invalid signature hex encoding"
        }))),
    };
    
    // Parse pubkey
    let pubkey_bytes = match hex::decode(&request.pubkey) {
        Ok(p) if p.len() == 32 => {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&p);
            arr
        },
        _ => return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Invalid pubkey: must be 32 bytes hex-encoded"
        }))),
    };
    
    match state.token_factory.claim_faucet_with_signature(
        &request.address,
        &request.nonce,
        &signature,
        &pubkey_bytes,
    ).await {
        Ok(amount) => Ok(warp::reply::json(&serde_json::json!({
            "success": true,
            "amount": amount.to_string(),
            "amount_sltn": amount / 1_000_000,
            "message": format!("Claimed {} SLTN to {}", amount / 1_000_000, request.address)
        }))),
        Err(e) => Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

// Faucet status handler
async fn handle_faucet_status(
    state: Arc<NodeState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let stats = state.token_factory.get_faucet_stats().await;
    Ok(warp::reply::json(&serde_json::json!({
        "success": true,
        "enabled": stats.enabled,
        "amount_per_claim": stats.amount_per_claim.to_string(),
        "amount_per_claim_sltn": stats.amount_per_claim / 1_000_000,
        "max_cap": stats.max_cap.to_string(),
        "max_cap_sltn": stats.max_cap / 1_000_000,
        "total_claims": stats.total_claims,
        "total_distributed": stats.total_distributed.to_string(),
        "total_distributed_sltn": stats.total_distributed / 1_000_000,
        "remaining": stats.remaining.to_string(),
        "remaining_sltn": stats.remaining / 1_000_000
    })))
}

// Admin: Toggle faucet (requires admin key from env)
async fn handle_faucet_toggle(
    enabled: bool,
    admin_key: String,
    state: Arc<NodeState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    use crate::token_factory::TokenFactory;
    
    // Check admin key from environment (constant-time comparison)
    let expected_key = std::env::var("SULTAN_ADMIN_KEY").unwrap_or_default();
    if expected_key.is_empty() || !TokenFactory::constant_time_compare(&admin_key, &expected_key) {
        return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Unauthorized: Invalid admin key"
        })));
    }
    
    if enabled {
        state.token_factory.enable_faucet();
    } else {
        state.token_factory.disable_faucet();
    }
    
    Ok(warp::reply::json(&serde_json::json!({
        "success": true,
        "faucet_enabled": state.token_factory.is_faucet_enabled(),
        "message": if enabled { "Faucet enabled (Phase 1)" } else { "Faucet disabled (Phase 2)" }
    })))
}

// DEX Handlers
async fn handle_create_pair(
    request: CreatePairRequest,
    state: Arc<NodeState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Check if native_dex feature is enabled
    if !state.config.read().await.features.native_dex_enabled {
        return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Native DEX not enabled. Enable via governance proposal."
        })));
    }
    
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
    // Check if native_dex feature is enabled
    if !state.config.read().await.features.native_dex_enabled {
        return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Native DEX not enabled. Enable via governance proposal."
        })));
    }
    
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
        &request.pair_id,        // pair_id first
        &request.from_address,   // then user address
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
    // Check if native_dex feature is enabled
    if !state.config.read().await.features.native_dex_enabled {
        return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Native DEX not enabled. Enable via governance proposal."
        })));
    }
    
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
    // Check if native_dex feature is enabled
    if !state.config.read().await.features.native_dex_enabled {
        return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "error": "Native DEX not enabled. Enable via governance proposal."
        })));
    }
    
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
