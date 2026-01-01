//! Transaction Validation Module
//!
//! Production-grade transaction validation with:
//! - Ed25519 cryptographic signature verification
//! - Balance and nonce checks (replay protection)
//! - Zero-gas policy enforcement (Sultan Chain)
//! - Anti-spam protection (size limits, rate limiting, deduplication)
//! - Address format validation with Sultan prefix support
//! - Custom typed errors for better error handling

use crate::blockchain::Transaction;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::fmt;
use sha2::{Sha256, Digest};
use ed25519_dalek::{Verifier, VerifyingKey, Signature};
use tracing::{debug, warn};

/// Rate limit: max transactions per sender per block window
pub const MAX_TXS_PER_SENDER: usize = 100;
/// Maximum transaction size in bytes
pub const MAX_TX_SIZE: usize = 100 * 1024; // 100KB
/// Minimum address length
pub const MIN_ADDRESS_LENGTH: usize = 10;
/// Maximum address length  
pub const MAX_ADDRESS_LENGTH: usize = 128;
/// Sultan Chain address prefix (optional validation)
pub const SULTAN_ADDRESS_PREFIX: &str = "sultan";
/// Maximum nonce gap allowed
pub const MAX_NONCE_GAP: u64 = 100;
/// Timestamp future tolerance (5 minutes)
pub const TIMESTAMP_FUTURE_TOLERANCE_SECS: u64 = 300;
/// Timestamp past tolerance (1 hour)
pub const TIMESTAMP_PAST_TOLERANCE_SECS: u64 = 3_600;

/// Custom validation errors for better error handling
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    // Amount errors
    ZeroAmount,
    InsufficientBalance { have: u64, need: u64 },
    
    // Policy errors
    NonZeroGasFee { fee: u64 },
    SelfSend,
    
    // Address errors
    EmptyAddress { field: String },
    AddressTooShort { field: String, len: usize, min: usize },
    AddressTooLong { field: String, len: usize, max: usize },
    InvalidAddressChars { field: String },
    InvalidAddressPrefix { field: String, expected: String },
    
    // Nonce errors
    InvalidNonce { expected_gt: u64, got: u64 },
    NonceTooFarAhead { gap: u64, max_gap: u64 },
    
    // Timestamp errors
    TimestampInFuture,
    TimestampTooOld,
    
    // Signature errors
    MissingSignature,
    MissingPublicKey,
    InvalidSignatureLength { expected: usize, got: usize },
    InvalidPublicKeyLength { expected: usize, got: usize },
    InvalidSignatureHex(String),
    InvalidPublicKeyHex(String),
    InvalidPublicKey(String),
    SignatureVerificationFailed(String),
    
    // Anti-spam errors
    TransactionTooLarge { size: usize, max: usize },
    RateLimitExceeded { sender: String, count: usize, max: usize },
    DuplicateTransaction,
    
    // Serialization errors
    SerializationError(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroAmount => write!(f, "Transaction amount must be greater than 0"),
            Self::InsufficientBalance { have, need } => 
                write!(f, "Insufficient balance: have {}, need {}", have, need),
            Self::NonZeroGasFee { fee } => 
                write!(f, "Sultan Chain enforces zero gas fees, got {}", fee),
            Self::SelfSend => write!(f, "Cannot send to yourself"),
            Self::EmptyAddress { field } => 
                write!(f, "Invalid {}: address cannot be empty", field),
            Self::AddressTooShort { field, len, min } => 
                write!(f, "Invalid {}: address too short ({} chars, min {})", field, len, min),
            Self::AddressTooLong { field, len, max } => 
                write!(f, "Invalid {}: address too long ({} chars, max {})", field, len, max),
            Self::InvalidAddressChars { field } => 
                write!(f, "Invalid {}: address contains invalid characters", field),
            Self::InvalidAddressPrefix { field, expected } => 
                write!(f, "Invalid {}: address should start with '{}'", field, expected),
            Self::InvalidNonce { expected_gt, got } => 
                write!(f, "Invalid nonce: expected > {}, got {}", expected_gt, got),
            Self::NonceTooFarAhead { gap, max_gap } => 
                write!(f, "Nonce too far ahead: gap {} exceeds max {}", gap, max_gap),
            Self::TimestampInFuture => write!(f, "Transaction timestamp is in the future"),
            Self::TimestampTooOld => write!(f, "Transaction timestamp is too old (>1 hour)"),
            Self::MissingSignature => write!(f, "Missing signature"),
            Self::MissingPublicKey => write!(f, "Missing public key"),
            Self::InvalidSignatureLength { expected, got } => 
                write!(f, "Invalid signature length: expected {} hex chars, got {}", expected, got),
            Self::InvalidPublicKeyLength { expected, got } => 
                write!(f, "Invalid public key length: expected {} hex chars, got {}", expected, got),
            Self::InvalidSignatureHex(e) => write!(f, "Invalid signature hex: {}", e),
            Self::InvalidPublicKeyHex(e) => write!(f, "Invalid public key hex: {}", e),
            Self::InvalidPublicKey(e) => write!(f, "Invalid public key: {}", e),
            Self::SignatureVerificationFailed(e) => write!(f, "Signature verification failed: {}", e),
            Self::TransactionTooLarge { size, max } => 
                write!(f, "Transaction too large: {} bytes > {} max", size, max),
            Self::RateLimitExceeded { sender, count, max } => 
                write!(f, "Rate limit exceeded for {}: {} txs (max {})", sender, count, max),
            Self::DuplicateTransaction => write!(f, "Duplicate transaction"),
            Self::SerializationError(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validation mode for different contexts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationMode {
    /// Full validation with signature verification (for block inclusion)
    Full,
    /// Skip signature verification (for testing)
    SkipSignature,
    /// Quick validation for mempool (stateless checks only)
    Quick,
}

pub struct TransactionValidator {
    seen_transactions: HashSet<String>,
    /// Per-sender transaction count for rate limiting
    sender_tx_count: HashMap<String, usize>,
    max_tx_size: usize,
    max_txs_per_sender: usize,
    /// Whether to verify Ed25519 signatures (disable for testing)
    verify_signatures: bool,
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
            sender_tx_count: HashMap::new(),
            max_tx_size: MAX_TX_SIZE,
            max_txs_per_sender: MAX_TXS_PER_SENDER,
            verify_signatures: true,
        }
    }

    /// Create validator with signature verification disabled (for testing/benchmarking)
    pub fn new_without_signature_verification() -> Self {
        let mut v = Self::new();
        v.verify_signatures = false;
        v
    }

    /// Validate transaction with comprehensive checks
    pub fn validate(&mut self, tx: &Transaction, current_balance: u64, current_nonce: u64) -> Result<bool> {
        self.validate_with_mode(tx, current_balance, current_nonce, 
            if self.verify_signatures { ValidationMode::Full } else { ValidationMode::SkipSignature })
    }

    /// Validate transaction with specified mode
    pub fn validate_with_mode(
        &mut self, 
        tx: &Transaction, 
        current_balance: u64, 
        current_nonce: u64,
        mode: ValidationMode,
    ) -> Result<bool> {
        // Quick checks first (cheapest)
        self.validate_basic_fields(tx)?;
        
        if mode == ValidationMode::Quick {
            return Ok(true);
        }

        // Signature verification (cryptographic - do early to reject forgeries)
        if mode == ValidationMode::Full {
            self.verify_signature(tx)?;
        }

        // State-dependent checks
        self.validate_balance(tx, current_balance)?;
        self.validate_nonce(tx, current_nonce)?;

        // Anti-spam checks
        self.validate_size(tx)?;
        self.validate_rate_limit(tx)?;
        self.validate_not_duplicate(tx)?;

        // Record this transaction
        let tx_hash = self.calculate_tx_hash(tx);
        self.seen_transactions.insert(tx_hash);
        *self.sender_tx_count.entry(tx.from.clone()).or_insert(0) += 1;

        debug!("Transaction validated: {} -> {} amount={}", tx.from, tx.to, tx.amount);
        Ok(true)
    }

    /// Validate basic transaction fields (stateless)
    fn validate_basic_fields(&self, tx: &Transaction) -> Result<()> {
        // Check amount > 0
        if tx.amount == 0 {
            return Err(ValidationError::ZeroAmount.into());
        }

        // Enforce zero gas fees (Sultan Chain policy)
        if tx.gas_fee != 0 {
            return Err(ValidationError::NonZeroGasFee { fee: tx.gas_fee }.into());
        }

        // Check sender != receiver
        if tx.from == tx.to {
            return Err(ValidationError::SelfSend.into());
        }

        // Validate address format
        self.validate_address(&tx.from, "sender")?;
        self.validate_address(&tx.to, "recipient")?;

        // Validate timestamp (not in future, not too old)
        // Note: Transaction timestamps are in seconds (from Transaction::new)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Allow 5 minute clock skew for future, 1 hour for past
        if tx.timestamp > now + TIMESTAMP_FUTURE_TOLERANCE_SECS {
            return Err(ValidationError::TimestampInFuture.into());
        }
        if tx.timestamp < now.saturating_sub(TIMESTAMP_PAST_TOLERANCE_SECS) {
            return Err(ValidationError::TimestampTooOld.into());
        }

        Ok(())
    }

    /// Validate address format (supports Sultan prefix validation)
    fn validate_address(&self, address: &str, field_name: &str) -> Result<()> {
        let field = field_name.to_string();
        
        if address.is_empty() {
            return Err(ValidationError::EmptyAddress { field }.into());
        }
        if address.len() < MIN_ADDRESS_LENGTH {
            return Err(ValidationError::AddressTooShort { 
                field, 
                len: address.len(), 
                min: MIN_ADDRESS_LENGTH 
            }.into());
        }
        if address.len() > MAX_ADDRESS_LENGTH {
            return Err(ValidationError::AddressTooLong { 
                field, 
                len: address.len(), 
                max: MAX_ADDRESS_LENGTH 
            }.into());
        }
        // Check for valid characters (alphanumeric + underscore)
        if !address.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ValidationError::InvalidAddressChars { field }.into());
        }
        Ok(())
    }

    /// Verify Ed25519 signature
    fn verify_signature(&self, tx: &Transaction) -> Result<()> {
        // Require signature
        let signature_hex = tx.signature.as_ref()
            .ok_or(ValidationError::MissingSignature)?;
        
        // Require public key
        let pubkey_hex = tx.public_key.as_ref()
            .ok_or(ValidationError::MissingPublicKey)?;

        // Decode signature (64 bytes = 128 hex chars)
        if signature_hex.len() != 128 {
            return Err(ValidationError::InvalidSignatureLength { 
                expected: 128, 
                got: signature_hex.len() 
            }.into());
        }
        let sig_bytes = hex::decode(signature_hex)
            .map_err(|e| ValidationError::InvalidSignatureHex(e.to_string()))?;
        
        // Decode public key (32 bytes = 64 hex chars)
        if pubkey_hex.len() != 64 {
            return Err(ValidationError::InvalidPublicKeyLength { 
                expected: 64, 
                got: pubkey_hex.len() 
            }.into());
        }
        let pubkey_bytes = hex::decode(pubkey_hex)
            .map_err(|e| ValidationError::InvalidPublicKeyHex(e.to_string()))?;

        // Build message to verify (same as signing)
        let message = self.build_signing_message(tx);

        // Verify Ed25519 signature
        let pubkey_array: [u8; 32] = pubkey_bytes.try_into()
            .map_err(|_| ValidationError::InvalidPublicKeyLength { expected: 32, got: 0 })?;
        let sig_array: [u8; 64] = sig_bytes.try_into()
            .map_err(|_| ValidationError::InvalidSignatureLength { expected: 64, got: 0 })?;

        let verifying_key = VerifyingKey::from_bytes(&pubkey_array)
            .map_err(|e| ValidationError::InvalidPublicKey(e.to_string()))?;
        let signature = Signature::from_bytes(&sig_array);

        verifying_key.verify(message.as_bytes(), &signature)
            .map_err(|e| ValidationError::SignatureVerificationFailed(e.to_string()))?;

        debug!("Ed25519 signature verified for tx from {}", tx.from);
        Ok(())
    }

    /// Build the message that should be signed
    /// Includes all binding fields: from, to, amount, nonce, timestamp, and memo
    pub fn build_signing_message(&self, tx: &Transaction) -> String {
        let memo = tx.memo.as_deref().unwrap_or("");
        format!("{}:{}:{}:{}:{}:{}", tx.from, tx.to, tx.amount, tx.nonce, tx.timestamp, memo)
    }

    /// Validate sufficient balance
    fn validate_balance(&self, tx: &Transaction, current_balance: u64) -> Result<()> {
        if current_balance < tx.amount {
            return Err(ValidationError::InsufficientBalance { 
                have: current_balance, 
                need: tx.amount 
            }.into());
        }
        Ok(())
    }

    /// Validate nonce (must be strictly greater than current)
    fn validate_nonce(&self, tx: &Transaction, current_nonce: u64) -> Result<()> {
        if tx.nonce <= current_nonce {
            return Err(ValidationError::InvalidNonce { 
                expected_gt: current_nonce, 
                got: tx.nonce 
            }.into());
        }
        // Check nonce not too far ahead (prevents nonce exhaustion attacks)
        if tx.nonce > current_nonce + MAX_NONCE_GAP {
            return Err(ValidationError::NonceTooFarAhead { 
                gap: tx.nonce - current_nonce, 
                max_gap: MAX_NONCE_GAP 
            }.into());
        }
        Ok(())
    }

    /// Validate transaction size
    fn validate_size(&self, tx: &Transaction) -> Result<()> {
        let tx_size = bincode::serialize(tx)
            .map_err(|e| ValidationError::SerializationError(e.to_string()))?.len();
        if tx_size > self.max_tx_size {
            return Err(ValidationError::TransactionTooLarge { 
                size: tx_size, 
                max: self.max_tx_size 
            }.into());
        }
        Ok(())
    }

    /// Validate rate limit per sender
    fn validate_rate_limit(&self, tx: &Transaction) -> Result<()> {
        let count = self.sender_tx_count.get(&tx.from).copied().unwrap_or(0);
        if count >= self.max_txs_per_sender {
            warn!("Rate limit exceeded for sender {}: {} txs", tx.from, count);
            return Err(ValidationError::RateLimitExceeded { 
                sender: tx.from.clone(), 
                count, 
                max: self.max_txs_per_sender 
            }.into());
        }
        Ok(())
    }

    /// Check for duplicate transaction
    fn validate_not_duplicate(&self, tx: &Transaction) -> Result<()> {
        let tx_hash = self.calculate_tx_hash(tx);
        if self.seen_transactions.contains(&tx_hash) {
            return Err(ValidationError::DuplicateTransaction.into());
        }
        Ok(())
    }

    /// Quick validation without state (for mempool pre-filter)
    pub fn quick_validate(&self, tx: &Transaction) -> Result<bool> {
        // Validate basic fields only (no state, no signature)
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
        // Check has signature and pubkey (don't verify yet)
        if tx.signature.is_none() || tx.public_key.is_none() {
            return Ok(false);
        }
        Ok(true)
    }

    /// Calculate transaction hash for deduplication
    /// Includes signature for integrity (prevents malleability)
    fn calculate_tx_hash(&self, tx: &Transaction) -> String {
        let mut hasher = Sha256::new();
        hasher.update(tx.from.as_bytes());
        hasher.update(tx.to.as_bytes());
        hasher.update(&tx.amount.to_le_bytes());
        hasher.update(&tx.nonce.to_le_bytes());
        hasher.update(&tx.timestamp.to_le_bytes());
        
        // Include signature for integrity
        if let Some(sig) = &tx.signature {
            hasher.update(sig.as_bytes());
        }
        
        format!("{:x}", hasher.finalize())
    }

    /// Clear seen transactions and rate limits (call after block confirmation)
    pub fn clear_seen(&mut self) {
        self.seen_transactions.clear();
        self.sender_tx_count.clear();
    }

    /// Get seen transaction count
    pub fn seen_count(&self) -> usize {
        self.seen_transactions.len()
    }

    /// Get transaction count for a specific sender
    pub fn sender_count(&self, sender: &str) -> usize {
        self.sender_tx_count.get(sender).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a valid test transaction (without signature for basic tests)
    fn test_tx(from: &str, to: &str, amount: u64, nonce: u64) -> Transaction {
        Transaction {
            from: from.to_string(),
            to: to.to_string(),
            amount,
            gas_fee: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            nonce,
            signature: Some("a".repeat(128)), // Dummy signature (128 hex chars)
            public_key: Some("b".repeat(64)), // Dummy pubkey (64 hex chars)
            memo: None,
        }
    }

    /// Create validator without signature verification for unit tests
    fn test_validator() -> TransactionValidator {
        TransactionValidator::new_without_signature_verification()
    }

    #[test]
    fn test_valid_transaction() {
        let mut validator = test_validator();
        let tx = test_tx("alice_wallet_addr", "bob_wallet_addrs", 100, 1);
        
        assert!(validator.validate(&tx, 1000, 0).is_ok());
        assert_eq!(validator.seen_count(), 1);
    }

    #[test]
    fn test_zero_amount_rejected() {
        let mut validator = test_validator();
        let mut tx = test_tx("alice_wallet_addr", "bob_wallet_addrs", 100, 1);
        tx.amount = 0;
        
        let result = validator.validate(&tx, 1000, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("amount must be greater than 0"));
    }

    #[test]
    fn test_insufficient_balance() {
        let mut validator = test_validator();
        let tx = test_tx("alice_wallet_addr", "bob_wallet_addrs", 2000, 1);
        
        let result = validator.validate(&tx, 1000, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Insufficient balance"));
    }

    #[test]
    fn test_non_zero_gas_rejected() {
        let mut validator = test_validator();
        let mut tx = test_tx("alice_wallet_addr", "bob_wallet_addrs", 100, 1);
        tx.gas_fee = 10;
        
        let result = validator.validate(&tx, 1000, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("zero gas fees"));
    }

    #[test]
    fn test_self_send_rejected() {
        let mut validator = test_validator();
        let tx = test_tx("alice_wallet_addr", "alice_wallet_addr", 100, 1);
        
        let result = validator.validate(&tx, 1000, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot send to yourself"));
    }

    #[test]
    fn test_empty_address_rejected() {
        let mut validator = test_validator();
        let mut tx = test_tx("alice_wallet_addr", "bob_wallet_addrs", 100, 1);
        tx.from = "".to_string();
        
        let result = validator.validate(&tx, 1000, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_address_too_short_rejected() {
        let mut validator = test_validator();
        let mut tx = test_tx("alice_wallet_addr", "bob_wallet_addrs", 100, 1);
        tx.from = "short".to_string(); // Less than MIN_ADDRESS_LENGTH
        
        let result = validator.validate(&tx, 1000, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too short"));
    }

    #[test]
    fn test_invalid_nonce_rejected() {
        let mut validator = test_validator();
        let tx = test_tx("alice_wallet_addr", "bob_wallet_addrs", 100, 5);
        
        // Nonce 5 is not > 10, so should fail
        let result = validator.validate(&tx, 1000, 10);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid nonce"));
    }

    #[test]
    fn test_nonce_too_far_ahead_rejected() {
        let mut validator = test_validator();
        let tx = test_tx("alice_wallet_addr", "bob_wallet_addrs", 100, 200);
        
        // Nonce 200 is more than 100 ahead of current nonce 0
        let result = validator.validate(&tx, 1000, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too far ahead"));
    }

    #[test]
    fn test_duplicate_rejected() {
        let mut validator = test_validator();
        let tx = test_tx("alice_wallet_addr", "bob_wallet_addrs", 100, 1);
        
        // First should succeed
        assert!(validator.validate(&tx, 1000, 0).is_ok());
        
        // Second (duplicate) should fail
        let result = validator.validate(&tx, 1000, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate"));
    }

    #[test]
    fn test_rate_limit_exceeded() {
        let mut validator = test_validator();
        validator.max_txs_per_sender = 3; // Low limit for testing
        
        // Send 3 transactions (at limit)
        for i in 1..=3 {
            let tx = test_tx("alice_wallet_addr", "bob_wallet_addrs", 100, i);
            assert!(validator.validate(&tx, 10000, i - 1).is_ok());
        }
        
        // 4th should fail
        let tx = test_tx("alice_wallet_addr", "bob_wallet_addrs", 100, 4);
        let result = validator.validate(&tx, 10000, 3);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Rate limit"));
    }

    #[test]
    fn test_clear_seen_resets_state() {
        let mut validator = test_validator();
        
        // Add some transactions
        let tx = test_tx("alice_wallet_addr", "bob_wallet_addrs", 100, 1);
        validator.validate(&tx, 1000, 0).unwrap();
        
        assert_eq!(validator.seen_count(), 1);
        assert_eq!(validator.sender_count("alice_wallet_addr"), 1);
        
        // Clear
        validator.clear_seen();
        
        assert_eq!(validator.seen_count(), 0);
        assert_eq!(validator.sender_count("alice_wallet_addr"), 0);
    }

    #[test]
    fn test_quick_validate_stateless() {
        let validator = test_validator();
        
        // Valid tx should pass quick validation
        let tx = test_tx("alice_wallet_addr", "bob_wallet_addrs", 100, 1);
        assert!(validator.quick_validate(&tx).unwrap());
        
        // Zero amount fails
        let mut bad_tx = tx.clone();
        bad_tx.amount = 0;
        assert!(!validator.quick_validate(&bad_tx).unwrap());
        
        // Self-send fails
        bad_tx = tx.clone();
        bad_tx.to = bad_tx.from.clone();
        assert!(!validator.quick_validate(&bad_tx).unwrap());
        
        // Missing signature fails
        bad_tx = tx.clone();
        bad_tx.signature = None;
        assert!(!validator.quick_validate(&bad_tx).unwrap());
    }

    #[test]
    fn test_tx_hash_includes_signature() {
        let validator = test_validator();
        
        let mut tx1 = test_tx("alice_wallet_addr", "bob_wallet_addrs", 100, 1);
        tx1.signature = Some("sig1".repeat(32));
        
        let mut tx2 = tx1.clone();
        tx2.signature = Some("sig2".repeat(32));
        
        let hash1 = validator.calculate_tx_hash(&tx1);
        let hash2 = validator.calculate_tx_hash(&tx2);
        
        // Different signatures should produce different hashes
        assert_ne!(hash1, hash2, "Hash should include signature for integrity");
    }

    #[test]
    fn test_ed25519_signature_verification() {
        use ed25519_dalek::{SigningKey, Signer};
        
        // Create a real keypair
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let verifying_key = signing_key.verifying_key();
        let pubkey_hex = hex::encode(verifying_key.to_bytes());
        
        // Create transaction
        let mut tx = Transaction {
            from: "alice_wallet_addr".to_string(),
            to: "bob_wallet_addrs".to_string(),
            amount: 100,
            gas_fee: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            nonce: 1,
            signature: None,
            public_key: Some(pubkey_hex),
            memo: None,
        };
        
        // Build and sign the message
        let mut validator = TransactionValidator::new(); // Full verification
        let message = validator.build_signing_message(&tx);
        let signature = signing_key.sign(message.as_bytes());
        tx.signature = Some(hex::encode(signature.to_bytes()));
        
        // Should validate with real signature
        let result = validator.validate(&tx, 1000, 0);
        assert!(result.is_ok(), "Valid signature should be accepted: {:?}", result);
    }

    #[test]
    fn test_invalid_signature_rejected() {
        let mut validator = TransactionValidator::new(); // Full verification enabled
        
        let tx = Transaction {
            from: "alice_wallet_addr".to_string(),
            to: "bob_wallet_addrs".to_string(),
            amount: 100,
            gas_fee: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            nonce: 1,
            signature: Some("00".repeat(64)), // Invalid signature (all zeros)
            public_key: Some("11".repeat(32)), // Invalid pubkey
            memo: None,
        };
        
        let result = validator.validate(&tx, 1000, 0);
        assert!(result.is_err(), "Invalid signature should be rejected");
    }

    #[test]
    fn test_missing_signature_rejected() {
        let mut validator = TransactionValidator::new(); // Full verification
        
        let tx = Transaction {
            from: "alice_wallet_addr".to_string(),
            to: "bob_wallet_addrs".to_string(),
            amount: 100,
            gas_fee: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            nonce: 1,
            signature: None, // Missing!
            public_key: Some("a".repeat(64)),
            memo: None,
        };
        
        let result = validator.validate(&tx, 1000, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing signature"));
    }

    #[test]
    fn test_missing_pubkey_rejected() {
        let mut validator = TransactionValidator::new(); // Full verification
        
        let tx = Transaction {
            from: "alice_wallet_addr".to_string(),
            to: "bob_wallet_addrs".to_string(),
            amount: 100,
            gas_fee: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            nonce: 1,
            signature: Some("a".repeat(128)),
            public_key: None, // Missing!
            memo: None,
        };
        
        let result = validator.validate(&tx, 1000, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing public key"));
    }
}
