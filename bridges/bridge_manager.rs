use std::collections::HashMap;

pub struct BridgeManager {
    bridges: HashMap<String, BridgeInfo>,
    total_bridged_value: f64,
    sultan_total_fees: f64,  // Always 0!
}

impl BridgeManager {
    pub fn new() -> Self {
        let mut bridges = HashMap::new();
        
        bridges.insert("Bitcoin".to_string(), BridgeInfo {
            active: true,
            wrapped_token: "sBTC".to_string(),
            sultan_fee: 0.00,
            total_bridged: 0.0,
        });
        
        bridges.insert("Ethereum".to_string(), BridgeInfo {
            active: true,
            wrapped_token: "sETH".to_string(),
            sultan_fee: 0.00,
            total_bridged: 0.0,
        });
        
        bridges.insert("Solana".to_string(), BridgeInfo {
            active: true,
            wrapped_token: "sSOL".to_string(),
            sultan_fee: 0.00,
            total_bridged: 0.0,
        });
        
        bridges.insert("TON".to_string(), BridgeInfo {
            active: true,
            wrapped_token: "sTON".to_string(),
            sultan_fee: 0.00,
            total_bridged: 0.0,
        });
        
        Self {
            bridges,
            total_bridged_value: 0.0,
            sultan_total_fees: 0.00,  // Always 0!
        }
    }
    
    pub fn get_fee(&self, chain: &str) -> f64 {
        0.00  // Sultan Chain always has zero fees!
    }
}

pub struct BridgeInfo {
    active: bool,
    wrapped_token: String,
    sultan_fee: f64,  // Always 0!
    total_bridged: f64,
}
