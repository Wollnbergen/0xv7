use crate::blockchain::{Block, Blockchain};
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};

pub struct SimpleConsensus {
    blockchain: Arc<Mutex<Blockchain>>,
    is_validator: bool,
}

impl SimpleConsensus {
    pub fn new(blockchain: Arc<Mutex<Blockchain>>) -> Self {
        SimpleConsensus {
            blockchain,
            is_validator: true,
        }
    }

    pub async fn start(&self) {
        let mut interval = interval(Duration::from_secs(5));
        
        loop {
            interval.tick().await;
            
            if self.is_validator {
                let mut chain = self.blockchain.lock().unwrap();
                
                if !chain.pending_transactions.is_empty() {
                    let block = chain.create_block();
                    println!("⛏️  Created block #{} with {} transactions", 
                        block.index, 
                        block.transactions.len()
                    );
                } else {
                    println!("⏳ No transactions to process");
                }
            }
        }
    }
}

impl Block {
    pub fn gas_fee_total(&self) -> u64 {
        self.transactions.iter().map(|tx| tx.gas_fee).sum()
    }
}
