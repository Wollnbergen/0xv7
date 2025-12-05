// Sultan Chain Core Library
pub mod blockchain;
pub mod config;
pub mod consensus;
pub mod rewards;
pub mod rpc_server;
pub mod scylla_db;
pub mod sdk;
pub mod transaction_validator;
pub mod types;
pub mod persistence;
pub mod p2p;
pub mod multi_consensus;
pub mod state_sync;

// Re-export main types
pub use blockchain::{Blockchain, Block};
pub use config::ChainConfig;
pub use sdk::SultanSDK;
pub use types::{Transaction, SultanToken, Account};  // Export Transaction

pub const VERSION: &str = "1.0.0";
pub const ZERO_GAS_FEE: u64 = 0;
pub const STAKING_APY: f64 = 0.2667;
