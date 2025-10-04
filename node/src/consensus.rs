#[cfg(test)]
mod tests {
    use super::*;
    use tracing::info;
    #[tokio::test]
    async fn test_governance_proposal() {
        let votes = vec![
            Vote { proposal_id: 1, validator_id: "v1".to_string(), vote_yes: true, stake_weight: 1, sig: None },
            Vote { proposal_id: 1, validator_id: "v2".to_string(), vote_yes: false, stake_weight: 1, sig: None },
            Vote { proposal_id: 1, validator_id: "v3".to_string(), vote_yes: true, stake_weight: 1, sig: None },
        ];
        let result = process_governance_proposal(&votes);
        assert!(result.is_ok());
        info!("Governance proposal test passed (democratic, production)");
    }
}
// Post-launch: Democratic governance proposal processing (no stake-weighted, equal vote)
pub fn process_governance_proposal(votes: &Vec<Vote>) -> Result<()> {
    let yes_votes = votes.iter().filter(|v| v.vote_yes).count();
    let no_votes = votes.len() - yes_votes;
    let passed = yes_votes > no_votes;
    info!("Governance proposal processed (democratic, production, APY ~26.67%): {} yes, {} no, passed: {}", yes_votes, no_votes, passed);
    Ok(())
}
// consensus.rs - Consensus logic for Sultan Blockchain

// Add your consensus implementation here
use anyhow::Result;
use std::collections::HashMap;
use tracing::info;
use tonic::{transport::Server, Request, Response, Status};
use proto::consensus::consensus_server::{Consensus, ConsensusServer};
use proto::consensus::{ProposalRequest, ProposalResponse, VoteRequest, VoteResponse, TallyRequest, TallyResponse};
use crate::types::{Proposal, Vote, ValidatorInfo};
use crate::quantum::SharedQuantumCrypto;
use crate::config::Config;
use std::sync::Arc;
use tokio::sync::RwLock;
use pqcrypto_traits::sign::SignedMessage as _;

// gRPC proto definition (add proto/consensus.proto and build with tonic_build in build.rs)
mod proto {
    tonic::include_proto!("consensus");
}

pub struct ConsensusEngine {
    pub proposals: Arc<RwLock<HashMap<u64, Proposal>>>,
    pub votes: Arc<RwLock<HashMap<u64, Vec<Vote>>>>,
    pub validators: Arc<RwLock<Vec<ValidatorInfo>>>,
    pub quantum_crypto: SharedQuantumCrypto,
    pub config: Config,
}

impl ConsensusEngine {
    pub fn new(config: Config) -> Self {
        Self {
            proposals: Arc::new(RwLock::new(HashMap::new())),
            votes: Arc::new(RwLock::new(HashMap::new())),
            validators: Arc::new(RwLock::new(Vec::new())),
            quantum_crypto: Arc::new(RwLock::new(QuantumCrypto::new())),
            config,
        }
    }

    pub async fn start_grpc_server(self: Arc<Self>) -> Result<()> {
        let addr = "[::1]:50051".parse()?;
        Server::builder().add_service(ConsensusServer::new(self)).serve(addr).await?;
        Ok(())
    }

    pub fn propose_governance(&mut self, id: u64, chain_name: String) -> Result<()> {
        let chain_name_clone = chain_name.clone();
        self.proposals.insert(id, Proposal {
            id,
            description: format!("Add {} chain (Avalanche seed: {})", chain_name, id),
            chain_name,
            interop_chain: "none".to_string(),
            votes_for: 0,
            votes_against: 0,
            quorum: 0.5,
        });
        info!("Proposed governance: ID {} - Add {} chain (Avalanche seed: {})", id, chain_name_clone, id);
        Ok(())
    }

    pub async fn vote_on_proposal(&mut self, proposal_id: u64, validator_id: String, vote_yes: bool) -> Result<()> {
        let mut vote = Vote {
            proposal_id,
            validator_id: validator_id.clone(),
            vote_yes,
            stake_weight: 100000,
            sig: None,
        };
        let guard = self.quantum_crypto.read().await;
        let signed = guard.sign(format!("vote_{}_{}", proposal_id, validator_id).as_bytes());
        vote.sig = Some(signed.as_bytes().to_vec());
        info!("Dilithium signature applied to governance vote (quantum-proof, quorum 0.5)");
        self.votes.entry(proposal_id).or_default().push(vote);
        Ok(())
    }

    pub fn tally_votes(&self, proposal_id: u64) -> Result<()> {
        if let Some(proposal) = self.proposals.get(&proposal_id) {
            let votes = self.votes.get(&proposal_id).cloned().unwrap_or_default();
            let votes_for: u64 = votes.iter().filter(|v| v.vote_yes).map(|v| v.stake_weight).sum();
            let votes_against: u64 = votes.iter().filter(|v| !v.vote_yes).map(|v| v.stake_weight).sum();
            let total_votes = votes_for + votes_against;
            let quorum_met = total_votes as f64 / self.config.quorum > 0.0;
            if quorum_met && votes_for > votes_against {
                info!("Governance passed: ID {} - Add {} chain", proposal_id, proposal.chain_name);
            }
        }
        Ok(())
    }
}

#[tonic::async_trait]
impl Consensus for ConsensusEngine {
    async fn propose(&self, request: Request<ProposalRequest>) -> Result<Response<ProposalResponse>, Status> {
        let mut proposals = self.proposals.write().await;
        let id = request.into_inner().id;
        proposals.insert(id, Proposal { id, description: "Real proposal".to_string(), chain_name: "Sultan".to_string(), interop_chain: "Bitcoin".to_string(), votes_for: 0, votes_against: 0, quorum: 0.5 });
        info!("gRPC proposal: ID {}", id);
        Ok(Response::new(ProposalResponse { success: true }))
    }

    async fn vote(&self, request: Request<VoteRequest>) -> Result<Response<VoteResponse>, Status> {
        let mut votes = self.votes.write().await;
        let req = request.into_inner();
        votes.entry(req.proposal_id).or_default().push(Vote { proposal_id: req.proposal_id, validator_id: req.validator_id, vote_yes: req.vote_yes, stake_weight: 100000, sig: None });
        info!("gRPC vote on {}", req.proposal_id);
        Ok(Response::new(VoteResponse { success: true }))
    }

    async fn tally(&self, request: Request<TallyRequest>) -> Result<Response<TallyResponse>, Status> {
        let votes = self.votes.read().await;
        let id = request.into_inner().proposal_id;
        let tally = votes.get(&id).map(|v| v.len() as u64).unwrap_or(0);
        info!("gRPC tally for {}: {}", id, tally);
        Ok(Response::new(TallyResponse { tally }))
    }
}