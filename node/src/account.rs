//! Sultan Chain Account Management
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub address: String,
    pub balance: u128,
    pub nonce: u64,
}

impl Account {
    pub fn new(address: String) -> Self {
        Self {
            address,
            balance: 0,
            nonce: 0,
        }
    }
    
    pub fn transfer(&mut self, amount: u128) -> Result<(), String> {
        if self.balance >= amount {
            self.balance -= amount;
            self.nonce += 1;
            Ok(())
        } else {
            Err("Insufficient balance".to_string())
        }
    }
}
