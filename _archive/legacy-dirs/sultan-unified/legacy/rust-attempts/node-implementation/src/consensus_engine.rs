use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::blockchain::{Blockchain, Block};

pub enum ConsensusState {
    Idle,
    Proposing,
    Voting,
    Committing,
}

pub struct ConsensusEngine {
    blockchain: Arc<Mutex<Blockchain>>,
    state: ConsensusState,
    round: u64,
    validators: Vec<String>,
    votes: Vec<(String, bool)>, // (validator, vote)
}

impl ConsensusEngine {
    pub fn new(blockchain: Arc<Mutex<Blockchain>>) -> Self {
        ConsensusEngine {
            blockchain,
            state: ConsensusState::Idle,
            round: 0,
            validators: Vec::new(),
            votes: Vec::new(),
        }
    }
    
    pub async fn propose_block(&mut self, transactions: Vec<String>) -> Result<Block> {
        self.state = ConsensusState::Proposing;
        
        let mut blockchain = self.blockchain.lock().await;
        let block = blockchain.add_block(transactions).await?;
        
        self.state = ConsensusState::Voting;
        Ok(block)
    }
    
    pub async fn vote_on_block(&mut self, block_hash: &str, approve: bool) -> Result<()> {
        // Record vote
        let validator = "validator1"; // TODO: Get from identity
        self.votes.push((validator.to_string(), approve));
        
        // Check if we have 2/3 majority
        let approvals = self.votes.iter().filter(|(_, v)| *v).count();
        let required = (self.validators.len() * 2) / 3 + 1;
        
        if approvals >= required {
            self.state = ConsensusState::Committing;
            self.commit_block().await?;
        }
        
        Ok(())
    }
    
    async fn commit_block(&mut self) -> Result<()> {
        // Block is already added, just finalize
        self.state = ConsensusState::Idle;
        self.round += 1;
        self.votes.clear();
        Ok(())
    }
}
