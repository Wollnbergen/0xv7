use serde_json::json;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

// Import blockchain types
use crate::blockchain::{Blockchain, Transaction};
use crate::config::ChainConfig;
use crate::consensus::Validator;
use crate::sdk_error::{SdkError, SdkResult};

pub struct SultanSDK {
    pub config: ChainConfig,
    pub blockchain: Arc<Mutex<Blockchain>>,
    // Make database optional for now
    pub wallets: Arc<Mutex<HashMap<String, i64>>>,
    proposals: Arc<Mutex<HashMap<u64, Proposal>>>,
}

#[derive(Clone)]
pub struct Proposal {
    id: u64,
    proposer: String,
    title: String,
    description: String,
    status: String,
    yes_votes: u64,
    no_votes: u64,
}

impl SultanSDK {
    pub async fn new(config: ChainConfig, _db_contact: Option<&str>) -> SdkResult<Self> {
        let blockchain = Arc::new(Mutex::new(Blockchain::new()));
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

    pub async fn create_wallet(&self, _owner: &str) -> SdkResult<String> {
        let address = format!("sultan1{}", uuid::Uuid::new_v4().simple());
        let mut wallets = self.wallets.lock()
            .map_err(|e| SdkError::LockPoisoned(e.to_string()))?;
        wallets.insert(address.clone(), 1000000);
        Ok(address)
    }

    pub async fn get_balance(&self, address: &str) -> SdkResult<i64> {
        let wallets = self.wallets.lock()
            .map_err(|e| SdkError::LockPoisoned(e.to_string()))?;
        Ok(*wallets.get(address).unwrap_or(&0))
    }

    pub async fn transfer(&self, from: &str, to: &str, amount: u64) -> SdkResult<String> {
        if amount == 0 {
            return Err(SdkError::InvalidAmount("Amount must be greater than zero".into()));
        }
        
        let tx_hash = format!("0x{:x}", rand::random::<u64>());
        
        // Update balances
        let mut wallets = self.wallets.lock()
            .map_err(|e| SdkError::LockPoisoned(e.to_string()))?;
        let from_balance = wallets.entry(from.to_string()).or_insert(0);
        
        if *from_balance < amount as i64 {
            return Err(SdkError::InsufficientBalance { 
                required: amount, 
                available: *from_balance 
            });
        }
        
        *from_balance -= amount as i64;
        let to_balance = wallets.entry(to.to_string()).or_insert(0);
        *to_balance += amount as i64;
        
        drop(wallets);
        
        // Add to blockchain
        let mut blockchain = self.blockchain.lock()
            .map_err(|e| SdkError::LockPoisoned(e.to_string()))?;
        let tx = Transaction {
            from: from.to_string(),
            to: to.to_string(),
            amount,
            gas_fee: 0,  // ZERO FEES!
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(std::time::Duration::from_secs(0))
                .as_secs(),
        };
        blockchain.add_transaction(tx);
        let _block = blockchain.create_block();
        
        Ok(tx_hash)
    }

    pub async fn stake(&self, validator: &str, amount: u64) -> SdkResult<bool> {
        if amount < self.config.min_stake {
            return Err(SdkError::BelowMinimumStake { 
                required: self.config.min_stake, 
                provided: amount 
            });
        }
        
        // In full implementation, this would add to consensus engine
        // For now, we just validate and return success
        let _val = Validator {
            address: validator.to_string(),
            stake: amount,
            voting_power: amount,
        };
        
        Ok(true)
    }

    pub async fn query_apy(&self, is_validator: bool) -> SdkResult<f64> {
        let base_apy = self.config.inflation_rate / 0.3;  // 8% / 0.3 = 26.67%
        Ok(if is_validator { 
            base_apy 
        } else { 
            base_apy * 0.8
        })
    }

    // Governance methods
    pub async fn proposal_create(&self, proposer: &str, title: &str, description: &str) -> SdkResult<u64> {
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
        
        let mut proposals = self.proposals.lock()
            .map_err(|e| SdkError::LockPoisoned(e.to_string()))?;
        proposals.insert(proposal_id, proposal);
        
        Ok(proposal_id)
    }
    
    pub async fn proposal_get(&self, id: u64) -> SdkResult<serde_json::Value> {
        let proposals = self.proposals.lock()
            .map_err(|e| SdkError::LockPoisoned(e.to_string()))?;
        
        if let Some(proposal) = proposals.get(&id) {
            Ok(json!({
                "id": proposal.id,
                "proposer": proposal.proposer,
                "title": proposal.title,
                "description": proposal.description,
                "status": proposal.status,
                "yes_votes": proposal.yes_votes,
                "no_votes": proposal.no_votes
            }))
        } else {
            Err(SdkError::ProposalNotFound(id))
        }
    }
    
    pub async fn votes_tally(&self, proposal_id: u64) -> SdkResult<(u64, u64)> {
        let proposals = self.proposals.lock()
            .map_err(|e| SdkError::LockPoisoned(e.to_string()))?;
        
        if let Some(proposal) = proposals.get(&proposal_id) {
            Ok((proposal.yes_votes, proposal.no_votes))
        } else {
            Err(SdkError::ProposalNotFound(proposal_id))
        }
    }
    
    pub async fn get_all_proposals(&self) -> SdkResult<Vec<serde_json::Value>> {
        let proposals = self.proposals.lock()
            .map_err(|e| SdkError::LockPoisoned(e.to_string()))?;
        let mut result = Vec::new();
        
        for proposal in proposals.values() {
            result.push(json!({
                "id": proposal.id,
                "proposer": proposal.proposer,
                "title": proposal.title,
                "description": proposal.description,
                "status": proposal.status,
                "yes_votes": proposal.yes_votes,
                "no_votes": proposal.no_votes
            }));
        }
        
        Ok(result)
    }
    
    pub async fn vote_on_proposal(&self, proposal_id: u64, _voter: &str, vote: bool) -> SdkResult<()> {
        let mut proposals = self.proposals.lock()
            .map_err(|e| SdkError::LockPoisoned(e.to_string()))?;
        
        if let Some(proposal) = proposals.get_mut(&proposal_id) {
            if vote {
                proposal.yes_votes += 1;
            } else {
                proposal.no_votes += 1;
            }
            Ok(())
        } else {
            Err(SdkError::ProposalNotFound(proposal_id))
        }
    }
    
    // Token methods
    pub async fn mint_token(&self, to: &str, amount: u64) -> SdkResult<String> {
        if amount == 0 {
            return Err(SdkError::InvalidAmount("Amount must be greater than zero".into()));
        }
        
        let mut wallets = self.wallets.lock()
            .map_err(|e| SdkError::LockPoisoned(e.to_string()))?;
        let balance = wallets.entry(to.to_string()).or_insert(0);
        *balance += amount as i64;
        
        Ok(format!("0x{:x}", rand::random::<u64>()))
    }
    
    pub async fn validator_register(&self, validator_addr: &str, stake: u64) -> SdkResult<String> {
        self.stake(validator_addr, stake).await?;
        Ok(format!("Validator {} registered with {} stake", validator_addr, stake))
    }
    
    // Query methods for dApps
    pub async fn get_block_height(&self) -> SdkResult<u64> {
        let blockchain = self.blockchain.lock()
            .map_err(|e| SdkError::LockPoisoned(e.to_string()))?;
        Ok(blockchain.chain.len() as u64)
    }
    
    pub async fn get_transaction_count(&self, address: &str) -> SdkResult<u64> {
        let blockchain = self.blockchain.lock()
            .map_err(|e| SdkError::LockPoisoned(e.to_string()))?;
        let count = blockchain.chain.iter()
            .flat_map(|b| &b.transactions)
            .filter(|tx| tx.from == address)
            .count();
        Ok(count as u64)
    }
    
    pub async fn list_wallets(&self) -> SdkResult<Vec<String>> {
        let wallets = self.wallets.lock()
            .map_err(|e| SdkError::LockPoisoned(e.to_string()))?;
        Ok(wallets.keys().cloned().collect())
    }
    
    // IBC methods for Cosmos ecosystem interoperability
    pub async fn ibc_transfer(
        &self,
        from: &str,
        to_address: &str,
        amount: u64,
        channel_id: &str
    ) -> SdkResult<String> {
        if amount == 0 {
            return Err(SdkError::InvalidAmount("Amount must be greater than zero".into()));
        }
        
        // Validate channel format
        if !channel_id.starts_with("channel-") && !channel_id.starts_with("transfer/channel-") {
            return Err(SdkError::InvalidAddress(format!("Invalid IBC channel: {}", channel_id)));
        }
        
        // Check balance
        let wallets = self.wallets.lock()
            .map_err(|e| SdkError::LockPoisoned(e.to_string()))?;
        let from_balance = wallets.get(from).ok_or_else(|| SdkError::WalletNotFound(from.to_string()))?;
        
        if *from_balance < amount as i64 {
            return Err(SdkError::InsufficientBalance { 
                required: amount, 
                available: *from_balance 
            });
        }
        drop(wallets);
        
        // In production, this would call IBC transfer module
        // For now, return transaction hash
        let tx_hash = format!("0x{:x}", rand::random::<u64>());
        Ok(tx_hash)
    }
    
    pub async fn ibc_query_channels(&self) -> SdkResult<Vec<serde_json::Value>> {
        // In production, query IBC keeper for active channels
        Ok(vec![
            json!({
                "channel_id": "channel-0",
                "port_id": "transfer",
                "counterparty_chain": "osmosis-1",
                "state": "OPEN"
            }),
            json!({
                "channel_id": "channel-1",
                "port_id": "transfer",
                "counterparty_chain": "cosmoshub-4",
                "state": "OPEN"
            })
        ])
    }
    
    // Batch operations for efficiency
    pub async fn batch_transfer(
        &self,
        from: &str,
        transfers: Vec<(String, u64)>
    ) -> SdkResult<Vec<String>> {
        let mut tx_hashes = Vec::new();
        
        for (to, amount) in transfers {
            let tx_hash = self.transfer(from, &to, amount).await?;
            tx_hashes.push(tx_hash);
        }
        
        Ok(tx_hashes)
    }
    
    // Transaction history for wallets and explorers
    pub async fn get_transaction_history(
        &self,
        address: &str,
        limit: Option<usize>
    ) -> SdkResult<Vec<serde_json::Value>> {
        let blockchain = self.blockchain.lock()
            .map_err(|e| SdkError::LockPoisoned(e.to_string()))?;
        
        let mut txs: Vec<serde_json::Value> = blockchain.chain.iter()
            .flat_map(|b| &b.transactions)
            .filter(|tx| tx.from == address || tx.to == address)
            .map(|tx| json!({
                "from": tx.from,
                "to": tx.to,
                "amount": tx.amount,
                "gas_fee": tx.gas_fee,
                "timestamp": tx.timestamp
            }))
            .collect();
        
        // Apply limit if specified
        if let Some(max) = limit {
            txs.truncate(max);
        }
        
        Ok(txs)
    }
    
    // Get specific transaction by hash
    pub async fn get_transaction(&self, tx_hash: &str) -> SdkResult<serde_json::Value> {
        let blockchain = self.blockchain.lock()
            .map_err(|e| SdkError::LockPoisoned(e.to_string()))?;
        
        // In production, maintain tx hash -> tx mapping
        // For now, return first transaction if any exist
        if let Some(block) = blockchain.chain.first() {
            if let Some(tx) = block.transactions.first() {
                return Ok(json!({
                    "hash": tx_hash,
                    "from": tx.from,
                    "to": tx.to,
                    "amount": tx.amount,
                    "gas_fee": tx.gas_fee,
                    "timestamp": tx.timestamp,
                    "status": "confirmed"
                }));
            }
        }
        
        Err(SdkError::TransactionFailed("Transaction not found".into()))
    }
    
    // Advanced validator queries
    pub async fn get_validator_set(&self) -> SdkResult<Vec<serde_json::Value>> {
        // In production, query consensus engine
        Ok(vec![
            json!({
                "address": "sultanvaloper1...",
                "voting_power": "1000000",
                "commission": "0.05",
                "status": "active"
            })
        ])
    }
    
    pub async fn get_delegations(&self, delegator: &str) -> SdkResult<Vec<serde_json::Value>> {
        Ok(vec![
            json!({
                "validator": "sultanvaloper1...",
                "amount": "10000",
                "rewards": "267"
            })
        ])
    }
}

// Alias for compatibility
pub type SDK = SultanSDK;
