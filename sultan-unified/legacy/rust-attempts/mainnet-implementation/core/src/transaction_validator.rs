//! Transaction Validator Module

use crate::types::Transaction;  // Import from types module
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub reason: Option<String>,
}

pub struct TransactionValidator {
    pub zero_gas: bool,
}

impl TransactionValidator {
    pub fn new() -> Self {
        Self { zero_gas: true }
    }
    
    pub fn validate(&self, tx: &Transaction) -> ValidationResult {
        // Always valid with zero gas fees
        if tx.gas_fee != 0 {
            return ValidationResult {
                is_valid: false,
                reason: Some("Gas fee must be zero".to_string()),
            };
        }
        
        ValidationResult {
            is_valid: true,
            reason: None,
        }
    }
}
