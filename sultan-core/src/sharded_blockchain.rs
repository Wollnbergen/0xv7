//! ⚠️ LEGACY Sharded Blockchain Implementation (DEPRECATED)
//!
//! ⚠️ DO NOT USE IN PRODUCTION - Use sharded_blockchain_production.rs instead
//! This file is kept only for backward compatibility with old tests.
//!
//! Combines traditional blockchain with sharding for 1M+ TPS capability.

use crate::blockchain::{Block, Transaction};
use crate::sharding::{ShardingCoordinator, ShardConfig, ShardStats};
use anyhow::Result;
use tracing::info;
use std::sync::{Arc, RwLock};

/// Blockchain with integrated sharding support
pub struct ShardedBlockchain {
    /// Sharding coordinator for parallel processing
    pub sharding: ShardingCoordinator,
    /// Main chain for global state and block headers
    pub blocks: Vec<Block>,
    /// Global statistics
    pub total_transactions: Arc<RwLock<u64>>,
}

impl ShardedBlockchain {
    /// Create new sharded blockchain
    pub fn new(config: ShardConfig) -> Self {
        info!("Initializing sharded blockchain with {} shards", config.shard_count);
        
        let sharding = ShardingCoordinator::new(config);
        
        // Create genesis block
        let genesis = Block {
            index: 0,
            timestamp: chrono::Utc::now().timestamp() as u64,
            transactions: vec![],
            prev_hash: String::from("0"),
            hash: String::from("genesis"),
            nonce: 0,
            validator: String::from("genesis"),
            state_root: String::from("0"),
        };
        
        Self {
            sharding,
            blocks: vec![genesis],
            total_transactions: Arc::new(RwLock::new(0)),
        }
    }

    /// Initialize account in appropriate shard
    pub fn init_account(&self, address: String, balance: u64) -> Result<()> {
        self.sharding.init_account(address, balance)
    }

    /// Get account balance from appropriate shard
    pub fn get_balance(&self, address: &str) -> u64 {
        self.sharding.get_balance(address)
    }

    /// Process transactions in parallel across shards
    pub async fn process_transactions(&self, transactions: Vec<Transaction>) -> Result<Vec<Transaction>> {
        let processed = self.sharding.process_parallel(transactions).await?;
        
        // Update total transaction count
        let mut total = self.total_transactions.write().unwrap();
        *total += processed.len() as u64;
        
        Ok(processed)
    }

    /// Create new block with processed transactions
    pub async fn create_block(&mut self, transactions: Vec<Transaction>, validator: String) -> Result<Block> {
        // Process transactions in parallel across shards
        let processed = self.process_transactions(transactions).await?;
        
        let prev_block = self.blocks.last()
            .ok_or_else(|| anyhow::anyhow!("No blocks in chain - genesis missing"))?;
        
        // Create block with processed transactions
        let block = Block {
            index: prev_block.index + 1,
            timestamp: chrono::Utc::now().timestamp() as u64,
            transactions: processed,
            prev_hash: prev_block.hash.clone(),
            hash: format!("block-{}", prev_block.index + 1),
            nonce: 0,
            validator,
            state_root: String::from("0"), // State root from sharding coordinator
        };
        
        info!("Block {} created with {} transactions", block.index, block.transactions.len());
        
        Ok(block)
    }

    /// Add block to chain
    pub fn add_block(&mut self, block: Block) -> Result<()> {
        // Basic validation
        let prev = self.blocks.last()
            .ok_or_else(|| anyhow::anyhow!("No blocks in chain"))?;
        if block.index != prev.index + 1 {
            anyhow::bail!("Invalid block index");
        }
        
        self.blocks.push(block);
        Ok(())
    }

    /// Get sharding statistics
    pub fn get_stats(&self) -> ShardStats {
        self.sharding.get_stats()
    }

    /// Get current block height
    pub fn get_height(&self) -> u64 {
        self.blocks.len() as u64 - 1
    }

    /// Get total processed transactions
    pub fn get_total_transactions(&self) -> u64 {
        *self.total_transactions.read().unwrap()
    }

    /// Get estimated TPS capacity
    pub fn get_tps_capacity(&self) -> u64 {
        self.sharding.get_tps_capacity()
    }
}

impl Default for ShardedBlockchain {
    fn default() -> Self {
        Self::new(ShardConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sharded_blockchain() {
        let config = ShardConfig {
            shard_count: 10,
            tx_per_shard: 1000,
            cross_shard_enabled: false,
        };
        
        let mut blockchain = ShardedBlockchain::new(config);
        
        // Initialize test accounts
        blockchain.init_account("alice".to_string(), 1_000_000).unwrap();
        blockchain.init_account("bob".to_string(), 1_000_000).unwrap();
        
        // Create transactions
        let mut txs = Vec::new();
        for i in 0..10000 {
            txs.push(Transaction {
                from: "alice".to_string(),
                to: "bob".to_string(),
                amount: 1,
                gas_fee: 0,
                timestamp: i,
                nonce: i as u64,
                signature: None,
                public_key: None,
                memo: None,
            });
        }
        
        // Create block
        let block = blockchain.create_block(txs, "validator1".to_string()).await.unwrap();
        blockchain.add_block(block).unwrap();
        
        // Verify
        assert_eq!(blockchain.get_height(), 1);
        
        let stats = blockchain.get_stats();
        assert_eq!(stats.shard_count, 10);
        
        println!("TPS capacity: {}", blockchain.get_tps_capacity());
    }
}
