//! Global state management for FFI bridge
//!
//! Thread-safe singleton for managing Sultan blockchain instances
//! across the FFI boundary.

use sultan_core::{Blockchain, ConsensusEngine};
use parking_lot::RwLock;
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Global state container
static BRIDGE_STATE: Lazy<RwLock<BridgeState>> = Lazy::new(|| {
    RwLock::new(BridgeState::new())
});

/// Bridge state managing multiple blockchain instances
pub struct BridgeState {
    blockchains: HashMap<usize, Blockchain>,
    consensus_engines: HashMap<usize, ConsensusEngine>,
    next_id: usize,
}

impl BridgeState {
    fn new() -> Self {
        BridgeState {
            blockchains: HashMap::new(),
            consensus_engines: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn add_blockchain(&mut self, blockchain: Blockchain) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.blockchains.insert(id, blockchain);
        id
    }

    pub fn get_blockchain(&self, id: usize) -> Option<&Blockchain> {
        self.blockchains.get(&id)
    }

    pub fn get_blockchain_mut(&mut self, id: usize) -> Option<&mut Blockchain> {
        self.blockchains.get_mut(&id)
    }

    pub fn remove_blockchain(&mut self, id: usize) -> Option<Blockchain> {
        self.blockchains.remove(&id)
    }

    pub fn add_consensus(&mut self, consensus: ConsensusEngine) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.consensus_engines.insert(id, consensus);
        id
    }

    pub fn get_consensus(&self, id: usize) -> Option<&ConsensusEngine> {
        self.consensus_engines.get(&id)
    }

    pub fn get_consensus_mut(&mut self, id: usize) -> Option<&mut ConsensusEngine> {
        self.consensus_engines.get_mut(&id)
    }

    pub fn remove_consensus(&mut self, id: usize) -> Option<ConsensusEngine> {
        self.consensus_engines.remove(&id)
    }
}

/// Get global bridge state
pub fn get_state() -> &'static RwLock<BridgeState> {
    &BRIDGE_STATE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_state_blockchain() {
        let mut state = BridgeState::new();
        let blockchain = Blockchain::new();
        
        let id = state.add_blockchain(blockchain);
        assert!(state.get_blockchain(id).is_some());
        
        let removed = state.remove_blockchain(id);
        assert!(removed.is_some());
        assert!(state.get_blockchain(id).is_none());
    }

    #[test]
    fn test_bridge_state_consensus() {
        let mut state = BridgeState::new();
        let consensus = ConsensusEngine::new();
        
        let id = state.add_consensus(consensus);
        assert!(state.get_consensus(id).is_some());
        
        let removed = state.remove_consensus(id);
        assert!(removed.is_some());
        assert!(state.get_consensus(id).is_none());
    }
}
