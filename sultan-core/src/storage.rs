//! Production-grade Persistent Storage using RocksDB
//!
//! Features:
//! - Block storage with height indexing
//! - Wallet balance persistence (with optional encryption)
//! - Transaction history with address indexing
//! - Staking state snapshots
//! - Governance state persistence
//! - Slashing event audit log
//! - LRU cache for hot blocks (1000 entries)
//! - Auto-compaction scheduling
//!
//! Security:
//! - Prefixed keys prevent collisions
//! - Atomic batch writes for consistency
//! - Append-only slashing log for audit
//! - Optional encryption for sensitive data (wallets)

use anyhow::{Result, Context};
use rocksdb::{DB, Options, WriteBatch, IteratorMode};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{info, warn};
use lru::LruCache;
use std::num::NonZeroUsize;

use crate::blockchain::Block;

/// Compact database every N blocks
const AUTO_COMPACT_INTERVAL_BLOCKS: u64 = 10_000;

/// Key prefixes for different data types (prevent collisions)
const PREFIX_BLOCK: &str = "block:";
const PREFIX_HEIGHT: &str = "height:";
const PREFIX_WALLET: &str = "wallet:";
const PREFIX_TX: &str = "tx:";
const PREFIX_TX_INDEX: &str = "txindex:";
const PREFIX_SLASH: &str = "slash:";
const PREFIX_GOV_PROPOSAL: &str = "gov:proposal:";
const PREFIX_GOV_VOTES: &str = "gov:votes:";
const PREFIX_GOV_STATE: &str = "gov:state";

/// AES-256-GCM authenticated encryption for sensitive data
/// Provides confidentiality, integrity, and authenticity guarantees
#[derive(Clone)]
pub struct StorageEncryption {
    key: [u8; 32], // 256-bit key for AES-256-GCM
}

impl StorageEncryption {
    /// Nonce size for AES-GCM (96 bits / 12 bytes)
    const NONCE_SIZE: usize = 12;
    
    /// HKDF info context for storage encryption
    const HKDF_INFO: &'static [u8] = b"sultan-storage-encryption-v1";
    
    /// HKDF salt for additional security (can be stored alongside encrypted data)
    const HKDF_SALT: &'static [u8] = b"sultan-l1-blockchain-storage";
    
    /// Create new encryption with HKDF-SHA256 key derivation (RFC 5869)
    /// HKDF provides cryptographically secure key derivation with:
    /// - Extract: Produces pseudorandom key from input key material
    /// - Expand: Derives output key with domain separation via info
    /// In production, derive from a secure key management system (HSM, KMS)
    pub fn new(key: &[u8]) -> Self {
        use hkdf::Hkdf;
        use sha2::Sha256;
        
        // HKDF-SHA256 key derivation (RFC 5869)
        // Extract phase: Create pseudorandom key from input key material
        // Expand phase: Derive 32-byte output key with domain separation
        let hk = Hkdf::<Sha256>::new(Some(Self::HKDF_SALT), key);
        let mut derived_key = [0u8; 32];
        hk.expand(Self::HKDF_INFO, &mut derived_key)
            .expect("32 bytes is valid output length for HKDF-SHA256");
        
        Self { key: derived_key }
    }
    
    /// Create encryption with custom salt for multi-tenant isolation
    pub fn with_salt(key: &[u8], salt: &[u8]) -> Self {
        use hkdf::Hkdf;
        use sha2::Sha256;
        
        let hk = Hkdf::<Sha256>::new(Some(salt), key);
        let mut derived_key = [0u8; 32];
        hk.expand(Self::HKDF_INFO, &mut derived_key)
            .expect("32 bytes is valid output length for HKDF-SHA256");
        
        Self { key: derived_key }
    }
    
    /// Generate a cryptographically secure random nonce
    fn generate_nonce() -> [u8; 12] {
        use rand::RngCore;
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);
        nonce
    }
    
    /// Encrypt data using AES-256-GCM with authenticated encryption
    /// Returns: nonce (12 bytes) || ciphertext || auth_tag (16 bytes)
    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };
        
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .expect("Valid 32-byte key");
        
        let nonce_bytes = Self::generate_nonce();
        let nonce = Nonce::from(nonce_bytes);
        
        // Encrypt with authentication tag appended
        let ciphertext = cipher
            .encrypt(&nonce, data)
            .expect("Encryption should not fail with valid inputs");
        
        // Prepend nonce to ciphertext for storage
        let mut result = Vec::with_capacity(Self::NONCE_SIZE + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        result
    }
    
    /// Decrypt data and verify authentication tag
    /// Input format: nonce (12 bytes) || ciphertext || auth_tag (16 bytes)
    /// Returns decrypted plaintext or error on authentication failure
    /// SECURITY: Uses try_decrypt internally to avoid panics on corrupted data
    pub fn decrypt(&self, data: &[u8]) -> Vec<u8> {
        // Use try_decrypt and convert error to empty vec to maintain API compatibility
        // Callers should prefer try_decrypt for proper error handling
        self.try_decrypt(data).unwrap_or_else(|e| {
            tracing::error!("Decryption failed: {}", e);
            Vec::new()
        })
    }
    
    /// Try to decrypt data, returning Result instead of panicking
    /// Useful for graceful error handling in production
    pub fn try_decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };
        
        if data.len() < Self::NONCE_SIZE + 16 {
            return Err(anyhow::anyhow!("Ciphertext too short"));
        }
        
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|_| anyhow::anyhow!("Invalid key"))?;
        
        // Extract nonce and ciphertext - convert slice to fixed array
        let mut nonce_arr = [0u8; 12];
        nonce_arr.copy_from_slice(&data[..Self::NONCE_SIZE]);
        let nonce = Nonce::from(nonce_arr);
        let ciphertext = &data[Self::NONCE_SIZE..];
        
        cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|_| anyhow::anyhow!("Decryption failed: authentication error"))
    }
}

/// Production-grade persistent storage using RocksDB
/// 
/// Thread-safe storage backend with LRU caching and auto-compaction.
/// Supports optional encryption for sensitive data.
pub struct PersistentStorage {
    db: Arc<DB>,
    block_cache: parking_lot::Mutex<LruCache<String, Block>>,
    /// Track last compaction height for auto-compaction scheduling
    last_compaction_height: AtomicU64,
    /// Optional encryption for sensitive data
    encryption: Option<StorageEncryption>,
}

impl PersistentStorage {
    /// Create new persistent storage instance (unencrypted)
    pub fn new(path: &str) -> Result<Self> {
        Self::with_encryption(path, None)
    }
    
    /// Create new persistent storage instance with optional encryption
    pub fn with_encryption(path: &str, encryption_key: Option<&[u8]>) -> Result<Self> {
        info!("Initializing RocksDB at: {}", path);
        
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_max_open_files(10000);
        opts.set_use_fsync(false); // Speed over paranoia for development
        opts.set_bytes_per_sync(8388608); // 8MB
        opts.set_level_compaction_dynamic_level_bytes(true);
        opts.set_max_background_jobs(4);
        
        let db = DB::open(&opts, path)?;
        
        let encryption = encryption_key.map(StorageEncryption::new);
        
        if encryption.is_some() {
            info!("✅ RocksDB initialized with encryption enabled");
        } else {
            info!("✅ RocksDB initialized successfully");
        }
        
        Ok(Self {
            db: Arc::new(db),
            block_cache: parking_lot::Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap())),
            last_compaction_height: AtomicU64::new(0),
            encryption,
        })
    }
    
    /// Check if encryption is enabled
    pub fn is_encrypted(&self) -> bool {
        self.encryption.is_some()
    }
    
    /// Save block to persistent storage
    /// 
    /// Also triggers auto-compaction every AUTO_COMPACT_INTERVAL_BLOCKS.
    pub fn save_block(&self, block: &Block) -> Result<()> {
        let key = format!("{}{}", PREFIX_BLOCK, block.hash);
        let value = bincode::serialize(block)?;
        
        // Save block data
        self.db.put(key.as_bytes(), value)?;
        
        // Update height index for fast lookup
        let height_key = format!("{}{}", PREFIX_HEIGHT, block.index);
        self.db.put(height_key.as_bytes(), block.hash.as_bytes())?;
        
        // Update latest block pointer
        self.db.put(b"latest", block.hash.as_bytes())?;
        
        // Cache the block
        self.block_cache.lock().put(block.hash.clone(), block.clone());
        
        // Auto-compaction check
        self.maybe_auto_compact(block.index);
        
        Ok(())
    }
    
    /// Check if auto-compaction should run based on block height
    fn maybe_auto_compact(&self, current_height: u64) {
        let last_compaction = self.last_compaction_height.load(Ordering::Relaxed);
        if current_height >= last_compaction + AUTO_COMPACT_INTERVAL_BLOCKS {
            // Try to update atomically - only one thread should compact
            if self.last_compaction_height
                .compare_exchange(last_compaction, current_height, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                info!("🗂️ Auto-compaction triggered at height {}", current_height);
                // Run compaction in background (non-blocking)
                let db = Arc::clone(&self.db);
                std::thread::spawn(move || {
                    db.compact_range::<&[u8], &[u8]>(None, None);
                    info!("✅ Auto-compaction complete");
                });
            }
        }
    }
    
    /// Get block by hash (checks cache first)
    pub fn get_block(&self, hash: &str) -> Result<Option<Block>> {
        // Check cache first for speed
        if let Some(block) = self.block_cache.lock().get(hash) {
            return Ok(Some(block.clone()));
        }
        
        // Query database
        let key = format!("{}{}", PREFIX_BLOCK, hash);
        if let Some(data) = self.db.get(key.as_bytes())? {
            let block: Block = bincode::deserialize(&data)
                .context("Failed to deserialize block")?;
            
            // Cache for next time
            self.block_cache.lock().put(hash.to_string(), block.clone());
            
            return Ok(Some(block));
        }
        
        Ok(None)
    }
    
    /// Get block by height
    pub fn get_block_by_height(&self, height: u64) -> Result<Option<Block>> {
        let height_key = format!("{}{}", PREFIX_HEIGHT, height);
        
        if let Some(hash_bytes) = self.db.get(height_key.as_bytes())? {
            let hash = String::from_utf8(hash_bytes)
                .context("Invalid UTF-8 in block hash")?;
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
    
    /// Save wallet balance (encrypted if encryption is enabled)
    pub fn save_wallet(&self, address: &str, balance: i64) -> Result<()> {
        let key = format!("{}{}", PREFIX_WALLET, address);
        let balance_bytes = balance.to_le_bytes();
        
        // Encrypt balance if encryption is enabled
        let value = if let Some(ref enc) = self.encryption {
            enc.encrypt(&balance_bytes)
        } else {
            balance_bytes.to_vec()
        };
        
        self.db.put(key.as_bytes(), value)?;
        Ok(())
    }
    
    /// Get wallet balance (decrypted if encryption is enabled)
    pub fn get_wallet(&self, address: &str) -> Result<Option<i64>> {
        let key = format!("{}{}", PREFIX_WALLET, address);
        
        if let Some(data) = self.db.get(key.as_bytes())? {
            // Decrypt if encryption is enabled
            let decrypted = if let Some(ref enc) = self.encryption {
                enc.decrypt(&data)
            } else {
                data.to_vec()
            };
            
            let bytes: [u8; 8] = decrypted.as_slice().try_into()
                .context("Invalid wallet balance data")?;
            let balance = i64::from_le_bytes(bytes);
            return Ok(Some(balance));
        }
        
        Ok(None)
    }
    
    /// Batch update wallets (atomic operation, encrypted if enabled)
    pub fn batch_update_wallets(&self, updates: Vec<(String, i64)>) -> Result<()> {
        let mut batch = WriteBatch::default();
        
        for (address, balance) in updates {
            let key = format!("{}{}", PREFIX_WALLET, address);
            let balance_bytes = balance.to_le_bytes();
            
            // Encrypt balance if encryption is enabled
            let value = if let Some(ref enc) = self.encryption {
                enc.encrypt(&balance_bytes)
            } else {
                balance_bytes.to_vec()
            };
            
            batch.put(key.as_bytes(), value);
        }
        
        self.db.write(batch)?;
        Ok(())
    }
    
    /// Get blockchain height
    pub fn get_height(&self) -> Result<u64> {
        if let Some(block) = self.get_latest_block()? {
            return Ok(block.index);
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

    /// Save a confirmed transaction to storage
    /// Stores: tx by hash, and indexes by sender/receiver address
    pub fn save_transaction(&self, tx: &crate::sharded_blockchain_production::ConfirmedTransaction) -> Result<()> {
        // Store transaction by hash
        let tx_key = format!("{}{}", PREFIX_TX, tx.hash);
        let tx_data = serde_json::to_vec(tx)
            .context("Failed to serialize transaction")?;
        self.db.put(tx_key.as_bytes(), &tx_data)?;

        // Add to sender's transaction list
        self.append_tx_to_address(&tx.from, &tx.hash)?;

        // Add to receiver's transaction list (if different)
        if tx.from != tx.to {
            self.append_tx_to_address(&tx.to, &tx.hash)?;
        }

        Ok(())
    }

    /// Append a transaction hash to an address's transaction index
    fn append_tx_to_address(&self, address: &str, tx_hash: &str) -> Result<()> {
        let index_key = format!("{}{}", PREFIX_TX_INDEX, address);
        
        // Get existing hashes
        let mut hashes: Vec<String> = if let Some(data) = self.db.get(index_key.as_bytes())? {
            serde_json::from_slice(&data).unwrap_or_default()
        } else {
            Vec::new()
        };

        // Add new hash (avoid duplicates)
        if !hashes.contains(&tx_hash.to_string()) {
            hashes.push(tx_hash.to_string());
            let data = serde_json::to_vec(&hashes)
                .context("Failed to serialize tx index")?;
            self.db.put(index_key.as_bytes(), &data)?;
        }

        Ok(())
    }

    /// Get transaction by hash
    pub fn get_transaction(&self, hash: &str) -> Result<Option<crate::sharded_blockchain_production::ConfirmedTransaction>> {
        let key = format!("{}{}", PREFIX_TX, hash);
        if let Some(data) = self.db.get(key.as_bytes())? {
            let tx: crate::sharded_blockchain_production::ConfirmedTransaction = 
                serde_json::from_slice(&data).context("Failed to deserialize transaction")?;
            return Ok(Some(tx));
        }
        Ok(None)
    }

    /// Get transaction history for an address (most recent first)
    pub fn get_transaction_history(&self, address: &str, limit: usize) -> Result<Vec<crate::sharded_blockchain_production::ConfirmedTransaction>> {
        let index_key = format!("{}{}", PREFIX_TX_INDEX, address);
        
        // Get transaction hashes for this address
        let hashes: Vec<String> = if let Some(data) = self.db.get(index_key.as_bytes())? {
            serde_json::from_slice(&data).unwrap_or_default()
        } else {
            return Ok(Vec::new());
        };

        // Fetch transactions (most recent last in storage, so reverse)
        let mut transactions = Vec::new();
        for hash in hashes.iter().rev().take(limit) {
            if let Some(tx) = self.get_transaction(hash)? {
                transactions.push(tx);
            }
        }

        // Sort by block height descending (most recent first)
        transactions.sort_by(|a, b| b.block_height.cmp(&a.block_height));

        Ok(transactions)
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
    
    // ============ Staking Persistence ============
    
    /// Save staking state to persistent storage
    /// Called after every staking operation (delegate, undelegate, slash)
    pub fn save_staking_state(&self, state: &StakingStateSnapshot) -> Result<()> {
        let value = bincode::serialize(state)?;
        self.db.put(b"staking:state", value)?;
        info!("💾 Staking state persisted ({} validators, {} unbonding entries)", 
            state.validators.len(), state.unbonding_queue.len());
        Ok(())
    }
    
    /// Load staking state from persistent storage
    /// Called on node startup to restore state
    pub fn load_staking_state(&self) -> Result<Option<StakingStateSnapshot>> {
        if let Some(data) = self.db.get(b"staking:state")? {
            let state: StakingStateSnapshot = bincode::deserialize(&data)?;
            info!("📥 Loaded staking state: {} validators, {} delegations, {} unbonding",
                state.validators.len(), state.delegations.len(), state.unbonding_queue.len());
            Ok(Some(state))
        } else {
            info!("📭 No existing staking state found");
            Ok(None)
        }
    }

    /// Delete staking state from persistent storage
    /// Used when resetting staking state (--reset-staking flag)
    pub fn delete_staking_state(&self) -> Result<()> {
        self.db.delete(b"staking:state")?;
        info!("🗑️ Staking state deleted from storage");
        Ok(())
    }
    
    /// Save slashing history (append-only log for audit)
    pub fn append_slashing_event(&self, event: &crate::staking::SlashingEvent) -> Result<()> {
        let key = format!("{}{}:{}", PREFIX_SLASH, event.height, event.validator_address);
        let value = bincode::serialize(event)
            .context("Failed to serialize slashing event")?;
        self.db.put(key.as_bytes(), value)?;
        warn!("⚔️  Slashing event persisted: {} slashed {} at height {}", 
            event.validator_address, event.amount_slashed, event.height);
        Ok(())
    }
    
    /// Get slashing history for a validator
    pub fn get_slashing_history(&self, validator_address: &str) -> Result<Vec<crate::staking::SlashingEvent>> {
        let mut events = Vec::new();
        
        for item in self.db.prefix_iterator(PREFIX_SLASH.as_bytes()) {
            let (_key, value) = item?;
            // Deserialize and filter by validator address
            if let Ok(event) = bincode::deserialize::<crate::staking::SlashingEvent>(&value) {
                if event.validator_address == validator_address {
                    events.push(event);
                }
            }
        }
        
        events.sort_by_key(|e| e.height);
        Ok(events)
    }
    
    /// Get all slashing events (for auditing)
    pub fn get_all_slashing_events(&self) -> Result<Vec<crate::staking::SlashingEvent>> {
        let mut events = Vec::new();
        
        for item in self.db.prefix_iterator(PREFIX_SLASH.as_bytes()) {
            let (_key, value) = item?;
            if let Ok(event) = bincode::deserialize::<crate::staking::SlashingEvent>(&value) {
                events.push(event);
            }
        }
        
        events.sort_by_key(|e| e.height);
        Ok(events)
    }
    
    // ============ Governance Persistence ============
    
    /// Save governance proposal to persistent storage
    pub fn save_proposal(&self, proposal: &crate::governance::Proposal) -> Result<()> {
        let key = format!("{}{}", PREFIX_GOV_PROPOSAL, proposal.id);
        let value = serde_json::to_vec(proposal)
            .context("Failed to serialize proposal")?;
        self.db.put(key.as_bytes(), value)?;
        info!("💾 Proposal #{} persisted: {}", proposal.id, proposal.title);
        Ok(())
    }
    
    /// Load a proposal by ID
    pub fn load_proposal(&self, proposal_id: u64) -> Result<Option<crate::governance::Proposal>> {
        let key = format!("{}{}", PREFIX_GOV_PROPOSAL, proposal_id);
        if let Some(data) = self.db.get(key.as_bytes())? {
            let proposal: crate::governance::Proposal = serde_json::from_slice(&data)
                .context("Failed to deserialize proposal")?;
            return Ok(Some(proposal));
        }
        Ok(None)
    }
    
    /// Load all proposals from storage
    pub fn load_all_proposals(&self) -> Result<Vec<crate::governance::Proposal>> {
        let mut proposals = Vec::new();
        
        for item in self.db.prefix_iterator(PREFIX_GOV_PROPOSAL.as_bytes()) {
            let (_key, value) = item?;
            if let Ok(proposal) = serde_json::from_slice::<crate::governance::Proposal>(&value) {
                proposals.push(proposal);
            }
        }
        
        // Sort by ID descending (newest first)
        proposals.sort_by(|a, b| b.id.cmp(&a.id));
        Ok(proposals)
    }
    
    /// Save votes for a proposal
    pub fn save_proposal_votes(&self, proposal_id: u64, votes: &[crate::governance::Vote]) -> Result<()> {
        let key = format!("{}{}", PREFIX_GOV_VOTES, proposal_id);
        let value = serde_json::to_vec(votes)
            .context("Failed to serialize votes")?;
        self.db.put(key.as_bytes(), value)?;
        Ok(())
    }
    
    /// Load votes for a proposal
    pub fn load_proposal_votes(&self, proposal_id: u64) -> Result<Vec<crate::governance::Vote>> {
        let key = format!("{}{}", PREFIX_GOV_VOTES, proposal_id);
        if let Some(data) = self.db.get(key.as_bytes())? {
            let votes: Vec<crate::governance::Vote> = serde_json::from_slice(&data)
                .context("Failed to deserialize votes")?;
            return Ok(votes);
        }
        Ok(Vec::new())
    }
    
    /// Save governance state (next_proposal_id, etc.)
    pub fn save_governance_state(&self, state: &GovernanceStateSnapshot) -> Result<()> {
        let value = serde_json::to_vec(state)
            .context("Failed to serialize governance state")?;
        self.db.put(PREFIX_GOV_STATE.as_bytes(), value)?;
        info!("💾 Governance state persisted: next_proposal_id={}", state.next_proposal_id);
        Ok(())
    }
    
    /// Load governance state
    pub fn load_governance_state(&self) -> Result<Option<GovernanceStateSnapshot>> {
        if let Some(data) = self.db.get(PREFIX_GOV_STATE.as_bytes())? {
            let state: GovernanceStateSnapshot = serde_json::from_slice(&data)
                .context("Failed to deserialize governance state")?;
            return Ok(Some(state));
        }
        Ok(None)
    }
    
    // ============ Encrypted Governance Storage ============
    // For sensitive proposals (slashing, emergency actions, etc.)
    
    /// Save proposal with encryption for sensitive content
    /// Use this for proposals containing validator addresses being slashed,
    /// security-sensitive parameter changes, or confidential governance actions
    pub fn save_proposal_encrypted(&self, proposal: &crate::governance::Proposal) -> Result<()> {
        let encryption = self.encryption.as_ref()
            .context("Encryption not enabled - use with_encryption() constructor")?;
        
        let key = format!("{}enc:{}", PREFIX_GOV_PROPOSAL, proposal.id);
        let plaintext = serde_json::to_vec(proposal)
            .context("Failed to serialize proposal")?;
        let ciphertext = encryption.encrypt(&plaintext);
        
        self.db.put(key.as_bytes(), ciphertext)?;
        info!("🔐 Proposal #{} encrypted and persisted: {}", proposal.id, proposal.title);
        Ok(())
    }
    
    /// Load encrypted proposal by ID
    pub fn load_proposal_encrypted(&self, proposal_id: u64) -> Result<Option<crate::governance::Proposal>> {
        let encryption = self.encryption.as_ref()
            .context("Encryption not enabled - use with_encryption() constructor")?;
        
        let key = format!("{}enc:{}", PREFIX_GOV_PROPOSAL, proposal_id);
        if let Some(ciphertext) = self.db.get(key.as_bytes())? {
            let plaintext = encryption.try_decrypt(&ciphertext)
                .context("Failed to decrypt proposal - possible key mismatch or data corruption")?;
            let proposal: crate::governance::Proposal = serde_json::from_slice(&plaintext)
                .context("Failed to deserialize proposal")?;
            return Ok(Some(proposal));
        }
        Ok(None)
    }
    
    /// Check if a proposal is stored encrypted
    pub fn is_proposal_encrypted(&self, proposal_id: u64) -> bool {
        let key = format!("{}enc:{}", PREFIX_GOV_PROPOSAL, proposal_id);
        self.db.get(key.as_bytes()).ok().flatten().is_some()
    }
}

/// Serializable snapshot of all staking state
/// Used for persistence and state sync
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StakingStateSnapshot {
    pub validators: std::collections::HashMap<String, crate::staking::ValidatorStake>,
    pub delegations: std::collections::HashMap<String, Vec<crate::staking::Delegation>>,
    pub unbonding_queue: Vec<crate::staking::UnbondingEntry>,
    pub total_staked: u64,
    pub current_height: u64,
    pub snapshot_time: u64,
}

/// Serializable snapshot of governance state
/// Used for persistence and state sync
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GovernanceStateSnapshot {
    pub next_proposal_id: u64,
    pub current_height: u64,
    pub total_bonded_tokens: u64,
    /// Rate limit tracking: last proposal height per address
    pub last_proposal_by_address: std::collections::HashMap<String, u64>,
    pub snapshot_time: u64,
}

impl Clone for PersistentStorage {
    fn clone(&self) -> Self {
        Self {
            db: Arc::clone(&self.db),
            block_cache: parking_lot::Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap())),
            last_compaction_height: AtomicU64::new(self.last_compaction_height.load(Ordering::Relaxed)),
            encryption: self.encryption.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_storage_persistence() {
        let dir = tempdir().unwrap();
        let storage = PersistentStorage::new(dir.path().to_str().unwrap()).unwrap();
        
        // Create and save block
        let block = Block {
            index: 1,
            hash: "test_hash".to_string(),
            prev_hash: "genesis".to_string(),
            timestamp: 1234567890,
            transactions: vec![],
            nonce: 0,
            validator: "test".to_string(),
            state_root: "root".to_string(),
        };
        
        storage.save_block(&block).unwrap();
        
        // Retrieve block
        let retrieved = storage.get_block("test_hash").unwrap().unwrap();
        assert_eq!(retrieved.hash, "test_hash");
        assert_eq!(retrieved.index, 1);
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
                index: i,
                hash: format!("hash_{}", i),
                prev_hash: if i == 1 { "genesis".to_string() } else { format!("hash_{}", i - 1) },
                timestamp: 1234567890 + i,
                transactions: vec![],
                nonce: 0,
                validator: "test".to_string(),
                state_root: "root".to_string(),
            };
            
            storage.save_block(&block).unwrap();
        }
        
        // Query by height
        let block_5 = storage.get_block_by_height(5).unwrap().unwrap();
        assert_eq!(block_5.index, 5);
        assert_eq!(block_5.hash, "hash_5");
        
        // Get latest
        let latest = storage.get_latest_block().unwrap().unwrap();
        assert_eq!(latest.index, 10);
    }
    
    #[test]
    fn test_staking_state_persistence() {
        let dir = tempdir().unwrap();
        let storage = PersistentStorage::new(dir.path().to_str().unwrap()).unwrap();
        
        // Create staking state snapshot
        let mut validators = std::collections::HashMap::new();
        validators.insert("validator1".to_string(), crate::staking::ValidatorStake {
            validator_address: "validator1".to_string(),
            self_stake: 10_000_000_000_000,
            delegated_stake: 5_000_000_000_000,
            total_stake: 15_000_000_000_000,
            commission_rate: 0.10,
            rewards_accumulated: 100_000_000,
            blocks_signed: 1000,
            blocks_missed: 5,
            total_blocks_missed: 10,
            jailed: false,
            jailed_until: 0,
            created_at: 1700000000,
            last_reward_height: 5000,
            reward_wallet: None,
        });
        
        let snapshot = StakingStateSnapshot {
            validators,
            delegations: std::collections::HashMap::new(),
            unbonding_queue: Vec::new(),
            total_staked: 15_000_000_000_000,
            current_height: 5000,
            snapshot_time: 1700000000,
        };
        
        // Save and reload
        storage.save_staking_state(&snapshot).unwrap();
        let loaded = storage.load_staking_state().unwrap().unwrap();
        
        assert_eq!(loaded.total_staked, 15_000_000_000_000);
        assert_eq!(loaded.current_height, 5000);
        assert!(loaded.validators.contains_key("validator1"));
    }
    
    #[test]
    fn test_slashing_event_persistence() {
        let dir = tempdir().unwrap();
        let storage = PersistentStorage::new(dir.path().to_str().unwrap()).unwrap();
        
        // Create slashing events
        let event1 = crate::staking::SlashingEvent {
            validator_address: "validator1".to_string(),
            height: 1000,
            timestamp: 1700000000,
            reason: crate::staking::SlashReason::Downtime,
            amount_slashed: 100_000_000_000,
            jail_duration: 3600,
        };
        
        let event2 = crate::staking::SlashingEvent {
            validator_address: "validator1".to_string(),
            height: 2000,
            timestamp: 1700001000,
            reason: crate::staking::SlashReason::DoubleSign,
            amount_slashed: 500_000_000_000,
            jail_duration: 10000,
        };
        
        let event3 = crate::staking::SlashingEvent {
            validator_address: "validator2".to_string(),
            height: 1500,
            timestamp: 1700000500,
            reason: crate::staking::SlashReason::Downtime,
            amount_slashed: 50_000_000_000,
            jail_duration: 3600,
        };
        
        storage.append_slashing_event(&event1).unwrap();
        storage.append_slashing_event(&event2).unwrap();
        storage.append_slashing_event(&event3).unwrap();
        
        // Get history for validator1
        let history = storage.get_slashing_history("validator1").unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].height, 1000); // Sorted by height
        assert_eq!(history[1].height, 2000);
        
        // Get all events
        let all_events = storage.get_all_slashing_events().unwrap();
        assert_eq!(all_events.len(), 3);
    }
    
    #[test]
    fn test_governance_state_persistence() {
        let dir = tempdir().unwrap();
        let storage = PersistentStorage::new(dir.path().to_str().unwrap()).unwrap();
        
        // Create governance state
        let mut last_proposal_by_address = std::collections::HashMap::new();
        last_proposal_by_address.insert("proposer1".to_string(), 1000u64);
        
        let state = GovernanceStateSnapshot {
            next_proposal_id: 5,
            current_height: 10000,
            total_bonded_tokens: 1_000_000_000_000_000,
            last_proposal_by_address,
            snapshot_time: 1700000000,
        };
        
        // Save and reload
        storage.save_governance_state(&state).unwrap();
        let loaded = storage.load_governance_state().unwrap().unwrap();
        
        assert_eq!(loaded.next_proposal_id, 5);
        assert_eq!(loaded.current_height, 10000);
        assert_eq!(loaded.total_bonded_tokens, 1_000_000_000_000_000);
        assert_eq!(*loaded.last_proposal_by_address.get("proposer1").unwrap(), 1000);
    }
    
    #[test]
    fn test_compaction() {
        let dir = tempdir().unwrap();
        let storage = PersistentStorage::new(dir.path().to_str().unwrap()).unwrap();
        
        // Manual compaction should not fail
        storage.compact().unwrap();
        
        // Stats should work
        let stats = storage.stats().unwrap();
        assert!(stats.contains("RocksDB Stats"));
    }
    
    #[test]
    fn test_checkpoint() {
        let dir = tempdir().unwrap();
        let storage = PersistentStorage::new(dir.path().to_str().unwrap()).unwrap();
        
        // Save some data
        storage.save_wallet("test", 1000).unwrap();
        
        // Checkpoint should flush to disk
        storage.checkpoint().unwrap();
        
        // Data should still be accessible
        assert_eq!(storage.get_wallet("test").unwrap().unwrap(), 1000);
    }
    
    #[test]
    fn test_encrypted_wallet_storage() {
        let dir = tempdir().unwrap();
        let encryption_key = b"super_secret_key_for_testing_123";
        let storage = PersistentStorage::with_encryption(
            dir.path().to_str().unwrap(), 
            Some(encryption_key)
        ).unwrap();
        
        assert!(storage.is_encrypted());
        
        // Save encrypted wallet
        storage.save_wallet("encrypted_addr", 999_999_999).unwrap();
        
        // Retrieve and verify decryption works
        let balance = storage.get_wallet("encrypted_addr").unwrap().unwrap();
        assert_eq!(balance, 999_999_999);
        
        // Batch update with encryption
        storage.batch_update_wallets(vec![
            ("enc1".to_string(), 100),
            ("enc2".to_string(), 200),
        ]).unwrap();
        
        assert_eq!(storage.get_wallet("enc1").unwrap().unwrap(), 100);
        assert_eq!(storage.get_wallet("enc2").unwrap().unwrap(), 200);
    }
    
    #[test]
    fn test_encryption_helper() {
        let enc = StorageEncryption::new(b"test_key");
        
        let original = b"sensitive data 12345";
        let encrypted = enc.encrypt(original);
        
        // Encrypted should differ from original
        assert_ne!(encrypted, original.to_vec());
        
        // Decryption should restore original
        let decrypted = enc.decrypt(&encrypted);
        assert_eq!(decrypted, original.to_vec());
    }
    
    #[test]
    fn test_proposal_persistence() {
        use crate::governance::{Proposal, ProposalStatus, ProposalType};
        
        let dir = tempdir().unwrap();
        let storage = PersistentStorage::new(dir.path().to_str().unwrap()).unwrap();
        
        // Create a proposal
        let proposal = Proposal {
            id: 1,
            proposer: "sultan1prpsr2qqqqqqqqqqqqqqqqqqqqqqqqqqqqprpsaa".to_string(),
            title: "Test Proposal".to_string(),
            description: "A test governance proposal".to_string(),
            proposal_type: ProposalType::ParameterChange,
            status: ProposalStatus::VotingPeriod,
            submit_height: 1000,
            submit_time: 1700000000,
            deposit_end_height: 2000,
            discussion_end_height: 87400,
            voting_start_height: 87400,
            voting_end_height: 389800,
            total_deposit: 1_000_000_000_000,
            depositors: vec![],
            final_tally: None,
            parameters: Some([("inflation_rate".to_string(), "0.05".to_string())].into()),
            voting_power_snapshot: None,
            telegram_discussion_url: Some("https://t.me/test".to_string()),
            discord_discussion_url: None,
            validator_signatures: vec![],
            emergency_pause_votes: vec![],
        };
        
        // Save and reload
        storage.save_proposal(&proposal).unwrap();
        let loaded = storage.load_proposal(1).unwrap().unwrap();
        
        assert_eq!(loaded.id, 1);
        assert_eq!(loaded.title, "Test Proposal");
        assert_eq!(loaded.proposer, proposal.proposer);
        assert_eq!(loaded.status, ProposalStatus::VotingPeriod);
        assert!(loaded.parameters.unwrap().contains_key("inflation_rate"));
    }
    
    #[test]
    fn test_proposal_votes_persistence() {
        use crate::governance::{Vote, VoteOption};
        
        let dir = tempdir().unwrap();
        let storage = PersistentStorage::new(dir.path().to_str().unwrap()).unwrap();
        
        let votes = vec![
            Vote {
                proposal_id: 1,
                voter: "sultan1vtr2qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqvtraa".to_string(),
                option: VoteOption::Yes,
                voting_power: 5_000_000_000_000,
                time: 1700000000,
            },
            Vote {
                proposal_id: 1,
                voter: "sultan1vtr3qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqvtrcc".to_string(),
                option: VoteOption::No,
                voting_power: 2_000_000_000_000,
                time: 1700001000,
            },
        ];
        
        // Save and reload
        storage.save_proposal_votes(1, &votes).unwrap();
        let loaded = storage.load_proposal_votes(1).unwrap();
        
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].option, VoteOption::Yes);
        assert_eq!(loaded[1].option, VoteOption::No);
        assert_eq!(loaded[0].voting_power, 5_000_000_000_000);
    }
    
    #[test]
    fn test_load_all_proposals() {
        use crate::governance::{Proposal, ProposalStatus, ProposalType};
        
        let dir = tempdir().unwrap();
        let storage = PersistentStorage::new(dir.path().to_str().unwrap()).unwrap();
        
        // Create multiple proposals
        for i in 1..=5 {
            let proposal = Proposal {
                id: i,
                proposer: format!("sultan1prp{}qqqqqqqqqqqqqqqqqqqqqqqqqqqqqprp{:02}", i, i),
                title: format!("Proposal #{}", i),
                description: format!("Description for proposal {}", i),
                proposal_type: ProposalType::TextProposal,
                status: ProposalStatus::VotingPeriod,
                submit_height: i * 1000,
                submit_time: 1700000000 + i,
                deposit_end_height: i * 1000 + 1000,
                discussion_end_height: 87400,
                voting_start_height: 87400,
                voting_end_height: 389800,
                total_deposit: 1_000_000_000_000,
                depositors: vec![],
                final_tally: None,
                parameters: None,
                voting_power_snapshot: None,
                telegram_discussion_url: Some("https://t.me/test".to_string()),
                discord_discussion_url: None,
                validator_signatures: vec![],
                emergency_pause_votes: vec![],
            };
            storage.save_proposal(&proposal).unwrap();
        }
        
        // Load all
        let all = storage.load_all_proposals().unwrap();
        assert_eq!(all.len(), 5);
        
        // Should be sorted by ID descending (newest first)
        assert_eq!(all[0].id, 5);
        assert_eq!(all[4].id, 1);
    }
}
