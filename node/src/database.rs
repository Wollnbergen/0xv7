//! Production Database Module for Sultan Chain
//! Fully decentralized - each node has its own database

use anyhow::{anyhow, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

// Mock database for now - will add real Scylla later
pub struct Database {
    keyspace: String,
    proposals: Arc<RwLock<std::collections::HashMap<String, ProposalInfo>>>,
    votes: Arc<RwLock<std::collections::HashMap<String, Vec<Vote>>>>,
    wallets: Arc<RwLock<std::collections::HashMap<String, i64>>>,
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
    pub voting_start: i64,
    pub voting_end: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct Vote {
    pub proposal_id: String,
    pub voter_address: String,
    pub vote: bool,
    pub voting_power: i64,
    pub signature: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProposalState {
    Draft,
    Active,
    Passed,
    Rejected,
    Executed,
}

impl ProposalState {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "draft" => ProposalState::Draft,
            "active" => ProposalState::Active,
            "passed" => ProposalState::Passed,
            "rejected" => ProposalState::Rejected,
            "executed" => ProposalState::Executed,
            _ => ProposalState::Draft,
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            ProposalState::Draft => "draft".to_string(),
            ProposalState::Active => "active".to_string(),
            ProposalState::Passed => "passed".to_string(),
            ProposalState::Rejected => "rejected".to_string(),
            ProposalState::Executed => "executed".to_string(),
        }
    }
}

impl Database {
    pub async fn new(_contact_points: Vec<&str>, keyspace: &str) -> Result<Self> {
        info!("🔗 Initializing database layer for {}", keyspace);
        
        Ok(Database {
            keyspace: keyspace.to_string(),
            proposals: Arc::new(RwLock::new(std::collections::HashMap::new())),
            votes: Arc::new(RwLock::new(std::collections::HashMap::new())),
            wallets: Arc::new(RwLock::new(std::collections::HashMap::new())),
        })
    }
    
    pub async fn run_migrations(&self) -> Result<()> {
        info!("✅ Mock migrations completed for {}", self.keyspace);
        Ok(())
    }
    
    pub async fn save_proposal(&self, proposal: &ProposalInfo) -> Result<()> {
        let mut proposals = self.proposals.write().await;
        proposals.insert(proposal.proposal_id.clone(), proposal.clone());
        info!("💾 Proposal {} saved", proposal.proposal_id);
        Ok(())
    }
    
    pub async fn get_proposal(&self, id: &str) -> Result<Option<ProposalInfo>> {
        let proposals = self.proposals.read().await;
        Ok(proposals.get(id).cloned())
    }
    
    pub async fn save_vote(&self, vote: &Vote) -> Result<()> {
        let mut votes = self.votes.write().await;
        votes.entry(vote.proposal_id.clone())
            .or_insert_with(Vec::new)
            .push(vote.clone());
        info!("🗳️ Vote saved for proposal {}", vote.proposal_id);
        Ok(())
    }
    
    pub async fn update_wallet_balance(&self, address: &str, balance: i64) -> Result<()> {
        let mut wallets = self.wallets.write().await;
        wallets.insert(address.to_string(), balance);
        Ok(())
    }
    
    pub async fn get_wallet_balance(&self, address: &str) -> Result<i64> {
        let wallets = self.wallets.read().await;
        Ok(wallets.get(address).copied().unwrap_or(1000))
    }
}
