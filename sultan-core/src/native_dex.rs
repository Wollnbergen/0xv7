// Native DEX Module - Automated Market Maker (AMM)
// Enables token swapping without smart contracts using constant product formula
//
// Security features:
// - Ed25519 signature verification on swap/add/remove operations
// - Slippage protection with minimum output amounts
// - Constant product formula (x * y = k) for fair pricing

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};

use crate::token_factory::TokenFactory;

/// Liquidity pool using constant product formula (x * y = k)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPool {
    pub pair_id: String,
    pub token_a: String,
    pub token_b: String,
    pub reserve_a: u128,
    pub reserve_b: u128,
    pub total_lp_tokens: u128,
    pub lp_token_holders: HashMap<String, u128>,
    pub created_at: u64,
    pub total_volume_a: u128,
    pub total_volume_b: u128,
    pub fee_rate: u32, // Basis points (30 = 0.3%)
}

/// Native DEX for token swapping
pub struct NativeDex {
    pub pools: Arc<RwLock<HashMap<String, LiquidityPool>>>,
    pub token_factory: Arc<TokenFactory>,
    pub default_fee_rate: u32, // 30 basis points = 0.3%
}

impl NativeDex {
    pub fn new(token_factory: Arc<TokenFactory>) -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            token_factory,
            default_fee_rate: 30, // 0.3% fee
        }
    }
    
    /// Internal: Create a new liquidity pool
    /// Private method - use `create_pair_with_signature` for public API
    async fn create_pair_internal(
        &self,
        creator: &str,
        token_a: &str,
        token_b: &str,
        amount_a: u128,
        amount_b: u128,
    ) -> Result<String> {
        if amount_a == 0 || amount_b == 0 {
            bail!("Initial liquidity must be positive");
        }
        
        // Sort tokens to prevent duplicate pairs (A/B == B/A)
        let (token_a, token_b, reserve_a, reserve_b) = if token_a < token_b {
            (token_a.to_string(), token_b.to_string(), amount_a, amount_b)
        } else {
            (token_b.to_string(), token_a.to_string(), amount_b, amount_a)
        };
        
        // Create pair ID
        let pair_id = format!("pair/{}/{}", token_a, token_b);
        
        // Check if pair already exists
        let pools = self.pools.read().await;
        if pools.contains_key(&pair_id) {
            bail!("Liquidity pool already exists for this pair");
        }
        drop(pools);
        
        // Transfer tokens from creator to pool
        self.token_factory.transfer_internal(&token_a, creator, &pair_id, reserve_a).await?;
        self.token_factory.transfer_internal(&token_b, creator, &pair_id, reserve_b).await?;
        
        // Calculate initial LP tokens (geometric mean: sqrt(a * b))
        let lp_supply = integer_sqrt(reserve_a * reserve_b);
        
        if lp_supply == 0 {
            bail!("Initial liquidity too small");
        }
        
        // Create pool
        let pool = LiquidityPool {
            pair_id: pair_id.clone(),
            token_a: token_a.clone(),
            token_b: token_b.clone(),
            reserve_a,
            reserve_b,
            total_lp_tokens: lp_supply,
            lp_token_holders: HashMap::from([(creator.to_string(), lp_supply)]),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            total_volume_a: 0,
            total_volume_b: 0,
            fee_rate: self.default_fee_rate,
        };
        
        // Store pool
        let mut pools = self.pools.write().await;
        pools.insert(pair_id.clone(), pool);
        
        info!("✅ Liquidity pool created: {} ({} + {})", 
            pair_id, reserve_a, reserve_b);
        info!("   LP tokens minted: {} to {}", lp_supply, creator);
        
        Ok(pair_id)
    }
    
    /// Internal: Swap tokens using constant product formula
    /// Private method - use `swap_with_signature` for public API
    async fn swap_internal(
        &self,
        pair_id: &str,
        user: &str,
        token_in: &str,
        amount_in: u128,
        min_amount_out: u128,
    ) -> Result<u128> {
        if amount_in == 0 {
            bail!("Swap amount must be positive");
        }
        
        let mut pools = self.pools.write().await;
        let pool = pools.get_mut(pair_id)
            .ok_or_else(|| anyhow::anyhow!("Liquidity pool not found"))?;
        
        // Determine swap direction
        let (reserve_in, reserve_out, token_out, is_a_to_b) = 
            if token_in == pool.token_a {
                (pool.reserve_a, pool.reserve_b, &pool.token_b, true)
            } else if token_in == pool.token_b {
                (pool.reserve_b, pool.reserve_a, &pool.token_a, false)
            } else {
                bail!("Invalid token for this pair");
            };
        
        // Calculate swap output using constant product formula
        // (x + Δx * 0.997) * (y - Δy) = x * y
        // Δy = (y * Δx * 0.997) / (x + Δx * 0.997)
        
        let fee = (amount_in * pool.fee_rate as u128) / 10000;
        let amount_in_after_fee = amount_in - fee;
        
        // Use checked arithmetic to prevent overflow with large reserves
        let numerator = reserve_out.checked_mul(amount_in_after_fee)
            .ok_or_else(|| anyhow::anyhow!("Swap calculation overflow"))?;
        let denominator = reserve_in.checked_add(amount_in_after_fee)
            .ok_or_else(|| anyhow::anyhow!("Swap calculation overflow"))?;
        let amount_out = numerator / denominator;
        
        if amount_out == 0 {
            bail!("Output amount too small");
        }
        
        // Slippage protection
        if amount_out < min_amount_out {
            bail!("Slippage tolerance exceeded: expected {}, got {}", 
                min_amount_out, amount_out);
        }
        
        // Update reserves and volume with overflow protection
        if is_a_to_b {
            pool.reserve_a = pool.reserve_a.checked_add(amount_in)
                .ok_or_else(|| anyhow::anyhow!("Reserve overflow"))?;
            pool.reserve_b = pool.reserve_b.checked_sub(amount_out)
                .ok_or_else(|| anyhow::anyhow!("Reserve underflow"))?;
            pool.total_volume_a += amount_in;
        } else {
            pool.reserve_b += amount_in;
            pool.reserve_a -= amount_out;
            pool.total_volume_b += amount_in;
        }
        
        // Clone values before dropping pool lock
        let token_out_clone = token_out.clone();
        let pair_id_clone = pair_id.to_string();
        drop(pools);
        
        // Execute token transfers
        self.token_factory.transfer_internal(token_in, user, &pair_id_clone, amount_in).await?;
        self.token_factory.transfer_internal(&token_out_clone, &pair_id_clone, user, amount_out).await?;
        
        info!("✅ Swap executed: {} {} → {} {}", 
            amount_in, token_in, amount_out, token_out_clone);
        
        Ok(amount_out)
    }
    
    /// Internal: Add liquidity to existing pool
    /// Private method - use `add_liquidity_with_signature` for public API
    async fn add_liquidity_internal(
        &self,
        pair_id: &str,
        user: &str,
        amount_a_desired: u128,
        amount_b_desired: u128,
        amount_a_min: u128,
        amount_b_min: u128,
    ) -> Result<(u128, u128, u128)> {
        let mut pools = self.pools.write().await;
        let pool = pools.get_mut(pair_id)
            .ok_or_else(|| anyhow::anyhow!("Liquidity pool not found"))?;
        
        // Calculate optimal amounts to maintain pool ratio
        let optimal_b = (amount_a_desired * pool.reserve_b) / pool.reserve_a;
        let optimal_a = (amount_b_desired * pool.reserve_a) / pool.reserve_b;
        
        let (amount_a, amount_b) = if optimal_b <= amount_b_desired {
            (amount_a_desired, optimal_b)
        } else {
            (optimal_a, amount_b_desired)
        };
        
        // Check minimum amounts
        if amount_a < amount_a_min {
            bail!("Amount A below minimum: {} < {}", amount_a, amount_a_min);
        }
        if amount_b < amount_b_min {
            bail!("Amount B below minimum: {} < {}", amount_b, amount_b_min);
        }
        
        // Calculate LP tokens to mint (proportional to liquidity added)
        let lp_tokens = std::cmp::min(
            (amount_a * pool.total_lp_tokens) / pool.reserve_a,
            (amount_b * pool.total_lp_tokens) / pool.reserve_b,
        );
        
        if lp_tokens == 0 {
            bail!("Liquidity amount too small");
        }
        
        // Update pool state with overflow protection
        pool.reserve_a = pool.reserve_a.checked_add(amount_a)
            .ok_or_else(|| anyhow::anyhow!("Reserve A overflow"))?;
        pool.reserve_b = pool.reserve_b.checked_add(amount_b)
            .ok_or_else(|| anyhow::anyhow!("Reserve B overflow"))?;
        pool.total_lp_tokens = pool.total_lp_tokens.checked_add(lp_tokens)
            .ok_or_else(|| anyhow::anyhow!("LP token overflow"))?;
        
        // Mint LP tokens to user
        let user_lp = pool.lp_token_holders.entry(user.to_string()).or_insert(0);
        *user_lp += lp_tokens;
        
        // Clone values before dropping lock
        let token_a = pool.token_a.clone();
        let token_b = pool.token_b.clone();
        let pair_id_clone = pair_id.to_string();
        drop(pools);
        
        // Transfer tokens to pool
        self.token_factory.transfer_internal(&token_a, user, &pair_id_clone, amount_a).await?;
        self.token_factory.transfer_internal(&token_b, user, &pair_id_clone, amount_b).await?;
        
        info!("✅ Liquidity added: {} {} + {} {} → {} LP tokens", 
            amount_a, token_a, amount_b, token_b, lp_tokens);
        
        Ok((amount_a, amount_b, lp_tokens))
    }
    
    /// Internal: Remove liquidity from pool
    /// Private method - use `remove_liquidity_with_signature` for public API
    async fn remove_liquidity_internal(
        &self,
        pair_id: &str,
        user: &str,
        lp_tokens: u128,
        amount_a_min: u128,
        amount_b_min: u128,
    ) -> Result<(u128, u128)> {
        if lp_tokens == 0 {
            bail!("LP token amount must be positive");
        }
        
        let mut pools = self.pools.write().await;
        let pool = pools.get_mut(pair_id)
            .ok_or_else(|| anyhow::anyhow!("Liquidity pool not found"))?;
        
        // Verify user has sufficient LP tokens
        let user_lp = pool.lp_token_holders.get(user)
            .ok_or_else(|| anyhow::anyhow!("No LP tokens found"))?;
        
        if *user_lp < lp_tokens {
            bail!("Insufficient LP tokens: has {}, needs {}", user_lp, lp_tokens);
        }
        
        // Calculate amounts to return (proportional to LP tokens)
        let amount_a = (pool.reserve_a * lp_tokens) / pool.total_lp_tokens;
        let amount_b = (pool.reserve_b * lp_tokens) / pool.total_lp_tokens;
        
        // Check minimum amounts
        if amount_a < amount_a_min {
            bail!("Amount A below minimum: {} < {}", amount_a, amount_a_min);
        }
        if amount_b < amount_b_min {
            bail!("Amount B below minimum: {} < {}", amount_b, amount_b_min);
        }
        
        // Update pool state
        pool.reserve_a -= amount_a;
        pool.reserve_b -= amount_b;
        pool.total_lp_tokens -= lp_tokens;
        
        // Burn LP tokens from user
        let user_lp_mut = pool.lp_token_holders.get_mut(user).unwrap();
        *user_lp_mut -= lp_tokens;
        
        // Clone values before dropping lock
        let token_a = pool.token_a.clone();
        let token_b = pool.token_b.clone();
        let pair_id_clone = pair_id.to_string();
        drop(pools);
        
        // Transfer tokens back to user
        self.token_factory.transfer_internal(&token_a, &pair_id_clone, user, amount_a).await?;
        self.token_factory.transfer_internal(&token_b, &pair_id_clone, user, amount_b).await?;
        
        info!("✅ Liquidity removed: {} LP tokens → {} {} + {} {}", 
            lp_tokens, amount_a, token_a, amount_b, token_b);
        
        Ok((amount_a, amount_b))
    }
    
    /// Get pool information
    pub async fn get_pool(&self, pair_id: &str) -> Option<LiquidityPool> {
        let pools = self.pools.read().await;
        pools.get(pair_id).cloned()
    }
    
    /// Get price of token A in terms of token B
    pub async fn get_price(&self, pair_id: &str) -> Result<f64> {
        let pools = self.pools.read().await;
        let pool = pools.get(pair_id)
            .ok_or_else(|| anyhow::anyhow!("Pool not found"))?;
        
        let price = pool.reserve_b as f64 / pool.reserve_a as f64;
        Ok(price)
    }
    
    /// Get user's LP token balance
    pub async fn get_lp_balance(&self, pair_id: &str, user: &str) -> u128 {
        let pools = self.pools.read().await;
        if let Some(pool) = pools.get(pair_id) {
            *pool.lp_token_holders.get(user).unwrap_or(&0)
        } else {
            0
        }
    }

    /// Verify Ed25519 signature for DEX operations
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

    /// Swap with Ed25519 signature verification
    pub async fn swap_with_signature(
        &self,
        pair_id: &str,
        user: &str,
        token_in: &str,
        amount_in: u128,
        min_amount_out: u128,
        signature: &[u8],
        pubkey: &[u8; 32],
    ) -> Result<u128> {
        // Verify signature over swap data
        let sign_data = format!("swap:{}:{}:{}:{}:{}", pair_id, user, token_in, amount_in, min_amount_out);
        if !Self::verify_ed25519_signature(pubkey, sign_data.as_bytes(), signature) {
            bail!("Invalid Ed25519 signature on swap");
        }
        debug!("✅ Swap signature verified for {} {} in {}", amount_in, token_in, pair_id);
        
        // Perform the actual swap
        self.swap_internal(pair_id, user, token_in, amount_in, min_amount_out).await
    }

    /// Create pair with Ed25519 signature verification
    pub async fn create_pair_with_signature(
        &self,
        creator: &str,
        token_a: &str,
        token_b: &str,
        amount_a: u128,
        amount_b: u128,
        signature: &[u8],
        pubkey: &[u8; 32],
    ) -> Result<String> {
        let sign_data = format!("create_pair:{}:{}:{}:{}:{}", creator, token_a, token_b, amount_a, amount_b);
        if !Self::verify_ed25519_signature(pubkey, sign_data.as_bytes(), signature) {
            bail!("Invalid Ed25519 signature on create_pair");
        }
        debug!("✅ Create pair signature verified for {}/{}", token_a, token_b);
        
        self.create_pair_internal(creator, token_a, token_b, amount_a, amount_b).await
    }

    /// Add liquidity with Ed25519 signature verification
    pub async fn add_liquidity_with_signature(
        &self,
        pair_id: &str,
        user: &str,
        amount_a_desired: u128,
        amount_b_desired: u128,
        amount_a_min: u128,
        amount_b_min: u128,
        signature: &[u8],
        pubkey: &[u8; 32],
    ) -> Result<(u128, u128, u128)> {
        let sign_data = format!("add_liquidity:{}:{}:{}:{}:{}:{}", 
            pair_id, user, amount_a_desired, amount_b_desired, amount_a_min, amount_b_min);
        if !Self::verify_ed25519_signature(pubkey, sign_data.as_bytes(), signature) {
            bail!("Invalid Ed25519 signature on add_liquidity");
        }
        debug!("✅ Add liquidity signature verified for {} in {}", user, pair_id);
        
        self.add_liquidity_internal(pair_id, user, amount_a_desired, amount_b_desired, amount_a_min, amount_b_min).await
    }

    /// Remove liquidity with Ed25519 signature verification
    pub async fn remove_liquidity_with_signature(
        &self,
        pair_id: &str,
        user: &str,
        lp_tokens: u128,
        amount_a_min: u128,
        amount_b_min: u128,
        signature: &[u8],
        pubkey: &[u8; 32],
    ) -> Result<(u128, u128)> {
        let sign_data = format!("remove_liquidity:{}:{}:{}:{}:{}", 
            pair_id, user, lp_tokens, amount_a_min, amount_b_min);
        if !Self::verify_ed25519_signature(pubkey, sign_data.as_bytes(), signature) {
            bail!("Invalid Ed25519 signature on remove_liquidity");
        }
        debug!("✅ Remove liquidity signature verified for {} in {}", user, pair_id);
        
        self.remove_liquidity_internal(pair_id, user, lp_tokens, amount_a_min, amount_b_min).await
    }

    /// Get DEX statistics
    pub async fn get_statistics(&self) -> DexStatistics {
        let pools = self.pools.read().await;
        
        let total_pools = pools.len();
        let total_volume: u128 = pools.values()
            .map(|p| p.total_volume_a + p.total_volume_b)
            .sum();
        let total_liquidity: u128 = pools.values()
            .map(|p| p.reserve_a + p.reserve_b)
            .sum();
        
        DexStatistics {
            total_pools,
            total_volume,
            total_liquidity,
            default_fee_rate: self.default_fee_rate,
        }
    }
}

/// DEX statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexStatistics {
    pub total_pools: usize,
    pub total_volume: u128,
    pub total_liquidity: u128,
    pub default_fee_rate: u32,
}

/// Integer square root (for LP token calculation)
fn integer_sqrt(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    
    let mut x = n;
    let mut y = (x + 1) / 2;
    
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    
    x
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_pool() {
        let token_factory = Arc::new(TokenFactory::new());
        let dex = NativeDex::new(token_factory.clone());
        
        // Create two tokens
        let token_a = token_factory.create_token_internal(
            "sultan1alice",
            "Token A".to_string(),
            "TKNA".to_string(),
            6,
            1_000_000,
            None,
            None,
            None,
        ).await.unwrap();
        
        let token_b = token_factory.create_token_internal(
            "sultan1alice",
            "Token B".to_string(),
            "TKNB".to_string(),
            6,
            1_000_000,
            None,
            None,
            None,
        ).await.unwrap();
        
        // Create pool
        let pair_id = dex.create_pair_internal(
            "sultan1alice",
            &token_a,
            &token_b,
            100_000,
            200_000,
        ).await.unwrap();
        
        // Verify pool created
        let pool = dex.get_pool(&pair_id).await.unwrap();
        assert_eq!(pool.reserve_a, 100_000);
        assert_eq!(pool.reserve_b, 200_000);
    }
    
    #[tokio::test]
    async fn test_swap() {
        let token_factory = Arc::new(TokenFactory::new());
        let dex = NativeDex::new(token_factory.clone());
        
        // Setup tokens - alice creates both tokens so she can create the pool
        let token_a = token_factory.create_token_internal(
            "sultan1alice",
            "Token A".to_string(),
            "TKNA".to_string(),
            6,
            1_000_000,
            None,
            None,
            None,
        ).await.unwrap();
        
        let token_b = token_factory.create_token_internal(
            "sultan1alice",
            "Token B".to_string(),
            "TKNB".to_string(),
            6,
            1_000_000,
            None,
            None,
            None,
        ).await.unwrap();
        
        // Transfer some token_a to bob for swapping
        token_factory.transfer_internal(&token_a, "sultan1alice", "sultan1bob", 10_000).await.unwrap();
        
        let pair_id = dex.create_pair_internal(
            "sultan1alice",
            &token_a,
            &token_b,
            100_000,
            100_000,
        ).await.unwrap();
        
        // Bob swaps 1000 token_a for token_b
        let amount_out = dex.swap_internal(
            &pair_id,
            "sultan1bob",
            &token_a,
            1000,
            0, // No slippage protection for test
        ).await.unwrap();
        
        // The constant product formula with 0.3% fee:
        // amount_out = (reserve_out * amount_in_after_fee) / (reserve_in + amount_in_after_fee)
        // fee = 1000 * 30 / 10000 = 3
        // amount_in_after_fee = 1000 - 3 = 997
        // amount_out = (100000 * 997) / (100000 + 997) = 99700000 / 100997 = 986
        // So we should expect ~986, not 990+
        assert!(amount_out > 980, "amount_out should be > 980, got {}", amount_out);
        assert!(amount_out < 1000, "amount_out should be < 1000, got {}", amount_out);
    }

    #[tokio::test]
    async fn test_add_and_remove_liquidity() {
        let token_factory = Arc::new(TokenFactory::new());
        let dex = NativeDex::new(token_factory.clone());
        
        // Create tokens
        let token_a = token_factory.create_token_internal(
            "sultan1alice", "Token A".to_string(), "TKNA".to_string(),
            6, 10_000_000, None, None, None,
        ).await.unwrap();
        
        let token_b = token_factory.create_token_internal(
            "sultan1alice", "Token B".to_string(), "TKNB".to_string(),
            6, 10_000_000, None, None, None,
        ).await.unwrap();
        
        // Create pool
        let pair_id = dex.create_pair_internal(
            "sultan1alice", &token_a, &token_b, 100_000, 200_000,
        ).await.unwrap();
        
        // Get initial LP balance
        let initial_lp = dex.get_lp_balance(&pair_id, "sultan1alice").await;
        assert!(initial_lp > 0);
        
        // Add more liquidity
        let (lp_tokens, _, _) = dex.add_liquidity_internal(
            &pair_id, "sultan1alice", 50_000, 100_000, 0, 0,
        ).await.unwrap();
        assert!(lp_tokens > 0);
        
        // Remove some liquidity
        let (amount_a, amount_b) = dex.remove_liquidity_internal(
            &pair_id, "sultan1alice", lp_tokens / 2, 0, 0,
        ).await.unwrap();
        assert!(amount_a > 0);
        assert!(amount_b > 0);
    }

    #[tokio::test]
    async fn test_get_price() {
        let token_factory = Arc::new(TokenFactory::new());
        let dex = NativeDex::new(token_factory.clone());
        
        let token_a = token_factory.create_token_internal(
            "sultan1alice", "Token A".to_string(), "TKNA".to_string(),
            6, 1_000_000, None, None, None,
        ).await.unwrap();
        
        let token_b = token_factory.create_token_internal(
            "sultan1alice", "Token B".to_string(), "TKNB".to_string(),
            6, 1_000_000, None, None, None,
        ).await.unwrap();
        
        let pair_id = dex.create_pair_internal(
            "sultan1alice", &token_a, &token_b, 100_000, 200_000,
        ).await.unwrap();
        
        let price = dex.get_price(&pair_id).await.unwrap();
        // Price of A in terms of B = reserve_b / reserve_a = 200000 / 100000 = 2.0
        assert!((price - 2.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_slippage_protection() {
        let token_factory = Arc::new(TokenFactory::new());
        let dex = NativeDex::new(token_factory.clone());
        
        let token_a = token_factory.create_token_internal(
            "sultan1alice", "Token A".to_string(), "TKNA".to_string(),
            6, 1_000_000, None, None, None,
        ).await.unwrap();
        
        let token_b = token_factory.create_token_internal(
            "sultan1alice", "Token B".to_string(), "TKNB".to_string(),
            6, 1_000_000, None, None, None,
        ).await.unwrap();
        
        // Transfer some to bob
        token_factory.transfer_internal(&token_a, "sultan1alice", "sultan1bob", 10_000).await.unwrap();
        
        let pair_id = dex.create_pair_internal(
            "sultan1alice", &token_a, &token_b, 100_000, 100_000,
        ).await.unwrap();
        
        // Try to swap with unrealistic slippage expectation
        let result = dex.swap_internal(
            &pair_id, "sultan1bob", &token_a, 1000, 999, // expect 999 but will get ~986
        ).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Slippage tolerance exceeded"));
    }

    #[tokio::test]
    async fn test_statistics() {
        let token_factory = Arc::new(TokenFactory::new());
        let dex = NativeDex::new(token_factory.clone());
        
        let token_a = token_factory.create_token_internal(
            "sultan1alice", "Token A".to_string(), "TKNA".to_string(),
            6, 1_000_000, None, None, None,
        ).await.unwrap();
        
        let token_b = token_factory.create_token_internal(
            "sultan1alice", "Token B".to_string(), "TKNB".to_string(),
            6, 1_000_000, None, None, None,
        ).await.unwrap();
        
        dex.create_pair_internal(
            "sultan1alice", &token_a, &token_b, 100_000, 100_000,
        ).await.unwrap();
        
        let stats = dex.get_statistics().await;
        assert_eq!(stats.total_pools, 1);
        assert_eq!(stats.total_liquidity, 200_000);
        assert_eq!(stats.default_fee_rate, 30);
    }

    #[test]
    fn test_integer_sqrt() {
        assert_eq!(integer_sqrt(0), 0);
        assert_eq!(integer_sqrt(1), 1);
        assert_eq!(integer_sqrt(4), 2);
        assert_eq!(integer_sqrt(100), 10);
        assert_eq!(integer_sqrt(10000), 100);
        // sqrt(100 * 200) = sqrt(20000) = 141 (rounded down)
        assert_eq!(integer_sqrt(20000), 141);
    }

    #[tokio::test]
    async fn test_create_pair_with_signature_rejects_invalid() {
        let token_factory = Arc::new(TokenFactory::new());
        let dex = NativeDex::new(token_factory.clone());
        
        let token_a = token_factory.create_token_internal(
            "sultan1alice", "Token A".to_string(), "TKNA".to_string(),
            6, 1_000_000, None, None, None,
        ).await.unwrap();
        
        let token_b = token_factory.create_token_internal(
            "sultan1alice", "Token B".to_string(), "TKNB".to_string(),
            6, 1_000_000, None, None, None,
        ).await.unwrap();
        
        let pubkey = [1u8; 32]; // Non-zero to trigger signature verification
        let invalid_sig = vec![0u8; 64];
        
        let result = dex.create_pair_with_signature(
            "sultan1alice", &token_a, &token_b, 100_000, 100_000,
            &invalid_sig, &pubkey,
        ).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid Ed25519 signature"));
    }

    #[tokio::test]
    async fn test_swap_with_signature_rejects_invalid() {
        let token_factory = Arc::new(TokenFactory::new());
        let dex = NativeDex::new(token_factory.clone());
        
        let token_a = token_factory.create_token_internal(
            "sultan1alice", "Token A".to_string(), "TKNA".to_string(),
            6, 1_000_000, None, None, None,
        ).await.unwrap();
        
        let token_b = token_factory.create_token_internal(
            "sultan1alice", "Token B".to_string(), "TKNB".to_string(),
            6, 1_000_000, None, None, None,
        ).await.unwrap();
        
        // Create pair first
        dex.create_pair_internal(
            "sultan1alice", &token_a, &token_b, 100_000, 100_000,
        ).await.unwrap();
        
        let pair_id = format!("pair/{}/{}", token_a, token_b);
        // Use non-zero pubkey to ensure signature verification runs and fails
        let pubkey = [1u8; 32];
        let invalid_sig = vec![0u8; 64];
        
        let result = dex.swap_with_signature(
            &pair_id, "sultan1bob", &token_a, 1000, 0,
            &invalid_sig, &pubkey,
        ).await;
        
        assert!(result.is_err(), "Expected error");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Ed25519 signature"), "Expected sig error, got: {}", err_msg);
    }

    #[tokio::test]
    async fn test_add_liquidity_with_signature_rejects_invalid() {
        let token_factory = Arc::new(TokenFactory::new());
        let dex = NativeDex::new(token_factory.clone());
        
        let token_a = token_factory.create_token_internal(
            "sultan1alice", "Token A".to_string(), "TKNA".to_string(),
            6, 1_000_000, None, None, None,
        ).await.unwrap();
        
        let token_b = token_factory.create_token_internal(
            "sultan1alice", "Token B".to_string(), "TKNB".to_string(),
            6, 1_000_000, None, None, None,
        ).await.unwrap();
        
        // Create pair first
        dex.create_pair_internal(
            "sultan1alice", &token_a, &token_b, 100_000, 100_000,
        ).await.unwrap();
        
        let pair_id = format!("pair/{}/{}", token_a, token_b);
        let pubkey = [1u8; 32]; // Non-zero to trigger signature verification
        let invalid_sig = vec![0u8; 64];
        
        let result = dex.add_liquidity_with_signature(
            &pair_id, "sultan1bob", 10_000, 10_000, 0, 0,
            &invalid_sig, &pubkey,
        ).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid Ed25519 signature"));
    }

    #[tokio::test]
    async fn test_remove_liquidity_with_signature_rejects_invalid() {
        let token_factory = Arc::new(TokenFactory::new());
        let dex = NativeDex::new(token_factory.clone());
        
        let token_a = token_factory.create_token_internal(
            "sultan1alice", "Token A".to_string(), "TKNA".to_string(),
            6, 1_000_000, None, None, None,
        ).await.unwrap();
        
        let token_b = token_factory.create_token_internal(
            "sultan1alice", "Token B".to_string(), "TKNB".to_string(),
            6, 1_000_000, None, None, None,
        ).await.unwrap();
        
        // Create pair first (this gives LP tokens to alice)
        dex.create_pair_internal(
            "sultan1alice", &token_a, &token_b, 100_000, 100_000,
        ).await.unwrap();
        
        let pair_id = format!("pair/{}/{}", token_a, token_b);
        let pubkey = [1u8; 32]; // Non-zero to trigger signature verification
        let invalid_sig = vec![0u8; 64];
        
        let result = dex.remove_liquidity_with_signature(
            &pair_id, "sultan1alice", 1000, 0, 0,
            &invalid_sig, &pubkey,
        ).await;
        
        assert!(result.is_err(), "Expected error");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Ed25519 signature"), "Expected sig error, got: {}", err_msg);
    }

    #[tokio::test]
    async fn test_dex_statistics_comprehensive() {
        let token_factory = Arc::new(TokenFactory::new());
        let dex = NativeDex::new(token_factory.clone());
        
        // Create tokens
        let token_a = token_factory.create_token_internal(
            "sultan1alice", "Token A".to_string(), "TOKA".to_string(),
            6, 10_000_000, None, None, None,
        ).await.unwrap();
        
        let token_b = token_factory.create_token_internal(
            "sultan1alice", "Token B".to_string(), "TOKB".to_string(),
            6, 10_000_000, None, None, None,
        ).await.unwrap();
        
        // Initial stats - no pools
        let stats = dex.get_statistics().await;
        assert_eq!(stats.total_pools, 0);
        assert_eq!(stats.total_volume, 0);
        assert_eq!(stats.total_liquidity, 0);
        assert_eq!(stats.default_fee_rate, 30); // 0.3% = 30 basis points
        
        // Create pair (pool)
        dex.create_pair_internal(
            "sultan1alice", &token_a, &token_b, 100_000, 100_000,
        ).await.unwrap();
        
        // Stats after pair creation
        let stats = dex.get_statistics().await;
        assert_eq!(stats.total_pools, 1);
        assert_eq!(stats.total_liquidity, 200_000); // 100k + 100k
        
        // Transfer tokens to bob so he can swap
        token_factory.transfer_internal(&token_a, "sultan1alice", "sultan1bob", 10_000).await.unwrap();
        
        // Perform swap - swap_internal takes (pair_id, user, token_in, amount_in, min_amount_out)
        let pair_id = format!("pair/{}/{}", token_a, token_b);
        dex.swap_internal(
            &pair_id, "sultan1bob", &token_a, 1000, 1,
        ).await.unwrap();
        
        // Stats after swap - volume increased
        let stats = dex.get_statistics().await;
        assert!(stats.total_volume > 0, "Volume should increase after swap");
        
        // Add more liquidity
        dex.add_liquidity_internal(
            &pair_id,
            "sultan1alice", 50_000, 50_000, 0, 0,
        ).await.unwrap();
        
        // Stats after adding liquidity
        let stats = dex.get_statistics().await;
        assert!(stats.total_liquidity > 200_000, "Liquidity should increase");
        
        // Remove liquidity - use get_lp_balance method
        let lp_balance = dex.get_lp_balance(&pair_id, "sultan1alice").await;
        dex.remove_liquidity_internal(
            &pair_id, "sultan1alice", lp_balance / 4, 0, 0,
        ).await.unwrap();
        
        // Final stats - liquidity decreased after removal
        let stats = dex.get_statistics().await;
        assert_eq!(stats.total_pools, 1);
        assert!(stats.total_volume > 0, "Volume persists");
    }

    #[tokio::test]
    async fn test_multiple_pools_stats() {
        let token_factory = Arc::new(TokenFactory::new());
        let dex = NativeDex::new(token_factory.clone());
        
        // Create 3 tokens
        let token_a = token_factory.create_token_internal(
            "sultan1creator", "Token A".to_string(), "AAA".to_string(),
            6, 10_000_000, None, None, None,
        ).await.unwrap();
        
        let token_b = token_factory.create_token_internal(
            "sultan1creator", "Token B".to_string(), "BBB".to_string(),
            6, 10_000_000, None, None, None,
        ).await.unwrap();
        
        let token_c = token_factory.create_token_internal(
            "sultan1creator", "Token C".to_string(), "CCC".to_string(),
            6, 10_000_000, None, None, None,
        ).await.unwrap();
        
        // Create 3 pairs: A-B, A-C, B-C
        dex.create_pair_internal("sultan1creator", &token_a, &token_b, 100_000, 100_000).await.unwrap();
        dex.create_pair_internal("sultan1creator", &token_a, &token_c, 200_000, 200_000).await.unwrap();
        dex.create_pair_internal("sultan1creator", &token_b, &token_c, 300_000, 300_000).await.unwrap();
        
        let stats = dex.get_statistics().await;
        assert_eq!(stats.total_pools, 3);
        
        // Total liquidity = (100k+100k) + (200k+200k) + (300k+300k) = 1.2M
        assert_eq!(stats.total_liquidity, 1_200_000);
        
        // Verify each pool exists
        let pair_ab = format!("pair/{}/{}", token_a, token_b);
        let pair_ac = format!("pair/{}/{}", token_a, token_c);
        let pair_bc = format!("pair/{}/{}", token_b, token_c);
        
        assert!(dex.get_pool(&pair_ab).await.is_some());
        assert!(dex.get_pool(&pair_ac).await.is_some());
        assert!(dex.get_pool(&pair_bc).await.is_some());
    }
}
