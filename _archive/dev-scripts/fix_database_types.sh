#!/bin/bash

# Update database.rs to define types locally or import correctly
cat > node/src/database.rs << 'DATABASE'
use std::collections::HashMap;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tracing::info;

// Define Wallet type locally
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
    pub created_at: i64,
    pub is_validator: bool,
}

// Define Stake type locally
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stake {
    pub validator: String,
    pub amount: u64,
    pub timestamp: i64,
}

// Transfer type for token transfers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub timestamp: i64,
    pub memo: Option<String>,
}

#[derive(Debug)]
pub struct Database {
    pub wallets: HashMap<String, Wallet>,
    pub stakes: HashMap<String, Stake>,
    pub transfers: HashMap<String, Transfer>,
    pub idempotency_keys: HashMap<String, String>,
    pub total_supply: u64,
}

impl Database {
    pub fn new() -> Self {
        Self {
            wallets: HashMap::new(),
            stakes: HashMap::new(),
            transfers: HashMap::new(),
            idempotency_keys: HashMap::new(),
            total_supply: 0,
        }
    }

    pub fn create_wallet(&mut self, address: &str) -> Result<()> {
        let full_address = if address.starts_with("sultan1") {
            address.to_string()
        } else {
            format!("sultan1{}", address)
        };
        
        if self.wallets.contains_key(&full_address) {
            return Ok(()); // Already exists
        }
        
        let wallet = Wallet {
            address: full_address.clone(),
            balance: 0,
            nonce: 0,
            created_at: chrono::Utc::now().timestamp(),
            is_validator: false,
        };
        
        self.wallets.insert(full_address.clone(), wallet);
        info!("Created new wallet: {}", full_address);
        Ok(())
    }
}
DATABASE

echo "✅ Database.rs fixed with local type definitions"
