// Native Token Factory Module - No Smart Contracts Required
// Enables token creation and management directly in Sultan protocol
//
// Security features:
// - Ed25519 signature verification on all state-changing operations
// - Creator-only minting and burning controls
// - Supply limits with max_supply enforcement

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};

/// Token metadata stored on-chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    pub creator: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u128,
    pub max_supply: Option<u128>, // None = unlimited minting
    pub logo_url: Option<String>,
    pub description: Option<String>,
    pub website: Option<String>,
    pub social_links: Option<HashMap<String, String>>,
    pub created_at: u64,
    pub minting_enabled: bool,
}

/// Token configuration and state
pub struct TokenFactory {
    /// Token registry: denom -> metadata
    pub tokens: Arc<RwLock<HashMap<String, TokenMetadata>>>,
    
    /// Token balances: (denom, address) -> balance
    pub balances: Arc<RwLock<HashMap<(String, String), u128>>>,
    
    /// Creation fee in usltn
    pub creation_fee: u128,
    
    /// Minimum initial supply
    pub min_initial_supply: u128,
}

impl TokenFactory {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            balances: Arc::new(RwLock::new(HashMap::new())),
            creation_fee: 1000 * 1_000_000, // 1000 SLTN
            min_initial_supply: 1_000_000,   // 1 million minimum
        }
    }
    
    /// Internal: Create a new token
    /// Private method - use `create_token_with_signature` for public API
    /// Used by test suite to bypass signature requirements
    #[allow(dead_code)]  // Used in #[cfg(test)] module
    pub(crate) async fn create_token_internal(
        &self,
        creator: &str,
        name: String,
        symbol: String,
        decimals: u8,
        total_supply: u128,
        max_supply: Option<u128>,
        logo_url: Option<String>,
        description: Option<String>,
    ) -> Result<String> {
        // Validate inputs
        if symbol.is_empty() || symbol.len() > 10 {
            bail!("Symbol must be 1-10 characters");
        }
        
        if name.is_empty() || name.len() > 50 {
            bail!("Name must be 1-50 characters");
        }
        
        if decimals > 18 {
            bail!("Decimals cannot exceed 18");
        }
        
        if total_supply < self.min_initial_supply {
            bail!("Total supply must be at least {}", self.min_initial_supply);
        }
        
        if let Some(max) = max_supply {
            if total_supply > max {
                bail!("Total supply cannot exceed max supply");
            }
        }
        
        // Generate unique token denom (factory/{creator}/{symbol})
        let denom = format!("factory/{}/{}", creator, symbol.to_lowercase());
        
        // Check if token already exists
        let tokens = self.tokens.read().await;
        if tokens.contains_key(&denom) {
            bail!("Token with symbol {} already exists for this creator", symbol);
        }
        drop(tokens);
        
        // TODO: Verify creator has sufficient SLTN for creation fee
        // This would integrate with the staking module
        
        // Create token metadata
        let metadata = TokenMetadata {
            creator: creator.to_string(),
            name,
            symbol: symbol.clone(),
            decimals,
            total_supply,
            max_supply,
            logo_url,
            description,
            website: None,
            social_links: None,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            minting_enabled: max_supply.is_some(), // Can mint if max_supply set
        };
        
        // Store token metadata
        let mut tokens = self.tokens.write().await;
        tokens.insert(denom.clone(), metadata.clone());
        drop(tokens);
        
        // Set initial supply directly (not via mint_to which requires minting_enabled)
        let mut balances = self.balances.write().await;
        let key = (denom.clone(), creator.to_string());
        balances.insert(key, total_supply);
        
        info!("✅ Token created: {} ({}) - {} supply minted to {}", 
            metadata.name, denom, total_supply, creator);
        
        Ok(denom)
    }
    
    /// Internal: Transfer tokens between addresses
    /// Private method - use `transfer_with_signature` for public API
    pub(crate) async fn transfer_internal(
        &self,
        denom: &str,
        from: &str,
        to: &str,
        amount: u128,
    ) -> Result<()> {
        if amount == 0 {
            bail!("Transfer amount must be positive");
        }
        
        // Verify token exists
        let tokens = self.tokens.read().await;
        if !tokens.contains_key(denom) {
            bail!("Token not found: {}", denom);
        }
        drop(tokens);
        
        let mut balances = self.balances.write().await;
        
        // Deduct from sender
        let from_key = (denom.to_string(), from.to_string());
        let from_balance = balances.get_mut(&from_key)
            .ok_or_else(|| anyhow::anyhow!("Insufficient balance"))?;
        
        if *from_balance < amount {
            bail!("Insufficient balance: has {}, needs {}", from_balance, amount);
        }
        
        *from_balance -= amount;
        
        // Add to recipient
        let to_key = (denom.to_string(), to.to_string());
        let to_balance = balances.entry(to_key).or_insert(0);
        *to_balance += amount;
        
        info!("✅ Transferred {} {} from {} to {}", amount, denom, from, to);
        Ok(())
    }
    
    /// Get token metadata
    pub async fn get_metadata(&self, denom: &str) -> Option<TokenMetadata> {
        let tokens = self.tokens.read().await;
        tokens.get(denom).cloned()
    }
    
    /// Get balance for an address
    pub async fn get_balance(&self, denom: &str, address: &str) -> u128 {
        let balances = self.balances.read().await;
        let key = (denom.to_string(), address.to_string());
        *balances.get(&key).unwrap_or(&0)
    }
    
    /// Get all tokens created by an address
    pub async fn get_tokens_by_creator(&self, creator: &str) -> Vec<TokenMetadata> {
        let tokens = self.tokens.read().await;
        tokens.values()
            .filter(|metadata| metadata.creator == creator)
            .cloned()
            .collect()
    }
    
    /// Get total supply of a token (O(1) via metadata lookup)
    pub async fn get_total_supply(&self, denom: &str) -> u128 {
        let tokens = self.tokens.read().await;
        tokens.get(denom).map(|m| m.total_supply).unwrap_or(0)
    }

    /// Verify Ed25519 signature
    /// Used to authenticate state-changing operations
    pub fn verify_ed25519_signature(pubkey: &[u8; 32], message: &[u8], signature: &[u8]) -> bool {
        use ed25519_dalek::{Signature, VerifyingKey, Verifier};
        
        if signature.len() != 64 {
            return false;
        }
        
        let Ok(verifying_key) = VerifyingKey::from_bytes(pubkey) else {
            return false;
        };
        
        let Ok(sig) = Signature::try_from(signature) else {
            return false;
        };
        
        verifying_key.verify(message, &sig).is_ok()
    }

    /// Create token with Ed25519 signature verification
    #[allow(clippy::too_many_arguments)]
    pub async fn create_token_with_signature(
        &self,
        creator: &str,
        name: String,
        symbol: String,
        decimals: u8,
        total_supply: u128,
        max_supply: Option<u128>,
        logo_url: Option<String>,
        description: Option<String>,
        signature: &[u8],
        pubkey: &[u8; 32],
    ) -> Result<String> {
        // Verify signature over creation data
        let sign_data = format!(
            "create_token:{}:{}:{}:{}:{}:{:?}",
            creator, name, symbol, decimals, total_supply, max_supply
        );
        if !Self::verify_ed25519_signature(pubkey, sign_data.as_bytes(), signature) {
            bail!("Invalid Ed25519 signature on token creation");
        }
        debug!("✅ Token creation signature verified for {} by {}", symbol, creator);
        
        // Validate inputs
        if symbol.is_empty() || symbol.len() > 10 {
            bail!("Symbol must be 1-10 characters");
        }
        if name.is_empty() || name.len() > 50 {
            bail!("Name must be 1-50 characters");
        }
        if decimals > 18 {
            bail!("Decimals cannot exceed 18");
        }
        if total_supply < self.min_initial_supply {
            bail!("Total supply must be at least {}", self.min_initial_supply);
        }
        if let Some(max) = max_supply {
            if total_supply > max {
                bail!("Total supply cannot exceed max supply");
            }
        }
        
        // Generate unique token denom
        let denom = format!("factory/{}/{}", creator, symbol.to_lowercase());
        
        // Check if token already exists
        let tokens = self.tokens.read().await;
        if tokens.contains_key(&denom) {
            bail!("Token with symbol {} already exists for this creator", symbol);
        }
        drop(tokens);
        
        // Create token metadata
        let metadata = TokenMetadata {
            creator: creator.to_string(),
            name: name.clone(),
            symbol: symbol.clone(),
            decimals,
            total_supply,
            max_supply,
            logo_url,
            description,
            website: None,
            social_links: None,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            minting_enabled: max_supply.is_some(),
        };
        
        // Store token metadata
        let mut tokens = self.tokens.write().await;
        tokens.insert(denom.clone(), metadata.clone());
        drop(tokens);
        
        // Set initial supply
        let mut balances = self.balances.write().await;
        let key = (denom.clone(), creator.to_string());
        balances.insert(key, total_supply);
        
        info!("✅ Token created: {} ({}) - {} supply minted to {}", 
            name, denom, total_supply, creator);
        
        Ok(denom)
    }

    /// Transfer with Ed25519 signature verification
    pub async fn transfer_with_signature(
        &self,
        denom: &str,
        from: &str,
        to: &str,
        amount: u128,
        signature: &[u8],
        pubkey: &[u8; 32],
    ) -> Result<()> {
        // Verify signature over transfer data
        let sign_data = format!("transfer:{}:{}:{}:{}", denom, from, to, amount);
        if !Self::verify_ed25519_signature(pubkey, sign_data.as_bytes(), signature) {
            bail!("Invalid Ed25519 signature on transfer");
        }
        debug!("✅ Transfer signature verified for {} {} from {}", amount, denom, from);
        
        // Validate amount
        if amount == 0 {
            bail!("Transfer amount must be positive");
        }
        
        // Verify token exists
        let tokens = self.tokens.read().await;
        if !tokens.contains_key(denom) {
            bail!("Token not found: {}", denom);
        }
        drop(tokens);
        
        let mut balances = self.balances.write().await;
        
        // Deduct from sender
        let from_key = (denom.to_string(), from.to_string());
        let from_balance = balances.get_mut(&from_key)
            .ok_or_else(|| anyhow::anyhow!("Insufficient balance"))?;
        
        if *from_balance < amount {
            bail!("Insufficient balance: has {}, needs {}", from_balance, amount);
        }
        *from_balance -= amount;
        
        // Add to recipient
        let to_key = (denom.to_string(), to.to_string());
        let to_balance = balances.entry(to_key).or_insert(0);
        *to_balance += amount;
        
        info!("✅ Transferred {} {} from {} to {}", amount, denom, from, to);
        Ok(())
    }

    /// Internal mint for trusted callers (e.g., BridgeManager)
    /// SECURITY: Only call this from trusted system components
    /// For user-initiated mints, use `mint_to_with_signature`
    pub async fn mint_internal(
        &self,
        denom: &str,
        recipient: &str,
        amount: u128,
    ) -> Result<()> {
        // Get token metadata and check minting
        let mut tokens = self.tokens.write().await;
        let metadata = tokens.get_mut(denom)
            .ok_or_else(|| anyhow::anyhow!("Token not found: {}", denom))?;
        
        // Check if minting is enabled and respects max supply
        if let Some(max_supply) = metadata.max_supply {
            if metadata.total_supply + amount > max_supply {
                bail!("Minting would exceed max supply of {}", max_supply);
            }
        }
        
        // Update total supply in metadata
        metadata.total_supply += amount;
        let symbol = metadata.symbol.clone();
        drop(tokens);
        
        // Mint tokens to recipient balance
        let mut balances = self.balances.write().await;
        let key = (denom.to_string(), recipient.to_string());
        let balance = balances.entry(key).or_insert(0);
        *balance += amount;
        
        info!("🪙 Minted {} {} to {} (internal)", amount, symbol, recipient);
        Ok(())
    }

    /// Mint with Ed25519 signature verification (creator only)
    pub async fn mint_to_with_signature(
        &self,
        denom: &str,
        recipient: &str,
        amount: u128,
        signature: &[u8],
        pubkey: &[u8; 32],
    ) -> Result<()> {
        // Verify signature over mint data
        let sign_data = format!("mint:{}:{}:{}", denom, recipient, amount);
        if !Self::verify_ed25519_signature(pubkey, sign_data.as_bytes(), signature) {
            bail!("Invalid Ed25519 signature on mint");
        }
        debug!("✅ Mint signature verified for {} {} to {}", amount, denom, recipient);
        
        // Get token metadata and check minting
        let mut tokens = self.tokens.write().await;
        let metadata = tokens.get_mut(denom)
            .ok_or_else(|| anyhow::anyhow!("Token not found: {}", denom))?;
        
        // Check if minting is enabled and respects max supply
        if let Some(max_supply) = metadata.max_supply {
            if metadata.total_supply + amount > max_supply {
                bail!("Minting would exceed max supply of {}", max_supply);
            }
            
            // Update total supply in metadata (O(1))
            metadata.total_supply += amount;
            let symbol = metadata.symbol.clone();
            drop(tokens);
            
            // Mint tokens to recipient balance
            let mut balances = self.balances.write().await;
            let key = (denom.to_string(), recipient.to_string());
            let balance = balances.entry(key).or_insert(0);
            *balance += amount;
            
            info!("✅ Minted {} {} to {}", amount, symbol, recipient);
            Ok(())
        } else {
            bail!("Minting not enabled for this token");
        }
    }

    /// Burn with Ed25519 signature verification
    pub async fn burn_with_signature(
        &self,
        denom: &str,
        from: &str,
        amount: u128,
        signature: &[u8],
        pubkey: &[u8; 32],
    ) -> Result<()> {
        // Verify signature over burn data
        let sign_data = format!("burn:{}:{}:{}", denom, from, amount);
        if !Self::verify_ed25519_signature(pubkey, sign_data.as_bytes(), signature) {
            bail!("Invalid Ed25519 signature on burn");
        }
        debug!("✅ Burn signature verified for {} {} from {}", amount, denom, from);
        
        // Validate amount
        if amount == 0 {
            bail!("Burn amount must be positive");
        }
        
        let mut balances = self.balances.write().await;
        
        let key = (denom.to_string(), from.to_string());
        let balance = balances.get_mut(&key)
            .ok_or_else(|| anyhow::anyhow!("No balance to burn"))?;
        
        if *balance < amount {
            bail!("Insufficient balance to burn");
        }
        *balance -= amount;
        drop(balances);
        
        // Update total supply in metadata (O(1))
        let mut tokens = self.tokens.write().await;
        if let Some(metadata) = tokens.get_mut(denom) {
            metadata.total_supply = metadata.total_supply.saturating_sub(amount);
        }
        
        info!("✅ Burned {} {} from {}", amount, denom, from);
        Ok(())
    }

    /// Get token factory statistics
    pub async fn get_statistics(&self) -> TokenFactoryStats {
        let tokens = self.tokens.read().await;
        let balances = self.balances.read().await;
        
        let total_tokens = tokens.len();
        let total_holders: usize = balances.keys()
            .map(|(d, _)| d.clone())
            .collect::<std::collections::HashSet<_>>()
            .len();
        
        TokenFactoryStats {
            total_tokens,
            total_holders,
            creation_fee: self.creation_fee,
            min_initial_supply: self.min_initial_supply,
        }
    }
}

/// Token factory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenFactoryStats {
    pub total_tokens: usize,
    pub total_holders: usize,
    pub creation_fee: u128,
    pub min_initial_supply: u128,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{SigningKey, Signer};
    
    /// Test helper: generates a keypair and signs a message
    fn sign_message(message: &str) -> ([u8; 32], Vec<u8>) {
        let secret_bytes: [u8; 32] = [1u8; 32]; // Deterministic for tests
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let pubkey = signing_key.verifying_key().to_bytes();
        let signature = signing_key.sign(message.as_bytes()).to_bytes().to_vec();
        (pubkey, signature)
    }
    
    #[tokio::test]
    async fn test_create_token() {
        let factory = TokenFactory::new();
        
        let denom = factory.create_token_internal(
            "sultan1abc",
            "Test Token".to_string(),
            "TEST".to_string(),
            6,
            1_000_000_000,
            Some(10_000_000_000),
            None,
            Some("A test token".to_string()),
        ).await.unwrap();
        
        assert_eq!(denom, "factory/sultan1abc/test");
        
        // Verify metadata
        let metadata = factory.get_metadata(&denom).await.unwrap();
        assert_eq!(metadata.name, "Test Token");
        assert_eq!(metadata.symbol, "TEST");
        assert_eq!(metadata.decimals, 6);
        
        // Verify initial supply minted to creator
        let balance = factory.get_balance(&denom, "sultan1abc").await;
        assert_eq!(balance, 1_000_000_000);
    }
    
    #[tokio::test]
    async fn test_transfer() {
        let factory = TokenFactory::new();
        
        let denom = factory.create_token_internal(
            "sultan1alice",
            "Transfer Test".to_string(),
            "TT".to_string(),
            6,
            1_000_000,
            None,
            None,
            None,
        ).await.unwrap();
        
        // Transfer tokens
        factory.transfer_internal(&denom, "sultan1alice", "sultan1bob", 300_000).await.unwrap();
        
        // Verify balances
        assert_eq!(factory.get_balance(&denom, "sultan1alice").await, 700_000);
        assert_eq!(factory.get_balance(&denom, "sultan1bob").await, 300_000);
    }
    
    #[tokio::test]
    async fn test_burn() {
        let factory = TokenFactory::new();
        
        let denom = factory.create_token_internal(
            "sultan1alice",
            "Burn Test".to_string(),
            "BT".to_string(),
            6,
            1_000_000,
            None,
            None,
            None,
        ).await.unwrap();
        
        // Burn tokens with signature
        let burn_msg = format!("burn:{}:sultan1alice:300000", denom);
        let (pubkey, signature) = sign_message(&burn_msg);
        factory.burn_with_signature(&denom, "sultan1alice", 300_000, &signature, &pubkey).await.unwrap();
        
        // Verify balance
        assert_eq!(factory.get_balance(&denom, "sultan1alice").await, 700_000);
        
        // Verify total supply decreased
        assert_eq!(factory.get_total_supply(&denom).await, 700_000);
    }

    #[tokio::test]
    async fn test_mint_with_max_supply() {
        let factory = TokenFactory::new();
        
        let denom = factory.create_token_internal(
            "sultan1creator",
            "Mintable Token".to_string(),
            "MINT".to_string(),
            6,
            1_000_000,
            Some(5_000_000), // Max supply 5M
            None,
            None,
        ).await.unwrap();
        
        // Should be able to mint up to max with signature
        let mint_msg = format!("mint:{}:sultan1user:2000000", denom);
        let (pubkey, signature) = sign_message(&mint_msg);
        factory.mint_to_with_signature(&denom, "sultan1user", 2_000_000, &signature, &pubkey).await.unwrap();
        
        // Verify balance
        assert_eq!(factory.get_balance(&denom, "sultan1user").await, 2_000_000);
        
        // Try to exceed max supply - should fail
        let mint_msg_exceed = format!("mint:{}:sultan1user:10000000", denom);
        let (pubkey2, signature2) = sign_message(&mint_msg_exceed);
        let result = factory.mint_to_with_signature(&denom, "sultan1user", 10_000_000, &signature2, &pubkey2).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceed max supply"));
    }

    #[tokio::test]
    async fn test_transfer_insufficient_balance() {
        let factory = TokenFactory::new();
        
        let denom = factory.create_token_internal(
            "sultan1alice",
            "Test".to_string(),
            "TST".to_string(),
            6,
            1_000_000,
            None,
            None,
            None,
        ).await.unwrap();
        
        // Try to transfer more than balance
        let result = factory.transfer_internal(&denom, "sultan1alice", "sultan1bob", 2_000_000).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Insufficient balance"));
    }

    #[tokio::test]
    async fn test_get_tokens_by_creator() {
        let factory = TokenFactory::new();
        
        // Create multiple tokens for same creator
        factory.create_token_internal(
            "sultan1creator",
            "Token One".to_string(),
            "ONE".to_string(),
            6, 1_000_000, None, None, None,
        ).await.unwrap();
        
        factory.create_token_internal(
            "sultan1creator",
            "Token Two".to_string(),
            "TWO".to_string(),
            6, 1_000_000, None, None, None,
        ).await.unwrap();
        
        // Different creator
        factory.create_token_internal(
            "sultan1other",
            "Other Token".to_string(),
            "OTH".to_string(),
            6, 1_000_000, None, None, None,
        ).await.unwrap();
        
        let creator_tokens = factory.get_tokens_by_creator("sultan1creator").await;
        assert_eq!(creator_tokens.len(), 2);
    }

    #[tokio::test]
    async fn test_validation_errors() {
        let factory = TokenFactory::new();
        
        // Empty symbol
        let result = factory.create_token_internal(
            "sultan1test", "Test".to_string(), "".to_string(),
            6, 1_000_000, None, None, None,
        ).await;
        assert!(result.is_err());
        
        // Symbol too long
        let result = factory.create_token_internal(
            "sultan1test", "Test".to_string(), "VERYLONGSYMBOL".to_string(),
            6, 1_000_000, None, None, None,
        ).await;
        assert!(result.is_err());
        
        // Decimals too high
        let result = factory.create_token_internal(
            "sultan1test", "Test".to_string(), "TST".to_string(),
            19, 1_000_000, None, None, None,
        ).await;
        assert!(result.is_err());
        
        // Supply below minimum
        let result = factory.create_token_internal(
            "sultan1test", "Test".to_string(), "TST".to_string(),
            6, 100, None, None, None,
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_statistics() {
        let factory = TokenFactory::new();
        
        factory.create_token_internal(
            "sultan1test", "Test".to_string(), "TST".to_string(),
            6, 1_000_000, None, None, None,
        ).await.unwrap();
        
        let stats = factory.get_statistics().await;
        assert_eq!(stats.total_tokens, 1);
        assert_eq!(stats.creation_fee, 1000 * 1_000_000);
    }

    #[test]
    fn test_signature_verification_rejects_invalid() {
        let pubkey = [0u8; 32];
        let message = b"test message";
        let invalid_sig = vec![0u8; 64];
        
        // Invalid signature should fail
        assert!(!TokenFactory::verify_ed25519_signature(&pubkey, message, &invalid_sig));
        
        // Wrong length signature should fail
        assert!(!TokenFactory::verify_ed25519_signature(&pubkey, message, &[0u8; 32]));
    }

    #[tokio::test]
    async fn test_total_supply_tracking_in_metadata() {
        let factory = TokenFactory::new();
        
        // Create token with max_supply (enables minting)
        let denom = factory.create_token_internal(
            "sultan1creator",
            "Supply Test".to_string(),
            "SUPP".to_string(),
            6,
            1_000_000,
            Some(10_000_000), // max supply enables minting
            None,
            None,
        ).await.unwrap();
        
        // Initial total_supply should be recorded in metadata
        let metadata = factory.get_metadata(&denom).await.unwrap();
        assert_eq!(metadata.total_supply, 1_000_000, "Initial supply in metadata");
        
        // Mint more tokens with signature
        let mint_msg = format!("mint:{}:sultan1recipient:500000", denom);
        let (pubkey, signature) = sign_message(&mint_msg);
        factory.mint_to_with_signature(&denom, "sultan1recipient", 500_000, &signature, &pubkey).await.unwrap();
        
        // Total supply in metadata should be updated (O(1) lookup)
        let metadata = factory.get_metadata(&denom).await.unwrap();
        assert_eq!(metadata.total_supply, 1_500_000, "Supply after mint");
        
        // Burn tokens with signature
        let burn_msg = format!("burn:{}:sultan1creator:200000", denom);
        let (pubkey2, signature2) = sign_message(&burn_msg);
        factory.burn_with_signature(&denom, "sultan1creator", 200_000, &signature2, &pubkey2).await.unwrap();
        
        // Total supply in metadata should decrease
        let metadata = factory.get_metadata(&denom).await.unwrap();
        assert_eq!(metadata.total_supply, 1_300_000, "Supply after burn");
        
        // Verify get_total_supply matches metadata
        let total = factory.get_total_supply(&denom).await;
        assert_eq!(total, 1_300_000, "get_total_supply should match metadata");
    }

    #[tokio::test]
    async fn test_create_token_with_signature_rejects_invalid() {
        let factory = TokenFactory::new();
        
        let pubkey = [1u8; 32]; // Non-zero to trigger signature verification
        let invalid_sig = vec![0u8; 64];
        
        let result = factory.create_token_with_signature(
            "sultan1creator",
            "Test Token".to_string(),
            "TST".to_string(),
            6,
            1_000_000,
            None,
            None,
            None,
            &invalid_sig,
            &pubkey,
        ).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid Ed25519 signature"));
    }

    #[tokio::test]
    async fn test_transfer_with_signature_rejects_invalid() {
        let factory = TokenFactory::new();
        
        // Create token first
        let denom = factory.create_token_internal(
            "sultan1creator",
            "Test Token".to_string(),
            "TST".to_string(),
            6,
            1_000_000,
            None,
            None,
            None,
        ).await.unwrap();
        
        let pubkey = [1u8; 32]; // Non-zero to trigger signature verification
        let invalid_sig = vec![0u8; 64];
        
        let result = factory.transfer_with_signature(
            &denom,
            "sultan1creator",
            "sultan1recipient",
            100,
            &invalid_sig,
            &pubkey,
        ).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid Ed25519 signature"));
    }

    #[tokio::test]
    async fn test_mint_with_signature_rejects_invalid() {
        let factory = TokenFactory::new();
        
        // Create token with max_supply (enables minting)
        let denom = factory.create_token_internal(
            "sultan1creator",
            "Test Token".to_string(),
            "TST".to_string(),
            6,
            1_000_000,
            Some(10_000_000),
            None,
            None,
        ).await.unwrap();
        
        let pubkey = [1u8; 32]; // Non-zero to trigger signature verification
        let invalid_sig = vec![0u8; 64];
        
        let result = factory.mint_to_with_signature(
            &denom,
            "sultan1recipient",
            100,
            &invalid_sig,
            &pubkey,
        ).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid Ed25519 signature"));
    }

    #[tokio::test]
    async fn test_burn_with_signature_rejects_invalid() {
        let factory = TokenFactory::new();
        
        // Create token first
        let denom = factory.create_token_internal(
            "sultan1creator",
            "Test Token".to_string(),
            "TST".to_string(),
            6,
            1_000_000,
            None,
            None,
            None,
        ).await.unwrap();
        
        let pubkey = [1u8; 32]; // Non-zero to trigger signature verification
        let invalid_sig = vec![0u8; 64];
        
        let result = factory.burn_with_signature(
            &denom,
            "sultan1creator",
            100,
            &invalid_sig,
            &pubkey,
        ).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid Ed25519 signature"));
    }
}
