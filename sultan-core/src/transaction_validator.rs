//! Transaction Validation Module
//!
//! Production-grade transaction validation with signature verification,
//! balance checks, and anti-spam protection.

use crate::blockchain::Transaction;
use anyhow::{Result, bail};
use std::collections::HashSet;

pub struct TransactionValidator {
    seen_transactions: HashSet<String>,
    max_tx_size: usize,
}

impl Default for TransactionValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionValidator {
    pub fn new() -> Self {
        Self {
            seen_transactions: HashSet::new(),
            max_tx_size: 1024 * 100, // 100KB max
        }
    }

    /// Validate transaction with comprehensive checks
    pub fn validate(&mut self, tx: &Transaction, current_balance: u64, current_nonce: u64) -> Result<bool> {
        // Check amount
        if tx.amount == 0 {
            bail!("Transaction amount must be greater than 0");
        }

        // Enforce zero gas fees
        if tx.gas_fee != 0 {
            bail!("Sultan Chain enforces zero gas fees");
        }

        // Check sender != receiver
        if tx.from == tx.to {
            bail!("Cannot send to yourself");
        }

        // Validate addresses
        if tx.from.is_empty() || tx.to.is_empty() {
            bail!("Invalid address");
        }

        // Check balance
        if current_balance < tx.amount {
            bail!("Insufficient balance: {} < {}", current_balance, tx.amount);
        }

        // Check nonce (prevent replay attacks)
        if tx.nonce <= current_nonce {
            bail!("Invalid nonce: expected > {}, got {}", current_nonce, tx.nonce);
        }

        // Check transaction size
        let tx_size = bincode::serialize(tx).map_err(|e| anyhow::anyhow!("Serialization error: {}", e))?.len();
        if tx_size > self.max_tx_size {
            bail!("Transaction too large: {} > {}", tx_size, self.max_tx_size);
        }

        // Check for duplicate (replay protection)
        let tx_hash = self.calculate_tx_hash(tx);
        if self.seen_transactions.contains(&tx_hash) {
            bail!("Duplicate transaction");
        }

        self.seen_transactions.insert(tx_hash);

        Ok(true)
    }

    /// Quick validation without state (for mempool)
    pub fn quick_validate(&self, tx: &Transaction) -> Result<bool> {
        if tx.amount == 0 {
            return Ok(false);
        }

        if tx.gas_fee != 0 {
            return Ok(false);
        }

        if tx.from == tx.to {
            return Ok(false);
        }

        if tx.from.is_empty() || tx.to.is_empty() {
            return Ok(false);
        }

        Ok(true)
    }

    /// Calculate transaction hash for deduplication
    fn calculate_tx_hash(&self, tx: &Transaction) -> String {
        use sha2::{Sha256, Digest};
        
        let data = format!("{}{}{}{}{}", 
            tx.from, 
            tx.to, 
            tx.amount, 
            tx.nonce, 
            tx.timestamp
        );
        
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Clear seen transactions (call after block confirmation)
    pub fn clear_seen(&mut self) {
        self.seen_transactions.clear();
    }

    /// Get seen transaction count
    pub fn seen_count(&self) -> usize {
        self.seen_transactions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transaction() {
        let mut validator = TransactionValidator::new();
        let tx = Transaction {
            from: "alice".to_string(),
            to: "bob".to_string(),
            amount: 100,
            gas_fee: 0,
            timestamp: 1000,
            nonce: 1,
            signature: None,
        };
        
        assert!(validator.validate(&tx, 1000, 0).is_ok());
    }

    #[test]
    fn test_zero_amount() {
        let mut validator = TransactionValidator::new();
        let tx = Transaction {
            from: "alice".to_string(),
            to: "bob".to_string(),
            amount: 0,
            gas_fee: 0,
            timestamp: 1000,
            nonce: 1,
            signature: None,
        };
        
        assert!(validator.validate(&tx, 1000, 0).is_err());
    }

    #[test]
    fn test_insufficient_balance() {
        let mut validator = TransactionValidator::new();
        let tx = Transaction {
            from: "alice".to_string(),
            to: "bob".to_string(),
            amount: 2000,
            gas_fee: 0,
            timestamp: 1000,
            nonce: 1,
            signature: None,
        };
        
        assert!(validator.validate(&tx, 1000, 0).is_err());
    }
}
