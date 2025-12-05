//! Sultan SDK Module

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SultanSDK {
    pub version: String,
    pub chain_id: String,
}

impl SultanSDK {
    pub fn new() -> Self {
        Self {
            version: "1.0.0".to_string(),
            chain_id: "sultan-1".to_string(),
        }
    }
}

impl Default for SultanSDK {
    fn default() -> Self {
        Self::new()
    }
}
