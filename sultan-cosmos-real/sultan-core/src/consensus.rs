//! Consensus Module

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub address: String,
    pub stake: u64,
    pub voting_power: u64,
}

pub struct ConsensusEngine {
    pub validators: HashMap<String, Validator>,
    pub current_proposer: Option<String>,
    pub round: u64,
}

impl Default for ConsensusEngine {
    fn default() -> Self { Self::new() }
}

impl ConsensusEngine {
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
            current_proposer: None,
            round: 0,
        }
    }

    pub fn add_validator(&mut self, validator: Validator) {
        self.validators.insert(validator.address.clone(), validator);
    }

    pub fn select_proposer(&mut self) -> Option<String> {
        // Simple round-robin for now
        let validators: Vec<String> = self.validators.keys().cloned().collect();
        if !validators.is_empty() {
            let index = (self.round as usize) % validators.len();
            self.current_proposer = Some(validators[index].clone());
            self.round += 1;
            self.current_proposer.clone()
        } else {
            None
        }
    }
}
