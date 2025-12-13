use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: String,
    pub gas_price: u64,
    pub block_time: u64,
    pub max_block_size: usize,
}

impl Default for ChainConfig {
    fn default() -> Self {
        ChainConfig {
            chain_id: String::from("sultan-1"),
            gas_price: 0, // Zero gas fees!
            block_time: 5, // 5 second blocks
            max_block_size: 1000,
        }
    }
}
