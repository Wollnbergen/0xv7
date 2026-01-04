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
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn};

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
    /// LP token denom for transferable LP positions
    pub lp_denom: String,
    pub created_at: u64,
    pub total_volume_a: u128,
    pub total_volume_b: u128,
    /// Total fee in basis points (30 = 0.3%)
    pub fee_rate: u32,
    /// Fee to LP providers in basis points (20 = 0.2%)
    pub lp_fee_rate: u32,
    /// Fee to protocol wallet in basis points (10 = 0.1%)
    pub protocol_fee_rate: u32,
    /// Total protocol fees collected (token_a)
    pub protocol_fees_a: u128,
    /// Total protocol fees collected (token_b)
    pub protocol_fees_b: u128,
}

/// Native DEX for token swapping
pub struct NativeDex {
    pub pools: Arc<RwLock<HashMap<String, LiquidityPool>>>,
    pub token_factory: Arc<TokenFactory>,
    /// Total fee rate in basis points (30 = 0.3%)
    pub default_fee_rate: u32,
    /// Fee to LP providers (20 = 0.2%)
    pub default_lp_fee_rate: u32,
    /// Fee to protocol wallet (10 = 0.1%)
    pub default_protocol_fee_rate: u32,
    /// Protocol fee recipient wallet address
    pub protocol_fee_address: String,
    /// Path to persist pool data
    pub storage_path: Option<PathBuf>,
}

impl NativeDex {
    /// Create a new DEX with default fee configuration
    /// Fee split: 0.2% to LP providers, 0.1% to protocol wallet
    pub fn new(token_factory: Arc<TokenFactory>) -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            token_factory,
            default_fee_rate: 30,         // 0.3% total fee
            default_lp_fee_rate: 20,      // 0.2% to LPs (stays in reserves)
            default_protocol_fee_rate: 10, // 0.1% to protocol wallet
            protocol_fee_address: String::new(), // Must be set before use
            storage_path: None,
        }
    }

    /// Create a new DEX with custom protocol fee address and optional persistence
    pub fn with_config(
        token_factory: Arc<TokenFactory>,
        protocol_fee_address: String,
        storage_path: Option<PathBuf>,
    ) -> Self {
        let mut dex = Self::new(token_factory);
        dex.protocol_fee_address = protocol_fee_address;
        dex.storage_path = storage_path;
        dex
    }

    /// Set the protocol fee address
    pub fn set_protocol_fee_address(&mut self, address: String) {
        self.protocol_fee_address = address;
    }

    /// Load pools from persistent storage
    pub async fn load_from_storage(&self) -> Result<()> {
        let Some(path) = &self.storage_path else {
            return Ok(());
        };
        
        let pools_file = path.join("pools.json");
        if !pools_file.exists() {
            info!("No existing DEX state found at {:?}", pools_file);
            return Ok(());
        }
        
        let data = tokio::fs::read_to_string(&pools_file).await?;
        let loaded_pools: HashMap<String, LiquidityPool> = serde_json::from_str(&data)?;
        
        let mut pools = self.pools.write().await;
        *pools = loaded_pools;
        
        info!("📂 Loaded {} pools from {:?}", pools.len(), pools_file);
        Ok(())
    }

    /// Save pools to persistent storage
    pub async fn save_to_storage(&self) -> Result<()> {
        let Some(path) = &self.storage_path else {
            return Ok(());
        };
        
        // Ensure directory exists
        tokio::fs::create_dir_all(path).await?;
        
        let pools_file = path.join("pools.json");
        let pools = self.pools.read().await;
        let data = serde_json::to_string_pretty(&*pools)?;
        
        tokio::fs::write(&pools_file, data).await?;
        debug!("💾 Saved {} pools to {:?}", pools.len(), pools_file);
        Ok(())
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
        
        // Create LP token as a real denom (transferable)
        let lp_denom = format!("lp/{}/{}", token_a, token_b);
        self.token_factory.create_lp_token_internal(
            &lp_denom,
            &pair_id,
            lp_supply,
            &token_a,
            &token_b,
        ).await?;
        
        // Transfer LP tokens to the creator
        self.token_factory.transfer_internal(&lp_denom, &pair_id, creator, lp_supply).await?;
        
        // Create pool
        let pool = LiquidityPool {
            pair_id: pair_id.clone(),
            token_a: token_a.clone(),
            token_b: token_b.clone(),
            reserve_a,
            reserve_b,
            total_lp_tokens: lp_supply,
            lp_denom: lp_denom.clone(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            total_volume_a: 0,
            total_volume_b: 0,
            fee_rate: self.default_fee_rate,
            lp_fee_rate: self.default_lp_fee_rate,
            protocol_fee_rate: self.default_protocol_fee_rate,
            protocol_fees_a: 0,
            protocol_fees_b: 0,
        };
        
        // Store pool
        let mut pools = self.pools.write().await;
        pools.insert(pair_id.clone(), pool);
        drop(pools);
        
        // Persist to storage
        if let Err(e) = self.save_to_storage().await {
            warn!("Failed to persist pool state: {}", e);
        }
        
        info!("✅ Liquidity pool created: {} ({} + {})", 
            pair_id, reserve_a, reserve_b);
        info!("   LP tokens minted: {} {} to {}", lp_supply, lp_denom, creator);
        
        Ok(pair_id)
    }
    
    /// Internal: Swap tokens using constant product formula
    /// Private method - use `swap_with_signature` for public API
    /// Fee split: 0.2% to LP providers (stays in reserves), 0.1% to protocol wallet
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
        
        // Calculate fees: 0.3% total = 0.2% to LP (stays in pool) + 0.1% to protocol
        // LP fee stays in reserves, protocol fee is extracted
        let total_fee = (amount_in * pool.fee_rate as u128) / 10000;
        let protocol_fee = (amount_in * pool.protocol_fee_rate as u128) / 10000;
        let lp_fee = total_fee - protocol_fee; // 0.2% stays in reserves
        
        // Amount used for swap calculation (after total fee)
        let amount_in_after_fee = amount_in - total_fee;
        
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
        
        // Update reserves with LP fee included, track protocol fees separately
        // LP providers earn via the lp_fee staying in reserves
        if is_a_to_b {
            pool.reserve_a = pool.reserve_a.checked_add(amount_in - protocol_fee)
                .ok_or_else(|| anyhow::anyhow!("Reserve overflow"))?;
            pool.reserve_b = pool.reserve_b.checked_sub(amount_out)
                .ok_or_else(|| anyhow::anyhow!("Reserve underflow"))?;
            pool.total_volume_a += amount_in;
            pool.protocol_fees_a += protocol_fee;
        } else {
            pool.reserve_b = pool.reserve_b.checked_add(amount_in - protocol_fee)
                .ok_or_else(|| anyhow::anyhow!("Reserve overflow"))?;
            pool.reserve_a = pool.reserve_a.checked_sub(amount_out)
                .ok_or_else(|| anyhow::anyhow!("Reserve underflow"))?;
            pool.total_volume_b += amount_in;
            pool.protocol_fees_b += protocol_fee;
        }
        
        // Clone values before dropping pool lock
        let token_out_clone = token_out.clone();
        let pair_id_clone = pair_id.to_string();
        let protocol_fee_addr = self.protocol_fee_address.clone();
        drop(pools);
        
        // Execute token transfers
        // User sends full amount_in (includes fees)
        self.token_factory.transfer_internal(token_in, user, &pair_id_clone, amount_in - protocol_fee).await?;
        
        // Protocol fee goes directly to protocol wallet (if configured)
        if protocol_fee > 0 && !protocol_fee_addr.is_empty() {
            self.token_factory.transfer_internal(token_in, user, &protocol_fee_addr, protocol_fee).await?;
            debug!("💰 Protocol fee: {} {} to {}", protocol_fee, token_in, protocol_fee_addr);
        } else if protocol_fee > 0 {
            // If no protocol address configured, fee stays in pool as extra LP rewards
            self.token_factory.transfer_internal(token_in, user, &pair_id_clone, protocol_fee).await?;
        }
        
        // User receives output tokens
        self.token_factory.transfer_internal(&token_out_clone, &pair_id_clone, user, amount_out).await?;
        
        // Persist state
        if let Err(e) = self.save_to_storage().await {
            warn!("Failed to persist pool state: {}", e);
        }
        
        info!("✅ Swap executed: {} {} → {} {} (LP fee: {}, protocol fee: {})", 
            amount_in, token_in, amount_out, token_out_clone, lp_fee, protocol_fee);
        
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
        
        // Clone values before dropping lock
        let token_a = pool.token_a.clone();
        let token_b = pool.token_b.clone();
        let lp_denom = pool.lp_denom.clone();
        let pair_id_clone = pair_id.to_string();
        drop(pools);
        
        // Transfer tokens to pool
        self.token_factory.transfer_internal(&token_a, user, &pair_id_clone, amount_a).await?;
        self.token_factory.transfer_internal(&token_b, user, &pair_id_clone, amount_b).await?;
        
        // Mint LP tokens to user (as real tokens)
        self.token_factory.mint_internal(&lp_denom, user, lp_tokens).await?;
        
        // Persist state
        if let Err(e) = self.save_to_storage().await {
            warn!("Failed to persist pool state: {}", e);
        }
        
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
        
        // Verify user has sufficient LP tokens (check actual token balance)
        let lp_denom = pool.lp_denom.clone();
        let user_lp_balance = self.token_factory.get_balance(&lp_denom, user).await;
        
        if user_lp_balance < lp_tokens {
            bail!("Insufficient LP tokens: has {}, needs {}", user_lp_balance, lp_tokens);
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
        
        // Clone values before dropping lock
        let token_a = pool.token_a.clone();
        let token_b = pool.token_b.clone();
        let pair_id_clone = pair_id.to_string();
        drop(pools);
        
        // Burn LP tokens from user (using burn_internal)
        self.token_factory.burn_internal(&lp_denom, user, lp_tokens).await?;
        
        // Transfer tokens back to user
        self.token_factory.transfer_internal(&token_a, &pair_id_clone, user, amount_a).await?;
        self.token_factory.transfer_internal(&token_b, &pair_id_clone, user, amount_b).await?;
        
        // Persist state
        if let Err(e) = self.save_to_storage().await {
            warn!("Failed to persist pool state: {}", e);
        }
        
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
    
    /// Get user's LP token balance (now uses token_factory for real token balance)
    pub async fn get_lp_balance(&self, pair_id: &str, user: &str) -> u128 {
        let pools = self.pools.read().await;
        if let Some(pool) = pools.get(pair_id) {
            let lp_denom = pool.lp_denom.clone();
            drop(pools);
            self.token_factory.get_balance(&lp_denom, user).await
        } else {
            0
        }
    }
    
    /// Get the LP token denom for a pool
    pub async fn get_lp_denom(&self, pair_id: &str) -> Option<String> {
        let pools = self.pools.read().await;
        pools.get(pair_id).map(|p| p.lp_denom.clone())
    }
    
    /// Collect accumulated protocol fees to the protocol wallet
    /// SECURITY: Only callable by protocol fee address (treasury) with signature verification
    pub async fn collect_protocol_fees_with_signature(
        &self,
        pair_id: &str,
        caller: &str,
        signature: &[u8],
        pubkey: &[u8; 32],
    ) -> Result<(u128, u128)> {
        // Verify caller is the protocol fee address (treasury only)
        if caller != self.protocol_fee_address {
            bail!("Only protocol treasury can collect fees");
        }
        
        // Verify signature
        let sign_data = format!("collect_fees:{}:{}", pair_id, caller);
        if !Self::verify_ed25519_signature(pubkey, sign_data.as_bytes(), signature) {
            bail!("Invalid Ed25519 signature on collect_fees");
        }
        
        self.collect_protocol_fees_internal(pair_id).await
    }
    
    /// Internal: Collect protocol fees (trusted caller only)
    async fn collect_protocol_fees_internal(&self, pair_id: &str) -> Result<(u128, u128)> {
        if self.protocol_fee_address.is_empty() {
            bail!("Protocol fee address not configured");
        }
        
        let mut pools = self.pools.write().await;
        let pool = pools.get_mut(pair_id)
            .ok_or_else(|| anyhow::anyhow!("Pool not found"))?;
        
        let fees_a = pool.protocol_fees_a;
        let fees_b = pool.protocol_fees_b;
        
        if fees_a == 0 && fees_b == 0 {
            return Ok((0, 0));
        }
        
        // Reset collected fees
        pool.protocol_fees_a = 0;
        pool.protocol_fees_b = 0;
        
        let token_a = pool.token_a.clone();
        let token_b = pool.token_b.clone();
        let protocol_addr = self.protocol_fee_address.clone();
        drop(pools);
        
        // Transfer fees to protocol wallet
        if fees_a > 0 {
            self.token_factory.transfer_internal(&token_a, pair_id, &protocol_addr, fees_a).await?;
        }
        if fees_b > 0 {
            self.token_factory.transfer_internal(&token_b, pair_id, &protocol_addr, fees_b).await?;
        }
        
        // Persist state
        if let Err(e) = self.save_to_storage().await {
            warn!("Failed to persist pool state: {}", e);
        }
        
        info!("💰 Protocol fees collected: {} {} + {} {} to {}", 
            fees_a, token_a, fees_b, token_b, protocol_addr);
        
        Ok((fees_a, fees_b))
    }
    
    /// Get accumulated protocol fees for a pool (read-only)
    pub async fn get_protocol_fees(&self, pair_id: &str) -> Result<(u128, u128)> {
        let pools = self.pools.read().await;
        let pool = pools.get(pair_id)
            .ok_or_else(|| anyhow::anyhow!("Pool not found"))?;
        Ok((pool.protocol_fees_a, pool.protocol_fees_b))
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
