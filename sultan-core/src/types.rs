use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address(pub String);

impl Address {
    pub fn new(addr: &str) -> Self {
        Address(addr.to_string())
    }
}

// Re-export Transaction from blockchain module for backwards compatibility
pub use crate::blockchain::Transaction;
