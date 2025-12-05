use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SultanToken {
    pub symbol: String,
    pub total_supply: u128,
    pub decimals: u8,
}

impl Default for SultanToken {
    fn default() -> Self {
        Self {
            symbol: "SLTN".to_string(),
            total_supply: 1_000_000_000_000_000_000, // 1 billion with 18 decimals
            decimals: 18,
        }
    }
}

// Make Transaction public
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub gas_fee: u64, // Always 0
    pub timestamp: u64,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
}
