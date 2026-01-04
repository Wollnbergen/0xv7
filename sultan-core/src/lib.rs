//! Sultan Core - Layer 1 Blockchain
//!
//! This is the core Sultan blockchain implementation in Rust.
//! It provides the fundamental blockchain logic, consensus, and state management.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use sultan_core::{init, init_with_genesis, VERSION};
//!
//! // Initialize with default configuration
//! let blockchain = init().unwrap();
//!
//! // Or with genesis accounts
//! let blockchain = init_with_genesis(vec![
//!     ("alice".to_string(), 1_000_000),
//!     ("bob".to_string(), 500_000),
//! ]).unwrap();
//!
//! println!("Sultan Core v{}", VERSION);
//! ```
//!
//! # Feature Flags (Hot-Upgrades)
//!
//! Sultan supports governance-activated feature flags for runtime upgrades:
//!
//! - `smart_contracts_enabled`: Smart contracts (VM to be selected post-launch)
//! - `quantum_signatures_enabled`: Dilithium3 quantum-resistant signatures
//! - `bridges_enabled`: Cross-chain bridges (BTC, ETH, SOL, TON)
//!
//! See [`Config`] and [`FeatureFlags`] for details.

pub mod blockchain;
pub mod block_sync;
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
pub use types::{Address, AddressError};
pub use config::{Config, FeatureFlags};
pub use transaction_validator::TransactionValidator;

// Production sharding - the unified Sultan blockchain
pub use sharding_production::{ShardingCoordinator as ProductionShardingCoordinator, ShardConfig as ProductionShardConfig};
#[allow(deprecated)]
pub use sharded_blockchain_production::{SultanBlockchain, ShardedBlockchainProduction, ConfirmedTransaction};

// Legacy types (deprecated - for backward compatibility only)
#[deprecated(note = "Use SultanBlockchain - sharding is always enabled")]
pub use sharding::{ShardingCoordinator, ShardConfig, ShardStats};
#[deprecated(note = "Use SultanBlockchain instead")]
pub use sharded_blockchain::ShardedBlockchain;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_blockchain() {
        let blockchain = init().unwrap();
        assert_eq!(blockchain.height(), 0);
    }

    #[test]
    fn test_init_with_genesis_accounts() {
        let blockchain = init_with_genesis(vec![
            ("alice".to_string(), 1_000_000),
            ("bob".to_string(), 500_000),
        ]).unwrap();
        
        assert_eq!(blockchain.get_balance("alice"), 1_000_000);
        assert_eq!(blockchain.get_balance("bob"), 500_000);
        assert_eq!(blockchain.get_balance("charlie"), 0);
    }

    #[test]
    fn test_version_exists() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_address_validation() {
        // Valid address
        let valid = "sultan1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqtest";
        assert!(Address::new(valid).is_ok());
        
        // Invalid prefix
        let invalid_prefix = "cosmos1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqtest";
        assert!(matches!(
            Address::new(invalid_prefix),
            Err(AddressError::InvalidPrefix)
        ));
        
        // Invalid length
        let invalid_len = "sultan1short";
        assert!(matches!(
            Address::new(invalid_len),
            Err(AddressError::InvalidLength(_))
        ));
    }

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        
        // Default feature flags
        assert!(config.features.sharding_enabled);
        assert!(config.features.governance_enabled);
        assert!(config.features.bridges_enabled);
        assert!(!config.features.wasm_contracts_enabled);
        assert!(!config.features.evm_contracts_enabled);
        assert!(!config.features.ibc_enabled);
        
        // Zero gas fees!
        assert_eq!(config.gas_price, 0);
    }

    #[test]
    fn test_feature_flag_update() {
        let mut config = Config::default();
        
        assert!(!config.features.wasm_contracts_enabled);
        config.update_feature("wasm_contracts_enabled", true).unwrap();
        assert!(config.features.wasm_contracts_enabled);
        
        // Unknown feature should fail
        assert!(config.update_feature("unknown_feature", true).is_err());
    }
}
