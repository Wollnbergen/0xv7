//! Persistence Module - Simplified for now

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PersistenceLayer {
    data: HashMap<String, Vec<u8>>,
}

impl PersistenceLayer {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn put(&mut self, key: String, value: Vec<u8>) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&Vec<u8>> {
        self.data.get(key)
    }
}
