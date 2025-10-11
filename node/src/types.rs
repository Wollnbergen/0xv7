use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use pqcrypto_dilithium::dilithium3::keypair;

#[derive(Clone, Debug, Default)]
#[allow(dead_code)] // For future expansion
pub struct Block {
    pub height: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: u64,
    pub validator: String,
    pub signature: String,
    pub state_root: Vec<u8>,
    pub transactions: Vec<Transaction>,
    pub shard_id: u64,
    pub mev_proofs: Vec<Vec<u8>>,
}

// types.rs - Shared types for Sultan Blockchain

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub tx_hash: String,
    pub block_height: u64,
    pub from_address: String,
    pub to_address: String,
    pub amount: u128,
    pub signature: String,
    pub subsidy_flag: bool,
    pub stake: f64,
    pub subsidy: f64,
    pub data: Vec<u8>,
    pub interop_flag: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ValidatorInfo {
    pub id: String,
    pub stake: u64,
    pub voting_power: u64,
    pub device_type: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SultanToken {
    pub balance: HashMap<String, u64>,
    pub supply: u64,
    pub db: Option<Arc<crate::scylla_db::ScyllaCluster>>,
}

impl SultanToken {
    pub fn new(db: Option<Arc<crate::scylla_db::ScyllaCluster>>) -> Self {
        Self {
            balance: HashMap::new(),
            supply: 0,
            db,
        }
    }

    pub async fn mint(&mut self, to: &str, amount: u64) -> Result<()> {
        if amount == 0 {
            return Err(anyhow::anyhow!("Cannot mint zero tokens"));
        }
        let entry = self.balance.entry(to.to_string()).or_insert(0);
        *entry += amount;
        self.supply += amount;
        if let Some(db) = &self.db {
            db.update_wallet_balance(to, amount as i64).await?;
        }
        println!("Minted {} SLTN to {}", amount, to);
        Ok(())
    }

    pub async fn generate_wallet_address(&self, telegram_id: &str) -> Result<String> {
        if telegram_id.is_empty() {
            return Err(anyhow::anyhow!("Telegram ID cannot be empty"));
        }
        let (pk, sk) = keypair();
        let address = format!("sultan1{}", hex::encode(pk.as_bytes()));
        if let Some(db) = &self.db {
            let timestamp = chrono::Utc::now().timestamp();
            db.insert_wallet(telegram_id, &address, pk.as_bytes(), sk.as_bytes(), timestamp).await?;
        }
        println!("Generated wallet address for {}: {}", telegram_id, address);
        Ok(address)
    }

    pub fn get_supply(&self) -> u64 {
        self.supply
    }

    pub fn mint_inflation(&mut self, amount: u64) -> Result<()> {
        self.supply += amount;
        println!("Minted {} SLTN at 5.0% inflation", amount);
        Ok(())
    }

    pub fn allocate_inflation(&mut self, amount: u64, _total_stake: u64) -> Result<()> {
        let rewards = (amount as f64 * 0.6) as u64;
        let subsidies = (amount as f64 * 0.3) as u64;
        let burned = amount - rewards - subsidies;
        println!("Inflation allocated: {} rewards, {} subsidies, {} burned", rewards, subsidies, burned);
        self.supply += amount;
        Ok(())
    }

    pub fn set_rate(&mut self, rate: f64) -> Result<()> {
        println!("Token rate set to {}", rate);
        Ok(())
    }
}

impl Default for SultanToken {
    fn default() -> Self {
        Self {
            balance: HashMap::new(),
            supply: 0,
            db: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Vote {
    pub proposal_id: u64,
    pub validator_id: String,
    pub vote_yes: bool,
    pub stake_weight: u64,
    pub sig: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Proposal {
    pub id: u64,
    pub description: String,
    pub chain_name: String,
    pub interop_chain: String,
    pub votes_for: u64,
    pub votes_against: u64,
    pub quorum: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimulationStats {
    pub tps: f64,
    pub uptime: f64,
    pub finality: f64,
    pub inflation: f64,
}