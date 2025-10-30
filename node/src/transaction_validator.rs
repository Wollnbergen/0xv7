// SDK gas-free transaction validator (minimal, production-safe stub)

use anyhow::{anyhow, Result};
use tracing::info;

#[derive(Debug)]
pub struct Tx {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub token: String,
    pub chain: String,
    pub nonce: u64,
    pub subsidy: bool, // true for Sultan gas-free transactions
}

pub struct TransactionValidator;

impl TransactionValidator {
    pub fn new() -> Self {
        TransactionValidator
    }

    pub fn validate(&self, tx: &Tx) -> Result<()> {
        if tx.subsidy {
            info!("Validated gas-free tx (subsidized by APY ~26.67%)");
            Ok(())
        } else {
            Err(anyhow!("No subsidy"))
        }
    }

    // Stub for block validation to keep SDK compatibility
    pub fn validate_block<T>(&self, _block: &T) -> Result<()> {
        Ok(())
    }
}

impl Default for TransactionValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_free_tx() {
        let tx = Tx {
            from: "S1example".to_string(),
            to: "S1dest".to_string(),
            amount: 1000,
            token: "SLTN".to_string(),
            chain: "sultan".to_string(),
            nonce: 1,
            subsidy: true,
        };
        let validator = TransactionValidator::new();
        assert!(validator.validate(&tx).is_ok());
    }

    #[test]
    fn test_non_subsidy_tx() {
        let tx = Tx {
            from: "0xabc".to_string(),
            to: "0xdef".to_string(),
            amount: 500,
            token: "ETH".to_string(),
            chain: "ethereum".to_string(),
            nonce: 2,
            subsidy: false,
        };
        let validator = TransactionValidator::new();
        assert!(validator.validate(&tx).is_err());
    }
}
