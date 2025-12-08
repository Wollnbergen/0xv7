//! Sultan Core - Layer 1 Blockchain
//!
//! This is the core Sultan blockchain implementation in Rust.
//! It provides the fundamental blockchain logic, consensus, and state management.

pub mod blockchain;
pub mod consensus;
pub mod p2p;
pub mod quantum;
pub mod database;
pub mod storage;
pub mod types;
pub mod config;
pub mod economics;
pub mod transaction_validator;
pub mod sharding;
pub mod sharding_production;
pub mod sharded_blockchain;
pub mod sharded_blockchain_production;
pub mod bridge_integration;
pub mod bridge_fees;
pub mod staking;
pub mod governance;
pub mod token_factory;
pub mod native_dex;

// Re-export main types for convenience
pub use blockchain::{Blockchain, Block, Transaction, Account};
pub use consensus::{ConsensusEngine, Validator};
pub use p2p::{P2PNetwork, NetworkMessage};
pub use quantum::QuantumCrypto;
pub use storage::PersistentStorage;
pub use types::Address;
pub use config::Config;
pub use transaction_validator::TransactionValidator;
// Legacy sharding (deprecated - tests only)
#[deprecated(note = "Use ShardedBlockchainProduction from sharding_production module")]
pub use sharding::{ShardingCoordinator, ShardConfig, ShardStats};
pub use sharding_production::{ShardingCoordinator as ProductionShardingCoordinator, ShardConfig as ProductionShardConfig};
#[deprecated(note = "Use ShardedBlockchainProduction instead")]
pub use sharded_blockchain::ShardedBlockchain;
pub use sharded_blockchain_production::ShardedBlockchainProduction;

/// Sultan Core version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize Sultan blockchain with default configuration
pub fn init() -> anyhow::Result<Blockchain> {
    Ok(Blockchain::new())
}

/// Initialize Sultan blockchain with genesis accounts
pub fn init_with_genesis(accounts: Vec<(String, u64)>) -> anyhow::Result<Blockchain> {
    let mut blockchain = Blockchain::new();
    
    for (address, balance) in accounts {
        blockchain.init_account(address, balance);
    }
    
    Ok(blockchain)
}
