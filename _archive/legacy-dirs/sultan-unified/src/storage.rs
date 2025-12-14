use anyhow::Result;
use rocksdb::{DB, Options, WriteBatch, IteratorMode};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tracing::{info, warn};
use lru::LruCache;
use std::num::NonZeroUsize;

use crate::blockchain::Block;
use crate::types::Address;

/// Production-grade persistent storage using RocksDB
pub struct PersistentStorage {
    db: Arc<DB>,
    block_cache: parking_lot::Mutex<LruCache<String, Block>>,
}

impl PersistentStorage {
    /// Create new persistent storage instance
    pub fn new(path: &str) -> Result<Self> {
        info!("Initializing RocksDB at: {}", path);
        
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_max_open_files(10000);
        opts.set_use_fsync(false); // Speed over paranoia for development
        opts.set_bytes_per_sync(8388608); // 8MB
        opts.set_level_compaction_dynamic_level_bytes(true);
        opts.set_max_background_jobs(4);
        
        let db = DB::open(&opts, path)?;
        
        info!("✅ RocksDB initialized successfully");
        
        Ok(Self {
            db: Arc::new(db),
            block_cache: parking_lot::Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap())),
        })
    }
    
    /// Save block to persistent storage
    pub fn save_block(&self, block: &Block) -> Result<()> {
        let key = format!("block:{}", block.hash);
        let value = bincode::serialize(block)?;
        
        // Save block data
        self.db.put(key.as_bytes(), value)?;
        
        // Update height index for fast lookup
        let height_key = format!("height:{}", block.height);
        self.db.put(height_key.as_bytes(), block.hash.as_bytes())?;
        
        // Update latest block pointer
        self.db.put(b"latest", block.hash.as_bytes())?;
        
        // Cache the block
        self.block_cache.lock().put(block.hash.clone(), block.clone());
        
        Ok(())
    }
    
    /// Get block by hash (checks cache first)
    pub fn get_block(&self, hash: &str) -> Result<Option<Block>> {
        // Check cache first for speed
        if let Some(block) = self.block_cache.lock().get(hash) {
            return Ok(Some(block.clone()));
        }
        
        // Query database
        let key = format!("block:{}", hash);
        if let Some(data) = self.db.get(key.as_bytes())? {
            let block: Block = bincode::deserialize(&data)?;
            
            // Cache for next time
            self.block_cache.lock().put(hash.to_string(), block.clone());
            
            return Ok(Some(block));
        }
        
        Ok(None)
    }
    
    /// Get block by height
    pub fn get_block_by_height(&self, height: u64) -> Result<Option<Block>> {
        let height_key = format!("height:{}", height);
        
        if let Some(hash_bytes) = self.db.get(height_key.as_bytes())? {
            let hash = String::from_utf8(hash_bytes)?;
            return self.get_block(&hash);
        }
        
        Ok(None)
    }
    
    /// Get latest block
    pub fn get_latest_block(&self) -> Result<Option<Block>> {
        if let Some(hash_bytes) = self.db.get(b"latest")? {
            let hash = String::from_utf8(hash_bytes)?;
            return self.get_block(&hash);
        }
        
        Ok(None)
    }
    
    /// Save wallet balance
    pub fn save_wallet(&self, address: &str, balance: i64) -> Result<()> {
        let key = format!("wallet:{}", address);
        self.db.put(key.as_bytes(), balance.to_le_bytes())?;
        Ok(())
    }
    
    /// Get wallet balance
    pub fn get_wallet(&self, address: &str) -> Result<Option<i64>> {
        let key = format!("wallet:{}", address);
        
        if let Some(data) = self.db.get(key.as_bytes())? {
            let bytes: [u8; 8] = data.as_slice().try_into()?;
            let balance = i64::from_le_bytes(bytes);
            return Ok(Some(balance));
        }
        
        Ok(None)
    }
    
    /// Batch update wallets (atomic operation)
    pub fn batch_update_wallets(&self, updates: Vec<(String, i64)>) -> Result<()> {
        let mut batch = WriteBatch::default();
        
        for (address, balance) in updates {
            let key = format!("wallet:{}", address);
            batch.put(key.as_bytes(), balance.to_le_bytes());
        }
        
        self.db.write(batch)?;
        Ok(())
    }
    
    /// Get blockchain height
    pub fn get_height(&self) -> Result<u64> {
        if let Some(block) = self.get_latest_block()? {
            return Ok(block.height);
        }
        
        Ok(0)
    }
    
    /// Checkpoint (force flush to disk)
    pub fn checkpoint(&self) -> Result<()> {
        self.db.flush()?;
        info!("✅ Database checkpoint complete");
        Ok(())
    }
    
    /// Get database statistics
    pub fn stats(&self) -> Result<String> {
        // Get approximate sizes
        let mut total_keys = 0;
        let iter = self.db.iterator(IteratorMode::Start);
        
        for _ in iter {
            total_keys += 1;
            if total_keys > 10000 {
                break; // Don't count everything, just estimate
            }
        }
        
        Ok(format!(
            "RocksDB Stats:\n\
             - Total keys: ~{}\n\
             - Cache size: {}\n\
             - Height: {}",
            total_keys,
            self.block_cache.lock().len(),
            self.get_height()?
        ))
    }
    
    /// Compact database (reduce disk usage)
    pub fn compact(&self) -> Result<()> {
        info!("Starting database compaction...");
        self.db.compact_range::<&[u8], &[u8]>(None, None);
        info!("✅ Database compaction complete");
        Ok(())
    }
    
    /// Clear all data (DANGEROUS - for testing only)
    #[cfg(test)]
    pub fn clear_all(&self) -> Result<()> {
        warn!("⚠️  Clearing all data!");
        
        let keys: Vec<Vec<u8>> = self.db
            .iterator(IteratorMode::Start)
            .map(|item| item.unwrap().0.to_vec())
            .collect();
        
        for key in keys {
            self.db.delete(&key)?;
        }
        
        self.block_cache.lock().clear();
        
        Ok(())
    }
}

impl Clone for PersistentStorage {
    fn clone(&self) -> Self {
        Self {
            db: Arc::clone(&self.db),
            block_cache: parking_lot::Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::Transaction;
    use tempfile::tempdir;
    
    #[test]
    fn test_storage_persistence() {
        let dir = tempdir().unwrap();
        let storage = PersistentStorage::new(dir.path().to_str().unwrap()).unwrap();
        
        // Create and save block
        let mut block = Block {
            height: 1,
            hash: "test_hash".to_string(),
            prev_hash: "genesis".to_string(),
            timestamp: 1234567890,
            transactions: vec![],
        };
        
        storage.save_block(&block).unwrap();
        
        // Retrieve block
        let retrieved = storage.get_block("test_hash").unwrap().unwrap();
        assert_eq!(retrieved.hash, "test_hash");
        assert_eq!(retrieved.height, 1);
    }
    
    #[test]
    fn test_wallet_operations() {
        let dir = tempdir().unwrap();
        let storage = PersistentStorage::new(dir.path().to_str().unwrap()).unwrap();
        
        // Save wallet
        storage.save_wallet("sultan1abc123", 1000000).unwrap();
        
        // Retrieve wallet
        let balance = storage.get_wallet("sultan1abc123").unwrap().unwrap();
        assert_eq!(balance, 1000000);
        
        // Update wallet
        storage.save_wallet("sultan1abc123", 500000).unwrap();
        let updated = storage.get_wallet("sultan1abc123").unwrap().unwrap();
        assert_eq!(updated, 500000);
    }
    
    #[test]
    fn test_batch_wallet_update() {
        let dir = tempdir().unwrap();
        let storage = PersistentStorage::new(dir.path().to_str().unwrap()).unwrap();
        
        let updates = vec![
            ("sultan1aaa".to_string(), 1000),
            ("sultan1bbb".to_string(), 2000),
            ("sultan1ccc".to_string(), 3000),
        ];
        
        storage.batch_update_wallets(updates).unwrap();
        
        assert_eq!(storage.get_wallet("sultan1aaa").unwrap().unwrap(), 1000);
        assert_eq!(storage.get_wallet("sultan1bbb").unwrap().unwrap(), 2000);
        assert_eq!(storage.get_wallet("sultan1ccc").unwrap().unwrap(), 3000);
    }
    
    #[test]
    fn test_height_index() {
        let dir = tempdir().unwrap();
        let storage = PersistentStorage::new(dir.path().to_str().unwrap()).unwrap();
        
        for i in 1..=10 {
            let block = Block {
                height: i,
                hash: format!("hash_{}", i),
                prev_hash: if i == 1 { "genesis".to_string() } else { format!("hash_{}", i - 1) },
                timestamp: 1234567890 + i,
                transactions: vec![],
            };
            
            storage.save_block(&block).unwrap();
        }
        
        // Query by height
        let block_5 = storage.get_block_by_height(5).unwrap().unwrap();
        assert_eq!(block_5.height, 5);
        assert_eq!(block_5.hash, "hash_5");
        
        // Get latest
        let latest = storage.get_latest_block().unwrap().unwrap();
        assert_eq!(latest.height, 10);
    }
}
