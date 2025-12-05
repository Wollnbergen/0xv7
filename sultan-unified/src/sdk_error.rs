use thiserror::Error;

#[derive(Error, Debug)]
pub enum SdkError {
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: u64, available: i64 },
    
    #[error("Invalid address format: {0}")]
    InvalidAddress(String),
    
    #[error("Wallet not found: {0}")]
    WalletNotFound(String),
    
    #[error("Proposal not found: {0}")]
    ProposalNotFound(u64),
    
    #[error("Below minimum stake: required {required}, provided {provided}")]
    BelowMinimumStake { required: u64, provided: u64 },
    
    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),
    
    #[error("Blockchain error: {0}")]
    BlockchainError(String),
    
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    
    #[error("Validator already exists: {0}")]
    ValidatorExists(String),
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
}

pub type SdkResult<T> = Result<T, SdkError>;
