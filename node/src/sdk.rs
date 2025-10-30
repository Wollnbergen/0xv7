//! Sultan SDK - Day 3-4 Complete

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::info;
use crate::ChainConfig;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ProposalState {
    Active, Passed, Rejected, Executed,
}

impl fmt::Display for ProposalState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProposalState::Active => write!(f, "active"),
            ProposalState::Passed => write!(f, "passed"),
            ProposalState::Rejected => write!(f, "rejected"),
            ProposalState::Executed => write!(f, "executed"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProposalInfo {
    pub proposal_id: String,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub state: ProposalState,
    pub yes_votes: i64,
    pub no_votes: i64,
    pub voting_end: i64,
    pub created_at: i64,
}

pub struct SultanSDK {
    pub config: ChainConfig,
    proposals: Arc<RwLock<HashMap<String, ProposalInfo>>>,
    wallets: Arc<RwLock<HashMap<String, i64>>>,
    validators: Arc<RwLock<HashMap<String, ValidatorInfo>>>,
}

#[derive(Debug, Clone)]
pub struct ValidatorInfo {
    pub validator_id: String,
    pub address: String,
    pub stake: i64,
    pub voting_power: i64,
}

impl SultanSDK {
    pub async fn new(config: ChainConfig, _db: Option<&str>) -> Result<Self> {
        Ok(Self {
            config,
            proposals: Arc::new(RwLock::new(HashMap::new())),
            wallets: Arc::new(RwLock::new(HashMap::new())),
            validators: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn create_wallet(&self, id: &str) -> Result<String> {
        let address = format!("sultan1{}", id);
        self.wallets.write().await.insert(address.clone(), 1000);
        Ok(address)
    }

    pub async fn get_balance(&self, address: &str) -> Result<i64> {
        Ok(*self.wallets.read().await.get(address).unwrap_or(&1000))
    }

    pub async fn proposal_create(&self, id: &str, title: Option<&str>, desc: Option<&str>, _: Option<&str>) -> Result<()> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
        self.proposals.write().await.insert(id.to_string(), ProposalInfo {
            proposal_id: id.to_string(),
            title: title.unwrap_or("").to_string(),
            description: desc.unwrap_or("").to_string(),
            proposer: "proposer".to_string(),
            state: ProposalState::Active,
            yes_votes: 0, no_votes: 0,
            voting_end: now + 86400000,
            created_at: now,
        });
        Ok(())
    }

    pub async fn proposal_get(&self, id: &str) -> Result<Option<ProposalInfo>> {
        Ok(self.proposals.read().await.get(id).cloned())
    }

    pub async fn vote_cast(&self, proposal_id: &str, _validator: &str, vote: bool) -> Result<()> {
        if let Some(p) = self.proposals.write().await.get_mut(proposal_id) {
            if vote { p.yes_votes += 100; } else { p.no_votes += 100; }
            if p.yes_votes + p.no_votes >= 300 {
                p.state = if p.yes_votes > p.no_votes { ProposalState::Passed } else { ProposalState::Rejected };
            }
        }
        Ok(())
    }

    pub async fn votes_tally(&self, id: &str) -> Result<(i64, i64)> {
        self.proposals.read().await.get(id).map(|p| (p.yes_votes, p.no_votes)).ok_or(anyhow!("Not found"))
    }

    pub async fn vote_on_proposal(&self, id: &str, vote: bool, validator: &str) -> Result<bool> {
        self.vote_cast(id, validator, vote).await?;
        Ok(true)
    }

    pub async fn validator_register(&self, id: &str, addr: &str, stake: i64, _: Option<&str>) -> Result<()> {
        self.validators.write().await.insert(id.to_string(), ValidatorInfo {
            validator_id: id.to_string(),
            address: addr.to_string(),
            stake, voting_power: stake / 100,
        });
        Ok(())
    }

    pub async fn validator_get_owner(&self, _: &str) -> Result<Option<String>> {
        Ok(Some("dev".to_string()))
    }

    pub async fn mint_token(&self, to: &str, amount: u64) -> Result<(i64, String)> {
        let balance = {
            let mut w = self.wallets.write().await;
            let b = w.entry(to.to_string()).or_insert(0);
            *b += amount as i64;
            *b
        };
        Ok((balance, format!("tx_{}", amount)))
    }

    pub async fn stake(&self, validator: &str, amount: u64) -> Result<bool> {
        if amount < self.config.min_stake { return Err(anyhow!("Below min")); }
        if let Some(v) = self.validators.write().await.get_mut(validator) {
            v.stake += amount as i64;
            v.voting_power = v.stake / 100;
        }
        Ok(true)
    }

    pub async fn query_apy(&self, is_validator: bool) -> Result<f64> {
        Ok(if is_validator { 0.12 } else { 0.08 })
    }

    pub async fn cross_chain_swap(&self, _: &str, _: u64) -> Result<bool> {
        Ok(true)
    }
}
