use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub chain_id: String,
    pub gas_price: u64,
    pub block_time: u64,
    pub max_block_size: usize,
    pub min_stake: u64,
    pub inflation_rate: f64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            chain_id: String::from("sultan-1"),
            gas_price: 0, // Zero gas fees!
            block_time: 2, // 2 second blocks
            max_block_size: 1000,
            min_stake: 1000,
            inflation_rate: 0.08, // 8% annual inflation
        }
    }
}
