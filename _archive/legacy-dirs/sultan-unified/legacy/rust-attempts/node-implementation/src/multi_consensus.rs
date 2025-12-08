use sha2::Digest;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

pub struct MultiNodeConsensus {
    node_id: String,
    peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
    current_round: u64,
    votes: Arc<RwLock<HashMap<u64, Vec<Vote>>>>,
}

#[derive(Debug, Clone)]
struct PeerInfo {
    id: String,
    stake: u64,
    is_validator: bool,
}

#[derive(Debug, Clone)]
struct Vote {
    node_id: String,
    block_hash: String,
    round: u64,
    signature: Vec<u8>,
}

impl MultiNodeConsensus {
    pub fn new(node_id: String) -> Self {
        MultiNodeConsensus {
            node_id,
            peers: Arc::new(RwLock::new(HashMap::new())),
            current_round: 0,
            votes: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn propose_block(&self, block_data: Vec<u8>) -> Result<String> {
        // Byzantine Fault Tolerant consensus
        // Need 2/3 + 1 votes to finalize
        let block_hash = format!("0x{:x}", {
        use sha2::Digest;
        sha2::Sha256::digest(&block_data)
    });
        
        println!("�� Proposing block: {}", block_hash);
        
        // Broadcast to all validators
        self.broadcast_proposal(block_hash.clone()).await?;
        
        // Wait for votes
        let votes_needed = (self.peers.read().await.len() * 2 / 3) + 1;
        
        Ok(block_hash)
    }
    
    async fn broadcast_proposal(&self, block_hash: String) -> Result<()> {
        // Send via P2P network
        println!("📡 Broadcasting block proposal: {}", block_hash);
        Ok(())
    }
    
    pub async fn receive_vote(&self, vote: Vote) -> Result<()> {
        let mut votes = self.votes.write().await;
        votes.entry(vote.round).or_insert_with(Vec::new).push(vote);
        
        // Check if we have enough votes
        if let Some(round_votes) = votes.get(&self.current_round) {
            let vote_count = round_votes.len();
            let total_validators = self.peers.read().await.len() + 1; // +1 for self
            
            if vote_count >= (total_validators * 2 / 3) + 1 {
                println!("✅ Block finalized with {}/{} votes", vote_count, total_validators);
            }
        }
        
        Ok(())
    }
}
use sha2::Digest;
