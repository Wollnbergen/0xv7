//! Chain Configuration Module

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: String,
    pub block_time_ms: u64,
    pub max_validators: usize,
    pub min_stake: u64,
    pub inflation_rate: f64,
    pub gas_price: u64,      // Added: Always 0 for zero gas fees
    pub staking_apy: f64,    // Added: 26.67% APY
    pub max_block_size: usize,
    pub enable_smart_contracts: bool,
    pub consensus_type: String,
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self {
            chain_id: "sultan-1".to_string(),
            block_time_ms: 5000,
            max_validators: 100,
            min_stake: 5000,
            inflation_rate: 0.08,
            gas_price: 0,  // Zero gas fees forever!
            staking_apy: 0.2667,  // 26.67% APY
            max_block_size: 1_000_000,
            enable_smart_contracts: true,
            consensus_type: "PoS".to_string(),
        }
    }
}
