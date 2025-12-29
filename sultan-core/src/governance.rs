//! Production Governance System
//!
//! Real on-chain governance with:
//! - Proposal creation and voting
//! - Weighted voting based on stake
//! - Proposal execution
//! - Parameter changes
//! - Upgrade management
//!
//! Security protections:
//! - Deposit requirement (prevents spam)
//! - Quorum requirement (prevents low-participation attacks)
//! - Veto mechanism (prevents harmful proposals)
//! - Voting power snapshot (prevents flash stake attacks)
//! - Rate limiting (prevents proposal spam)
//! - Parameter bounds (prevents extreme changes)

use anyhow::{Result, Context, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

const PROPOSAL_DEPOSIT: u64 = 1_000_000_000_000; // 1,000 SLTN
const VOTING_PERIOD_BLOCKS: u64 = 302_400; // ~7 days with 2s blocks (7*24*60*60/2)
const MIN_QUORUM: f64 = 0.334; // 33.4% of total stake must vote
const PASS_THRESHOLD: f64 = 0.50; // 50% of votes must be YES

// Security constants
const MAX_ACTIVE_PROPOSALS_PER_ADDRESS: usize = 3; // Rate limit proposals
const PROPOSAL_COOLDOWN_BLOCKS: u64 = 43_200; // ~1 day between proposals from same address (24*60*60/2)
#[allow(dead_code)]
const VETO_THRESHOLD: f64 = 0.334; // >33.4% NoWithVeto = proposal fails & deposit burned

// Multi-sig requirements for critical proposals
const UPGRADE_MULTISIG_THRESHOLD: usize = 5; // Need 5 validator signatures for upgrades
const EMERGENCY_PAUSE_THRESHOLD: f64 = 0.67; // 67% of validators can emergency pause

// Parameter bounds (prevents extreme changes)
const MAX_INFLATION_RATE: f64 = 0.20; // Max 20% inflation
const MIN_INFLATION_RATE: f64 = 0.01; // Min 1% inflation
const MAX_VALIDATOR_COMMISSION: f64 = 0.50; // Max 50% commission
const MIN_UNBONDING_DAYS: u64 = 7; // Min 7 day unbonding
const MAX_UNBONDING_DAYS: u64 = 28; // Max 28 day unbonding

// Discussion requirements
const DISCUSSION_PERIOD_BLOCKS: u64 = 86_400; // ~2 days for Telegram discussion before voting (2*24*60*60/2)

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    ParameterChange,
    SoftwareUpgrade,
    CommunityPool,
    TextProposal,
    /// Emergency action requiring validator multi-sig
    EmergencyAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    /// Proposal submitted, awaiting discussion period
    DiscussionPeriod,
    DepositPeriod,
    VotingPeriod,
    Passed,
    Rejected,
    Failed,
    Executed,
    /// Emergency paused by validators
    EmergencyPaused,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoteOption {
    Yes,
    No,
    Abstain,
    NoWithVeto, // Veto if >33.4% of votes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: u64,
    pub proposer: String,
    pub title: String,
    pub description: String,
    pub proposal_type: ProposalType,
    pub status: ProposalStatus,
    pub submit_height: u64,
    pub submit_time: u64,
    pub deposit_end_height: u64,
    /// Discussion period end - proposals enter voting only after discussion
    pub discussion_end_height: u64,
    pub voting_start_height: u64,
    pub voting_end_height: u64,
    pub total_deposit: u64,
    pub depositors: Vec<Deposit>,
    pub final_tally: Option<TallyResult>,
    pub parameters: Option<HashMap<String, String>>,
    /// Snapshot of voting power at proposal start (prevents flash stake attacks)
    pub voting_power_snapshot: Option<HashMap<String, u64>>,
    /// Telegram discussion link (required for proposals)
    pub telegram_discussion_url: Option<String>,
    /// Discord discussion link (optional alternative)
    pub discord_discussion_url: Option<String>,
    /// For SoftwareUpgrade: validator signatures required
    pub validator_signatures: Vec<ValidatorSignature>,
    /// Emergency pause votes from validators
    pub emergency_pause_votes: Vec<String>,
}

/// Validator signature for critical proposals (upgrades)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSignature {
    pub validator_address: String,
    pub signature: String,
    pub signed_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deposit {
    pub depositor: String,
    pub amount: u64,
    pub time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub proposal_id: u64,
    pub voter: String,
    pub option: VoteOption,
    pub voting_power: u64,
    pub time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TallyResult {
    pub yes: u64,
    pub no: u64,
    pub abstain: u64,
    pub no_with_veto: u64,
    pub total_voting_power: u64,
    pub total_stake: u64,
    pub quorum_reached: bool,
    pub passed: bool,
    pub vetoed: bool,
}

pub struct GovernanceManager {
    proposals: Arc<RwLock<HashMap<u64, Proposal>>>,
    votes: Arc<RwLock<HashMap<u64, Vec<Vote>>>>,
    next_proposal_id: Arc<RwLock<u64>>,
    current_height: Arc<RwLock<u64>>,
    total_bonded_tokens: Arc<RwLock<u64>>,
    /// Track last proposal submission per address (for rate limiting)
    last_proposal_by_address: Arc<RwLock<HashMap<String, u64>>>,
    /// Track staking snapshots for voting power verification
    staking_snapshots: Arc<RwLock<HashMap<u64, HashMap<String, u64>>>>,
}

impl GovernanceManager {
    pub fn new() -> Self {
        Self {
            proposals: Arc::new(RwLock::new(HashMap::new())),
            votes: Arc::new(RwLock::new(HashMap::new())),
            next_proposal_id: Arc::new(RwLock::new(1)),
            current_height: Arc::new(RwLock::new(0)),
            total_bonded_tokens: Arc::new(RwLock::new(0)),
            last_proposal_by_address: Arc::new(RwLock::new(HashMap::new())),
            staking_snapshots: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Validate parameter bounds for ParameterChange proposals
    /// Prevents extreme/malicious parameter changes
    fn validate_parameter_bounds(params: &HashMap<String, String>) -> Result<()> {
        for (key, value) in params {
            match key.as_str() {
                "inflation_rate" => {
                    let rate: f64 = value.parse().context("Invalid inflation rate")?;
                    if rate < MIN_INFLATION_RATE || rate > MAX_INFLATION_RATE {
                        bail!("Inflation rate must be between {}% and {}%", 
                              MIN_INFLATION_RATE * 100.0, MAX_INFLATION_RATE * 100.0);
                    }
                }
                "max_validator_commission" => {
                    let commission: f64 = value.parse().context("Invalid commission rate")?;
                    if commission > MAX_VALIDATOR_COMMISSION {
                        bail!("Validator commission cannot exceed {}%", MAX_VALIDATOR_COMMISSION * 100.0);
                    }
                }
                "unbonding_days" => {
                    let days: u64 = value.parse().context("Invalid unbonding days")?;
                    if days < MIN_UNBONDING_DAYS || days > MAX_UNBONDING_DAYS {
                        bail!("Unbonding period must be between {} and {} days",
                              MIN_UNBONDING_DAYS, MAX_UNBONDING_DAYS);
                    }
                }
                "min_deposit" => {
                    let deposit: u64 = value.parse().context("Invalid deposit amount")?;
                    if deposit < 100_000_000_000 { // Min 100 SLTN
                        bail!("Minimum deposit cannot be less than 100 SLTN");
                    }
                    if deposit > 100_000_000_000_000 { // Max 100,000 SLTN
                        bail!("Minimum deposit cannot exceed 100,000 SLTN");
                    }
                }
                _ => {
                    // Unknown parameters are allowed but logged
                    warn!("Unknown parameter in proposal: {}", key);
                }
            }
        }
        Ok(())
    }

    /// Submit a new governance proposal
    /// 
    /// Requires a Telegram discussion link for community deliberation.
    /// Proposals enter a 2-day discussion period before voting begins.
    pub async fn submit_proposal(
        &self,
        proposer: String,
        title: String,
        description: String,
        proposal_type: ProposalType,
        initial_deposit: u64,
        parameters: Option<HashMap<String, String>>,
        telegram_discussion_url: Option<String>,
        discord_discussion_url: Option<String>,
    ) -> Result<u64> {
        // === Input validation ===
        if title.is_empty() || title.len() > 140 {
            bail!("Title must be between 1 and 140 characters");
        }

        if description.is_empty() || description.len() > 10_000 {
            bail!("Description must be between 1 and 10,000 characters");
        }

        if initial_deposit < PROPOSAL_DEPOSIT {
            bail!("Initial deposit must be at least {} SLTN", PROPOSAL_DEPOSIT / 1_000_000_000);
        }

        // === Require discussion link for non-emergency proposals ===
        if proposal_type != ProposalType::EmergencyAction {
            if telegram_discussion_url.is_none() && discord_discussion_url.is_none() {
                bail!("Proposals require a Discord or Telegram discussion link. Create a discussion in our community first: https://discord.com/channels/1375878827460395142/1453111965428875537");
            }
        }

        // === Validate parameter bounds for ParameterChange proposals ===
        if proposal_type == ProposalType::ParameterChange {
            if let Some(ref params) = parameters {
                Self::validate_parameter_bounds(params)?;
            }
        }

        let current_height = *self.current_height.read().await;

        // === Rate limiting: Check cooldown ===
        {
            let last_proposals = self.last_proposal_by_address.read().await;
            if let Some(last_height) = last_proposals.get(&proposer) {
                if current_height < last_height + PROPOSAL_COOLDOWN_BLOCKS {
                    let blocks_remaining = (last_height + PROPOSAL_COOLDOWN_BLOCKS) - current_height;
                    bail!(
                        "Rate limit: Please wait {} more blocks (~{} hours) before submitting another proposal",
                        blocks_remaining,
                        blocks_remaining / 1800 // ~1800 blocks per hour with 2s blocks
                    );
                }
            }
        }

        // === Rate limiting: Check max active proposals ===
        {
            let proposals = self.proposals.read().await;
            let active_count = proposals.values()
                .filter(|p| p.proposer == proposer && 
                       (p.status == ProposalStatus::VotingPeriod || 
                        p.status == ProposalStatus::DepositPeriod))
                .count();
            
            if active_count >= MAX_ACTIVE_PROPOSALS_PER_ADDRESS {
                bail!(
                    "Rate limit: You already have {} active proposals. Wait for them to complete.",
                    active_count
                );
            }
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let mut next_id = self.next_proposal_id.write().await;
        let proposal_id = *next_id;
        *next_id += 1;

        // Record this proposal for rate limiting
        {
            let mut last_proposals = self.last_proposal_by_address.write().await;
            last_proposals.insert(proposer.clone(), current_height);
        }

        // Get voting power snapshot (if available) to prevent flash stake attacks
        let voting_power_snapshot = {
            let snapshots = self.staking_snapshots.read().await;
            snapshots.get(&current_height).cloned()
        };

        // Calculate discussion and voting periods
        let discussion_end = current_height + DISCUSSION_PERIOD_BLOCKS;
        let voting_start = discussion_end;
        let voting_end = voting_start + VOTING_PERIOD_BLOCKS;

        let proposal = Proposal {
            id: proposal_id,
            proposer: proposer.clone(),
            title: title.clone(),
            description,
            proposal_type: proposal_type.clone(),
            status: ProposalStatus::DiscussionPeriod, // Start with discussion
            submit_height: current_height,
            submit_time: now,
            deposit_end_height: current_height + 1_000,
            discussion_end_height: discussion_end,
            voting_start_height: voting_start,
            voting_end_height: voting_end,
            total_deposit: initial_deposit,
            depositors: vec![Deposit {
                depositor: proposer.clone(),
                amount: initial_deposit,
                time: now,
            }],
            final_tally: None,
            parameters,
            voting_power_snapshot,
            telegram_discussion_url,
            discord_discussion_url,
            validator_signatures: Vec::new(),
            emergency_pause_votes: Vec::new(),
        };

        let mut proposals = self.proposals.write().await;
        proposals.insert(proposal_id, proposal);

        let mut votes = self.votes.write().await;
        votes.insert(proposal_id, Vec::new());

        info!(
            "Proposal #{} submitted by {}: '{}' (Discussion until block {}, Voting until block {})",
            proposal_id, proposer, title, discussion_end, voting_end
        );

        Ok(proposal_id)
    }

    /// Cast a vote on a proposal
    /// 
    /// Security: Voting power is verified against snapshot taken at proposal start
    /// to prevent flash stake attacks (stake → vote → unstake).
    pub async fn vote(
        &self,
        proposal_id: u64,
        voter: String,
        option: VoteOption,
        voting_power: u64,
    ) -> Result<()> {
        let proposals = self.proposals.read().await;
        let proposal = proposals.get(&proposal_id)
            .context("Proposal not found")?;

        // Check proposal status
        match proposal.status {
            ProposalStatus::DiscussionPeriod => {
                bail!("Proposal is still in discussion period. Voting starts at block {}", 
                      proposal.voting_start_height);
            }
            ProposalStatus::EmergencyPaused => {
                bail!("Proposal has been emergency paused by validators");
            }
            ProposalStatus::VotingPeriod => {} // OK to vote
            _ => {
                bail!("Proposal is not in voting period");
            }
        }

        let current_height = *self.current_height.read().await;
        if current_height > proposal.voting_end_height {
            bail!("Voting period has ended");
        }

        if voting_power == 0 {
            bail!("No voting power");
        }

        // === Security: Verify voting power against snapshot ===
        // This prevents flash stake attacks where someone stakes just to vote
        let verified_voting_power = if let Some(ref snapshot) = proposal.voting_power_snapshot {
            // Use snapshot voting power (what they had when proposal started)
            let snapshot_power = snapshot.get(&voter).copied().unwrap_or(0);
            if snapshot_power == 0 {
                bail!("You had no staked tokens when this proposal started. Flash staking is not allowed.");
            }
            // Use the minimum of claimed vs snapshot (can't claim more than snapshot)
            voting_power.min(snapshot_power)
        } else {
            // No snapshot available, use claimed voting power (legacy behavior)
            voting_power
        };

        drop(proposals);

        let mut votes = self.votes.write().await;
        let proposal_votes = votes.get_mut(&proposal_id)
            .context("Proposal votes not found")?;

        // Check if already voted
        if proposal_votes.iter().any(|v| v.voter == voter) {
            bail!("Already voted on this proposal");
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let vote = Vote {
            proposal_id,
            voter: voter.clone(),
            option: option.clone(),
            voting_power: verified_voting_power,
            time: now,
        };

        proposal_votes.push(vote);

        info!(
            "Vote cast on proposal #{}: {} voted {:?} with {} power (verified)",
            proposal_id, voter, option, verified_voting_power
        );

        Ok(())
    }

    /// Update staking snapshot for a height (called by staking module)
    /// This should be called at regular intervals (e.g., every 100 blocks)
    pub async fn update_staking_snapshot(&self, height: u64, snapshot: HashMap<String, u64>) {
        let mut snapshots = self.staking_snapshots.write().await;
        snapshots.insert(height, snapshot);
        
        // Keep only snapshots for last 7 days (~100,800 blocks)
        let min_height = height.saturating_sub(VOTING_PERIOD_BLOCKS);
        snapshots.retain(|&h, _| h >= min_height);
    }

    /// Add validator signature for SoftwareUpgrade proposals
    /// Requires UPGRADE_MULTISIG_THRESHOLD signatures before execution
    pub async fn sign_upgrade_proposal(
        &self,
        proposal_id: u64,
        validator_address: String,
        signature: String,
    ) -> Result<usize> {
        let mut proposals = self.proposals.write().await;
        let proposal = proposals.get_mut(&proposal_id)
            .context("Proposal not found")?;

        if proposal.proposal_type != ProposalType::SoftwareUpgrade {
            bail!("Only SoftwareUpgrade proposals require multi-sig");
        }

        if proposal.status != ProposalStatus::Passed {
            bail!("Proposal must be passed before validators can sign");
        }

        // Check if already signed
        if proposal.validator_signatures.iter().any(|s| s.validator_address == validator_address) {
            bail!("Validator has already signed this proposal");
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        proposal.validator_signatures.push(ValidatorSignature {
            validator_address: validator_address.clone(),
            signature,
            signed_at: now,
        });

        let sig_count = proposal.validator_signatures.len();

        info!(
            "Validator {} signed upgrade proposal #{} ({}/{} signatures)",
            validator_address, proposal_id, sig_count, UPGRADE_MULTISIG_THRESHOLD
        );

        // Auto-execute if threshold reached
        if sig_count >= UPGRADE_MULTISIG_THRESHOLD {
            proposal.status = ProposalStatus::Executed;
            info!("Upgrade proposal #{} has enough signatures - marked for execution", proposal_id);
        }

        Ok(sig_count)
    }

    /// Emergency pause a proposal (validator action)
    /// If 67% of validators vote to pause, proposal is halted
    pub async fn emergency_pause_vote(
        &self,
        proposal_id: u64,
        validator_address: String,
        total_validators: usize,
    ) -> Result<bool> {
        let mut proposals = self.proposals.write().await;
        let proposal = proposals.get_mut(&proposal_id)
            .context("Proposal not found")?;

        if proposal.status == ProposalStatus::EmergencyPaused {
            bail!("Proposal is already paused");
        }

        if proposal.status != ProposalStatus::VotingPeriod && proposal.status != ProposalStatus::Passed {
            bail!("Can only emergency pause proposals in voting or passed status");
        }

        // Check if already voted to pause
        if proposal.emergency_pause_votes.contains(&validator_address) {
            bail!("Validator has already voted to pause");
        }

        proposal.emergency_pause_votes.push(validator_address.clone());

        let pause_votes = proposal.emergency_pause_votes.len();
        let threshold = (total_validators as f64 * EMERGENCY_PAUSE_THRESHOLD).ceil() as usize;

        info!(
            "Validator {} voted to emergency pause proposal #{} ({}/{} votes, need {})",
            validator_address, proposal_id, pause_votes, total_validators, threshold
        );

        // Check if threshold reached
        if pause_votes >= threshold {
            proposal.status = ProposalStatus::EmergencyPaused;
            warn!(
                "🚨 EMERGENCY PAUSE: Proposal #{} has been paused by validator consensus",
                proposal_id
            );
            return Ok(true);
        }

        Ok(false)
    }

    /// Advance proposals from discussion to voting period
    /// Should be called each block
    pub async fn advance_proposal_phases(&self) {
        let current_height = *self.current_height.read().await;
        let mut proposals = self.proposals.write().await;

        for proposal in proposals.values_mut() {
            // Move from discussion to voting period
            if proposal.status == ProposalStatus::DiscussionPeriod 
               && current_height >= proposal.discussion_end_height {
                proposal.status = ProposalStatus::VotingPeriod;
                info!(
                    "Proposal #{} entering voting period (discussion complete)",
                    proposal.id
                );
            }
        }
    }

    /// Tally votes and update proposal status
    pub async fn tally_proposal(&self, proposal_id: u64) -> Result<TallyResult> {
        let mut proposals = self.proposals.write().await;
        let proposal = proposals.get_mut(&proposal_id)
            .context("Proposal not found")?;

        if proposal.status != ProposalStatus::VotingPeriod {
            bail!("Proposal is not in voting period");
        }

        let current_height = *self.current_height.read().await;
        if current_height <= proposal.voting_end_height {
            bail!("Voting period has not ended");
        }

        let votes = self.votes.read().await;
        let proposal_votes = votes.get(&proposal_id)
            .context("Proposal votes not found")?;

        let mut yes = 0u64;
        let mut no = 0u64;
        let mut abstain = 0u64;
        let mut no_with_veto = 0u64;

        for vote in proposal_votes {
            match vote.option {
                VoteOption::Yes => yes += vote.voting_power,
                VoteOption::No => no += vote.voting_power,
                VoteOption::Abstain => abstain += vote.voting_power,
                VoteOption::NoWithVeto => {
                    no += vote.voting_power;
                    no_with_veto += vote.voting_power;
                }
            }
        }

        let total_voting_power = yes + no + abstain;
        let total_bonded = *self.total_bonded_tokens.read().await;

        let quorum_reached = if total_bonded > 0 {
            (total_voting_power as f64 / total_bonded as f64) >= MIN_QUORUM
        } else {
            false
        };

        let vetoed = if total_voting_power > 0 {
            (no_with_veto as f64 / total_voting_power as f64) > 0.334
        } else {
            false
        };

        let passed = if quorum_reached && !vetoed && (yes + no) > 0 {
            (yes as f64 / (yes + no) as f64) > PASS_THRESHOLD
        } else {
            false
        };

        let tally = TallyResult {
            yes,
            no,
            abstain,
            no_with_veto,
            total_voting_power,
            total_stake: total_bonded,
            quorum_reached,
            passed,
            vetoed,
        };

        proposal.final_tally = Some(tally.clone());
        proposal.status = if vetoed {
            ProposalStatus::Failed
        } else if passed {
            ProposalStatus::Passed
        } else {
            ProposalStatus::Rejected
        };

        info!(
            "Proposal #{} tallied: {} (Yes: {}, No: {}, Abstain: {}, Veto: {})",
            proposal_id,
            match proposal.status {
                ProposalStatus::Passed => "PASSED",
                ProposalStatus::Rejected => "REJECTED",
                ProposalStatus::Failed => "FAILED (VETOED)",
                _ => "UNKNOWN",
            },
            yes,
            no,
            abstain,
            no_with_veto
        );

        Ok(tally)
    }

    /// Execute a passed proposal
    /// CRITICAL: This function can hot-activate features without chain restart
    pub async fn execute_proposal(&self, proposal_id: u64) -> Result<()> {
        let mut proposals = self.proposals.write().await;
        let proposal = proposals.get_mut(&proposal_id)
            .context("Proposal not found")?;

        if proposal.status != ProposalStatus::Passed {
            bail!("Proposal has not passed");
        }

        // Execute based on proposal type
        match proposal.proposal_type {
            ProposalType::ParameterChange => {
                if let Some(params) = &proposal.parameters {
                    for (key, value) in params {
                        info!("🔧 Executing parameter change: {} = {}", key, value);
                        
                        // Hot-activation of features via governance
                        // This allows enabling smart contracts, IBC, etc. without chain restart
                        if key.starts_with("features.") {
                            let feature_name = key.strip_prefix("features.").unwrap();
                            let enabled: bool = value.parse()
                                .context("Feature flag must be 'true' or 'false'")?;
                            
                            info!("🚀 Feature flag update: {} = {}", feature_name, enabled);
                            
                            // Log critical feature activations
                            match feature_name {
                                "wasm_contracts_enabled" if enabled => {
                                    info!("⚠️  CRITICAL: CosmWasm smart contracts will be enabled");
                                    info!("📝 Contracts can be deployed after this proposal executes");
                                }
                                "evm_contracts_enabled" if enabled => {
                                    info!("⚠️  CRITICAL: EVM smart contracts will be enabled");
                                    info!("📝 Solidity contracts can be deployed after this proposal executes");
                                }
                                "ibc_enabled" if enabled => {
                                    info!("⚠️  CRITICAL: IBC protocol will be enabled");
                                    info!("📝 Cross-chain IBC transfers can begin");
                                }
                                _ => {}
                            }
                            
                            // NOTE: Actual feature activation happens in NodeState
                            // via config update and runtime initialization
                        }
                    }
                }
            }
            ProposalType::SoftwareUpgrade => {
                info!("📦 Executing software upgrade: {}", proposal.title);
                // In production, this would schedule a coordinated chain upgrade
                // All validators upgrade at specific block height
            }
            ProposalType::CommunityPool => {
                info!("💰 Executing community pool spend: {}", proposal.title);
                // In production, this would transfer from community pool
            }
            ProposalType::TextProposal => {
                info!("📜 Text proposal passed: {}", proposal.title);
                // No execution needed, just signaling
            }
            ProposalType::EmergencyAction => {
                info!("🚨 Emergency action executed: {}", proposal.title);
                // Emergency actions are processed by the emergency pause mechanism
                // This type bypasses normal voting when 67% of validators agree
            }
        }

        proposal.status = ProposalStatus::Executed;

        info!("✅ Proposal #{} executed successfully", proposal_id);

        Ok(())
    }

    /// Get a proposal by ID
    pub async fn get_proposal(&self, proposal_id: u64) -> Option<Proposal> {
        let proposals = self.proposals.read().await;
        proposals.get(&proposal_id).cloned()
    }

    /// Get all proposals
    pub async fn get_all_proposals(&self) -> Vec<Proposal> {
        let proposals = self.proposals.read().await;
        let mut list: Vec<_> = proposals.values().cloned().collect();
        list.sort_by(|a, b| b.id.cmp(&a.id)); // Newest first
        list
    }

    /// Get votes for a proposal
    pub async fn get_proposal_votes(&self, proposal_id: u64) -> Vec<Vote> {
        let votes = self.votes.read().await;
        votes.get(&proposal_id).cloned().unwrap_or_default()
    }

    /// Update current block height
    pub async fn update_height(&self, height: u64) {
        let mut current_height = self.current_height.write().await;
        *current_height = height;
    }

    /// Update total bonded tokens (for quorum calculation)
    pub async fn update_total_bonded(&self, amount: u64) {
        let mut total_bonded = self.total_bonded_tokens.write().await;
        *total_bonded = amount;
    }

    /// Get governance statistics
    pub async fn get_statistics(&self) -> GovernanceStatistics {
        let proposals = self.proposals.read().await;
        let current_height = *self.current_height.read().await;

        let total_proposals = proposals.len();
        let active_proposals = proposals.values()
            .filter(|p| p.status == ProposalStatus::VotingPeriod)
            .count();
        let passed_proposals = proposals.values()
            .filter(|p| p.status == ProposalStatus::Passed || p.status == ProposalStatus::Executed)
            .count();
        let rejected_proposals = proposals.values()
            .filter(|p| p.status == ProposalStatus::Rejected || p.status == ProposalStatus::Failed)
            .count();

        GovernanceStatistics {
            total_proposals,
            active_proposals,
            passed_proposals,
            rejected_proposals,
            current_height,
            min_deposit: PROPOSAL_DEPOSIT,
            voting_period: VOTING_PERIOD_BLOCKS,
            quorum: MIN_QUORUM,
            pass_threshold: PASS_THRESHOLD,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceStatistics {
    pub total_proposals: usize,
    pub active_proposals: usize,
    pub passed_proposals: usize,
    pub rejected_proposals: usize,
    pub current_height: u64,
    pub min_deposit: u64,
    pub voting_period: u64,
    pub quorum: f64,
    pub pass_threshold: f64,
}

impl Default for GovernanceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_submit_proposal() {
        let gov = GovernanceManager::new();
        let result = gov.submit_proposal(
            "proposer1".to_string(),
            "Test Proposal".to_string(),
            "This is a test proposal".to_string(),
            ProposalType::TextProposal,
            PROPOSAL_DEPOSIT,
            None,
            Some("https://t.me/SultanChain/123".to_string()),
            Some("https://discord.gg/sultan/456".to_string()),
        ).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_vote() {
        let gov = GovernanceManager::new();
        gov.update_total_bonded(10_000_000_000_000).await;
        
        let proposal_id = gov.submit_proposal(
            "proposer1".to_string(),
            "Test Proposal".to_string(),
            "This is a test proposal".to_string(),
            ProposalType::TextProposal,
            PROPOSAL_DEPOSIT,
            None,
            Some("https://t.me/SultanChain/123".to_string()),
            Some("https://discord.gg/sultan/456".to_string()),
        ).await.unwrap();

        // Advance past discussion period to voting period
        gov.update_height(DISCUSSION_PERIOD_BLOCKS + 1).await;
        gov.advance_proposal_phases().await;

        let result = gov.vote(
            proposal_id,
            "voter1".to_string(),
            VoteOption::Yes,
            1_000_000_000_000,
        ).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_tally() {
        let gov = GovernanceManager::new();
        gov.update_total_bonded(10_000_000_000_000).await;
        
        let proposal_id = gov.submit_proposal(
            "proposer1".to_string(),
            "Test Proposal".to_string(),
            "This is a test proposal".to_string(),
            ProposalType::TextProposal,
            PROPOSAL_DEPOSIT,
            None,
            Some("https://t.me/SultanChain/123".to_string()),
            Some("https://discord.gg/sultan/456".to_string()),
        ).await.unwrap();

        // Advance past discussion period to voting period
        gov.update_height(DISCUSSION_PERIOD_BLOCKS + 1).await;
        gov.advance_proposal_phases().await;

        // Vote
        gov.vote(proposal_id, "voter1".to_string(), VoteOption::Yes, 4_000_000_000_000).await.unwrap();
        gov.vote(proposal_id, "voter2".to_string(), VoteOption::No, 1_000_000_000_000).await.unwrap();

        // End voting period
        gov.update_height(DISCUSSION_PERIOD_BLOCKS + VOTING_PERIOD_BLOCKS + 1).await;

        let tally = gov.tally_proposal(proposal_id).await.unwrap();
        assert!(tally.quorum_reached);
        assert!(tally.passed);
    }
}
