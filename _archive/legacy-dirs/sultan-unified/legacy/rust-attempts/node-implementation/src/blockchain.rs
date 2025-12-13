use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub prev_hash: String,
    pub hash: String,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub gas_fee: u64, // Always 0 for Sultan Chain
    pub timestamp: u64,
}

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis = Block {
            index: 0,
            timestamp: Self::current_timestamp(),
            transactions: vec![],
            prev_hash: String::from("0"),
            hash: String::from("genesis"),
            nonce: 0,
        };
        
        Blockchain {
            chain: vec![genesis],
            pending_transactions: vec![],
        }
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.pending_transactions.push(tx);
    }

    pub fn create_block(&mut self) -> Block {
        let prev_block = self.chain.last().unwrap();
        let mut block = Block {
            index: prev_block.index + 1,
            timestamp: Self::current_timestamp(),
            transactions: self.pending_transactions.clone(),
            prev_hash: prev_block.hash.clone(),
            hash: String::new(),
            nonce: 0,
        };
        
        block.hash = Self::calculate_hash(&block);
        self.pending_transactions.clear();
        self.chain.push(block.clone());
        block
    }

    fn calculate_hash(block: &Block) -> String {
        let data = format!("{}{}{:?}{}", 
            block.index, 
            block.timestamp, 
            block.transactions,
            block.prev_hash
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub fn get_latest_block(&self) -> &Block {
        self.chain.last().unwrap()
    }
}

impl Transaction {
    pub fn new(from: String, to: String, amount: u64) -> Self {
        Transaction {
            from,
            to,
            amount,
            gas_fee: 0, // Zero gas fees!
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}
