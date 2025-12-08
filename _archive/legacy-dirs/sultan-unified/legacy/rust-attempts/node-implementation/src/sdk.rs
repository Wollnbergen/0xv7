use anyhow::Result;
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

// Import blockchain types
use crate::blockchain::{Blockchain, ChainConfig, Transaction, Validator};

pub struct SultanSDK {
    pub config: ChainConfig,
    pub blockchain: Arc<Mutex<Blockchain>>,
    // Make database optional for now
    pub wallets: Arc<Mutex<HashMap<String, i64>>>,
    pub proposals: Arc<Mutex<HashMap<u64, Proposal>>>,
}

#[derive(Clone)]
struct Proposal {
    id: u64,
    proposer: String,
    title: String,
    description: String,
    status: String,
    yes_votes: u64,
    no_votes: u64,
}

impl SultanSDK {
    pub async fn new(config: ChainConfig, _db_contact: Option<&str>) -> Result<Self> {
        let blockchain = Arc::new(Mutex::new(Blockchain::new(config.clone())));
        let mut wallets = HashMap::new();
        
        // Initialize with some test wallets
        wallets.insert("alice".to_string(), 1000000);
        wallets.insert("bob".to_string(), 1000000);
        wallets.insert("validator1".to_string(), 5000000);
        
        Ok(SultanSDK {
            config,
            blockchain,
            wallets: Arc::new(Mutex::new(wallets)),
            proposals: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn create_wallet(&self, owner: &str) -> Result<String> {
        let address = format!("sultan1{}", uuid::Uuid::new_v4().simple());
        let mut wallets = self.wallets.lock().unwrap();
        wallets.insert(address.clone(), 1000000);
        Ok(address)
    }

    pub async fn get_balance(&self, address: &str) -> Result<i64> {
        let wallets = self.wallets.lock().unwrap();
        Ok(*wallets.get(address).unwrap_or(&0))
    }

    pub async fn transfer(&self, from: &str, to: &str, amount: u64) -> Result<String> {
        let tx_hash = format!("0x{:x}", rand::random::<u64>());
        
        // Update balances
        let mut wallets = self.wallets.lock().unwrap();
        let from_balance = wallets.entry(from.to_string()).or_insert(0);
        
        if *from_balance < amount as i64 {
            return Err(anyhow::anyhow!("Insufficient balance"));
        }
        
        *from_balance -= amount as i64;
        let to_balance = wallets.entry(to.to_string()).or_insert(0);
        *to_balance += amount as i64;
        
        drop(wallets);
        
        // Add to blockchain
        let mut blockchain = self.blockchain.lock().unwrap();
        let tx = Transaction {
            from: from.to_string(),
            to: to.to_string(),
            amount,
            fee: 0,  // ZERO FEES!
            nonce: 1,
            signature: vec![],
        };
        blockchain.produce_block(vec![tx])?;
        
        Ok(tx_hash)
    }

    pub async fn stake(&self, validator: &str, amount: u64) -> Result<bool> {
        if amount < self.config.min_stake {
            return Err(anyhow::anyhow!("Below minimum stake"));
        }
        
        let mut blockchain = self.blockchain.lock().unwrap();
        blockchain.add_validator(Validator {
            address: validator.to_string(),
            stake: amount,
            is_mobile: false,
            commission: 0.10,
        })?;
        
        Ok(true)
    }

    pub async fn query_apy(&self, is_validator: bool) -> Result<f64> {
        let base_apy = self.config.inflation_rate / 0.3;  // 4% / 0.3 = 13.33%
        Ok(if is_validator { 
            base_apy 
        } else { 
            base_apy * 0.8
        })
    }

    // Governance methods
    pub async fn proposal_create(&self, proposer: &str, title: &str, description: &str) -> Result<u64> {
        let proposal_id = rand::random::<u64>() % 1000;
        
        let proposal = Proposal {
            id: proposal_id,
            proposer: proposer.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            status: "active".to_string(),
            yes_votes: 0,
            no_votes: 0,
        };
        
        let mut proposals = self.proposals.lock().unwrap();
        proposals.insert(proposal_id, proposal);
        
        Ok(proposal_id)
    }
    
    pub async fn proposal_get(&self, id: u64) -> Result<serde_json::Value> {
        let proposals = self.proposals.lock().unwrap();
        
        if let Some(proposal) = proposals.get(&id) {
            Ok(json!({
                "id": proposal.id,
                "title": proposal.title,
                "status": proposal.status,
                "yes_votes": proposal.yes_votes,
                "no_votes": proposal.no_votes
            }))
        } else {
            Ok(json!({
                "id": id,
                "title": "Not found",
                "status": "unknown"
            }))
        }
    }
    
    pub async fn votes_tally(&self, proposal_id: u64) -> Result<(u64, u64)> {
        let proposals = self.proposals.lock().unwrap();
        
        if let Some(proposal) = proposals.get(&proposal_id) {
            Ok((proposal.yes_votes, proposal.no_votes))
        } else {
            Ok((0, 0))
        }
    }
    
    pub async fn get_all_proposals(&self) -> Result<Vec<serde_json::Value>> {
        let proposals = self.proposals.lock().unwrap();
        let mut result = Vec::new();
        
        for proposal in proposals.values() {
            result.push(json!({
                "id": proposal.id,
                "title": proposal.title,
                "status": proposal.status
            }));
        }
        
        Ok(result)
    }
    
    pub async fn vote_on_proposal(&self, proposal_id: u64, _voter: &str, vote: bool) -> Result<()> {
        let mut proposals = self.proposals.lock().unwrap();
        
        if let Some(proposal) = proposals.get_mut(&proposal_id) {
            if vote {
                proposal.yes_votes += 1;
            } else {
                proposal.no_votes += 1;
            }
        }
        
        Ok(())
    }
    
    // Token methods
    pub async fn mint_token(&self, to: &str, amount: u64) -> Result<String> {
        let mut wallets = self.wallets.lock().unwrap();
        let balance = wallets.entry(to.to_string()).or_insert(0);
        *balance += amount as i64;
        
        Ok(format!("0x{:x}", rand::random::<u64>()))
    }
}

// Alias for compatibility
pub type SDK = SultanSDK;

    pub async fn validator_register(&self, validator_addr: &str, stake: u64) -> Result<String> {
        // Register a new validator
        Ok(format!("Validator {} registered with {} stake", validator_addr, stake))
    }
