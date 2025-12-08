use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};

// Import Transaction from types module
use crate::types::Transaction;
use crate::config::ChainConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub validator: String,
}

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub config: ChainConfig,
}

impl Blockchain {
    pub fn new(config: ChainConfig) -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            pending_transactions: Vec::new(),
            config,
        };
        blockchain.create_genesis_block();
        blockchain
    }

    pub fn create_genesis_block(&mut self) {
        let genesis = Block {
            index: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            transactions: vec![],
            previous_hash: "0".to_string(),
            hash: "genesis".to_string(),
            validator: "sultan".to_string(),
        };
        self.chain.push(genesis);
    }

    pub fn create_block(&mut self, transactions: Vec<Transaction>) -> Block {
        let previous_block = self.chain.last().unwrap();
        let mut block = Block {
            index: self.chain.len() as u64,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            transactions,
            previous_hash: previous_block.hash.clone(),
            hash: String::new(),
            validator: "validator1".to_string(),
        };
        
        block.hash = self.calculate_hash(&block);
        self.chain.push(block.clone());
        block
    }

    fn calculate_hash(&self, block: &Block) -> String {
        let data = format!(
            "{}{}{}{}",
            block.index, block.timestamp, block.transactions.len(), block.previous_hash
        );
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
    
    pub fn get_latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }
    
    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
    }
}
