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

// Add your shared structs and enums here
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub tx_hash: String,
    pub block_height: u64,
    pub from_address: String,
    pub to_address: String,
    pub amount: u128,
    pub signature: String,
    pub subsidy_flag: bool,
    pub stake: f64, // For min stake check
    pub subsidy: f64, // For daily subsidy
    pub data: Vec<u8>,
    pub interop_flag: bool, // Patch: For atomic swaps/light clients in interop
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ValidatorInfo {
    pub id: String,
    pub stake: u64,
    pub voting_power: u64, // Democratic: always 1
    pub device_type: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SultanToken {
    pub balance: HashMap<String, u64>,
    pub supply: u64,
}

impl SultanToken {
    pub fn new() -> Self {
        Self {
            balance: HashMap::new(),
            supply: 0,
        }
    }
    pub fn get_supply(&self) -> u64 {
        self.supply
    }
    pub fn mint_inflation(&mut self, amount: u64) -> Result<()> {
        self.supply += amount;
        println!("Minted {} SLTN at 5.0% inflation", amount);
        Ok(())
    }
    pub fn allocate_inflation(&mut self, amount: u64, _total_stake: u64) -> Result<()> { // Prefix _ for unused
        // Stub: Print for log, split inflation (60% rewards, 30% subsidies, 10% burned, adaptive load-based rate capped 10%)
        let rewards = (amount as f64 * 0.6) as u64;
        let subsidies = (amount as f64 * 0.3) as u64;
        let burned = amount - rewards - subsidies;
        println!("Inflation allocated: {} rewards, {} subsidies, {} burned", rewards, subsidies, burned);
        self.supply += amount;
        Ok(())
    }
    pub fn set_rate(&mut self, rate: f64) -> Result<()> {
        // Stub set_rate for transaction_validator.rs (dynamic rate based on load)
        println!("Token rate set to {}", rate);
        Ok(())
    }
}

impl Default for SultanToken {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Vote {
    pub proposal_id: u64,
    pub validator_id: String,
    pub vote_yes: bool,
    pub stake_weight: u64,
    pub sig: Option<Vec<u8>>, // Patch: Add for Dilithium signature storage (post-quantum secure)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Proposal {
    pub id: u64,
    pub description: String,
    pub chain_name: String,
    pub interop_chain: String, // Patch: For BTC/ETH/SOL/TON interop gating
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