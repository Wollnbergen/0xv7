// node/src/transaction_validator.rs - SDK gas-free (minimal stub)

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
    pub subsidy: bool, // true for Sultan gas-free tx
}

pub struct TransactionValidator;

impl TransactionValidator {
    pub fn new() -> Self {
        TransactionValidator
    }

    pub fn validate(&self, tx: &Tx) -> Result<()> {
        if tx.subsidy {
            // Gas-free on Sultan
            info!("Validated gas-free tx (subsidized by APY ~26.67%)");
            Ok(())
        } else {
            Err(anyhow!("No subsidy"))
        }
    }

    // Stub for block validation (for SDK compatibility)
    pub fn validate_block<T>(&self, _block: &T) -> Result<()> {
        Ok(())
    }
}

// Removed unused: sultan_interop::zk_proofs::StateProof (ZK postponed Q3 2026)

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
