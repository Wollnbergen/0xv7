// Native DEX Module - Automated Market Maker (AMM)
// Enables token swapping without smart contracts using constant product formula

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

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
    
    /// Create a new liquidity pool
    pub async fn create_pair(
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
        self.token_factory.transfer(&token_a, creator, &pair_id, reserve_a).await?;
        self.token_factory.transfer(&token_b, creator, &pair_id, reserve_b).await?;
        
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
    
    /// Swap tokens using constant product formula
    pub async fn swap(
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
        
        let numerator = reserve_out * amount_in_after_fee;
        let denominator = reserve_in + amount_in_after_fee;
        let amount_out = numerator / denominator;
        
        if amount_out == 0 {
            bail!("Output amount too small");
        }
        
        // Slippage protection
        if amount_out < min_amount_out {
            bail!("Slippage tolerance exceeded: expected {}, got {}", 
                min_amount_out, amount_out);
        }
        
        // Update reserves and volume
        if is_a_to_b {
            pool.reserve_a += amount_in;
            pool.reserve_b -= amount_out;
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
        self.token_factory.transfer(token_in, user, &pair_id_clone, amount_in).await?;
        self.token_factory.transfer(&token_out_clone, &pair_id_clone, user, amount_out).await?;
        
        info!("✅ Swap executed: {} {} → {} {}", 
            amount_in, token_in, amount_out, token_out_clone);
        
        Ok(amount_out)
    }
    
    /// Add liquidity to existing pool
    pub async fn add_liquidity(
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
        
        // Update pool state
        pool.reserve_a += amount_a;
        pool.reserve_b += amount_b;
        pool.total_lp_tokens += lp_tokens;
        
        // Mint LP tokens to user
        let user_lp = pool.lp_token_holders.entry(user.to_string()).or_insert(0);
        *user_lp += lp_tokens;
        
        // Clone values before dropping lock
        let token_a = pool.token_a.clone();
        let token_b = pool.token_b.clone();
        let pair_id_clone = pair_id.to_string();
        drop(pools);
        
        // Transfer tokens to pool
        self.token_factory.transfer(&token_a, user, &pair_id_clone, amount_a).await?;
        self.token_factory.transfer(&token_b, user, &pair_id_clone, amount_b).await?;
        
        info!("✅ Liquidity added: {} {} + {} {} → {} LP tokens", 
            amount_a, token_a, amount_b, token_b, lp_tokens);
        
        Ok((amount_a, amount_b, lp_tokens))
    }
    
    /// Remove liquidity from pool
    pub async fn remove_liquidity(
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
        self.token_factory.transfer(&token_a, &pair_id_clone, user, amount_a).await?;
        self.token_factory.transfer(&token_b, &pair_id_clone, user, amount_b).await?;
        
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
        let token_a = token_factory.create_token(
            "sultan1alice",
            "Token A".to_string(),
            "TKNA".to_string(),
            6,
            1_000_000,
            None,
            None,
            None,
        ).await.unwrap();
        
        let token_b = token_factory.create_token(
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
        let pair_id = dex.create_pair(
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
        
        // Setup tokens and pool
        let token_a = token_factory.create_token(
            "sultan1alice",
            "Token A".to_string(),
            "TKNA".to_string(),
            6,
            1_000_000,
            None,
            None,
            None,
        ).await.unwrap();
        
        let token_b = token_factory.create_token(
            "sultan1bob",
            "Token B".to_string(),
            "TKNB".to_string(),
            6,
            1_000_000,
            None,
            None,
            None,
        ).await.unwrap();
        
        // Transfer some token_a to bob for swapping
        token_factory.transfer(&token_a, "sultan1alice", "sultan1bob", 10_000).await.unwrap();
        
        let pair_id = dex.create_pair(
            "sultan1alice",
            &token_a,
            &token_b,
            100_000,
            100_000,
        ).await.unwrap();
        
        // Bob swaps 1000 token_a for token_b
        let amount_out = dex.swap(
            &pair_id,
            "sultan1bob",
            &token_a,
            1000,
            0, // No slippage protection for test
        ).await.unwrap();
        
        // Verify swap executed (should get ~997 token_b minus fees)
        assert!(amount_out > 990);
        assert!(amount_out < 1000);
    }
}
