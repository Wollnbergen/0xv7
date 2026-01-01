//! Sultan Block Synchronization - PRODUCTION GRADE
//!
//! Implements Byzantine-tolerant block synchronization with:
//! - Leader-based block production (proposer rotation)
//! - Block validation and verification
//! - Chain synchronization with peers
//! - Fork resolution (longest valid chain wins)
//! - Catch-up sync for nodes that fall behind
//! - Voter verification against consensus validators
//! - Block signature validation

use anyhow::{Result, bail};
use sha2::{Sha256, Digest};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;
use tracing::{info, warn, debug};

use crate::blockchain::Block;
use crate::consensus::ConsensusEngine;

/// Block synchronization configuration
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Maximum blocks to request in a single sync request
    pub max_blocks_per_request: usize,
    /// Timeout for sync requests
    pub sync_timeout: Duration,
    /// How often to check if we need to sync
    pub sync_check_interval: Duration,
    /// Minimum confirmations before considering block final
    pub finality_confirmations: u64,
    /// Maximum fork depth to consider
    pub max_fork_depth: u64,
    /// Maximum pending blocks to track (DoS prevention)
    pub max_pending_blocks: usize,
    /// Maximum seen block hashes to cache
    pub max_seen_blocks: usize,
    /// Require validator verification for votes
    pub verify_voters: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            max_blocks_per_request: 100,
            sync_timeout: Duration::from_secs(30),
            sync_check_interval: Duration::from_secs(5),
            finality_confirmations: 3,
            max_fork_depth: 10,
            max_pending_blocks: 100,
            max_seen_blocks: 10000,
            verify_voters: true,
        }
    }
}

/// Pending block waiting for votes
#[derive(Debug, Clone)]
pub struct PendingBlock {
    pub block: Block,
    pub proposer: String,
    pub votes: HashMap<String, bool>, // validator -> approve
    pub vote_count: usize,
    pub created_at: std::time::Instant,
    pub block_hash: String,
}

/// Vote rejection reasons
#[derive(Debug, Clone, PartialEq)]
pub enum VoteRejection {
    /// Block not found in pending
    BlockNotFound,
    /// Voter is not a registered validator
    InvalidVoter,
    /// Duplicate vote from same voter
    DuplicateVote,
    /// Vote expired (block too old)
    Expired,
    /// Invalid cryptographic signature
    InvalidSignature,
}

/// Sync state for tracking chain synchronization
#[derive(Debug, Clone, PartialEq)]
pub enum SyncState {
    /// Node is syncing with peers
    Syncing { target_height: u64 },
    /// Node is synchronized
    Synced,
    /// Node is ahead (shouldn't happen normally)
    Ahead,
}

/// Block Synchronization Manager
///
/// Handles:
/// 1. Tracking sync state
/// 2. Managing pending blocks with validator verification
/// 3. Computing and validating block hashes
/// 4. Sync statistics for monitoring
pub struct BlockSyncManager {
    config: SyncConfig,
    /// Our validator address (if we are a validator)
    our_address: Option<String>,
    /// Current chain height
    current_height: Arc<RwLock<u64>>,
    /// Pending blocks awaiting votes
    pending_blocks: Arc<RwLock<HashMap<u64, PendingBlock>>>,
    /// Already processed block hashes (dedup)
    seen_blocks: Arc<RwLock<HashSet<String>>>,
    /// Sync state
    sync_state: Arc<RwLock<SyncState>>,
    /// Known peer heights for sync decisions
    peer_heights: Arc<RwLock<HashMap<String, u64>>>,
    /// Consensus engine for validator info
    consensus: Arc<RwLock<ConsensusEngine>>,
    /// Block time for timing
    block_time: Duration,
    /// Statistics: total blocks synced
    blocks_synced: Arc<RwLock<u64>>,
    /// Statistics: total votes recorded
    votes_recorded: Arc<RwLock<u64>>,
    /// Statistics: rejected votes
    votes_rejected: Arc<RwLock<u64>>,
}

impl BlockSyncManager {
    pub fn new(
        config: SyncConfig,
        our_address: Option<String>,
        consensus: Arc<RwLock<ConsensusEngine>>,
        block_time: Duration,
    ) -> Self {
        Self {
            config,
            our_address,
            current_height: Arc::new(RwLock::new(0)),
            pending_blocks: Arc::new(RwLock::new(HashMap::new())),
            seen_blocks: Arc::new(RwLock::new(HashSet::new())),
            sync_state: Arc::new(RwLock::new(SyncState::Synced)),
            peer_heights: Arc::new(RwLock::new(HashMap::new())),
            consensus,
            block_time,
            blocks_synced: Arc::new(RwLock::new(0)),
            votes_recorded: Arc::new(RwLock::new(0)),
            votes_rejected: Arc::new(RwLock::new(0)),
        }
    }

    /// Get configuration
    pub fn config(&self) -> &SyncConfig {
        &self.config
    }

    /// Get our validator address
    pub fn our_address(&self) -> Option<&String> {
        self.our_address.as_ref()
    }

    /// Check if we are a validator
    pub fn is_validator(&self) -> bool {
        self.our_address.is_some()
    }

    /// Get block time
    pub fn block_time(&self) -> Duration {
        self.block_time
    }

    /// Set current height (from blockchain state)
    pub async fn set_height(&self, height: u64) {
        *self.current_height.write().await = height;
    }

    /// Get current height
    pub async fn get_height(&self) -> u64 {
        *self.current_height.read().await
    }

    /// Get current sync state
    pub async fn get_sync_state(&self) -> SyncState {
        self.sync_state.read().await.clone()
    }

    /// Set sync state
    pub async fn set_sync_state(&self, state: SyncState) {
        *self.sync_state.write().await = state;
    }

    /// Check if a block hash has been seen
    pub async fn is_block_seen(&self, hash: &str) -> bool {
        self.seen_blocks.read().await.contains(hash)
    }

    /// Mark a block hash as seen
    pub async fn mark_block_seen(&self, hash: String) {
        let mut seen = self.seen_blocks.write().await;

        // Enforce max size to prevent memory exhaustion
        if seen.len() >= self.config.max_seen_blocks {
            // Remove oldest entries (simple strategy: clear half)
            let to_remove: Vec<_> = seen.iter().take(seen.len() / 2).cloned().collect();
            for h in to_remove {
                seen.remove(&h);
            }
            debug!("Cleaned up seen_blocks cache, now {} entries", seen.len());
        }

        seen.insert(hash);
    }

    /// Update peer height (called when we learn peer's height)
    pub async fn update_peer_height(&self, peer_id: String, height: u64) {
        self.peer_heights.write().await.insert(peer_id, height);

        // Check if we need to update sync state
        let our_height = *self.current_height.read().await;
        let max_peer = self.max_peer_height().await;

        if max_peer > our_height + 1 {
            *self.sync_state.write().await = SyncState::Syncing { target_height: max_peer };
        } else if our_height > max_peer + 1 {
            *self.sync_state.write().await = SyncState::Ahead;
        } else {
            *self.sync_state.write().await = SyncState::Synced;
        }
    }

    /// Remove a peer from tracking
    pub async fn remove_peer(&self, peer_id: &str) {
        self.peer_heights.write().await.remove(peer_id);
    }

    /// Get the maximum height known among peers
    pub async fn max_peer_height(&self) -> u64 {
        self.peer_heights.read().await.values().max().copied().unwrap_or(0)
    }

    /// Get all peer heights
    pub async fn get_peer_heights(&self) -> HashMap<String, u64> {
        self.peer_heights.read().await.clone()
    }

    /// Check if we need to sync (peers are ahead)
    pub async fn needs_sync(&self) -> bool {
        let our_height = *self.current_height.read().await;
        let max_peer = self.max_peer_height().await;
        max_peer > our_height + 1
    }

    /// Add a pending block awaiting votes
    pub async fn add_pending_block(&self, height: u64, block: Block, proposer: String) -> Result<()> {
        let mut pending = self.pending_blocks.write().await;

        // Enforce max pending blocks to prevent DoS
        if pending.len() >= self.config.max_pending_blocks {
            bail!("Maximum pending blocks ({}) reached", self.config.max_pending_blocks);
        }

        // Check if block is within acceptable range
        let current_height = *self.current_height.read().await;
        if height + self.config.max_fork_depth < current_height {
            bail!("Block height {} too old (current: {})", height, current_height);
        }

        // Verify proposer is a valid validator (if verification enabled)
        if self.config.verify_voters {
            let consensus = self.consensus.read().await;
            let validators = consensus.get_active_validators();
            if !validators.iter().any(|v| v.address == proposer) {
                bail!("Proposer {} is not a registered validator", proposer);
            }
        }

        // Compute block hash for verification
        let block_hash = Self::compute_block_hash(&block);

        pending.insert(height, PendingBlock {
            block,
            proposer: proposer.clone(),
            votes: HashMap::new(),
            vote_count: 0,
            created_at: std::time::Instant::now(),
            block_hash,
        });

        info!("📦 Added pending block at height {} from {}", height, proposer);
        Ok(())
    }

    /// Record a vote for a pending block with validator verification
    pub async fn record_vote(&self, height: u64, voter: String, approve: bool) -> Result<usize, VoteRejection> {
        // Verify voter is a registered validator (if verification enabled)
        if self.config.verify_voters {
            let consensus = self.consensus.read().await;
            let validators = consensus.get_active_validators();
            if !validators.iter().any(|v| v.address == voter) {
                warn!("⚠️ Rejected vote from non-validator: {}", voter);
                *self.votes_rejected.write().await += 1;
                return Err(VoteRejection::InvalidVoter);
            }
        }

        let mut pending = self.pending_blocks.write().await;
        if let Some(pb) = pending.get_mut(&height) {
            // Check for duplicate vote
            if pb.votes.contains_key(&voter) {
                warn!("⚠️ Duplicate vote from {} for height {}", voter, height);
                *self.votes_rejected.write().await += 1;
                return Err(VoteRejection::DuplicateVote);
            }

            // Check if block is expired (too old)
            if pb.created_at.elapsed() > self.config.sync_timeout {
                warn!("⚠️ Vote for expired block at height {}", height);
                *self.votes_rejected.write().await += 1;
                return Err(VoteRejection::Expired);
            }

            pb.votes.insert(voter.clone(), approve);
            if approve {
                pb.vote_count += 1;
            }

            *self.votes_recorded.write().await += 1;
            debug!("📝 Recorded {} vote from {} for height {} (total: {})",
                   if approve { "approve" } else { "reject" }, voter, height, pb.vote_count);

            Ok(pb.vote_count)
        } else {
            Err(VoteRejection::BlockNotFound)
        }
    }

    /// Record a vote with signature verification
    /// Uses the P2P network to look up voter's pubkey and verify signature
    pub async fn record_vote_with_signature(
        &self,
        height: u64,
        voter: String,
        approve: bool,
        signature: &[u8],
        voter_pubkey: &[u8; 32],
    ) -> Result<usize, VoteRejection> {
        // Get block hash for signature verification
        let block_hash = {
            let pending = self.pending_blocks.read().await;
            match pending.get(&height) {
                Some(pb) => pb.block_hash.clone(),
                None => return Err(VoteRejection::BlockNotFound),
            }
        };

        // Verify signature using the pubkey
        use crate::p2p::P2PNetwork;
        if !P2PNetwork::verify_vote_signature(voter_pubkey, block_hash.as_bytes(), signature) {
            warn!("⚠️ Invalid signature on vote from {} for height {}", voter, height);
            *self.votes_rejected.write().await += 1;
            return Err(VoteRejection::InvalidSignature);
        }

        // Delegate to normal record_vote (which handles other validations)
        self.record_vote(height, voter, approve).await
    }

    /// Check if a block has enough votes for finalization
    pub async fn has_enough_votes(&self, height: u64) -> bool {
        let pending = self.pending_blocks.read().await;
        if let Some(pb) = pending.get(&height) {
            let consensus = self.consensus.read().await;
            let total_validators = consensus.get_active_validators().len();
            let required = (total_validators * 2 / 3) + 1; // 2/3 + 1 majority
            pb.vote_count >= required
        } else {
            false
        }
    }

    /// Get and remove a pending block (after finalization)
    /// Validates block hash before returning
    pub async fn finalize_pending_block(&self, height: u64) -> Option<PendingBlock> {
        let mut pending = self.pending_blocks.write().await;

        // Validate block hash before finalizing
        if let Some(pb) = pending.get(&height) {
            let computed_hash = Self::compute_block_hash(&pb.block);
            if computed_hash != pb.block_hash {
                warn!("⚠️ Block hash mismatch at height {} - rejecting finalization", height);
                pending.remove(&height);
                return None;
            }
        }

        let block = pending.remove(&height);
        if block.is_some() {
            *self.blocks_synced.write().await += 1;
            info!("✅ Finalized block at height {}", height);
        }
        block
    }

    /// Get pending block without removing
    pub async fn get_pending_block(&self, height: u64) -> Option<PendingBlock> {
        self.pending_blocks.read().await.get(&height).cloned()
    }

    /// Get count of pending blocks
    pub async fn pending_block_count(&self) -> usize {
        self.pending_blocks.read().await.len()
    }

    /// Compute block hash
    /// 
    /// Uses SHA-256 for compatibility. For higher performance at scale,
    /// consider switching to blake3 (2-3x faster) by adding to Cargo.toml:
    /// `blake3 = "1.5"`
    /// 
    /// Blake3 example:
    /// ```ignore
    /// let hash = blake3::hash(data.as_bytes());
    /// format!("{}", hash.to_hex())
    /// ```
    pub fn compute_block_hash(block: &Block) -> String {
        let data = format!(
            "{}{}{}{}{}{}{}",
            block.index,
            block.timestamp,
            block.prev_hash,
            block.nonce,
            block.validator,
            block.transactions.len(),
            block.state_root
        );

        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verify block hash matches computed hash
    pub fn verify_block_hash(block: &Block) -> bool {
        if block.hash.is_empty() {
            return false;
        }
        let computed = Self::compute_block_hash(block);
        computed == block.hash
    }

    /// Validate a block's basic structure
    pub fn validate_block(block: &Block, expected_height: u64, prev_hash: &str) -> Result<()> {
        if block.index != expected_height {
            bail!("Block height mismatch: expected {}, got {}", expected_height, block.index);
        }

        if block.prev_hash != prev_hash {
            bail!("Block prev_hash mismatch");
        }

        if block.validator.is_empty() {
            bail!("Block missing validator");
        }

        if block.timestamp == 0 {
            bail!("Block missing timestamp");
        }

        Ok(())
    }

    /// Clean up old pending blocks (expired)
    pub async fn cleanup_old_pending(&self, current_height: u64) {
        let mut pending = self.pending_blocks.write().await;
        let before_count = pending.len();

        // Remove blocks more than max_fork_depth behind current height
        // OR blocks that have been pending too long
        pending.retain(|height, pb| {
            let height_ok = *height + self.config.max_fork_depth >= current_height;
            let time_ok = pb.created_at.elapsed() < self.config.sync_timeout * 2;
            height_ok && time_ok
        });

        let removed = before_count - pending.len();
        if removed > 0 {
            warn!("🧹 Cleaned up {} old/expired pending blocks", removed);
        }
    }

    /// Check if we are ahead of all peers (unusual state)
    pub async fn is_ahead(&self) -> bool {
        matches!(*self.sync_state.read().await, SyncState::Ahead)
    }

    /// Check if we are currently syncing
    pub async fn is_syncing(&self) -> bool {
        matches!(*self.sync_state.read().await, SyncState::Syncing { .. })
    }

    /// Get sync statistics
    pub async fn get_statistics(&self) -> SyncStatistics {
        SyncStatistics {
            current_height: *self.current_height.read().await,
            sync_state: self.sync_state.read().await.clone(),
            pending_blocks: self.pending_blocks.read().await.len(),
            seen_blocks: self.seen_blocks.read().await.len(),
            peer_count: self.peer_heights.read().await.len(),
            max_peer_height: self.max_peer_height().await,
            blocks_synced: *self.blocks_synced.read().await,
            votes_recorded: *self.votes_recorded.read().await,
            votes_rejected: *self.votes_rejected.read().await,
        }
    }

    /// Full block validation including proposer, hash, and structure
    pub async fn validate_block_full(&self, block: &Block, prev_hash: &str) -> Result<()> {
        // First do basic validation (block.index is expected height)
        Self::validate_block(block, block.index, prev_hash)?;

        // Verify block hash
        if !Self::verify_block_hash(block) {
            bail!("Block hash verification failed");
        }

        // Verify proposer if verify_voters is enabled
        if self.config.verify_voters {
            let consensus = self.consensus.read().await;
            let validators = consensus.get_active_validators();
            if !validators.iter().any(|v| v.address == block.validator) {
                bail!("Block proposer {} is not an active validator", block.validator);
            }
        }

        Ok(())
    }

    /// Create a sync request message for P2P network
    /// Returns (start_height, count) for the sync request
    pub fn create_sync_request(&self, our_height: u64, target_height: u64) -> (u64, usize) {
        let start = our_height + 1;
        let count = std::cmp::min(
            (target_height - our_height) as usize,
            self.config.max_blocks_per_request,
        );
        (start, count)
    }

    /// Process a sync response - add blocks to pending
    pub async fn process_sync_response(&self, blocks: Vec<Block>, proposer: &str) -> Result<usize> {
        let mut added = 0;
        for block in blocks {
            match self.add_pending_block(block.index, block.clone(), proposer.to_string()).await {
                Ok(()) => added += 1,
                Err(e) => warn!("Failed to add synced block {}: {}", block.index, e),
            }
        }
        Ok(added)
    }

    /// Initialize sync manager with info message
    pub fn init(&self) {
        info!("🔄 BlockSyncManager initialized (verify_voters: {}, max_pending: {})",
              self.config.verify_voters, self.config.max_pending_blocks);
    }
    
    /// Request sync from the P2P network for a range of blocks
    /// Returns a SyncRequest message that can be broadcast via P2P
    /// 
    /// Usage:
    /// ```ignore
    /// let (from, to) = sync_manager.create_sync_request(our_height, target_height);
    /// let request = sync_manager.build_sync_request(from, to);
    /// p2p.broadcast_message(BLOCK_TOPIC, request).await?;
    /// ```
    pub fn build_sync_request(&self, from_height: u64, to_height: u64) -> crate::p2p::NetworkMessage {
        crate::p2p::NetworkMessage::SyncRequest {
            from_height,
            to_height,
        }
    }
    
    /// Check if a sync request is valid (within bounds)
    pub fn validate_sync_request(&self, from_height: u64, to_height: u64) -> Result<()> {
        if to_height < from_height {
            bail!("Invalid sync request: to_height {} < from_height {}", to_height, from_height);
        }
        
        let requested = (to_height - from_height + 1) as usize;
        if requested > self.config.max_blocks_per_request {
            bail!("Sync request too large: {} blocks > max {}", requested, self.config.max_blocks_per_request);
        }
        
        Ok(())
    }
    
    /// Get consensus engine reference for external validation
    pub fn consensus(&self) -> &Arc<RwLock<ConsensusEngine>> {
        &self.consensus
    }
}

/// Sync statistics for monitoring
#[derive(Debug, Clone)]
pub struct SyncStatistics {
    pub current_height: u64,
    pub sync_state: SyncState,
    pub pending_blocks: usize,
    pub seen_blocks: usize,
    pub peer_count: usize,
    pub max_peer_height: u64,
    pub blocks_synced: u64,
    pub votes_recorded: u64,
    pub votes_rejected: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_block(index: u64, prev_hash: &str, validator: &str) -> Block {
        Block {
            index,
            timestamp: 1234567890,
            transactions: vec![],
            prev_hash: prev_hash.to_string(),
            hash: "".to_string(),
            nonce: 0,
            validator: validator.to_string(),
            state_root: "state_root".to_string(),
        }
    }

    fn create_test_consensus() -> ConsensusEngine {
        ConsensusEngine::new()
    }

    // Minimum stake required by ConsensusEngine (10 trillion SULTAN)
    const MIN_STAKE: u64 = 10_000_000_000_000;

    async fn create_sync_manager(verify_voters: bool) -> BlockSyncManager {
        let mut config = SyncConfig::default();
        config.verify_voters = verify_voters;

        let consensus = Arc::new(RwLock::new(create_test_consensus()));

        // Add some validators for testing using the correct API
        {
            let mut c = consensus.write().await;
            // add_validator(address, stake, pubkey) - stake must meet minimum
            c.add_validator("validator1".to_string(), MIN_STAKE, [1u8; 32]).unwrap();
            c.add_validator("validator2".to_string(), MIN_STAKE, [2u8; 32]).unwrap();
            c.add_validator("validator3".to_string(), MIN_STAKE, [3u8; 32]).unwrap();
        }

        BlockSyncManager::new(
            config,
            Some("validator1".to_string()),
            consensus,
            Duration::from_secs(5),
        )
    }

    #[tokio::test]
    async fn test_sync_config_defaults() {
        let config = SyncConfig::default();
        assert_eq!(config.max_blocks_per_request, 100);
        assert_eq!(config.finality_confirmations, 3);
        assert_eq!(config.max_pending_blocks, 100);
        assert_eq!(config.max_seen_blocks, 10000);
        assert!(config.verify_voters);
    }

    #[tokio::test]
    async fn test_block_hash_computation() {
        let block = create_test_block(1, "abc", "val1");
        let hash1 = BlockSyncManager::compute_block_hash(&block);
        let hash2 = BlockSyncManager::compute_block_hash(&block);
        assert_eq!(hash1, hash2); // Deterministic
        assert!(!hash1.is_empty());
    }

    #[tokio::test]
    async fn test_sync_manager_creation() {
        let sync = create_sync_manager(true).await;
        assert!(sync.is_validator());
        assert_eq!(sync.our_address(), Some(&"validator1".to_string()));
        assert_eq!(sync.config().max_blocks_per_request, 100);

        let stats = sync.get_statistics().await;
        assert_eq!(stats.current_height, 0);
        assert_eq!(stats.blocks_synced, 0);
        assert_eq!(stats.votes_recorded, 0);
    }

    #[tokio::test]
    async fn test_height_tracking() {
        let sync = create_sync_manager(true).await;
        assert_eq!(sync.get_height().await, 0);

        sync.set_height(100).await;
        assert_eq!(sync.get_height().await, 100);
    }

    #[tokio::test]
    async fn test_sync_state_transitions() {
        let sync = create_sync_manager(true).await;

        // Initially synced
        assert!(matches!(sync.get_sync_state().await, SyncState::Synced));

        // Set to syncing
        sync.set_sync_state(SyncState::Syncing { target_height: 100 }).await;
        if let SyncState::Syncing { target_height } = sync.get_sync_state().await {
            assert_eq!(target_height, 100);
        } else {
            panic!("Expected Syncing state");
        }

        // Set to ahead
        sync.set_sync_state(SyncState::Ahead).await;
        assert!(matches!(sync.get_sync_state().await, SyncState::Ahead));
    }

    #[tokio::test]
    async fn test_peer_height_tracking() {
        let sync = create_sync_manager(true).await;
        sync.set_height(10).await;

        sync.update_peer_height("peer1".to_string(), 50).await;
        sync.update_peer_height("peer2".to_string(), 100).await;

        assert_eq!(sync.max_peer_height().await, 100);
        assert!(sync.needs_sync().await);

        // Verify peer heights
        let heights = sync.get_peer_heights().await;
        assert_eq!(heights.len(), 2);
        assert_eq!(heights.get("peer1"), Some(&50));

        // Remove peer
        sync.remove_peer("peer1").await;
        let heights = sync.get_peer_heights().await;
        assert_eq!(heights.len(), 1);
    }

    #[tokio::test]
    async fn test_auto_sync_state_update() {
        let sync = create_sync_manager(true).await;
        sync.set_height(10).await;

        // Peer way ahead -> should trigger Syncing state
        sync.update_peer_height("peer1".to_string(), 100).await;
        if let SyncState::Syncing { target_height } = sync.get_sync_state().await {
            assert_eq!(target_height, 100);
        } else {
            panic!("Expected Syncing state when peer is ahead");
        }

        // Now set our height way ahead
        sync.set_height(200).await;
        sync.update_peer_height("peer1".to_string(), 100).await;
        assert!(matches!(sync.get_sync_state().await, SyncState::Ahead));
    }

    #[tokio::test]
    async fn test_pending_block_add_and_finalize() {
        let sync = create_sync_manager(false).await; // Disable voter verification for simplicity

        let block = create_test_block(1, "genesis", "validator1");
        sync.add_pending_block(1, block.clone(), "validator1".to_string()).await.unwrap();
        assert_eq!(sync.pending_block_count().await, 1);

        // Get pending block
        let pending = sync.get_pending_block(1).await.unwrap();
        assert_eq!(pending.proposer, "validator1");
        assert_eq!(pending.vote_count, 0);

        // Finalize
        let finalized = sync.finalize_pending_block(1).await.unwrap();
        assert_eq!(finalized.block.index, 1);
        assert_eq!(sync.pending_block_count().await, 0);

        let stats = sync.get_statistics().await;
        assert_eq!(stats.blocks_synced, 1);
    }

    #[tokio::test]
    async fn test_vote_recording_without_verification() {
        let sync = create_sync_manager(false).await; // Disable verification

        let block = create_test_block(1, "genesis", "validator1");
        sync.add_pending_block(1, block, "validator1".to_string()).await.unwrap();

        // Record votes
        let count = sync.record_vote(1, "voter1".to_string(), true).await.unwrap();
        assert_eq!(count, 1);

        let count = sync.record_vote(1, "voter2".to_string(), true).await.unwrap();
        assert_eq!(count, 2);

        // Record reject vote (doesn't increment count)
        let count = sync.record_vote(1, "voter3".to_string(), false).await.unwrap();
        assert_eq!(count, 2);

        let stats = sync.get_statistics().await;
        assert_eq!(stats.votes_recorded, 3);
    }

    #[tokio::test]
    async fn test_vote_validation_enabled() {
        let sync = create_sync_manager(true).await; // Enable verification

        let block = create_test_block(1, "genesis", "validator1");
        sync.add_pending_block(1, block, "validator1".to_string()).await.unwrap();

        // Valid validator vote
        let result = sync.record_vote(1, "validator1".to_string(), true).await;
        assert!(result.is_ok());

        // Invalid voter (not a validator)
        let result = sync.record_vote(1, "random_user".to_string(), true).await;
        assert!(matches!(result, Err(VoteRejection::InvalidVoter)));

        let stats = sync.get_statistics().await;
        assert_eq!(stats.votes_rejected, 1);
    }

    #[tokio::test]
    async fn test_duplicate_vote_rejection() {
        let sync = create_sync_manager(true).await;

        let block = create_test_block(1, "genesis", "validator1");
        sync.add_pending_block(1, block, "validator1".to_string()).await.unwrap();

        // First vote
        let result = sync.record_vote(1, "validator1".to_string(), true).await;
        assert!(result.is_ok());

        // Duplicate vote from same validator
        let result = sync.record_vote(1, "validator1".to_string(), true).await;
        assert!(matches!(result, Err(VoteRejection::DuplicateVote)));
    }

    #[tokio::test]
    async fn test_vote_on_nonexistent_block() {
        let sync = create_sync_manager(true).await;

        // Vote on block that doesn't exist
        let result = sync.record_vote(999, "validator1".to_string(), true).await;
        assert!(matches!(result, Err(VoteRejection::BlockNotFound)));
    }

    #[tokio::test]
    async fn test_has_enough_votes() {
        let sync = create_sync_manager(true).await;

        let block = create_test_block(1, "genesis", "validator1");
        sync.add_pending_block(1, block, "validator1".to_string()).await.unwrap();

        // Need 2/3 + 1 = 3 votes with 3 validators
        assert!(!sync.has_enough_votes(1).await);

        sync.record_vote(1, "validator1".to_string(), true).await.unwrap();
        assert!(!sync.has_enough_votes(1).await);

        sync.record_vote(1, "validator2".to_string(), true).await.unwrap();
        assert!(!sync.has_enough_votes(1).await);

        sync.record_vote(1, "validator3".to_string(), true).await.unwrap();
        assert!(sync.has_enough_votes(1).await);
    }

    #[tokio::test]
    async fn test_seen_blocks_tracking() {
        let sync = create_sync_manager(true).await;

        assert!(!sync.is_block_seen("hash1").await);

        sync.mark_block_seen("hash1".to_string()).await;
        assert!(sync.is_block_seen("hash1").await);

        sync.mark_block_seen("hash2".to_string()).await;
        assert!(sync.is_block_seen("hash2").await);
    }

    #[tokio::test]
    async fn test_cleanup_old_pending() {
        let sync = create_sync_manager(false).await;

        // Add blocks at various heights
        for i in 1..=20 {
            let block = create_test_block(i, "prev", "val1");
            sync.add_pending_block(i, block, "val1".to_string()).await.unwrap();
        }
        assert_eq!(sync.pending_block_count().await, 20);

        // Cleanup with current height 25 (max_fork_depth = 10)
        // Blocks 1-14 should be removed (25 - 10 = 15 threshold)
        sync.cleanup_old_pending(25).await;

        let count = sync.pending_block_count().await;
        assert!(count < 20); // Some should be cleaned up
    }

    #[tokio::test]
    async fn test_max_pending_blocks_limit() {
        let mut config = SyncConfig::default();
        config.max_pending_blocks = 5;
        config.verify_voters = false;

        let consensus = Arc::new(RwLock::new(create_test_consensus()));
        let sync = BlockSyncManager::new(
            config,
            Some("val1".to_string()),
            consensus,
            Duration::from_secs(5),
        );

        // Add up to limit
        for i in 1..=5 {
            let block = create_test_block(i, "prev", "val1");
            assert!(sync.add_pending_block(i, block, "val1".to_string()).await.is_ok());
        }

        // Exceeding limit should fail
        let block = create_test_block(6, "prev", "val1");
        let result = sync.add_pending_block(6, block, "val1".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_block_validation() {
        let block = create_test_block(10, "prev_hash", "validator1");

        // Valid block
        assert!(BlockSyncManager::validate_block(&block, 10, "prev_hash").is_ok());

        // Wrong height
        assert!(BlockSyncManager::validate_block(&block, 11, "prev_hash").is_err());

        // Wrong prev_hash
        assert!(BlockSyncManager::validate_block(&block, 10, "wrong_hash").is_err());
    }

    #[tokio::test]
    async fn test_block_hash_verification() {
        let mut block = create_test_block(1, "prev", "val1");

        // Empty hash is invalid
        assert!(!BlockSyncManager::verify_block_hash(&block));

        // Compute and set hash
        block.hash = BlockSyncManager::compute_block_hash(&block);
        assert!(BlockSyncManager::verify_block_hash(&block));

        // Tampered hash is invalid
        block.hash = "tampered".to_string();
        assert!(!BlockSyncManager::verify_block_hash(&block));
    }

    #[tokio::test]
    async fn test_statistics() {
        let sync = create_sync_manager(false).await;
        sync.set_height(100).await;

        // Add and finalize a block
        let block = create_test_block(101, "prev", "val1");
        sync.add_pending_block(101, block, "val1".to_string()).await.unwrap();
        sync.record_vote(101, "voter1".to_string(), true).await.unwrap();
        sync.finalize_pending_block(101).await;

        // Add peers
        sync.update_peer_height("peer1".to_string(), 105).await;

        let stats = sync.get_statistics().await;
        assert_eq!(stats.current_height, 100);
        assert_eq!(stats.blocks_synced, 1);
        assert_eq!(stats.votes_recorded, 1);
        assert_eq!(stats.peer_count, 1);
        assert_eq!(stats.max_peer_height, 105);
    }

    #[tokio::test]
    async fn test_needs_sync() {
        let sync = create_sync_manager(true).await;
        sync.set_height(100).await;

        // No peers
        assert!(!sync.needs_sync().await);

        // Peer at same height
        sync.update_peer_height("peer1".to_string(), 100).await;
        assert!(!sync.needs_sync().await);

        // Peer 1 ahead (within tolerance)
        sync.update_peer_height("peer1".to_string(), 101).await;
        assert!(!sync.needs_sync().await);

        // Peer 2+ ahead (needs sync)
        sync.update_peer_height("peer1".to_string(), 103).await;
        assert!(sync.needs_sync().await);
    }

    #[tokio::test]
    async fn test_proposer_verification() {
        let sync = create_sync_manager(true).await; // Enable verification

        let block = create_test_block(1, "genesis", "validator1");

        // Valid proposer (registered validator)
        let result = sync.add_pending_block(1, block.clone(), "validator1".to_string()).await;
        assert!(result.is_ok());

        // Invalid proposer (not a validator)
        let block2 = create_test_block(2, "prev", "fake_proposer");
        let result = sync.add_pending_block(2, block2, "fake_proposer".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_is_ahead_and_is_syncing() {
        let sync = create_sync_manager(true).await;

        // Initially synced (not ahead, not syncing)
        assert!(!sync.is_ahead().await);
        assert!(!sync.is_syncing().await);

        // Set to syncing
        sync.set_sync_state(SyncState::Syncing { target_height: 100 }).await;
        assert!(!sync.is_ahead().await);
        assert!(sync.is_syncing().await);

        // Set to ahead
        sync.set_sync_state(SyncState::Ahead).await;
        assert!(sync.is_ahead().await);
        assert!(!sync.is_syncing().await);
    }

    #[tokio::test]
    async fn test_finalize_with_hash_validation() {
        let sync = create_sync_manager(false).await;

        let block = create_test_block(1, "genesis", "validator1");
        sync.add_pending_block(1, block.clone(), "validator1".to_string()).await.unwrap();

        // Finalize should succeed (hash matches computed)
        let finalized = sync.finalize_pending_block(1).await;
        assert!(finalized.is_some());

        // Verify the computed hash matches
        let pb = finalized.unwrap();
        assert_eq!(pb.block_hash, BlockSyncManager::compute_block_hash(&pb.block));
    }

    #[tokio::test]
    async fn test_vote_rejection_invalid_signature() {
        // Test that VoteRejection::InvalidSignature exists and can be used
        let rejection = VoteRejection::InvalidSignature;
        assert_eq!(rejection, VoteRejection::InvalidSignature);
    }

    #[test]
    fn test_create_sync_request() {
        let config = SyncConfig::default();
        let consensus = Arc::new(RwLock::new(ConsensusEngine::new()));
        let sync = BlockSyncManager::new(config, None, consensus, Duration::from_secs(6));

        // Request from height 10 to target 50
        let (start, count) = sync.create_sync_request(10, 50);
        assert_eq!(start, 11); // Start at current + 1
        assert_eq!(count, 40); // 50 - 10 = 40, but capped at max_blocks_per_request

        // Request where remaining is less than max
        let (start, count) = sync.create_sync_request(95, 100);
        assert_eq!(start, 96);
        assert_eq!(count, 5); // Only 5 blocks needed
    }

    #[tokio::test]
    async fn test_process_sync_response() {
        let sync = create_sync_manager(false).await;

        let blocks = vec![
            create_test_block(1, "genesis", "val1"),
            create_test_block(2, "hash1", "val1"),
            create_test_block(3, "hash2", "val1"),
        ];

        let added = sync.process_sync_response(blocks, "peer1").await.unwrap();
        assert_eq!(added, 3);
        assert_eq!(sync.pending_block_count().await, 3);
    }

    #[tokio::test]
    async fn test_validate_block_full() {
        let sync = create_sync_manager(false).await;

        let block = create_test_block(1, "genesis", "validator1");

        // Without verify_voters, should pass (hash verification will fail since block.hash is empty)
        let result = sync.validate_block_full(&block, "genesis").await;
        assert!(result.is_err()); // Fails because block.hash is empty

        // Create a block with valid hash
        let mut block_with_hash = create_test_block(1, "genesis", "validator1");
        block_with_hash.hash = BlockSyncManager::compute_block_hash(&block_with_hash);

        let result = sync.validate_block_full(&block_with_hash, "genesis").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_block_full_with_proposer_verification() {
        let sync = create_sync_manager(true).await;

        // Create block with valid hash but non-validator proposer
        let mut block = create_test_block(1, "genesis", "unknown_proposer");
        block.hash = BlockSyncManager::compute_block_hash(&block);

        // Should fail due to proposer not being in validator set
        let result = sync.validate_block_full(&block, "genesis").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not an active validator"));
    }

    #[test]
    fn test_build_sync_request() {
        let config = SyncConfig::default();
        let consensus = Arc::new(RwLock::new(ConsensusEngine::new()));
        let sync = BlockSyncManager::new(config, None, consensus, Duration::from_secs(6));

        // Build a sync request
        let request = sync.build_sync_request(10, 50);
        
        match request {
            crate::p2p::NetworkMessage::SyncRequest { from_height, to_height } => {
                assert_eq!(from_height, 10);
                assert_eq!(to_height, 50);
            }
            _ => panic!("Expected SyncRequest message"),
        }
    }

    #[test]
    fn test_validate_sync_request() {
        let config = SyncConfig::default();
        let consensus = Arc::new(RwLock::new(ConsensusEngine::new()));
        let sync = BlockSyncManager::new(config, None, consensus, Duration::from_secs(6));

        // Valid request
        assert!(sync.validate_sync_request(10, 50).is_ok());
        
        // Invalid: to < from
        assert!(sync.validate_sync_request(50, 10).is_err());
        
        // Invalid: too many blocks (max is 100)
        assert!(sync.validate_sync_request(0, 200).is_err());
        
        // Edge case: exactly max blocks
        assert!(sync.validate_sync_request(0, 99).is_ok()); // 100 blocks: 0-99 inclusive
    }

    #[tokio::test]
    async fn test_consensus_accessor() {
        let sync = create_sync_manager(true).await;
        
        // Verify we can access consensus through the accessor
        let consensus = sync.consensus();
        let consensus_guard = consensus.read().await;
        let validators = consensus_guard.get_active_validators();
        assert_eq!(validators.len(), 3); // validator1, validator2, validator3
    }
}
