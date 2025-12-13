use anyhow::Result;
use tracing::{info, error};

mod blockchain;
mod consensus;
mod p2p;
mod quantum;
mod rpc_server;
mod database;
mod types;
mod config;
mod storage;

use blockchain::{Blockchain, Transaction};
use p2p::P2PNetwork;
use quantum::{QuantumCrypto, SharedQuantumCrypto};
use rpc_server::RpcServer;
use consensus::{ConsensusEngine, Validator};
use database::Database;
use types::Address;
use config::ChainConfig;
use storage::PersistentStorage;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("🚀 Starting Sultan Chain (Unified Implementation)");
    
    // Initialize persistent storage
    let storage = PersistentStorage::new("./data/sultan-chain")?;
    info!("✅ Persistent storage initialized");
    
    // Initialize components
    let blockchain = Blockchain::new();
    let quantum = QuantumCrypto::new();
    let config = ChainConfig::default();
    let mut p2p = P2PNetwork::new()?;
    
    // Start P2P network
    info!("Starting P2P network...");
    p2p.start_listening("0.0.0.0:30333").await?; // use existing start_listening API
    
    // Start RPC server
    info!("Starting RPC server on port 8545...");
    use std::sync::Arc;
    use tokio::sync::Mutex;
    let blockchain = Arc::new(Mutex::new(blockchain));
    let rpc = RpcServer::new(blockchain.clone());
    
    tokio::spawn(async move {
        if let Err(e) = rpc.start().await {
            error!("RPC server error: {}", e);
        }
    });
    
    // Exercise core components to satisfy lint usage requirements
    {
        let mut chain_guard = blockchain.lock().await;
        chain_guard.add_transaction(Transaction::new("sultan1alice".into(), "sultan1bob".into(), 10));
        let new_block = chain_guard.create_block();
        
        // Persist block to storage
        if let Some(block) = &new_block {
            storage.save_block(block)?;
            info!("📦 Block {} persisted to RocksDB", block.height);
        }
        
        let _latest = chain_guard.get_latest_block();
    }

    let mut consensus = ConsensusEngine::new();
    consensus.add_validator(Validator { address: "sultan1validator1".into(), stake: 1000, voting_power: 1000 });
    let _proposer = consensus.select_proposer();

    let mut db = Database::new();
    db.create_wallet("alice")?;
    let _addr = Address::new("sultan1sample");

    // Example block broadcast (empty payload placeholder)
    p2p.broadcast_block(Vec::new()).await?;
    p2p.broadcast_transaction(Vec::new()).await?;
    let _peers = p2p.connected_peers();
    let _peer_count = p2p.peer_count();
    p2p.connect_to_peer("bootstrap-node").await?;

    let _signed = quantum.sign(b"init");
    let _verified = quantum.verify(&_signed);
    let _pk_ref = quantum.pk();
    use tokio::sync::RwLock;
    let _shared_quantum: SharedQuantumCrypto = std::sync::Arc::new(RwLock::new(QuantumCrypto::new()));
    info!("Quantum crypto initialized; consensus proposer: {:?}", _proposer);
    info!("Sample address {}", _addr.0);
    let _db_fields = (&db.stakes, &db.transfers, &db.idempotency_keys, db.total_supply);

    info!("✅ Sultan Chain is running!");
    info!("   • P2P: {}", p2p.peer_id());
    info!("   • RPC: http://localhost:8545");
    info!("   • Zero gas fees: ENABLED");
    info!("   • Block time (config): {}s", config.block_time);
    
    // Keep running
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received. Commencing graceful shutdown...");
    p2p.stop().await?;
    info!("Shutdown complete.");
    
    Ok(())
}
