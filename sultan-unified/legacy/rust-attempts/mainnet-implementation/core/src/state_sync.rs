//! State Sync Module

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub height: u64,
    pub hash: String,
    pub accounts: HashMap<String, u64>,
}

pub struct StateSync {
    pub latest_snapshot: Option<StateSnapshot>,
}

impl StateSync {
    pub fn new() -> Self {
        Self {
            latest_snapshot: None,
        }
    }
    
    pub fn create_snapshot(&mut self, height: u64) -> StateSnapshot {
        let snapshot = StateSnapshot {
            height,
            hash: format!("snapshot_{}", height),
            accounts: HashMap::new(),
        };
        self.latest_snapshot = Some(snapshot.clone());
        snapshot
    }
}
