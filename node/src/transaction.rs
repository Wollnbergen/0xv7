//! Sultan Chain Transaction Processing
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u128,
    pub gas_fee: u128, // Always 0 for Sultan Chain
    pub nonce: u64,
    pub signature: Vec<u8>,
}

impl Transaction {
    pub fn new(from: String, to: String, amount: u128, nonce: u64) -> Self {
        Self {
            from,
            to,
            amount,
            gas_fee: 0, // Zero gas fees forever
            nonce,
            signature: Vec::new(),
        }
    }
    
    pub fn validate(&self) -> Result<(), String> {
        // Ensure gas fee is always zero
        if self.gas_fee != 0 {
            return Err("Gas fees must be zero".to_string());
        }
        
        if self.amount == 0 {
            return Err("Amount must be greater than zero".to_string());
        }
        
        Ok(())
    }
}
