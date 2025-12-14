//! Sultan Core Blockchain Implementation
//! 
//! Production-grade blockchain with proper validation, state management,
//! and transaction processing.

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use sha3::Keccak256;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use anyhow::{Result, bail};
use tracing::{info, warn};

/// Block represents a single block in the Sultan blockchain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub prev_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub validator: String,
    pub state_root: String,
}

/// Transaction represents a value transfer on Sultan Chain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub gas_fee: u64,
    pub timestamp: u64,
    pub nonce: u64,
    pub signature: Option<String>,
}

/// Account state in the blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub balance: u64,
    pub nonce: u64,
}

/// Main blockchain structure with state management
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub state: HashMap<String, Account>,
    pub transaction_pool: HashMap<String, Transaction>,
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}

impl Blockchain {
    /// Create new blockchain with genesis block
    pub fn new() -> Self {
        let genesis = Block {
            index: 0,
            timestamp: Self::current_timestamp(),
            transactions: vec![],
            prev_hash: String::from("0"),
            hash: String::new(),
            nonce: 0,
            validator: String::from("genesis"),
            state_root: String::from("0"),
        };
        
        let genesis_hash = Self::calculate_block_hash(&genesis);
        let mut genesis_final = genesis;
        genesis_final.hash = genesis_hash;
        
        info!("Genesis block created: {}", genesis_final.hash);
        
        Blockchain {
            chain: vec![genesis_final],
            pending_transactions: vec![],
            state: HashMap::new(),
            transaction_pool: HashMap::new(),
        }
    }

    /// Add transaction to pool with validation
    pub fn add_transaction(&mut self, tx: Transaction) -> Result<()> {
        // Validate transaction
        if tx.amount == 0 {
            bail!("Transaction amount must be greater than 0");
        }

        if tx.gas_fee != 0 {
            bail!("Sultan Chain has zero gas fees");
        }

        // Check sender has sufficient balance
        if let Some(account) = self.state.get(&tx.from) {
            if account.balance < tx.amount {
                bail!("Insufficient balance: {} < {}", account.balance, tx.amount);
            }
            if account.nonce >= tx.nonce {
                bail!("Invalid nonce: expected > {}, got {}", account.nonce, tx.nonce);
            }
        } else if tx.amount > 0 {
            // New account must have balance (genesis allocation or previous block)
            bail!("Account {} not found in state", tx.from);
        }

        let tx_hash = Self::calculate_tx_hash(&tx);
        
        if self.transaction_pool.contains_key(&tx_hash) {
            bail!("Transaction already in pool");
        }

        info!("Transaction added to pool: {} -> {} ({})", tx.from, tx.to, tx.amount);
        self.transaction_pool.insert(tx_hash, tx.clone());
        self.pending_transactions.push(tx);
        
        Ok(())
    }

    /// Create new block with pending transactions
    pub fn create_block(&mut self, validator: String) -> Result<Block> {
        let prev_block = self.get_latest_block();
        let prev_index = prev_block.index;
        let prev_hash = prev_block.hash.clone();
        
        // Move transactions from pool to block
        let transactions = self.pending_transactions.drain(..).collect::<Vec<_>>();
        
        if transactions.is_empty() {
            warn!("Creating empty block at height {}", prev_index + 1);
        }

        // Apply transactions and calculate new state root
        let mut new_state = self.state.clone();
        for tx in &transactions {
            self.apply_transaction(&mut new_state, tx)?;
        }
        
        let state_root = Self::calculate_state_root(&new_state);
        
        let mut block = Block {
            index: prev_index + 1,
            timestamp: Self::current_timestamp(),
            transactions: transactions.clone(),
            prev_hash,
            hash: String::new(),
            nonce: 0,
            validator: validator.clone(),
            state_root,
        };
        
        block.hash = Self::calculate_block_hash(&block);
        
        // Commit state changes
        self.state = new_state;
        
        // Clear transaction pool for included transactions
        for tx in &transactions {
            let tx_hash = Self::calculate_tx_hash(tx);
            self.transaction_pool.remove(&tx_hash);
        }
        
        // Note: Block is NOT added to chain here - caller must validate and add
        
        info!(
            "Block {} created by {} with {} transactions",
            block.index,
            validator,
            transactions.len()
        );
        
        Ok(block)
    }

    /// Validate block before acceptance
    pub fn validate_block(&self, block: &Block) -> Result<bool> {
        let prev_block = self.get_latest_block();
        
        // Check index
        if block.index != prev_block.index + 1 {
            bail!("Invalid block index: expected {}, got {}", prev_block.index + 1, block.index);
        }

        // Check previous hash
        if block.prev_hash != prev_block.hash {
            bail!("Invalid previous hash");
        }

        // Check timestamp
        if block.timestamp <= prev_block.timestamp {
            bail!("Block timestamp must be greater than previous block");
        }

        // Verify block hash
        let calculated_hash = Self::calculate_block_hash(block);
        if block.hash != calculated_hash {
            bail!("Invalid block hash");
        }

        // Validate all transactions
        let mut temp_state = self.state.clone();
        for tx in &block.transactions {
            if !self.validate_transaction(tx)? {
                bail!("Invalid transaction in block");
            }
            self.apply_transaction(&mut temp_state, tx)?;
        }

        // Verify state root
        let calculated_state_root = Self::calculate_state_root(&temp_state);
        if block.state_root != calculated_state_root {
            bail!("Invalid state root");
        }

        Ok(true)
    }

    /// Validate individual transaction
    fn validate_transaction(&self, tx: &Transaction) -> Result<bool> {
        if tx.amount == 0 {
            return Ok(false);
        }

        if tx.gas_fee != 0 {
            return Ok(false);
        }

        if let Some(account) = self.state.get(&tx.from) {
            if account.balance < tx.amount {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }

        Ok(true)
    }

    /// Apply transaction to state
    pub fn apply_transaction(&self, state: &mut HashMap<String, Account>, tx: &Transaction) -> Result<()> {
        // Deduct from sender
        let sender = state.entry(tx.from.clone()).or_insert(Account {
            balance: 0,
            nonce: 0,
        });
        
        if sender.balance < tx.amount {
            bail!("Insufficient balance");
        }
        
        sender.balance -= tx.amount;
        sender.nonce += 1;

        // Add to receiver
        let receiver = state.entry(tx.to.clone()).or_insert(Account {
            balance: 0,
            nonce: 0,
        });
        receiver.balance += tx.amount;

        Ok(())
    }

    /// Calculate block hash using SHA256
    fn calculate_block_hash(block: &Block) -> String {
        let data = format!(
            "{}{}{}{}{}{}{}",
            block.index,
            block.timestamp,
            bincode::serialize(&block.transactions).unwrap_or_default().len(),
            block.prev_hash,
            block.nonce,
            block.validator,
            block.state_root
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Calculate transaction hash
    fn calculate_tx_hash(tx: &Transaction) -> String {
        let data = format!("{}{}{}{}{}", tx.from, tx.to, tx.amount, tx.nonce, tx.timestamp);
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Calculate state root using Keccak256 (Ethereum-style)
    fn calculate_state_root(state: &HashMap<String, Account>) -> String {
        let mut accounts: Vec<_> = state.iter().collect();
        accounts.sort_by_key(|(addr, _)| *addr);
        
        let mut hasher = Keccak256::new();
        for (addr, account) in accounts {
            hasher.update(addr.as_bytes());
            hasher.update(&account.balance.to_le_bytes());
            hasher.update(&account.nonce.to_le_bytes());
        }
        
        format!("{:x}", hasher.finalize())
    }

    /// Get current timestamp in seconds
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX epoch")
            .as_secs()
    }

    /// Get latest block
    pub fn get_latest_block(&self) -> &Block {
        self.chain.last().expect("Chain must have at least genesis block")
    }

    /// Get block by index
    pub fn get_block(&self, index: u64) -> Option<&Block> {
        self.chain.get(index as usize)
    }

    /// Get account balance
    pub fn get_balance(&self, address: &str) -> u64 {
        self.state.get(address).map(|acc| acc.balance).unwrap_or(0)
    }

    /// Get account nonce
    pub fn get_nonce(&self, address: &str) -> u64 {
        self.state.get(address).map(|acc| acc.nonce).unwrap_or(0)
    }

    /// Initialize account with balance (for genesis or testing)
    pub fn init_account(&mut self, address: String, balance: u64) {
        self.state.insert(address.clone(), Account { balance, nonce: 0 });
        info!("Account initialized: {} with balance {}", address, balance);
    }

    /// Get chain height
    pub fn height(&self) -> u64 {
        self.chain.len() as u64 - 1
    }
}

impl Transaction {
    /// Create new transaction
    pub fn new(from: String, to: String, amount: u64, nonce: u64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX epoch")
            .as_secs();
            
        Transaction {
            from,
            to,
            amount,
            gas_fee: 0,
            timestamp,
            nonce,
            signature: None,
        }
    }

    /// Sign transaction (placeholder for quantum crypto integration)
    pub fn sign(&mut self, _private_key: &str) {
        // Will integrate with quantum.rs for actual signatures
        self.signature = Some(String::from("signed"));
    }
}
