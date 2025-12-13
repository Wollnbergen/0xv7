// Core modules - only include what exists
pub mod blockchain;
pub mod config;
pub mod consensus;
pub mod types;

// Re-exports
pub use blockchain::Blockchain;
pub use config::ChainConfig;
pub use types::Transaction;
