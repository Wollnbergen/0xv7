// Temporary file to show database updates needed
// These changes need to be added to the existing database.rs

use std::collections::HashMap;
use crate::token_transfer::Transfer;

// Add to the Database struct:
pub struct Database {
    // ... existing fields ...
    pub transfers: HashMap<String, Transfer>,  // NEW: Transfer history
    pub reward_claims: HashMap<String, Vec<i64>>, // NEW: Track reward claim timestamps
}

// Add to the Database::new() implementation:
impl Database {
    pub fn new() -> Self {
        Self {
            // ... existing fields ...
            transfers: HashMap::new(),
            reward_claims: HashMap::new(),
        }
    }
}
