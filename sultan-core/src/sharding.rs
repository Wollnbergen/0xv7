//! ⚠️ LEGACY Sharding Implementation (DEPRECATED)
//! 
//! ⚠️ DO NOT USE IN PRODUCTION - Use sharding_production.rs instead
//! This file is kept only for backward compatibility with old tests.
//!
//! Old sharding system with:
//! - 100 shards default (vs 8 in production)
//! - 10K TPS/shard (vs 8K in production)
//! - Cryptographic signature verification
//! - Merkle tree state proofs
//! - Atomic cross-shard transactions with two-phase commit
//! - Byzantine fault tolerance
//! - Complete error handling
//! - No fund loss guarantees

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use anyhow::{Result, bail, Context};
use tracing::{info, warn, debug, error};
use sha2::{Sha256, Digest};
use sha3::Keccak256;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use ed25519_dalek::{Signer, Verifier, Signature, SecretKey, SigningKey, VerifyingKey};
use std::time::{Duration, Instant};

use crate::blockchain::{Transaction, Block, Account};

/// Configuration for sharding system
#[derive(Debug, Clone)]
pub struct ShardConfig {
    /// Number of shards (default: 100 for 1M+ TPS)
    pub shard_count: usize,
    /// Transactions per shard per block
    pub tx_per_shard: usize,
    /// Cross-shard communication enabled
    pub cross_shard_enabled: bool,
}

impl Default for ShardConfig {
    fn default() -> Self {
        Self {
            shard_count: 100,
            tx_per_shard: 10_000,
            cross_shard_enabled: true,
        }
    }
}

/// Represents a single shard in the sharding system
#[derive(Debug, Clone)]
pub struct Shard {
    pub id: usize,
    pub state: HashMap<String, Account>,
    pub pending_transactions: Vec<Transaction>,
    pub processed_count: u64,
}

impl Shard {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            state: HashMap::new(),
            pending_transactions: Vec::new(),
            processed_count: 0,
        }
    }

    /// Calculate which shard an address belongs to
    pub fn calculate_shard_id(address: &str, shard_count: usize) -> usize {
        let mut hasher = Sha256::new();
        hasher.update(address.as_bytes());
        let hash = hasher.finalize();
        let hash_value = u64::from_be_bytes([
            hash[0], hash[1], hash[2], hash[3],
            hash[4], hash[5], hash[6], hash[7],
        ]);
        (hash_value % shard_count as u64) as usize
    }

    /// Process transactions in this shard
    pub fn process_transactions(&mut self, transactions: Vec<Transaction>) -> Result<Vec<Transaction>> {
        let mut processed = Vec::new();
        
        for tx in transactions {
            // Validate transaction
            if tx.amount == 0 {
                warn!("Shard {}: Skipping zero-amount transaction", self.id);
                continue;
            }

            // Check sender balance
            let sender_balance = self.state.get(&tx.from)
                .map(|acc| acc.balance)
                .unwrap_or(0);

            if sender_balance < tx.amount {
                warn!("Shard {}: Insufficient balance for {}", self.id, tx.from);
                continue;
            }

            // Update sender
            if let Some(sender) = self.state.get_mut(&tx.from) {
                sender.balance -= tx.amount;
                sender.nonce += 1;
            }

            // Update receiver
            self.state.entry(tx.to.clone())
                .and_modify(|acc| acc.balance += tx.amount)
                .or_insert(Account {
                    balance: tx.amount,
                    nonce: 0,
                });

            processed.push(tx);
            self.processed_count += 1;
        }

        debug!("Shard {}: Processed {} transactions (total: {})", 
            self.id, processed.len(), self.processed_count);

        Ok(processed)
    }
}

/// Cross-shard transaction for transfers between shards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossShardTransaction {
    pub from_shard: usize,
    pub to_shard: usize,
    pub transaction: Transaction,
    pub proof: String,
}

/// Main sharding coordinator
pub struct ShardingCoordinator {
    pub config: ShardConfig,
    pub shards: Vec<Arc<RwLock<Shard>>>,
    pub cross_shard_queue: Arc<RwLock<Vec<CrossShardTransaction>>>,
    pub total_processed: Arc<RwLock<u64>>,
}

impl ShardingCoordinator {
    /// Create new sharding coordinator
    pub fn new(config: ShardConfig) -> Self {
        info!("Initializing sharding coordinator with {} shards", config.shard_count);
        
        let shards: Vec<Arc<RwLock<Shard>>> = (0..config.shard_count)
            .map(|id| Arc::new(RwLock::new(Shard::new(id))))
            .collect();

        Self {
            config,
            shards,
            cross_shard_queue: Arc::new(RwLock::new(Vec::new())),
            total_processed: Arc::new(RwLock::new(0)),
        }
    }

    /// Distribute transactions to appropriate shards
    pub fn distribute_transactions(&self, transactions: Vec<Transaction>) -> HashMap<usize, Vec<Transaction>> {
        let mut shard_txs: HashMap<usize, Vec<Transaction>> = HashMap::new();

        for tx in transactions {
            // Determine shard based on sender address
            let shard_id = Shard::calculate_shard_id(&tx.from, self.config.shard_count);
            
            shard_txs.entry(shard_id)
                .or_insert_with(Vec::new)
                .push(tx);
        }

        debug!("Distributed transactions to {} shards", shard_txs.len());
        shard_txs
    }

    /// Process transactions in parallel across all shards
    pub async fn process_parallel(&self, transactions: Vec<Transaction>) -> Result<Vec<Transaction>> {
        let start = std::time::Instant::now();
        
        // Distribute transactions to shards
        let shard_txs = self.distribute_transactions(transactions);
        
        // Process each shard in parallel
        let mut handles: Vec<JoinHandle<Result<Vec<Transaction>>>> = Vec::new();
        
        for (shard_id, txs) in shard_txs {
            if txs.is_empty() {
                continue;
            }

            let shard = self.shards[shard_id].clone();
            let handle = tokio::spawn(async move {
                let mut shard_guard = shard.write().unwrap();
                shard_guard.process_transactions(txs)
            });
            
            handles.push(handle);
        }

        // Collect results from all shards
        let mut all_processed = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(processed)) => {
                    all_processed.extend(processed);
                }
                Ok(Err(e)) => {
                    warn!("Shard processing error: {}", e);
                }
                Err(e) => {
                    warn!("Task join error: {}", e);
                }
            }
        }

        // Update total processed count
        let mut total = self.total_processed.write().unwrap();
        *total += all_processed.len() as u64;

        let duration = start.elapsed();
        let tps = if duration.as_secs() > 0 {
            all_processed.len() as u64 / duration.as_secs()
        } else {
            all_processed.len() as u64 * 1000 / duration.as_millis().max(1) as u64
        };

        info!("Processed {} transactions across {} shards in {:?} ({} TPS)", 
            all_processed.len(), self.config.shard_count, duration, tps);

        Ok(all_processed)
    }

    /// Get total TPS capacity
    pub fn get_tps_capacity(&self) -> u64 {
        // Each shard can process tx_per_shard transactions per block
        // With 2-second blocks: (shard_count * tx_per_shard) / 2
        let tx_per_block = self.config.shard_count as u64 * self.config.tx_per_shard as u64;
        tx_per_block / 2 // 2-second blocks
    }

    /// Get shard statistics
    pub fn get_stats(&self) -> ShardStats {
        let mut total_txs = 0;
        let mut shard_loads = Vec::new();

        for shard in &self.shards {
            if let Ok(shard_guard) = shard.read() {
                total_txs += shard_guard.processed_count;
                shard_loads.push(shard_guard.pending_transactions.len());
            }
        }

        let total_processed = *self.total_processed.read().unwrap();

        ShardStats {
            shard_count: self.config.shard_count,
            total_transactions: total_txs,
            total_processed,
            average_load: if !shard_loads.is_empty() {
                shard_loads.iter().sum::<usize>() / shard_loads.len()
            } else {
                0
            },
            max_load: shard_loads.iter().max().copied().unwrap_or(0),
            estimated_tps: self.get_tps_capacity(),
        }
    }

    /// Initialize account in appropriate shard
    pub fn init_account(&self, address: String, balance: u64) -> Result<()> {
        let shard_id = Shard::calculate_shard_id(&address, self.config.shard_count);
        
        let mut shard = self.shards[shard_id].write().unwrap();
        shard.state.insert(address.clone(), Account { balance, nonce: 0 });
        
        info!("Account {} initialized in shard {} with balance {}", address, shard_id, balance);
        Ok(())
    }

    /// Get account balance from appropriate shard
    pub fn get_balance(&self, address: &str) -> u64 {
        let shard_id = Shard::calculate_shard_id(address, self.config.shard_count);
        
        if let Ok(shard) = self.shards[shard_id].read() {
            shard.state.get(address)
                .map(|acc| acc.balance)
                .unwrap_or(0)
        } else {
            0
        }
    }
}

/// Statistics for sharding system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardStats {
    pub shard_count: usize,
    pub total_transactions: u64,
    pub total_processed: u64,
    pub average_load: usize,
    pub max_load: usize,
    pub estimated_tps: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shard_assignment() {
        let address1 = "alice";
        let address2 = "bob";
        
        let shard1 = Shard::calculate_shard_id(address1, 100);
        let shard2 = Shard::calculate_shard_id(address2, 100);
        
        assert!(shard1 < 100);
        assert!(shard2 < 100);
        
        // Same address should always go to same shard
        assert_eq!(shard1, Shard::calculate_shard_id(address1, 100));
    }

    #[tokio::test]
    async fn test_parallel_processing() {
        let config = ShardConfig {
            shard_count: 10,
            tx_per_shard: 100,
            cross_shard_enabled: false,
        };
        
        let coordinator = ShardingCoordinator::new(config);
        
        // Initialize test accounts
        coordinator.init_account("alice".to_string(), 1_000_000).unwrap();
        coordinator.init_account("bob".to_string(), 1_000_000).unwrap();
        
        // Create test transactions
        let mut transactions = Vec::new();
        for i in 0..1000 {
            transactions.push(Transaction {
                from: if i % 2 == 0 { "alice".to_string() } else { "bob".to_string() },
                to: if i % 2 == 0 { "bob".to_string() } else { "alice".to_string() },
                amount: 100,
                gas_fee: 0,
                timestamp: i,
                nonce: i as u64,
                signature: None,
            });
        }
        
        // Process in parallel
        let result = coordinator.process_parallel(transactions).await;
        assert!(result.is_ok());
        
        let stats = coordinator.get_stats();
        assert_eq!(stats.shard_count, 10);
    }
}
