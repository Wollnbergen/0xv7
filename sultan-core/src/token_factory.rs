// Native Token Factory Module - No Smart Contracts Required
// Enables token creation and management directly in Sultan protocol

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

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
    
    /// Create a new token
    pub async fn create_token(
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
    
    /// Mint tokens to an address (only if minting enabled)
    pub async fn mint_to(
        &self,
        denom: &str,
        recipient: &str,
        amount: u128,
    ) -> Result<()> {
        // Get token metadata
        let tokens = self.tokens.read().await;
        let metadata = tokens.get(denom)
            .ok_or_else(|| anyhow::anyhow!("Token not found: {}", denom))?
            .clone();
        drop(tokens);
        
        // Check if minting is enabled and respects max supply
        if let Some(max_supply) = metadata.max_supply {
            let mut balances = self.balances.write().await;
            
            // Calculate current total supply
            let current_supply: u128 = balances.iter()
                .filter(|((d, _), _)| d == denom)
                .map(|(_, balance)| balance)
                .sum();
            
            if current_supply + amount > max_supply {
                bail!("Minting would exceed max supply of {}", max_supply);
            }
            
            // Mint tokens
            let key = (denom.to_string(), recipient.to_string());
            let balance = balances.entry(key).or_insert(0);
            *balance += amount;
            
            info!("✅ Minted {} {} to {}", amount, metadata.symbol, recipient);
            Ok(())
        } else {
            bail!("Minting not enabled for this token");
        }
    }
    
    /// Transfer tokens between addresses
    pub async fn transfer(
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
    
    /// Burn tokens from an address
    pub async fn burn(
        &self,
        denom: &str,
        from: &str,
        amount: u128,
    ) -> Result<()> {
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
        
        info!("✅ Burned {} {} from {}", amount, denom, from);
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
    
    /// Get total supply of a token
    pub async fn get_total_supply(&self, denom: &str) -> u128 {
        let balances = self.balances.read().await;
        balances.iter()
            .filter(|((d, _), _)| d == denom)
            .map(|(_, balance)| balance)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_token() {
        let factory = TokenFactory::new();
        
        let denom = factory.create_token(
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
        
        let denom = factory.create_token(
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
        factory.transfer(&denom, "sultan1alice", "sultan1bob", 300_000).await.unwrap();
        
        // Verify balances
        assert_eq!(factory.get_balance(&denom, "sultan1alice").await, 700_000);
        assert_eq!(factory.get_balance(&denom, "sultan1bob").await, 300_000);
    }
    
    #[tokio::test]
    async fn test_burn() {
        let factory = TokenFactory::new();
        
        let denom = factory.create_token(
            "sultan1alice",
            "Burn Test".to_string(),
            "BT".to_string(),
            6,
            1_000_000,
            None,
            None,
            None,
        ).await.unwrap();
        
        // Burn tokens
        factory.burn(&denom, "sultan1alice", 300_000).await.unwrap();
        
        // Verify balance
        assert_eq!(factory.get_balance(&denom, "sultan1alice").await, 700_000);
        
        // Verify total supply decreased
        assert_eq!(factory.get_total_supply(&denom).await, 700_000);
    }
}
