pub mod ethereum;
pub mod solana;
pub mod bitcoin;
pub mod ton;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteroperabilityLayer {
    pub ethereum_bridge: ethereum::bridge::EthereumBridge,
    pub solana_bridge: solana::bridge::SolanaBridge,
    pub bitcoin_bridge: bitcoin::bridge::BitcoinBridge,
    pub ton_bridge: ton::bridge::TonBridge,
    pub total_bridged_value: u64,
}

impl InteroperabilityLayer {
    pub fn new() -> Self {
        InteroperabilityLayer {
            ethereum_bridge: ethereum::bridge::EthereumBridge::new(),
            solana_bridge: solana::bridge::SolanaBridge::new(),
            bitcoin_bridge: bitcoin::bridge::BitcoinBridge::new(),
            ton_bridge: ton::bridge::TonBridge::new(),
            total_bridged_value: 0,
        }
    }
    
    pub fn get_supported_chains(&self) -> Vec<String> {
        vec![
            "Ethereum".to_string(),
            "Solana".to_string(),
            "Bitcoin".to_string(),
            "TON".to_string(),
        ]
    }
}
