//! Sultan Consensus Engine
//!
//! Production-grade consensus implementation with validator management,
//! proposer selection, and block validation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Result, bail};
use tracing::{info, warn, debug};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Validator {
    pub address: String,
    pub stake: u64,
    pub voting_power: u64,
    pub is_active: bool,
    pub blocks_proposed: u64,
    pub blocks_signed: u64,
}

pub struct ConsensusEngine {
    pub validators: HashMap<String, Validator>,
    pub current_proposer: Option<String>,
    pub round: u64,
    pub min_stake: u64,
    pub total_stake: u64,
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
        }
    }

    /// Add validator to active set
    pub fn add_validator(&mut self, address: String, stake: u64) -> Result<()> {
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
            stake,
            voting_power,
            is_active: true,
            blocks_proposed: 0,
            blocks_signed: 0,
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

    /// Record block signature by validator
    pub fn record_signature(&mut self, validator_address: &str) -> Result<()> {
        if let Some(validator) = self.validators.get_mut(validator_address) {
            validator.blocks_signed += 1;
            Ok(())
        } else {
            bail!("Validator {} not found", validator_address);
        }
    }

    /// Calculate voting power based on stake (linear for now)
    fn calculate_voting_power(&self, stake: u64) -> u64 {
        // Linear relationship: voting power = stake
        // Can be made non-linear for more sophisticated economics
        stake
    }

    /// Calculate deterministic seed for proposer selection
    fn calculate_selection_seed(&self) -> u64 {
        let data = format!("{}{}", self.round, self.total_stake);
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let result = hasher.finalize();
        
        // Use first 8 bytes as u64
        u64::from_le_bytes([
            result[0], result[1], result[2], result[3],
            result[4], result[5], result[6], result[7],
        ])
    }

    /// Calculate deterministic seed based on block height (for synchronized proposer selection)
    fn calculate_height_seed(&self, height: u64) -> u64 {
        let data = format!("sultan_proposer_{}_{}", height, self.total_stake);
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
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
            .filter(|(_, v)| v.is_active)
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

    #[test]
    fn test_consensus_basic() {
        let mut consensus = ConsensusEngine::new();
        
        // Use realistic stake amounts (10k, 20k, 15k SLTN in base units)
        assert!(consensus.add_validator("validator1".to_string(), MIN_STAKE).is_ok());
        assert!(consensus.add_validator("validator2".to_string(), MIN_STAKE * 2).is_ok());
        assert!(consensus.add_validator("validator3".to_string(), MIN_STAKE + MIN_STAKE / 2).is_ok());
        
        assert_eq!(consensus.validator_count(), 3);
        assert_eq!(consensus.total_stake, MIN_STAKE + MIN_STAKE * 2 + MIN_STAKE + MIN_STAKE / 2);
        assert_eq!(consensus.required_signatures(), 3);
        
        let proposer = consensus.select_proposer();
        assert!(proposer.is_some());
    }

    #[test]
    fn test_min_stake() {
        let mut consensus = ConsensusEngine::new();
        // Should reject stake below 10,000 SLTN minimum
        assert!(consensus.add_validator("low_stake".to_string(), MIN_STAKE - 1).is_err());
        // Should accept stake at or above 10,000 SLTN
        assert!(consensus.add_validator("valid_stake".to_string(), MIN_STAKE).is_ok());
    }

    #[test]
    fn test_duplicate_validator_rejected() {
        let mut consensus = ConsensusEngine::new();
        // First add should succeed
        assert!(consensus.add_validator("validator1".to_string(), MIN_STAKE).is_ok());
        assert_eq!(consensus.total_stake, MIN_STAKE);
        
        // Duplicate add should fail and not double-count stake
        assert!(consensus.add_validator("validator1".to_string(), MIN_STAKE).is_err());
        assert_eq!(consensus.total_stake, MIN_STAKE); // Should still be MIN_STAKE, not 2x
        assert_eq!(consensus.validator_count(), 1);
    }
}
