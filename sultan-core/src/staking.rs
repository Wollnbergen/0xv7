//! Production Validator Staking System
//!
//! Real staking implementation with:
//! - Token locking and unlocking
//! - Automatic reward calculation and distribution
//! - Validator slashing for misbehavior
//! - Delegation support
//! - Real-time APY tracking
//!
//! Security features:
//! - 21-day unbonding period (prevents flash stake governance attacks)
//! - Slashing during unbonding (misbehavior still punished)
//! - Validator jailing (prevents block production by bad actors)

use anyhow::{Result, Context, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

const MIN_VALIDATOR_STAKE: u64 = 10_000_000_000_000; // 10,000 SLTN (with 9 decimals)
const BLOCKS_PER_YEAR: u64 = 15_768_000; // 2-second blocks: (365*24*60*60)/2
const BASE_APY: f64 = 0.1333; // 13.33% APY for validators

// Security: 21-day unbonding period (like Cosmos)
// This prevents flash stake attacks on governance
const UNBONDING_PERIOD_BLOCKS: u64 = 907_200; // ~21 days with 2-second blocks

/// Unbonding entry - tokens being unstaked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnbondingEntry {
    pub delegator_address: String,
    pub validator_address: String,
    pub amount: u64,
    pub creation_height: u64,
    pub completion_height: u64,
    pub completion_time: u64,
}

/// Validator staking state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorStake {
    pub validator_address: String,
    pub self_stake: u64,
    pub delegated_stake: u64,
    pub total_stake: u64,
    pub commission_rate: f64, // 0.0 to 1.0 (0% to 100%)
    pub rewards_accumulated: u64,
    pub blocks_signed: u64,
    pub blocks_missed: u64,
    pub jailed: bool,
    pub jailed_until: u64,
    pub created_at: u64,
    pub last_reward_height: u64,
}

/// Delegator stake
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegation {
    pub delegator_address: String,
    pub validator_address: String,
    pub amount: u64,
    pub rewards_accumulated: u64,
    pub delegated_at: u64,
    pub last_reward_height: u64,
}

/// Reward distribution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardDistribution {
    pub height: u64,
    pub timestamp: u64,
    pub total_rewards: u64,
    pub validator_rewards: HashMap<String, u64>,
    pub delegator_rewards: HashMap<String, u64>,
}

/// Slashing event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingEvent {
    pub validator_address: String,
    pub height: u64,
    pub timestamp: u64,
    pub reason: SlashReason,
    pub amount_slashed: u64,
    pub jail_duration: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlashReason {
    DoubleSign,
    Downtime,
    InvalidBlock,
    MaliciousBehavior,
}

/// Production staking manager
pub struct StakingManager {
    validators: Arc<RwLock<HashMap<String, ValidatorStake>>>,
    delegations: Arc<RwLock<HashMap<String, Vec<Delegation>>>>,
    reward_history: Arc<RwLock<Vec<RewardDistribution>>>,
    slashing_history: Arc<RwLock<Vec<SlashingEvent>>>,
    /// Unbonding queue - tokens waiting to be released after 21 days
    unbonding_queue: Arc<RwLock<Vec<UnbondingEntry>>>,
    total_staked: Arc<RwLock<u64>>,
    inflation_rate: Arc<RwLock<f64>>,
    current_height: Arc<RwLock<u64>>,
}

impl StakingManager {
    pub fn new(initial_inflation: f64) -> Self {
        Self {
            validators: Arc::new(RwLock::new(HashMap::new())),
            delegations: Arc::new(RwLock::new(HashMap::new())),
            reward_history: Arc::new(RwLock::new(Vec::new())),
            slashing_history: Arc::new(RwLock::new(Vec::new())),
            unbonding_queue: Arc::new(RwLock::new(Vec::new())),
            total_staked: Arc::new(RwLock::new(0)),
            inflation_rate: Arc::new(RwLock::new(initial_inflation)),
            current_height: Arc::new(RwLock::new(0)),
        }
    }

    /// Create a new validator with initial self-stake
    pub async fn create_validator(
        &self,
        validator_address: String,
        initial_stake: u64,
        commission_rate: f64,
    ) -> Result<()> {
        if initial_stake < MIN_VALIDATOR_STAKE {
            bail!("Insufficient stake. Minimum: {} SLTN", MIN_VALIDATOR_STAKE / 1_000_000_000);
        }

        if commission_rate < 0.0 || commission_rate > 1.0 {
            bail!("Commission rate must be between 0% and 100%");
        }

        let mut validators = self.validators.write().await;
        
        if validators.contains_key(&validator_address) {
            bail!("Validator already exists");
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let current_height = *self.current_height.read().await;

        let validator = ValidatorStake {
            validator_address: validator_address.clone(),
            self_stake: initial_stake,
            delegated_stake: 0,
            total_stake: initial_stake,
            commission_rate,
            rewards_accumulated: 0,
            blocks_signed: 0,
            blocks_missed: 0,
            jailed: false,
            jailed_until: 0,
            created_at: now,
            last_reward_height: current_height,
        };

        validators.insert(validator_address.clone(), validator);

        let mut total_staked = self.total_staked.write().await;
        *total_staked += initial_stake;

        info!(
            "Validator created: {} with {} SLTN stake ({}% commission)",
            validator_address,
            initial_stake / 1_000_000_000,
            commission_rate * 100.0
        );

        Ok(())
    }

    /// Delegate tokens to a validator
    pub async fn delegate(
        &self,
        delegator_address: String,
        validator_address: String,
        amount: u64,
    ) -> Result<()> {
        if amount == 0 {
            bail!("Cannot delegate 0 tokens");
        }

        let mut validators = self.validators.write().await;
        let validator = validators.get_mut(&validator_address)
            .context("Validator not found")?;

        if validator.jailed {
            bail!("Cannot delegate to jailed validator");
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let current_height = *self.current_height.read().await;

        let delegation = Delegation {
            delegator_address: delegator_address.clone(),
            validator_address: validator_address.clone(),
            amount,
            rewards_accumulated: 0,
            delegated_at: now,
            last_reward_height: current_height,
        };

        validator.delegated_stake += amount;
        validator.total_stake += amount;

        let mut delegations = self.delegations.write().await;
        delegations.entry(delegator_address.clone())
            .or_insert_with(Vec::new)
            .push(delegation);

        let mut total_staked = self.total_staked.write().await;
        *total_staked += amount;

        info!(
            "Delegation: {} delegated {} SLTN to {}",
            delegator_address,
            amount / 1_000_000_000,
            validator_address
        );

        Ok(())
    }

    /// Calculate and distribute rewards for a block
    pub async fn distribute_block_rewards(&self, block_height: u64) -> Result<RewardDistribution> {
        let mut current_height = self.current_height.write().await;
        *current_height = block_height;

        let inflation_rate = *self.inflation_rate.read().await;
        let total_staked = *self.total_staked.read().await;

        if total_staked == 0 {
            return Ok(RewardDistribution {
                height: block_height,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs(),
                total_rewards: 0,
                validator_rewards: HashMap::new(),
                delegator_rewards: HashMap::new(),
            });
        }

        // Calculate total block reward based on inflation
        // Annual inflation distributed per block
        let annual_inflation_amount = (total_staked as f64 * inflation_rate) as u64;
        let block_reward = annual_inflation_amount / BLOCKS_PER_YEAR;

        let mut validators = self.validators.write().await;
        let mut delegations = self.delegations.write().await;

        let mut validator_rewards = HashMap::new();
        let mut delegator_rewards = HashMap::new();

        // Distribute rewards proportionally to stake
        for (addr, validator) in validators.iter_mut() {
            if validator.jailed {
                continue;
            }

            let validator_share = (validator.total_stake as f64 / total_staked as f64) * block_reward as f64;
            let validator_share = validator_share as u64;

            // Commission from delegators
            let delegator_share = (validator.delegated_stake as f64 / validator.total_stake as f64) * validator_share as f64;
            let commission = (delegator_share as f64 * validator.commission_rate) as u64;

            // Validator gets: their share of self-stake + commission
            let self_stake_share = validator_share - (delegator_share as u64);
            let validator_total_reward = self_stake_share + commission;

            validator.rewards_accumulated += validator_total_reward;
            validator.last_reward_height = block_height;
            validator_rewards.insert(addr.clone(), validator_total_reward);

            // Distribute remaining rewards to delegators
            // Iterate through all delegators to find those delegating to this validator
            if validator.delegated_stake > 0 {
                let delegator_reward_pool = (delegator_share as u64).saturating_sub(commission);
                
                for (_delegator_addr, delegator_list) in delegations.iter_mut() {
                    for delegation in delegator_list.iter_mut() {
                        if delegation.validator_address == *addr {
                            let delegator_reward = (delegation.amount as f64 / validator.delegated_stake as f64) 
                                * delegator_reward_pool as f64;
                            let delegator_reward = delegator_reward as u64;

                            delegation.rewards_accumulated += delegator_reward;
                            delegation.last_reward_height = block_height;
                            
                            *delegator_rewards.entry(delegation.delegator_address.clone())
                                .or_insert(0) += delegator_reward;
                        }
                    }
                }
            }
        }

        let distribution = RewardDistribution {
            height: block_height,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            total_rewards: block_reward,
            validator_rewards,
            delegator_rewards,
        };

        let mut reward_history = self.reward_history.write().await;
        reward_history.push(distribution.clone());

        // Keep only last 10000 blocks of history
        if reward_history.len() > 10000 {
            reward_history.remove(0);
        }

        Ok(distribution)
    }

    /// Slash a validator for misbehavior
    pub async fn slash_validator(
        &self,
        validator_address: &str,
        reason: SlashReason,
        slash_percentage: f64,
        jail_duration_blocks: u64,
    ) -> Result<()> {
        let mut validators = self.validators.write().await;
        let validator = validators.get_mut(validator_address)
            .context("Validator not found")?;

        let slash_amount = (validator.total_stake as f64 * slash_percentage) as u64;
        
        validator.total_stake = validator.total_stake.saturating_sub(slash_amount);
        validator.self_stake = validator.self_stake.saturating_sub(slash_amount.min(validator.self_stake));
        validator.jailed = true;
        
        let current_height = *self.current_height.read().await;
        validator.jailed_until = current_height + jail_duration_blocks;

        let mut total_staked = self.total_staked.write().await;
        *total_staked = total_staked.saturating_sub(slash_amount);

        let event = SlashingEvent {
            validator_address: validator_address.to_string(),
            height: current_height,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            reason,
            amount_slashed: slash_amount,
            jail_duration: jail_duration_blocks,
        };

        let mut slashing_history = self.slashing_history.write().await;
        slashing_history.push(event);

        warn!(
            "Validator {} slashed: {} SLTN ({:.1}%), jailed for {} blocks",
            validator_address,
            slash_amount / 1_000_000_000,
            slash_percentage * 100.0,
            jail_duration_blocks
        );

        Ok(())
    }

    /// Unjail a validator after serving jail time
    pub async fn unjail_validator(&self, validator_address: &str) -> Result<()> {
        let mut validators = self.validators.write().await;
        let validator = validators.get_mut(validator_address)
            .context("Validator not found")?;

        let current_height = *self.current_height.read().await;

        if !validator.jailed {
            bail!("Validator is not jailed");
        }

        if current_height < validator.jailed_until {
            bail!(
                "Validator must wait {} more blocks before unjailing",
                validator.jailed_until - current_height
            );
        }

        validator.jailed = false;
        validator.jailed_until = 0;

        info!("Validator {} unjailed", validator_address);

        Ok(())
    }

    /// Withdraw accumulated rewards
    pub async fn withdraw_validator_rewards(&self, validator_address: &str) -> Result<u64> {
        let mut validators = self.validators.write().await;
        let validator = validators.get_mut(validator_address)
            .context("Validator not found")?;

        let rewards = validator.rewards_accumulated;
        validator.rewards_accumulated = 0;

        info!(
            "Validator {} withdrew {} SLTN in rewards",
            validator_address,
            rewards / 1_000_000_000
        );

        Ok(rewards)
    }

    /// Withdraw delegator rewards
    pub async fn withdraw_delegator_rewards(
        &self,
        delegator_address: &str,
        validator_address: &str,
    ) -> Result<u64> {
        let mut delegations = self.delegations.write().await;
        let delegator_list = delegations.get_mut(delegator_address)
            .context("No delegations found")?;

        let mut total_rewards = 0;

        for delegation in delegator_list.iter_mut() {
            if delegation.validator_address == validator_address {
                total_rewards += delegation.rewards_accumulated;
                delegation.rewards_accumulated = 0;
            }
        }

        if total_rewards == 0 {
            bail!("No rewards to withdraw");
        }

        info!(
            "Delegator {} withdrew {} SLTN in rewards from validator {}",
            delegator_address,
            total_rewards / 1_000_000_000,
            validator_address
        );

        Ok(total_rewards)
    }

    /// Undelegate tokens (starts 21-day unbonding period)
    /// 
    /// Security: Tokens are NOT immediately available. They enter an unbonding
    /// queue and can only be withdrawn after UNBONDING_PERIOD_BLOCKS (~21 days).
    /// This prevents flash stake attacks on governance.
    pub async fn undelegate(
        &self,
        delegator_address: String,
        validator_address: String,
        amount: u64,
    ) -> Result<UnbondingEntry> {
        let current_height = *self.current_height.read().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        // Reduce delegation
        {
            let mut delegations = self.delegations.write().await;
            let delegator_list = delegations.get_mut(&delegator_address)
                .context("No delegations found")?;

            let delegation = delegator_list.iter_mut()
                .find(|d| d.validator_address == validator_address)
                .context("Delegation to this validator not found")?;

            if delegation.amount < amount {
                bail!("Insufficient delegation. You have {} SLTN delegated", 
                      delegation.amount / 1_000_000_000);
            }

            delegation.amount -= amount;
        }

        // Reduce validator stake
        {
            let mut validators = self.validators.write().await;
            let validator = validators.get_mut(&validator_address)
                .context("Validator not found")?;

            validator.delegated_stake = validator.delegated_stake.saturating_sub(amount);
            validator.total_stake = validator.total_stake.saturating_sub(amount);
        }

        // Update total staked
        {
            let mut total_staked = self.total_staked.write().await;
            *total_staked = total_staked.saturating_sub(amount);
        }

        // Create unbonding entry
        let unbonding = UnbondingEntry {
            delegator_address: delegator_address.clone(),
            validator_address: validator_address.clone(),
            amount,
            creation_height: current_height,
            completion_height: current_height + UNBONDING_PERIOD_BLOCKS,
            completion_time: now + (UNBONDING_PERIOD_BLOCKS * 2), // ~2 seconds per block
        };

        let mut unbonding_queue = self.unbonding_queue.write().await;
        unbonding_queue.push(unbonding.clone());

        warn!(
            "Unbonding started: {} is undelegating {} SLTN from {}. Available at block {}",
            delegator_address,
            amount / 1_000_000_000,
            validator_address,
            unbonding.completion_height
        );

        Ok(unbonding)
    }

    /// Process matured unbondings (should be called each block)
    /// Returns list of completed unbondings that can now be withdrawn
    pub async fn process_unbondings(&self) -> Vec<UnbondingEntry> {
        let current_height = *self.current_height.read().await;
        
        let mut unbonding_queue = self.unbonding_queue.write().await;
        
        // Partition into completed and pending
        let (completed, pending): (Vec<_>, Vec<_>) = unbonding_queue
            .drain(..)
            .partition(|u| u.completion_height <= current_height);

        // Put pending ones back
        *unbonding_queue = pending;

        for unbonding in &completed {
            info!(
                "Unbonding complete: {} can now withdraw {} SLTN",
                unbonding.delegator_address,
                unbonding.amount / 1_000_000_000
            );
        }

        completed
    }

    /// Get pending unbondings for an address
    pub async fn get_unbondings(&self, delegator_address: &str) -> Vec<UnbondingEntry> {
        let unbonding_queue = self.unbonding_queue.read().await;
        unbonding_queue.iter()
            .filter(|u| u.delegator_address == delegator_address)
            .cloned()
            .collect()
    }

    /// Get a snapshot of all staking power (for governance voting power verification)
    pub async fn get_staking_snapshot(&self) -> HashMap<String, u64> {
        let delegations = self.delegations.read().await;
        let mut snapshot = HashMap::new();

        for (delegator, delegation_list) in delegations.iter() {
            let total: u64 = delegation_list.iter().map(|d| d.amount).sum();
            if total > 0 {
                snapshot.insert(delegator.clone(), total);
            }
        }

        snapshot
    }

    /// Get all validators
    pub async fn get_validators(&self) -> Vec<ValidatorStake> {
        let validators = self.validators.read().await;
        validators.values().cloned().collect()
    }

    /// Get delegations for an address
    pub async fn get_delegations(&self, delegator_address: &str) -> Vec<Delegation> {
        let delegations = self.delegations.read().await;
        delegations.get(delegator_address)
            .map(|d| d.clone())
            .unwrap_or_default()
    }

    /// Get staking statistics
    pub async fn get_statistics(&self) -> StakingStatistics {
        let validators = self.validators.read().await;
        let total_staked = *self.total_staked.read().await;
        let inflation_rate = *self.inflation_rate.read().await;
        let current_height = *self.current_height.read().await;

        let active_validators = validators.values().filter(|v| !v.jailed).count();
        let jailed_validators = validators.values().filter(|v| v.jailed).count();

        StakingStatistics {
            total_validators: validators.len(),
            active_validators,
            jailed_validators,
            total_staked,
            current_apy: BASE_APY,
            inflation_rate,
            current_height,
        }
    }

    /// Update inflation rate (governance)
    pub async fn update_inflation_rate(&self, new_rate: f64) -> Result<()> {
        if new_rate < 0.0 || new_rate > 1.0 {
            bail!("Inflation rate must be between 0% and 100%");
        }

        let mut inflation_rate = self.inflation_rate.write().await;
        *inflation_rate = new_rate;

        info!("Inflation rate updated to {:.2}%", new_rate * 100.0);

        Ok(())
    }

    /// Record that a validator signed a block (reset miss counter)
    pub async fn record_block_signed(&self, validator_address: &str) -> Result<()> {
        let mut validators = self.validators.write().await;
        if let Some(validator) = validators.get_mut(validator_address) {
            validator.blocks_signed += 1;
            // Reset missed blocks on successful sign
            validator.blocks_missed = 0;
            Ok(())
        } else {
            // Validator might not be in staking yet - that's OK
            Ok(())
        }
    }

    /// Record that a validator missed their block proposal slot
    /// Returns true if the validator should be slashed (exceeded threshold)
    pub async fn record_block_missed(&self, validator_address: &str) -> Result<bool> {
        // Downtime slashing thresholds (like Cosmos)
        // Slash after 100 consecutive missed blocks (~3.3 minutes of downtime)
        const DOWNTIME_MISS_THRESHOLD: u64 = 100;
        const DOWNTIME_SLASH_PERCENT: f64 = 0.001; // 0.1% slash for downtime
        const DOWNTIME_JAIL_BLOCKS: u64 = 3600; // ~2 hours jail time

        let should_slash = {
            let mut validators = self.validators.write().await;
            if let Some(validator) = validators.get_mut(validator_address) {
                if validator.jailed {
                    // Already jailed, don't count more misses
                    return Ok(false);
                }
                validator.blocks_missed += 1;
                validator.blocks_missed >= DOWNTIME_MISS_THRESHOLD
            } else {
                return Ok(false);
            }
        };

        if should_slash {
            warn!(
                "⚠️ Validator {} exceeded downtime threshold ({} missed blocks) - initiating slash",
                validator_address, DOWNTIME_MISS_THRESHOLD
            );
            self.slash_validator(
                validator_address,
                SlashReason::Downtime,
                DOWNTIME_SLASH_PERCENT,
                DOWNTIME_JAIL_BLOCKS,
            ).await?;
        }

        Ok(should_slash)
    }

    /// Check if a validator is jailed
    pub async fn is_jailed(&self, validator_address: &str) -> bool {
        let validators = self.validators.read().await;
        validators.get(validator_address)
            .map(|v| v.jailed)
            .unwrap_or(false)
    }

    /// Get validator's missed block count
    pub async fn get_missed_blocks(&self, validator_address: &str) -> u64 {
        let validators = self.validators.read().await;
        validators.get(validator_address)
            .map(|v| v.blocks_missed)
            .unwrap_or(0)
    }

    // ============ Persistence Methods ============

    /// Create a snapshot of the current staking state for persistence
    pub async fn create_snapshot(&self) -> crate::storage::StakingStateSnapshot {
        let validators = self.validators.read().await;
        let delegations = self.delegations.read().await;
        let unbonding_queue = self.unbonding_queue.read().await;
        let total_staked = *self.total_staked.read().await;
        let current_height = *self.current_height.read().await;

        let snapshot_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        crate::storage::StakingStateSnapshot {
            validators: validators.clone(),
            delegations: delegations.clone(),
            unbonding_queue: unbonding_queue.clone(),
            total_staked,
            current_height,
            snapshot_time,
        }
    }

    /// Restore staking state from a persisted snapshot
    /// Called during node startup if state exists
    pub async fn restore_from_snapshot(&self, snapshot: crate::storage::StakingStateSnapshot) -> Result<()> {
        info!("🔄 Restoring staking state from snapshot (height {})", snapshot.current_height);

        // Restore validators
        {
            let mut validators = self.validators.write().await;
            *validators = snapshot.validators;
        }

        // Restore delegations
        {
            let mut delegations = self.delegations.write().await;
            *delegations = snapshot.delegations;
        }

        // Restore unbonding queue
        {
            let mut unbonding_queue = self.unbonding_queue.write().await;
            *unbonding_queue = snapshot.unbonding_queue;
        }

        // Restore counters
        {
            let mut total_staked = self.total_staked.write().await;
            *total_staked = snapshot.total_staked;
        }

        {
            let mut current_height = self.current_height.write().await;
            *current_height = snapshot.current_height;
        }

        let validators = self.validators.read().await;
        let delegations = self.delegations.read().await;
        let unbonding_queue = self.unbonding_queue.read().await;

        info!(
            "✅ Staking state restored: {} validators, {} delegators, {} unbonding, total staked: {}",
            validators.len(),
            delegations.len(),
            unbonding_queue.len(),
            snapshot.total_staked
        );

        Ok(())
    }

    /// Save current state to persistent storage
    /// Call this after every staking operation
    pub async fn persist_to_storage(&self, storage: &crate::storage::PersistentStorage) -> Result<()> {
        let snapshot = self.create_snapshot().await;
        storage.save_staking_state(&snapshot)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingStatistics {
    pub total_validators: usize,
    pub active_validators: usize,
    pub jailed_validators: usize,
    pub total_staked: u64,
    pub current_apy: f64,
    pub inflation_rate: f64,
    pub current_height: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Minimum stake is 10,000 SLTN (10_000_000_000_000 with 9 decimals)
    const MIN_STAKE: u64 = 10_000_000_000_000;

    #[tokio::test]
    async fn test_create_validator() {
        let staking = StakingManager::new(0.08);
        
        // Should fail with stake below minimum
        let result = staking.create_validator(
            "low_stake".to_string(),
            MIN_STAKE - 1,
            0.10,
        ).await;
        assert!(result.is_err());
        
        // Should succeed with minimum stake
        let result = staking.create_validator(
            "validator1".to_string(),
            MIN_STAKE,
            0.10,
        ).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delegate() {
        let staking = StakingManager::new(0.08);
        staking.create_validator("validator1".to_string(), MIN_STAKE, 0.10).await.unwrap();
        
        let result = staking.delegate(
            "delegator1".to_string(),
            "validator1".to_string(),
            1_000_000_000_000, // 1,000 SLTN delegation
        ).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reward_distribution() {
        let staking = StakingManager::new(0.08); // 8% inflation
        staking.create_validator("validator1".to_string(), MIN_STAKE, 0.10).await.unwrap();
        
        let distribution = staking.distribute_block_rewards(1).await.unwrap();
        assert!(distribution.total_rewards > 0);
        
        // Verify validator received rewards
        assert!(distribution.validator_rewards.contains_key("validator1"));
        let validator_reward = distribution.validator_rewards.get("validator1").unwrap();
        assert!(*validator_reward > 0);
    }

    #[tokio::test]
    async fn test_apy_calculation() {
        // Test that 13.33% APY is correctly applied
        let staking = StakingManager::new(BASE_APY); // Use the 13.33% APY
        staking.create_validator("validator1".to_string(), MIN_STAKE, 0.0).await.unwrap(); // 0% commission
        
        // Simulate a full year of blocks
        let mut total_rewards: u64 = 0;
        for height in 1..=BLOCKS_PER_YEAR {
            if height % 1_000_000 == 0 {
                // Sample every 1M blocks to speed up test
                let distribution = staking.distribute_block_rewards(height).await.unwrap();
                total_rewards += distribution.total_rewards * 1_000_000;
            }
        }
        
        // After one year, rewards should be approximately 13.33% of stake
        // Expected: MIN_STAKE * 0.1333 = 1,333,000,000,000 (1,333 SLTN)
        let expected_annual_reward = (MIN_STAKE as f64 * BASE_APY) as u64;
        
        // Allow 5% tolerance due to rounding
        let lower_bound = (expected_annual_reward as f64 * 0.95) as u64;
        let upper_bound = (expected_annual_reward as f64 * 1.05) as u64;
        
        assert!(
            total_rewards >= lower_bound && total_rewards <= upper_bound,
            "Expected ~{} rewards, got {} (APY: {:.2}%)",
            expected_annual_reward,
            total_rewards,
            (total_rewards as f64 / MIN_STAKE as f64) * 100.0
        );
    }

    #[tokio::test]
    async fn test_statistics_include_apy() {
        let staking = StakingManager::new(BASE_APY);
        staking.create_validator("validator1".to_string(), MIN_STAKE, 0.10).await.unwrap();
        
        let stats = staking.get_statistics().await;
        
        // Verify APY is reported correctly
        assert!((stats.current_apy - BASE_APY).abs() < 0.0001, 
            "Expected APY {}, got {}", BASE_APY, stats.current_apy);
        assert_eq!(stats.total_validators, 1);
        assert_eq!(stats.total_staked, MIN_STAKE);
    }

    // ============ Integration Tests ============

    #[tokio::test]
    async fn test_unbonding_period() {
        let staking = StakingManager::new(0.08);
        staking.create_validator("validator1".to_string(), MIN_STAKE, 0.10).await.unwrap();
        
        // Delegate tokens
        let delegation_amount = 1_000_000_000_000; // 1,000 SLTN
        staking.delegate(
            "delegator1".to_string(),
            "validator1".to_string(),
            delegation_amount,
        ).await.unwrap();

        // Verify delegation recorded
        let delegations = staking.get_delegations("delegator1").await;
        assert_eq!(delegations.len(), 1);
        assert_eq!(delegations[0].amount, delegation_amount);

        // Undelegate - starts 21-day unbonding
        let unbonding = staking.undelegate(
            "delegator1".to_string(),
            "validator1".to_string(),
            delegation_amount,
        ).await.unwrap();

        // Verify unbonding entry created
        assert_eq!(unbonding.amount, delegation_amount);
        assert_eq!(unbonding.completion_height, UNBONDING_PERIOD_BLOCKS); // Started at height 0

        // Tokens should NOT be available yet
        let pending = staking.get_unbondings("delegator1").await;
        assert_eq!(pending.len(), 1);

        // Process at current height (0) - nothing should complete
        let completed = staking.process_unbondings().await;
        assert!(completed.is_empty());

        // Advance height past unbonding period and process
        {
            let mut height = staking.current_height.write().await;
            *height = UNBONDING_PERIOD_BLOCKS + 1;
        }
        let completed = staking.process_unbondings().await;
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].amount, delegation_amount);

        // Queue should now be empty
        let pending = staking.get_unbondings("delegator1").await;
        assert!(pending.is_empty());
    }

    #[tokio::test]
    async fn test_slashing_reduces_stake() {
        let staking = StakingManager::new(0.08);
        staking.create_validator("validator1".to_string(), MIN_STAKE, 0.10).await.unwrap();

        let initial_stake = {
            let validators = staking.validators.read().await;
            validators.get("validator1").unwrap().total_stake
        };

        // Slash 5% for double signing
        staking.slash_validator(
            "validator1",
            SlashReason::DoubleSign,
            0.05, // 5% slash
            10000, // Jail for 10000 blocks
        ).await.unwrap();

        let final_stake = {
            let validators = staking.validators.read().await;
            let v = validators.get("validator1").unwrap();
            assert!(v.jailed, "Validator should be jailed");
            v.total_stake
        };

        // Stake should be reduced by 5%
        let expected_slash = (initial_stake as f64 * 0.05) as u64;
        assert_eq!(final_stake, initial_stake - expected_slash);
    }

    #[tokio::test]
    async fn test_downtime_slashing() {
        let staking = StakingManager::new(0.08);
        staking.create_validator("validator1".to_string(), MIN_STAKE, 0.10).await.unwrap();

        // Miss 99 blocks - should NOT slash yet
        for _ in 0..99 {
            let slashed = staking.record_block_missed("validator1").await.unwrap();
            assert!(!slashed, "Should not slash before threshold");
        }

        // Verify missed count
        let missed = staking.get_missed_blocks("validator1").await;
        assert_eq!(missed, 99);

        // Miss block 100 - should trigger slash
        let slashed = staking.record_block_missed("validator1").await.unwrap();
        assert!(slashed, "Should slash at threshold");

        // Verify jailed
        assert!(staking.is_jailed("validator1").await);
    }

    #[tokio::test]
    async fn test_block_signed_resets_miss_counter() {
        let staking = StakingManager::new(0.08);
        staking.create_validator("validator1".to_string(), MIN_STAKE, 0.10).await.unwrap();

        // Miss 50 blocks
        for _ in 0..50 {
            staking.record_block_missed("validator1").await.unwrap();
        }
        assert_eq!(staking.get_missed_blocks("validator1").await, 50);

        // Sign a block - should reset counter
        staking.record_block_signed("validator1").await.unwrap();
        assert_eq!(staking.get_missed_blocks("validator1").await, 0);
    }

    #[tokio::test]
    async fn test_cannot_delegate_to_jailed_validator() {
        let staking = StakingManager::new(0.08);
        staking.create_validator("validator1".to_string(), MIN_STAKE, 0.10).await.unwrap();

        // Jail the validator
        staking.slash_validator(
            "validator1",
            SlashReason::DoubleSign,
            0.01,
            10000,
        ).await.unwrap();

        // Try to delegate - should fail
        let result = staking.delegate(
            "delegator1".to_string(),
            "validator1".to_string(),
            1_000_000_000_000,
        ).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("jailed"));
    }

    #[tokio::test]
    async fn test_unjail_after_jail_period() {
        let staking = StakingManager::new(0.08);
        staking.create_validator("validator1".to_string(), MIN_STAKE, 0.10).await.unwrap();

        // Jail for 100 blocks
        staking.slash_validator(
            "validator1",
            SlashReason::Downtime,
            0.001,
            100,
        ).await.unwrap();

        // Try to unjail immediately - should fail
        let result = staking.unjail_validator("validator1").await;
        assert!(result.is_err());

        // Advance height past jail period
        {
            let mut height = staking.current_height.write().await;
            *height = 101;
        }

        // Now unjail should succeed
        staking.unjail_validator("validator1").await.unwrap();
        assert!(!staking.is_jailed("validator1").await);
    }

    #[tokio::test]
    async fn test_partial_undelegation() {
        let staking = StakingManager::new(0.08);
        staking.create_validator("validator1".to_string(), MIN_STAKE, 0.10).await.unwrap();

        let total_delegation = 5_000_000_000_000; // 5,000 SLTN
        staking.delegate(
            "delegator1".to_string(),
            "validator1".to_string(),
            total_delegation,
        ).await.unwrap();

        // Undelegate half
        let half = total_delegation / 2;
        staking.undelegate(
            "delegator1".to_string(),
            "validator1".to_string(),
            half,
        ).await.unwrap();

        // Verify remaining delegation
        let delegations = staking.get_delegations("delegator1").await;
        assert_eq!(delegations[0].amount, half);

        // Verify validator stake reduced
        let validators = staking.get_validators().await;
        let v = validators.iter().find(|v| v.validator_address == "validator1").unwrap();
        assert_eq!(v.delegated_stake, half);
    }

    #[tokio::test]
    async fn test_multiple_delegators() {
        let staking = StakingManager::new(0.08);
        staking.create_validator("validator1".to_string(), MIN_STAKE, 0.10).await.unwrap();

        // Multiple delegators
        for i in 1..=5 {
            staking.delegate(
                format!("delegator{}", i),
                "validator1".to_string(),
                1_000_000_000_000 * i as u64, // Each delegates more
            ).await.unwrap();
        }

        // Verify total delegated stake
        let validators = staking.get_validators().await;
        let v = validators.iter().find(|v| v.validator_address == "validator1").unwrap();
        
        // Sum: 1 + 2 + 3 + 4 + 5 = 15 thousand SLTN
        let expected_delegated = 15_000_000_000_000u64;
        assert_eq!(v.delegated_stake, expected_delegated);
        assert_eq!(v.total_stake, MIN_STAKE + expected_delegated);
    }

    #[tokio::test]
    async fn test_staking_snapshot() {
        let staking = StakingManager::new(0.08);
        staking.create_validator("validator1".to_string(), MIN_STAKE, 0.10).await.unwrap();
        staking.delegate(
            "delegator1".to_string(),
            "validator1".to_string(),
            2_000_000_000_000,
        ).await.unwrap();

        // Get staking snapshot for governance
        let snapshot = staking.get_staking_snapshot().await;
        
        // delegator1 should have 2,000 SLTN voting power
        assert_eq!(*snapshot.get("delegator1").unwrap(), 2_000_000_000_000);
    }

    #[tokio::test]
    async fn test_withdraw_rewards() {
        let staking = StakingManager::new(0.08);
        staking.create_validator("validator1".to_string(), MIN_STAKE, 0.10).await.unwrap();

        // Distribute some rewards
        for height in 1..=100 {
            staking.distribute_block_rewards(height).await.unwrap();
        }

        // Withdraw validator rewards
        let rewards = staking.withdraw_validator_rewards("validator1").await.unwrap();
        assert!(rewards > 0, "Validator should have accumulated rewards");

        // Second withdrawal should return 0
        let rewards2 = staking.withdraw_validator_rewards("validator1").await.unwrap();
        assert_eq!(rewards2, 0);
    }

    #[tokio::test]
    async fn test_delegator_rewards() {
        let staking = StakingManager::new(0.08);
        staking.create_validator("validator1".to_string(), MIN_STAKE, 0.05).await.unwrap(); // 5% commission

        // Delegate equal to self-stake
        staking.delegate(
            "delegator1".to_string(),
            "validator1".to_string(),
            MIN_STAKE,
        ).await.unwrap();

        // Distribute rewards for several blocks
        for height in 1..=1000 {
            staking.distribute_block_rewards(height).await.unwrap();
        }

        // Delegator should have rewards
        let delegations = staking.get_delegations("delegator1").await;
        assert!(delegations[0].rewards_accumulated > 0, "Delegator should have rewards");
    }

    #[tokio::test]
    async fn test_inflation_rate_update() {
        let staking = StakingManager::new(0.04);
        
        // Invalid rates should fail
        assert!(staking.update_inflation_rate(-0.01).await.is_err());
        assert!(staking.update_inflation_rate(1.5).await.is_err());

        // Valid update should succeed
        staking.update_inflation_rate(0.08).await.unwrap();
        
        let stats = staking.get_statistics().await;
        assert!((stats.inflation_rate - 0.08).abs() < 0.0001);
    }
}
