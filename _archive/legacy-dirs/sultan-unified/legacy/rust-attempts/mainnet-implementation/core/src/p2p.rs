//! P2P Network Module - Simplified

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,
    pub address: String,
}

pub struct P2PNetwork {
    pub peers: HashSet<String>,
    pub local_peer_id: String,
}

impl P2PNetwork {
    pub fn new() -> Self {
        Self {
            peers: HashSet::new(),
            local_peer_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    pub fn add_peer(&mut self, peer_id: String) {
        self.peers.insert(peer_id);
    }

    pub fn remove_peer(&mut self, peer_id: &str) {
        self.peers.remove(peer_id);
    }

    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }
}
