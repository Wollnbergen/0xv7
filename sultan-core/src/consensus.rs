//! Sultan Consensus Engine
//!
//! Production-grade consensus implementation with:
//! - Validator management (add/remove/update with min stake)
//! - Stake-weighted proposer selection (deterministic SHA256)
//! - Byzantine fault tolerance (2/3 + 1 quorum)
//! - Slashing for downtime (0.1%) and double-signing (5%)
//! - Ed25519 signature verification for Byzantine consensus
//! - Slashing evidence collection for on-chain proofs
//! - Signature collection and threshold validation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Result, bail};
use tracing::{info, warn, debug, error};
use sha2::{Sha256, Digest};
use ed25519_dalek::{Verifier, VerifyingKey, Signature};

/// Slashing parameters matching Sultan's PoS design
pub const DOWNTIME_SLASH_PERCENT: u64 = 1;    // 0.1% = 1/1000
pub const DOUBLE_SIGN_SLASH_PERCENT: u64 = 50; // 5% = 50/1000
pub const SLASH_DENOMINATOR: u64 = 1000;
pub const MAX_MISSED_BLOCKS_BEFORE_SLASH: u64 = 100;
pub const JAIL_DURATION_BLOCKS: u64 = 10_000; // ~5.5 hours at 2s blocks

/// Fallback proposer parameters
pub const FALLBACK_THRESHOLD_MISSED_BLOCKS: u64 = 5; // Fallback kicks in after this many misses
pub const MAX_FALLBACK_POSITIONS: usize = 3; // Only top N fallbacks can step in
pub const MISSED_BLOCK_TRACKING_WINDOW: u64 = 1000; // Keep track of last N heights

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Validator {
    pub address: String,
    /// Ed25519 public key (32 bytes) for signature verification
    pub pubkey: [u8; 32],
    pub stake: u64,
    pub voting_power: u64,
    pub is_active: bool,
    pub blocks_proposed: u64,
    pub blocks_signed: u64,
    /// Consecutive missed blocks (reset on sign)
    pub missed_blocks: u64,
    /// Whether validator is jailed (cannot participate)
    pub is_jailed: bool,
    /// Block height when jail ends (can unjail after this)
    pub jail_until: u64,
    /// Total amount slashed from this validator
    pub total_slashed: u64,
}

/// Evidence of slashing offense (stored for transparency/appeals)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingEvidence {
    pub validator_address: String,
    pub offense_type: SlashingOffense,
    pub height: u64,
    pub timestamp: u64,
    pub slash_amount: u64,
    /// For double-sign: the two conflicting block hashes
    pub evidence_hashes: Option<([u8; 32], [u8; 32])>,
    /// For downtime: number of consecutive missed blocks
    pub missed_blocks: Option<u64>,
}

/// Type of slashing offense
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SlashingOffense {
    Downtime,
    DoubleSign,
}

/// Block vote for Byzantine consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockVote {
    pub block_hash: [u8; 32],
    pub height: u64,
    pub validator_address: String,
    pub signature: Vec<u8>,
}

/// Signature collection for a specific block
#[derive(Debug, Clone, Default)]
pub struct SignatureSet {
    pub block_hash: [u8; 32],
    pub height: u64,
    pub signatures: HashMap<String, Vec<u8>>,
    pub voting_power_collected: u64,
}

/// Snapshot of consensus state for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusSnapshot {
    pub validators: HashMap<String, Validator>,
    pub round: u64,
    pub total_stake: u64,
    pub prev_block_hash: [u8; 32],
    pub slashing_evidence: Vec<SlashingEvidence>,
}

pub struct ConsensusEngine {
    pub validators: HashMap<String, Validator>,
    pub current_proposer: Option<String>,
    pub round: u64,
    pub min_stake: u64,
    pub total_stake: u64,
    /// Previous block hash for unpredictable seed
    pub prev_block_hash: [u8; 32],
    /// Pending signature sets for blocks awaiting consensus
    pub pending_signatures: HashMap<u64, SignatureSet>,
    /// Track double-signing: height -> validator -> block_hash
    pub height_signatures: HashMap<u64, HashMap<String, [u8; 32]>>,
    /// Slashing evidence for transparency and potential appeals
    pub slashing_evidence: Vec<SlashingEvidence>,
    /// Whether to verify Ed25519 signatures (disable for testing)
    pub verify_signatures: bool,
    /// Track which heights we've already recorded missed blocks for (prevents double-counting)
    /// Maps height -> validator_address that missed
    pub recorded_misses: HashMap<u64, String>,
}

impl Default for ConsensusEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsensusEngine {
    /// Create new consensus engine
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
            current_proposer: None,
            round: 0,
            min_stake: 10_000_000_000_000, // 10,000 SLTN (with 9 decimals) - matches staking.rs
            total_stake: 0,
            prev_block_hash: [0u8; 32],
            pending_signatures: HashMap::new(),
            height_signatures: HashMap::new(),
            slashing_evidence: Vec::new(),
            verify_signatures: true, // Enable signature verification by default
            recorded_misses: HashMap::new(),
        }
    }

    /// Create consensus engine with signature verification disabled (for testing)
    #[cfg(test)]
    pub fn new_without_verification() -> Self {
        let mut engine = Self::new();
        engine.verify_signatures = false;
        engine
    }
    
    /// Cleanup old recorded misses to prevent memory growth
    /// Called periodically during block processing
    pub fn cleanup_recorded_misses(&mut self, current_height: u64) {
        if current_height > MISSED_BLOCK_TRACKING_WINDOW {
            let cutoff = current_height - MISSED_BLOCK_TRACKING_WINDOW;
            self.recorded_misses.retain(|height, _| *height > cutoff);
        }
    }

    /// Update previous block hash (called after block finalization)
    pub fn set_prev_block_hash(&mut self, hash: [u8; 32]) {
        self.prev_block_hash = hash;
    }

    /// Add validator to active set
    pub fn add_validator(&mut self, address: String, stake: u64, pubkey: [u8; 32]) -> Result<()> {
        if stake < self.min_stake {
            bail!("Stake {} below minimum {}", stake, self.min_stake);
        }

        // Check if validator already exists
        if self.validators.contains_key(&address) {
            bail!("Validator {} already exists in consensus", address);
        }

        let voting_power = self.calculate_voting_power(stake);
        
        let validator = Validator {
            address: address.clone(),
            pubkey,
            stake,
            voting_power,
            is_active: true,
            blocks_proposed: 0,
            blocks_signed: 0,
            missed_blocks: 0,
            is_jailed: false,
            jail_until: 0,
            total_slashed: 0,
        };

        self.total_stake += stake;
        self.validators.insert(address.clone(), validator);
        
        info!("Validator added: {} with stake {} (voting power: {})", 
              address, stake, voting_power);
        
        Ok(())
    }

    /// Remove validator from active set
    pub fn remove_validator(&mut self, address: &str) -> Result<()> {
        if let Some(validator) = self.validators.get_mut(address) {
            validator.is_active = false;
            self.total_stake -= validator.stake;
            info!("Validator removed: {}", address);
            Ok(())
        } else {
            bail!("Validator {} not found", address);
        }
    }

    /// Update validator stake
    pub fn update_stake(&mut self, address: &str, new_stake: u64) -> Result<()> {
        if new_stake < self.min_stake {
            bail!("Stake {} below minimum {}", new_stake, self.min_stake);
        }

        let voting_power = self.calculate_voting_power(new_stake);
        if let Some(validator) = self.validators.get_mut(address) {
            let old_stake = validator.stake;
            self.total_stake = self.total_stake - old_stake + new_stake;
            validator.stake = new_stake;
            validator.voting_power = voting_power;
            
            info!("Validator {} stake updated: {} -> {}", address, old_stake, new_stake);
            Ok(())
        } else {
            bail!("Validator {} not found", address);
        }
    }

    /// Select next block proposer using weighted randomness
    pub fn select_proposer(&mut self) -> Option<String> {
        let active_validators: Vec<_> = self.validators
            .iter()
            .filter(|(_, v)| v.is_active)
            .collect();

        if active_validators.is_empty() {
            warn!("No active validators");
            return None;
        }

        // Deterministic weighted selection based on round and voting power
        let seed = self.calculate_selection_seed();
        let total_power: u64 = active_validators.iter().map(|(_, v)| v.voting_power).sum();
        
        if total_power == 0 {
            warn!("Total voting power is zero");
            return None;
        }

        let target = seed % total_power;
        let mut cumulative = 0u64;

        for (address, validator) in &active_validators {
            cumulative += validator.voting_power;
            if cumulative > target {
                self.current_proposer = Some((*address).clone());
                self.round += 1;
                
                info!("Round {}: Proposer selected: {}", self.round, address);
                return Some((*address).clone());
            }
        }

        // Fallback (should never happen with correct math)
        let fallback = active_validators[0].0.clone();
        self.current_proposer = Some(fallback.clone());
        self.round += 1;
        
        Some(fallback)
    }

    /// Record block proposal by validator
    pub fn record_proposal(&mut self, validator_address: &str) -> Result<()> {
        if let Some(validator) = self.validators.get_mut(validator_address) {
            validator.blocks_proposed += 1;
            Ok(())
        } else {
            bail!("Validator {} not found", validator_address);
        }
    }

    /// Record block signature by validator (resets missed block counter)
    pub fn record_signature(&mut self, validator_address: &str) -> Result<()> {
        if let Some(validator) = self.validators.get_mut(validator_address) {
            validator.blocks_signed += 1;
            validator.missed_blocks = 0; // Reset on successful sign
            Ok(())
        } else {
            bail!("Validator {} not found", validator_address);
        }
    }

    /// Record missed block for validator (triggers slashing if threshold exceeded)
    /// Returns the slash amount if slashing occurred
    /// 
    /// SAFETY: Uses height-based deduplication to prevent double-counting
    /// the same missed block from multiple sources (P2P gossip, sync, etc.)
    pub fn record_missed_block(&mut self, validator_address: &str, current_height: u64) -> Result<Option<u64>> {
        // CRITICAL: Check if we already recorded a miss for this height
        // This prevents double-counting when multiple nodes report the same miss
        if let Some(existing) = self.recorded_misses.get(&current_height) {
            if existing == validator_address {
                debug!("Already recorded missed block at height {} for {}", current_height, validator_address);
                return Ok(None);
            }
        }
        
        // First check if validator exists and get missed_blocks
        let (is_jailed, missed_blocks) = {
            let validator = self.validators.get(validator_address)
                .ok_or_else(|| anyhow::anyhow!("Validator {} not found", validator_address))?;
            (validator.is_jailed, validator.missed_blocks)
        };
        
        if is_jailed {
            return Ok(None); // Already jailed, don't accumulate
        }
        
        // Record that we've processed this height (before incrementing to prevent races)
        self.recorded_misses.insert(current_height, validator_address.to_string());
        
        // Periodic cleanup to prevent memory growth
        if current_height % 100 == 0 {
            self.cleanup_recorded_misses(current_height);
        }
        
        // Increment missed blocks
        if let Some(validator) = self.validators.get_mut(validator_address) {
            validator.missed_blocks += 1;
        }
        
        let new_missed = missed_blocks + 1;
        
        if new_missed >= MAX_MISSED_BLOCKS_BEFORE_SLASH {
            // Apply downtime slashing (0.1%)
            let slash_amount = self.apply_slash_with_evidence(
                validator_address, 
                DOWNTIME_SLASH_PERCENT, 
                current_height,
                SlashingOffense::Downtime,
                None,
                Some(new_missed),
            )?;
            info!("Validator {} slashed {} for downtime ({} missed blocks)", 
                  validator_address, slash_amount, MAX_MISSED_BLOCKS_BEFORE_SLASH);
            return Ok(Some(slash_amount));
        }
        
        Ok(None)
    }

    /// Record a block vote and check for double-signing
    /// Returns slash amount if double-sign detected
    pub fn record_block_vote(&mut self, vote: &BlockVote, current_height: u64) -> Result<Option<u64>> {
        let height_sigs = self.height_signatures.entry(vote.height).or_insert_with(HashMap::new);
        
        if let Some(existing_hash) = height_sigs.get(&vote.validator_address) {
            // Check if signing different block at same height (double-sign!)
            if *existing_hash != vote.block_hash {
                error!("DOUBLE-SIGN DETECTED: {} signed different blocks at height {}", 
                       vote.validator_address, vote.height);
                
                // Record evidence with both conflicting block hashes
                let evidence_hashes = Some((*existing_hash, vote.block_hash));
                
                // Apply double-sign slashing (5%)
                let slash_amount = self.apply_slash_with_evidence(
                    &vote.validator_address, 
                    DOUBLE_SIGN_SLASH_PERCENT, 
                    current_height,
                    SlashingOffense::DoubleSign,
                    evidence_hashes,
                    None,
                )?;
                return Ok(Some(slash_amount));
            }
        } else {
            // Record this signature
            height_sigs.insert(vote.validator_address.clone(), vote.block_hash);
        }
        
        Ok(None)
    }

    /// Apply slashing to a validator with evidence recording
    /// Returns the amount slashed
    fn apply_slash_with_evidence(
        &mut self, 
        validator_address: &str, 
        slash_percent: u64, 
        current_height: u64,
        offense_type: SlashingOffense,
        evidence_hashes: Option<([u8; 32], [u8; 32])>,
        missed_blocks: Option<u64>,
    ) -> Result<u64> {
        if let Some(validator) = self.validators.get_mut(validator_address) {
            let slash_amount = validator.stake * slash_percent / SLASH_DENOMINATOR;
            
            // Deduct from stake
            validator.stake = validator.stake.saturating_sub(slash_amount);
            let new_stake = validator.stake;
            validator.voting_power = Self::calculate_voting_power_static(new_stake, self.total_stake.saturating_sub(slash_amount));
            validator.total_slashed += slash_amount;
            
            // Update total stake: remove slashed amount AND remaining stake (validator is inactive)
            // When unjailed, remaining stake will be re-added to total_stake
            self.total_stake = self.total_stake.saturating_sub(slash_amount + new_stake);
            
            // Jail the validator
            validator.is_jailed = true;
            validator.is_active = false;
            validator.jail_until = current_height + JAIL_DURATION_BLOCKS;
            validator.missed_blocks = 0; // Reset counter
            
            warn!("Validator {} jailed until block {} (slashed {} = {}%)", 
                  validator_address, validator.jail_until, slash_amount, 
                  slash_percent as f64 / 10.0);
            
            // Record slashing evidence for transparency
            let evidence = SlashingEvidence {
                validator_address: validator_address.to_string(),
                offense_type,
                height: current_height,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                slash_amount,
                evidence_hashes,
                missed_blocks,
            };
            self.slashing_evidence.push(evidence);
            
            Ok(slash_amount)
        } else {
            bail!("Validator {} not found", validator_address);
        }
    }

    /// Unjail a validator (if jail period has passed and stake still sufficient)
    pub fn unjail_validator(&mut self, validator_address: &str, current_height: u64) -> Result<()> {
        if let Some(validator) = self.validators.get_mut(validator_address) {
            if !validator.is_jailed {
                bail!("Validator {} is not jailed", validator_address);
            }
            
            if current_height < validator.jail_until {
                bail!("Validator {} still jailed until block {} (current: {})", 
                      validator_address, validator.jail_until, current_height);
            }
            
            if validator.stake < self.min_stake {
                bail!("Validator {} stake {} below minimum {} after slashing", 
                      validator_address, validator.stake, self.min_stake);
            }
            
            validator.is_jailed = false;
            validator.is_active = true;
            // Re-add stake to active total (was removed when jailed in slash_and_jail)
            self.total_stake += validator.stake;
            
            info!("Validator {} unjailed and reactivated", validator_address);
            Ok(())
        } else {
            bail!("Validator {} not found", validator_address);
        }
    }

    /// Collect signature for a block (for Byzantine consensus)
    /// Verifies Ed25519 signature before accepting
    pub fn collect_signature(&mut self, height: u64, block_hash: [u8; 32], 
                             validator_address: &str, signature: Vec<u8>) -> Result<bool> {
        // Verify validator exists and is active, get voting power and pubkey
        let (voting_power, pubkey) = match self.validators.get(validator_address) {
            Some(v) if v.is_active && !v.is_jailed => (v.voting_power, v.pubkey),
            Some(_) => bail!("Validator {} is not active or is jailed", validator_address),
            None => bail!("Validator {} not found", validator_address),
        };
        
        // Verify Ed25519 signature if enabled
        if self.verify_signatures {
            self.verify_ed25519_signature(&pubkey, &block_hash, &signature)?;
        }
        
        let sig_set = self.pending_signatures.entry(height).or_insert_with(|| {
            SignatureSet {
                block_hash,
                height,
                signatures: HashMap::new(),
                voting_power_collected: 0,
            }
        });
        
        // Verify block hash matches
        if sig_set.block_hash != block_hash {
            warn!("Block hash mismatch at height {}: validator {} signed different block", 
                  height, validator_address);
            return Ok(false);
        }
        
        // Add signature if not already present
        if !sig_set.signatures.contains_key(validator_address) {
            sig_set.signatures.insert(validator_address.to_string(), signature);
            sig_set.voting_power_collected += voting_power;
        }
        
        // Check if we have consensus
        let has_consensus = self.check_consensus(height);
        Ok(has_consensus)
    }
    
    /// Verify Ed25519 signature
    fn verify_ed25519_signature(&self, pubkey: &[u8; 32], message: &[u8; 32], signature: &[u8]) -> Result<()> {
        if signature.len() != 64 {
            bail!("Invalid signature length: expected 64 bytes, got {}", signature.len());
        }
        
        let verifying_key = VerifyingKey::from_bytes(pubkey)
            .map_err(|e| anyhow::anyhow!("Invalid public key: {}", e))?;
        
        let sig_bytes: [u8; 64] = signature.try_into()
            .map_err(|_| anyhow::anyhow!("Failed to convert signature to fixed array"))?;
        let sig = Signature::from_bytes(&sig_bytes);
        
        verifying_key.verify(message, &sig)
            .map_err(|e| anyhow::anyhow!("Signature verification failed: {}", e))?;
        
        debug!("Ed25519 signature verified successfully");
        Ok(())
    }

    /// Check if we have enough signatures for consensus at given height
    pub fn check_consensus(&self, height: u64) -> bool {
        if let Some(sig_set) = self.pending_signatures.get(&height) {
            let required_power = self.required_voting_power();
            sig_set.voting_power_collected >= required_power
        } else {
            false
        }
    }

    /// Calculate required voting power for consensus (2/3 of total)
    pub fn required_voting_power(&self) -> u64 {
        // 2/3 + 1 of total active voting power
        let active_power: u64 = self.validators
            .values()
            .filter(|v| v.is_active && !v.is_jailed)
            .map(|v| v.voting_power)
            .sum();
        
        (active_power * 2 / 3) + 1
    }

    /// Clean up old signature sets (call periodically)
    pub fn cleanup_old_signatures(&mut self, current_height: u64, keep_blocks: u64) {
        let cutoff = current_height.saturating_sub(keep_blocks);
        self.pending_signatures.retain(|h, _| *h > cutoff);
        self.height_signatures.retain(|h, _| *h > cutoff);
    }

    /// Get all slashing evidence (for transparency/on-chain storage)
    pub fn get_slashing_evidence(&self) -> &[SlashingEvidence] {
        &self.slashing_evidence
    }

    /// Get slashing evidence for a specific validator
    pub fn get_validator_slashing_history(&self, validator_address: &str) -> Vec<&SlashingEvidence> {
        self.slashing_evidence
            .iter()
            .filter(|e| e.validator_address == validator_address)
            .collect()
    }

    /// Export state for persistence (serializable snapshot)
    pub fn export_state(&self) -> ConsensusSnapshot {
        ConsensusSnapshot {
            validators: self.validators.clone(),
            round: self.round,
            total_stake: self.total_stake,
            prev_block_hash: self.prev_block_hash,
            slashing_evidence: self.slashing_evidence.clone(),
        }
    }

    /// Import state from persistence
    pub fn import_state(&mut self, snapshot: ConsensusSnapshot) {
        self.validators = snapshot.validators;
        self.round = snapshot.round;
        self.total_stake = snapshot.total_stake;
        self.prev_block_hash = snapshot.prev_block_hash;
        self.slashing_evidence = snapshot.slashing_evidence;
        // Clear transient state
        self.pending_signatures.clear();
        self.height_signatures.clear();
        info!("Consensus state imported: {} validators, round {}", 
              self.validators.len(), self.round);
    }

    /// Calculate voting power based on stake (linear for now)
    fn calculate_voting_power(&self, stake: u64) -> u64 {
        // Linear relationship: voting power = stake
        // Can be made non-linear for more sophisticated economics
        stake
    }

    /// Static version for use in borrow-constrained contexts
    fn calculate_voting_power_static(_stake: u64, _total_stake: u64) -> u64 {
        // Linear relationship: voting power = stake
        _stake
    }

    /// Calculate deterministic seed for proposer selection
    /// Includes previous block hash for unpredictability (prevents grinding)
    fn calculate_selection_seed(&self) -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(&self.round.to_le_bytes());
        hasher.update(&self.total_stake.to_le_bytes());
        hasher.update(&self.prev_block_hash);
        let result = hasher.finalize();
        
        // Use first 8 bytes as u64
        u64::from_le_bytes([
            result[0], result[1], result[2], result[3],
            result[4], result[5], result[6], result[7],
        ])
    }

    /// Calculate deterministic seed based on block height (for synchronized proposer selection)
    /// Includes previous block hash for unpredictability
    fn calculate_height_seed(&self, height: u64) -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(b"sultan_proposer_");
        hasher.update(&height.to_le_bytes());
        hasher.update(&self.total_stake.to_le_bytes());
        hasher.update(&self.prev_block_hash);
        let result = hasher.finalize();
        
        u64::from_le_bytes([
            result[0], result[1], result[2], result[3],
            result[4], result[5], result[6], result[7],
        ])
    }

    /// Select proposer for a specific block height (synchronized across network)
    /// This ensures all validators agree on who should propose each block
    pub fn select_proposer_for_height(&self, height: u64) -> Option<String> {
        let mut active_validators: Vec<_> = self.validators
            .iter()
            .filter(|(_, v)| v.is_active && !v.is_jailed)
            .collect();

        if active_validators.is_empty() {
            warn!("No active validators");
            return None;
        }

        // Sort validators deterministically by address for consistent ordering
        active_validators.sort_by(|a, b| a.0.cmp(b.0));

        let seed = self.calculate_height_seed(height);
        let total_power: u64 = active_validators.iter().map(|(_, v)| v.voting_power).sum();
        
        if total_power == 0 {
            warn!("Total voting power is zero");
            return None;
        }

        let target = seed % total_power;
        let mut cumulative = 0u64;

        for (address, validator) in &active_validators {
            cumulative += validator.voting_power;
            if cumulative > target {
                debug!("Height {}: Proposer determined: {}", height, address);
                return Some((*address).clone());
            }
        }

        // Fallback - shouldn't happen with correct math
        Some(active_validators[0].0.clone())
    }

    /// Get ordered list of proposers for a height (primary + fallbacks)
    /// Returns validators in priority order: primary proposer first, then by stake
    /// Used for timeout-based fallback when primary proposer is offline
    pub fn get_proposer_order_for_height(&self, height: u64) -> Vec<String> {
        let mut active_validators: Vec<_> = self.validators
            .iter()
            .filter(|(_, v)| v.is_active && !v.is_jailed)
            .collect();

        if active_validators.is_empty() {
            return Vec::new();
        }

        // Sort by address for deterministic ordering
        active_validators.sort_by(|a, b| a.0.cmp(b.0));

        // Get the primary proposer
        let primary = self.select_proposer_for_height(height);
        
        // Build ordered list: primary first, then rest sorted by stake (descending)
        let mut result = Vec::new();
        
        if let Some(ref primary_addr) = primary {
            result.push(primary_addr.clone());
        }
        
        // Add remaining validators sorted by stake (highest first for faster fallback)
        let mut remaining: Vec<_> = active_validators
            .iter()
            .filter(|(addr, _)| primary.as_ref() != Some(*addr))
            .collect();
        remaining.sort_by(|a, b| b.1.voting_power.cmp(&a.1.voting_power));
        
        for (addr, _) in remaining {
            result.push((*addr).clone());
        }
        
        result
    }

    /// Check if a validator should produce as fallback
    /// slot_offset: 0 = primary, 1 = first fallback, etc.
    /// Returns true if this validator should produce at this offset
    pub fn is_fallback_proposer(&self, height: u64, validator_address: &str, slot_offset: usize) -> bool {
        let order = self.get_proposer_order_for_height(height);
        order.get(slot_offset).map(|s| s == validator_address).unwrap_or(false)
    }

    /// Get validator by address
    pub fn get_validator(&self, address: &str) -> Option<&Validator> {
        self.validators.get(address)
    }

    /// Get all active validators
    pub fn get_active_validators(&self) -> Vec<&Validator> {
        self.validators
            .values()
            .filter(|v| v.is_active)
            .collect()
    }

    /// Get validator count
    pub fn validator_count(&self) -> usize {
        self.validators.values().filter(|v| v.is_active).count()
    }

    /// Check if address is validator
    pub fn is_validator(&self, address: &str) -> bool {
        self.validators
            .get(address)
            .map(|v| v.is_active)
            .unwrap_or(false)
    }

    /// Calculate required signatures for consensus (2/3 + 1)
    pub fn required_signatures(&self) -> usize {
        let active_count = self.validator_count();
        if active_count == 0 {
            return 0;
        }
        (active_count * 2 / 3) + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 10,000 SLTN in base units (9 decimals)
    const MIN_STAKE: u64 = 10_000_000_000_000;
    
    // Test pubkey (zeros work since we disable verification in tests)
    const TEST_PUBKEY: [u8; 32] = [0u8; 32];

    /// Helper to create test consensus engine with verification disabled
    fn test_consensus() -> ConsensusEngine {
        ConsensusEngine::new_without_verification()
    }

    #[test]
    fn test_consensus_basic() {
        let mut consensus = test_consensus();
        
        // Use realistic stake amounts (10k, 20k, 15k SLTN in base units)
        assert!(consensus.add_validator("validator1".to_string(), MIN_STAKE, TEST_PUBKEY).is_ok());
        assert!(consensus.add_validator("validator2".to_string(), MIN_STAKE * 2, TEST_PUBKEY).is_ok());
        assert!(consensus.add_validator("validator3".to_string(), MIN_STAKE + MIN_STAKE / 2, TEST_PUBKEY).is_ok());
        
        assert_eq!(consensus.validator_count(), 3);
        assert_eq!(consensus.total_stake, MIN_STAKE + MIN_STAKE * 2 + MIN_STAKE + MIN_STAKE / 2);
        assert_eq!(consensus.required_signatures(), 3);
        
        let proposer = consensus.select_proposer();
        assert!(proposer.is_some());
    }

    #[test]
    fn test_min_stake() {
        let mut consensus = test_consensus();
        // Should reject stake below 10,000 SLTN minimum
        assert!(consensus.add_validator("low_stake".to_string(), MIN_STAKE - 1, TEST_PUBKEY).is_err());
        // Should accept stake at or above 10,000 SLTN
        assert!(consensus.add_validator("valid_stake".to_string(), MIN_STAKE, TEST_PUBKEY).is_ok());
    }

    #[test]
    fn test_duplicate_validator_rejected() {
        let mut consensus = test_consensus();
        // First add should succeed
        assert!(consensus.add_validator("validator1".to_string(), MIN_STAKE, TEST_PUBKEY).is_ok());
        assert_eq!(consensus.total_stake, MIN_STAKE);
        
        // Duplicate add should fail and not double-count stake
        assert!(consensus.add_validator("validator1".to_string(), MIN_STAKE, TEST_PUBKEY).is_err());
        assert_eq!(consensus.total_stake, MIN_STAKE); // Should still be MIN_STAKE, not 2x
        assert_eq!(consensus.validator_count(), 1);
    }

    #[test]
    fn test_proposer_selection_deterministic() {
        let mut consensus1 = test_consensus();
        let mut consensus2 = test_consensus();
        
        // Same validators, same state
        consensus1.add_validator("alice".to_string(), MIN_STAKE * 2, TEST_PUBKEY).unwrap();
        consensus1.add_validator("bob".to_string(), MIN_STAKE, TEST_PUBKEY).unwrap();
        consensus2.add_validator("alice".to_string(), MIN_STAKE * 2, TEST_PUBKEY).unwrap();
        consensus2.add_validator("bob".to_string(), MIN_STAKE, TEST_PUBKEY).unwrap();
        
        // Same height should select same proposer
        let proposer1 = consensus1.select_proposer_for_height(100);
        let proposer2 = consensus2.select_proposer_for_height(100);
        
        assert_eq!(proposer1, proposer2, "Proposer selection should be deterministic");
    }

    #[test]
    fn test_proposer_weighted_by_stake() {
        let mut consensus = test_consensus();
        
        // Add validators with different stakes
        consensus.add_validator("small".to_string(), MIN_STAKE, TEST_PUBKEY).unwrap();
        consensus.add_validator("large".to_string(), MIN_STAKE * 10, TEST_PUBKEY).unwrap();
        
        // Run many selections and count
        let mut small_count = 0;
        let mut large_count = 0;
        
        for height in 0..1000 {
            match consensus.select_proposer_for_height(height).as_deref() {
                Some("small") => small_count += 1,
                Some("large") => large_count += 1,
                _ => {}
            }
        }
        
        // Large should be selected ~10x more often (within reasonable variance)
        assert!(large_count > small_count * 5, 
            "Higher stake should be selected more often: large={}, small={}", large_count, small_count);
    }

    #[test]
    fn test_downtime_slashing() {
        let mut consensus = test_consensus();
        consensus.add_validator("validator1".to_string(), MIN_STAKE * 10, TEST_PUBKEY).unwrap();
        
        let initial_stake = MIN_STAKE * 10;
        
        // Record missed blocks up to threshold - use unique heights for deduplication
        for i in 0..(MAX_MISSED_BLOCKS_BEFORE_SLASH - 1) {
            let result = consensus.record_missed_block("validator1", 1000 + i).unwrap();
            assert!(result.is_none(), "Should not slash before threshold");
        }
        
        // This one should trigger slashing
        let slash_result = consensus.record_missed_block("validator1", 1000 + MAX_MISSED_BLOCKS_BEFORE_SLASH).unwrap();
        assert!(slash_result.is_some(), "Should slash at threshold");
        
        let slash_amount = slash_result.unwrap();
        let expected_slash = initial_stake * DOWNTIME_SLASH_PERCENT / SLASH_DENOMINATOR;
        assert_eq!(slash_amount, expected_slash, "Slash amount should be 0.1%");
        
        // Validator should be jailed
        let validator = consensus.get_validator("validator1").unwrap();
        assert!(validator.is_jailed, "Validator should be jailed");
        assert!(!validator.is_active, "Validator should be inactive");
        assert_eq!(validator.jail_until, 1000 + MAX_MISSED_BLOCKS_BEFORE_SLASH + JAIL_DURATION_BLOCKS);
    }

    #[test]
    fn test_double_sign_slashing() {
        let mut consensus = test_consensus();
        consensus.add_validator("validator1".to_string(), MIN_STAKE * 10, TEST_PUBKEY).unwrap();
        
        let initial_stake = MIN_STAKE * 10;
        
        // First vote
        let vote1 = BlockVote {
            block_hash: [1u8; 32],
            height: 100,
            validator_address: "validator1".to_string(),
            signature: vec![1, 2, 3],
        };
        let result1 = consensus.record_block_vote(&vote1, 100).unwrap();
        assert!(result1.is_none(), "First vote should not trigger slash");
        
        // Second vote for DIFFERENT block at same height (double-sign!)
        let vote2 = BlockVote {
            block_hash: [2u8; 32], // Different block!
            height: 100,
            validator_address: "validator1".to_string(),
            signature: vec![4, 5, 6],
        };
        let result2 = consensus.record_block_vote(&vote2, 100).unwrap();
        assert!(result2.is_some(), "Double-sign should trigger slash");
        
        let slash_amount = result2.unwrap();
        let expected_slash = initial_stake * DOUBLE_SIGN_SLASH_PERCENT / SLASH_DENOMINATOR;
        assert_eq!(slash_amount, expected_slash, "Slash amount should be 5%");
        
        // Validator should be jailed
        let validator = consensus.get_validator("validator1").unwrap();
        assert!(validator.is_jailed);
    }

    #[test]
    fn test_unjail_validator() {
        let mut consensus = test_consensus();
        consensus.add_validator("validator1".to_string(), MIN_STAKE * 10, TEST_PUBKEY).unwrap();
        
        // Slash to jail - use unique heights
        for i in 0..(MAX_MISSED_BLOCKS_BEFORE_SLASH as u64) {
            consensus.record_missed_block("validator1", 1000 + i).ok();
        }
        
        let validator = consensus.get_validator("validator1").unwrap();
        assert!(validator.is_jailed);
        let jail_until = validator.jail_until;
        
        // Try to unjail too early
        assert!(consensus.unjail_validator("validator1", jail_until - 1).is_err());
        
        // Unjail after jail period
        assert!(consensus.unjail_validator("validator1", jail_until).is_ok());
        
        let validator = consensus.get_validator("validator1").unwrap();
        assert!(!validator.is_jailed);
        assert!(validator.is_active);
    }

    #[test]
    fn test_signature_collection_consensus() {
        let mut consensus = test_consensus();
        
        // Add 4 validators with equal stake
        consensus.add_validator("v1".to_string(), MIN_STAKE, TEST_PUBKEY).unwrap();
        consensus.add_validator("v2".to_string(), MIN_STAKE, TEST_PUBKEY).unwrap();
        consensus.add_validator("v3".to_string(), MIN_STAKE, TEST_PUBKEY).unwrap();
        consensus.add_validator("v4".to_string(), MIN_STAKE, TEST_PUBKEY).unwrap();
        
        let block_hash = [42u8; 32];
        let height = 100;
        
        // Need 2/3 + 1 = 3 signatures for consensus
        let has_consensus = consensus.collect_signature(height, block_hash, "v1", vec![1]).unwrap();
        assert!(!has_consensus, "1/4 should not have consensus");
        
        let has_consensus = consensus.collect_signature(height, block_hash, "v2", vec![2]).unwrap();
        assert!(!has_consensus, "2/4 should not have consensus");
        
        let has_consensus = consensus.collect_signature(height, block_hash, "v3", vec![3]).unwrap();
        assert!(has_consensus, "3/4 should have consensus (2/3 + 1)");
    }

    #[test]
    fn test_required_voting_power() {
        let mut consensus = test_consensus();
        
        // 3 validators with 100, 200, 300 stake
        consensus.add_validator("v1".to_string(), MIN_STAKE, TEST_PUBKEY).unwrap();
        consensus.add_validator("v2".to_string(), MIN_STAKE * 2, TEST_PUBKEY).unwrap();
        consensus.add_validator("v3".to_string(), MIN_STAKE * 3, TEST_PUBKEY).unwrap();
        
        // Total = 6 * MIN_STAKE, required = (6 * 2/3) + 1 = 5 units
        let required = consensus.required_voting_power();
        let total = MIN_STAKE * 6;
        let expected = (total * 2 / 3) + 1;
        assert_eq!(required, expected);
    }

    #[test]
    fn test_validator_jailed_excluded_from_selection() {
        let mut consensus = test_consensus();
        
        consensus.add_validator("good".to_string(), MIN_STAKE, TEST_PUBKEY).unwrap();
        consensus.add_validator("bad".to_string(), MIN_STAKE * 100, TEST_PUBKEY).unwrap(); // Much higher stake
        
        // Jail the high-stake validator by triggering downtime slashing
        for i in 0..(MAX_MISSED_BLOCKS_BEFORE_SLASH as u64) {
            consensus.record_missed_block("bad", 1000 + i).ok();
        }
        
        // Now only "good" should be selected (even though "bad" has higher stake)
        for height in 0..100 {
            let proposer = consensus.select_proposer_for_height(height);
            assert_eq!(proposer.as_deref(), Some("good"), 
                "Jailed validator should not be selected");
        }
    }

    #[test]
    fn test_record_signature_resets_missed_blocks() {
        let mut consensus = test_consensus();
        consensus.add_validator("validator1".to_string(), MIN_STAKE, TEST_PUBKEY).unwrap();
        
        // Accumulate some missed blocks (but not enough to slash) - use unique heights
        for i in 0..50u64 {
            consensus.record_missed_block("validator1", 100 + i).ok();
        }
        
        let validator = consensus.get_validator("validator1").unwrap();
        assert_eq!(validator.missed_blocks, 50);
        
        // Sign a block - should reset counter
        consensus.record_signature("validator1").unwrap();
        
        let validator = consensus.get_validator("validator1").unwrap();
        assert_eq!(validator.missed_blocks, 0, "Signing should reset missed blocks");
    }

    #[test]
    fn test_cleanup_old_signatures() {
        let mut consensus = test_consensus();
        consensus.add_validator("v1".to_string(), MIN_STAKE, TEST_PUBKEY).unwrap();
        
        // Add signatures at various heights (0-99 inclusive = 100 entries)
        for height in 0..100 {
            consensus.collect_signature(height, [0u8; 32], "v1", vec![]).ok();
        }
        
        assert_eq!(consensus.pending_signatures.len(), 100);
        
        // Cleanup keeping last 10 blocks: cutoff = 100 - 10 = 90
        // Retain heights > 90, so heights 91-99 = 9 entries
        consensus.cleanup_old_signatures(100, 10);
        
        assert_eq!(consensus.pending_signatures.len(), 9);
        assert!(consensus.pending_signatures.contains_key(&91));
        assert!(consensus.pending_signatures.contains_key(&99));
        assert!(!consensus.pending_signatures.contains_key(&90));
    }

    #[test]
    fn test_slashing_evidence_recorded() {
        let mut consensus = test_consensus();
        consensus.add_validator("validator1".to_string(), MIN_STAKE * 10, TEST_PUBKEY).unwrap();
        
        // Trigger downtime slashing - use unique heights
        for i in 0..(MAX_MISSED_BLOCKS_BEFORE_SLASH as u64) {
            consensus.record_missed_block("validator1", 1000 + i).ok();
        }
        
        // Check evidence was recorded
        let evidence = consensus.get_slashing_evidence();
        assert_eq!(evidence.len(), 1, "Should have one slashing evidence record");
        assert_eq!(evidence[0].offense_type, SlashingOffense::Downtime);
        assert_eq!(evidence[0].validator_address, "validator1");
        assert_eq!(evidence[0].missed_blocks, Some(MAX_MISSED_BLOCKS_BEFORE_SLASH));
    }

    #[test]
    fn test_double_sign_evidence_includes_hashes() {
        let mut consensus = test_consensus();
        consensus.add_validator("validator1".to_string(), MIN_STAKE * 10, TEST_PUBKEY).unwrap();
        
        // First vote
        let vote1 = BlockVote {
            block_hash: [1u8; 32],
            height: 100,
            validator_address: "validator1".to_string(),
            signature: vec![1, 2, 3],
        };
        consensus.record_block_vote(&vote1, 100).unwrap();
        
        // Double-sign
        let vote2 = BlockVote {
            block_hash: [2u8; 32],
            height: 100,
            validator_address: "validator1".to_string(),
            signature: vec![4, 5, 6],
        };
        consensus.record_block_vote(&vote2, 100).unwrap();
        
        // Check evidence includes both block hashes
        let evidence = consensus.get_slashing_evidence();
        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].offense_type, SlashingOffense::DoubleSign);
        
        let (hash1, hash2) = evidence[0].evidence_hashes.unwrap();
        assert_eq!(hash1, [1u8; 32]);
        assert_eq!(hash2, [2u8; 32]);
    }

    #[test]
    fn test_state_export_import() {
        let mut consensus = test_consensus();
        consensus.add_validator("v1".to_string(), MIN_STAKE, TEST_PUBKEY).unwrap();
        consensus.add_validator("v2".to_string(), MIN_STAKE * 2, TEST_PUBKEY).unwrap();
        
        // Make some state changes
        consensus.select_proposer();
        consensus.set_prev_block_hash([42u8; 32]);
        
        // Export state
        let snapshot = consensus.export_state();
        
        // Create new engine and import
        let mut new_consensus = test_consensus();
        new_consensus.import_state(snapshot);
        
        // Verify state was restored
        assert_eq!(new_consensus.validators.len(), 2);
        assert_eq!(new_consensus.total_stake, MIN_STAKE * 3);
        assert_eq!(new_consensus.prev_block_hash, [42u8; 32]);
        assert!(new_consensus.get_validator("v1").is_some());
        assert!(new_consensus.get_validator("v2").is_some());
    }

    #[test]
    fn test_ed25519_verification_rejects_invalid() {
        use ed25519_dalek::{SigningKey, Signer};
        
        // Create a real keypair
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let verifying_key = signing_key.verifying_key();
        let pubkey: [u8; 32] = verifying_key.to_bytes();
        
        let mut consensus = ConsensusEngine::new(); // Verification enabled
        consensus.add_validator("v1".to_string(), MIN_STAKE, pubkey).unwrap();
        
        let block_hash = [42u8; 32];
        
        // Sign the block hash
        let signature = signing_key.sign(&block_hash);
        let sig_bytes = signature.to_bytes().to_vec();
        
        // Valid signature should work
        let result = consensus.collect_signature(100, block_hash, "v1", sig_bytes.clone());
        assert!(result.is_ok(), "Valid signature should be accepted");
        
        // Invalid signature should fail
        let bad_sig = vec![0u8; 64];
        let mut consensus2 = ConsensusEngine::new();
        consensus2.add_validator("v2".to_string(), MIN_STAKE, pubkey).unwrap();
        let result = consensus2.collect_signature(100, block_hash, "v2", bad_sig);
        assert!(result.is_err(), "Invalid signature should be rejected");
    }

    // ============ ENTERPRISE-GRADE PROPOSER SELECTION TESTS ============

    #[test]
    fn test_proposer_order_primary_first() {
        let mut consensus = test_consensus();
        
        // Add validators with different stakes
        consensus.add_validator("alice".to_string(), MIN_STAKE * 5, TEST_PUBKEY).unwrap();
        consensus.add_validator("bob".to_string(), MIN_STAKE * 10, TEST_PUBKEY).unwrap();
        consensus.add_validator("charlie".to_string(), MIN_STAKE * 3, TEST_PUBKEY).unwrap();
        
        let height = 100;
        let order = consensus.get_proposer_order_for_height(height);
        
        // First in order should be the primary proposer
        let primary = consensus.select_proposer_for_height(height);
        assert_eq!(order[0], primary.unwrap(), "First position should be primary proposer");
        
        // Order should contain all validators
        assert_eq!(order.len(), 3);
    }

    #[test]
    fn test_proposer_order_fallbacks_by_stake() {
        let mut consensus = test_consensus();
        
        // Add validators with different stakes (names sorted alphabetically for determinism)
        consensus.add_validator("a_low".to_string(), MIN_STAKE, TEST_PUBKEY).unwrap();
        consensus.add_validator("b_mid".to_string(), MIN_STAKE * 5, TEST_PUBKEY).unwrap();
        consensus.add_validator("c_high".to_string(), MIN_STAKE * 10, TEST_PUBKEY).unwrap();
        
        // Find a height where "a_low" is primary (might need to search)
        let mut found_height = None;
        for height in 0..1000 {
            if consensus.select_proposer_for_height(height) == Some("a_low".to_string()) {
                found_height = Some(height);
                break;
            }
        }
        
        if let Some(height) = found_height {
            let order = consensus.get_proposer_order_for_height(height);
            assert_eq!(order[0], "a_low", "Primary should be first");
            // Remaining should be sorted by stake descending
            assert_eq!(order[1], "c_high", "Highest stake fallback should be second");
            assert_eq!(order[2], "b_mid", "Lower stake fallback should be third");
        }
    }

    #[test]
    fn test_proposer_order_excludes_jailed() {
        let mut consensus = test_consensus();
        
        consensus.add_validator("good".to_string(), MIN_STAKE, TEST_PUBKEY).unwrap();
        consensus.add_validator("bad".to_string(), MIN_STAKE * 100, TEST_PUBKEY).unwrap();
        
        // Jail the high-stake validator
        for i in 0..(MAX_MISSED_BLOCKS_BEFORE_SLASH as u64) {
            consensus.record_missed_block("bad", 1000 + i).ok();
        }
        
        // Verify "bad" is jailed
        assert!(consensus.get_validator("bad").unwrap().is_jailed);
        
        // Check proposer order only includes "good"
        let order = consensus.get_proposer_order_for_height(100);
        assert_eq!(order.len(), 1);
        assert_eq!(order[0], "good");
    }

    #[test]
    fn test_missed_block_deduplication() {
        let mut consensus = test_consensus();
        consensus.add_validator("validator1".to_string(), MIN_STAKE * 10, TEST_PUBKEY).unwrap();
        
        // Record missed block at height 100
        let result1 = consensus.record_missed_block("validator1", 100).unwrap();
        assert!(result1.is_none()); // Not slashed yet
        
        // Try to record SAME height again - should be deduplicated
        let result2 = consensus.record_missed_block("validator1", 100).unwrap();
        assert!(result2.is_none());
        
        // Missed blocks should only be 1, not 2
        let validator = consensus.get_validator("validator1").unwrap();
        assert_eq!(validator.missed_blocks, 1, "Duplicate should not be counted");
        
        // Different height should work
        let result3 = consensus.record_missed_block("validator1", 101).unwrap();
        assert!(result3.is_none());
        
        let validator = consensus.get_validator("validator1").unwrap();
        assert_eq!(validator.missed_blocks, 2, "Different height should be counted");
    }

    #[test]
    fn test_missed_block_cleanup() {
        let mut consensus = test_consensus();
        consensus.add_validator("validator1".to_string(), MIN_STAKE * 10, TEST_PUBKEY).unwrap();
        
        // Record missed blocks at various heights
        for h in 100..200u64 {
            consensus.record_missed_block("validator1", h).ok();
        }
        
        assert_eq!(consensus.recorded_misses.len(), 100);
        
        // Cleanup at height 2000 (window is 1000)
        consensus.cleanup_recorded_misses(2000);
        
        // Heights 100-199 should be cleaned up (all < 2000 - 1000 = 1000)
        assert_eq!(consensus.recorded_misses.len(), 0, "Old misses should be cleaned up");
    }

    #[test]
    fn test_is_fallback_proposer() {
        let mut consensus = test_consensus();
        
        consensus.add_validator("alice".to_string(), MIN_STAKE * 10, TEST_PUBKEY).unwrap();
        consensus.add_validator("bob".to_string(), MIN_STAKE * 5, TEST_PUBKEY).unwrap();
        consensus.add_validator("charlie".to_string(), MIN_STAKE * 3, TEST_PUBKEY).unwrap();
        
        let height = 100;
        let order = consensus.get_proposer_order_for_height(height);
        
        // Test is_fallback_proposer matches the order
        for (i, addr) in order.iter().enumerate() {
            assert!(consensus.is_fallback_proposer(height, addr, i), 
                    "{} should be at position {}", addr, i);
        }
        
        // Wrong position should return false
        assert!(!consensus.is_fallback_proposer(height, &order[0], 1));
    }

    #[test]
    fn test_fallback_threshold_constant() {
        // Ensure the constant is reasonable
        assert!(FALLBACK_THRESHOLD_MISSED_BLOCKS >= 3, 
                "Threshold too low - could cause unnecessary fallbacks");
        assert!(FALLBACK_THRESHOLD_MISSED_BLOCKS <= 20, 
                "Threshold too high - could cause long downtime");
    }

    #[test]
    fn test_max_fallback_positions_constant() {
        assert!(MAX_FALLBACK_POSITIONS >= 2, 
                "Need at least 2 fallback positions for resilience");
        assert!(MAX_FALLBACK_POSITIONS <= 5, 
                "Too many fallbacks could cause competing block production");
    }

    #[test]
    fn test_slashing_atomicity() {
        let mut consensus = test_consensus();
        let initial_stake = MIN_STAKE * 100;
        consensus.add_validator("validator1".to_string(), initial_stake, TEST_PUBKEY).unwrap();
        
        let initial_total = consensus.total_stake;
        
        // Trigger slashing
        for i in 0..(MAX_MISSED_BLOCKS_BEFORE_SLASH as u64) {
            consensus.record_missed_block("validator1", 1000 + i).ok();
        }
        
        let validator = consensus.get_validator("validator1").unwrap();
        
        // Verify all state was updated atomically:
        // 1. Validator is jailed
        assert!(validator.is_jailed);
        // 2. Validator is inactive
        assert!(!validator.is_active);
        // 3. Stake was reduced
        let expected_slash = initial_stake * DOWNTIME_SLASH_PERCENT / SLASH_DENOMINATOR;
        assert_eq!(validator.stake, initial_stake - expected_slash);
        // 4. Total slashed is tracked
        assert_eq!(validator.total_slashed, expected_slash);
        // 5. Total stake is reduced (both slash AND remaining stake removed since jailed)
        assert_eq!(consensus.total_stake, 0, "Jailed validator stake should be removed from total");
        // 6. Missed blocks reset
        assert_eq!(validator.missed_blocks, 0);
        // 7. Evidence recorded
        assert_eq!(consensus.slashing_evidence.len(), 1);
    }
}
