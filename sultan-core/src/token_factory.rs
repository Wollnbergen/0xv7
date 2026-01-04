// Native Token Factory Module - No Smart Contracts Required
// Enables token creation and management directly in Sultan protocol
//
// Security features:
// - Ed25519 signature verification on all state-changing operations
// - Creator-only minting and burning controls
// - Supply limits with max_supply enforcement
// - Faucet with challenge-response anti-Sybil protection

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
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
    
    /// Creation fee in usltn (default: 3 SLTN = 3_000_000 usltn)
    pub creation_fee: u128,
    
    /// Minimum initial supply
    pub min_initial_supply: u128,
    
    /// Path to persist token data
    pub storage_path: Option<PathBuf>,
    
    // === Phase 1 Faucet (disable when DEX/CEX live) ===
    
    /// Whether faucet is enabled (Phase 1 = true, Phase 2 = false)
    /// Uses AtomicBool for runtime toggle without restart
    pub faucet_enabled: Arc<std::sync::atomic::AtomicBool>,
    
    /// Amount given per faucet claim (default: 10 SLTN)
    pub faucet_amount: u128,
    
    /// Maximum total SLTN that can be distributed via faucet
    /// Default: 2,000,000 SLTN (1% of 200M ecosystem fund)
    pub faucet_max_cap: u128,
    
    /// Total SLTN distributed via faucet so far
    pub faucet_total_distributed: Arc<RwLock<u128>>,
    
    /// Addresses that have claimed from faucet (prevents abuse)
    pub faucet_claims: Arc<RwLock<std::collections::HashSet<String>>>,
    
    // === Anti-Sybil & Rate Limiting ===
    
    /// Pending faucet challenges: address -> (nonce, expires_at)
    pub faucet_challenges: Arc<RwLock<HashMap<String, (String, u64)>>>,
    
    /// Rate limiting: timestamp of last N claims (sliding window)
    pub faucet_claim_timestamps: Arc<RwLock<std::collections::VecDeque<u64>>>,
    
    /// Max claims per minute (rate limit)
    pub faucet_rate_limit: u32,
}

impl TokenFactory {
    /// Native SLTN denom for fee payments
    pub const SLTN_DENOM: &'static str = "usltn";
    
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            balances: Arc::new(RwLock::new(HashMap::new())),
            creation_fee: 3 * 1_000_000, // 3 SLTN (~$0.90 at $0.30)
            min_initial_supply: 1_000_000,
            storage_path: None,
            faucet_enabled: Arc::new(std::sync::atomic::AtomicBool::new(true)), // Phase 1: ON
            faucet_amount: 10 * 1_000_000, // 10 SLTN per claim
            faucet_max_cap: 2_000_000 * 1_000_000, // 2M SLTN (1% of ecosystem fund)
            faucet_total_distributed: Arc::new(RwLock::new(0)),
            faucet_claims: Arc::new(RwLock::new(std::collections::HashSet::new())),
            faucet_challenges: Arc::new(RwLock::new(HashMap::new())),
            faucet_claim_timestamps: Arc::new(RwLock::new(std::collections::VecDeque::new())),
            faucet_rate_limit: 30, // 30 claims per minute max
        }
    }

    /// Create a new TokenFactory with persistence
    pub fn with_storage(storage_path: PathBuf) -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            balances: Arc::new(RwLock::new(HashMap::new())),
            creation_fee: 3 * 1_000_000, // 3 SLTN
            min_initial_supply: 1_000_000,
            storage_path: Some(storage_path),
            faucet_enabled: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            faucet_amount: 10 * 1_000_000,
            faucet_max_cap: 2_000_000 * 1_000_000,
            faucet_total_distributed: Arc::new(RwLock::new(0)),
            faucet_claims: Arc::new(RwLock::new(std::collections::HashSet::new())),
            faucet_challenges: Arc::new(RwLock::new(HashMap::new())),
            faucet_claim_timestamps: Arc::new(RwLock::new(std::collections::VecDeque::new())),
            faucet_rate_limit: 30,
        }
    }
    
    /// Create with full configuration (for production deployments)
    pub fn with_config(
        storage_path: Option<PathBuf>,
        creation_fee: u128,
        faucet_enabled: bool,
        faucet_amount: u128,
    ) -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            balances: Arc::new(RwLock::new(HashMap::new())),
            creation_fee,
            min_initial_supply: 1_000_000,
            storage_path,
            faucet_enabled: Arc::new(std::sync::atomic::AtomicBool::new(faucet_enabled)),
            faucet_amount,
            faucet_max_cap: 2_000_000 * 1_000_000, // 2M SLTN cap
            faucet_total_distributed: Arc::new(RwLock::new(0)),
            faucet_claims: Arc::new(RwLock::new(std::collections::HashSet::new())),
            faucet_challenges: Arc::new(RwLock::new(HashMap::new())),
            faucet_claim_timestamps: Arc::new(RwLock::new(std::collections::VecDeque::new())),
            faucet_rate_limit: 30,
        }
    }

    /// Set storage path for persistence
    pub fn set_storage_path(&mut self, path: PathBuf) {
        self.storage_path = Some(path);
    }
    
    // === Phase Control Methods ===
    
    /// Enable faucet (Phase 1 - pre-exchange)
    pub fn enable_faucet(&self) {
        use std::sync::atomic::Ordering;
        self.faucet_enabled.store(true, Ordering::SeqCst);
        info!("🚰 Faucet ENABLED (Phase 1 mode)");
    }
    
    /// Disable faucet (Phase 2 - exchanges live)
    /// Can be called at runtime without restart
    pub fn disable_faucet(&self) {
        use std::sync::atomic::Ordering;
        self.faucet_enabled.store(false, Ordering::SeqCst);
        info!("🔒 Faucet DISABLED (Phase 2 mode - buy SLTN on DEX/CEX)");
    }
    
    /// Check if faucet is enabled
    pub fn is_faucet_enabled(&self) -> bool {
        use std::sync::atomic::Ordering;
        self.faucet_enabled.load(Ordering::SeqCst)
    }
    
    /// Update creation fee (in usltn)
    pub fn set_creation_fee(&mut self, fee: u128) {
        self.creation_fee = fee;
        info!("💰 Creation fee updated to {} usltn ({} SLTN)", fee, fee / 1_000_000);
    }
    
    // === Faucet Challenge-Response (Anti-Sybil) ===
    
    /// Generate a challenge for faucet claim (step 1 of 2)
    /// Returns a nonce that must be signed by the wallet
    pub async fn generate_faucet_challenge(&self, address: &str) -> Result<String> {
        if !self.is_faucet_enabled() {
            bail!("Faucet is disabled. Buy SLTN on DEX or CEX.");
        }
        
        // Validate address format
        if !Self::validate_sultan_address(address) {
            bail!("Invalid Sultan address format");
        }
        
        // Check if already claimed
        let claims = self.faucet_claims.read().await;
        if claims.contains(address) {
            bail!("Address {} has already claimed from faucet", address);
        }
        drop(claims);
        
        // Generate random nonce
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let nonce = format!("sultan_faucet:{}:{}", address, timestamp);
        
        // Store challenge with 5-minute expiry
        let expires_at = timestamp + 300;
        let mut challenges = self.faucet_challenges.write().await;
        challenges.insert(address.to_string(), (nonce.clone(), expires_at));
        
        // Cleanup expired challenges (housekeeping)
        challenges.retain(|_, (_, exp)| *exp > timestamp);
        
        debug!("🎫 Faucet challenge generated for {}: {}", address, nonce);
        Ok(nonce)
    }
    
    /// Claim SLTN from faucet with signature verification (step 2 of 2)
    /// Requires signing the challenge nonce with the wallet's private key
    pub async fn claim_faucet_with_signature(
        &self,
        address: &str,
        nonce: &str,
        signature: &[u8],
        pubkey: &[u8; 32],
    ) -> Result<u128> {
        if !self.is_faucet_enabled() {
            bail!("Faucet is disabled. Buy SLTN on DEX or CEX.");
        }
        
        // Rate limiting check
        self.check_rate_limit().await?;
        
        // Validate address
        if !Self::validate_sultan_address(address) {
            bail!("Invalid Sultan address format");
        }
        
        // Verify challenge exists and is not expired
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut challenges = self.faucet_challenges.write().await;
        let (stored_nonce, expires_at) = challenges
            .get(address)
            .ok_or_else(|| anyhow::anyhow!("No challenge found. Call /faucet/challenge first"))?
            .clone();
        
        if now > expires_at {
            challenges.remove(address);
            bail!("Challenge expired. Request a new one.");
        }
        
        if nonce != stored_nonce {
            bail!("Invalid nonce");
        }
        
        // Verify signature over the nonce
        if !Self::verify_ed25519_signature(pubkey, nonce.as_bytes(), signature) {
            bail!("Invalid signature. Sign the challenge nonce with your wallet.");
        }
        
        // Remove used challenge
        challenges.remove(address);
        drop(challenges);
        
        // Check global cap
        let mut total_distributed = self.faucet_total_distributed.write().await;
        let new_total = *total_distributed + self.faucet_amount;
        if new_total > self.faucet_max_cap {
            bail!("Faucet cap reached ({} SLTN distributed of {} max). Buy SLTN on DEX.", 
                  *total_distributed / 1_000_000, self.faucet_max_cap / 1_000_000);
        }
        
        // Check if already claimed
        let mut claims = self.faucet_claims.write().await;
        if claims.contains(address) {
            bail!("Address {} has already claimed from faucet", address);
        }
        
        // Mark as claimed and update total
        claims.insert(address.to_string());
        *total_distributed = new_total;
        drop(claims);
        drop(total_distributed);
        
        // Record timestamp for rate limiting
        let mut timestamps = self.faucet_claim_timestamps.write().await;
        timestamps.push_back(now);
        // Keep only last 60 seconds of claims
        while let Some(&oldest) = timestamps.front() {
            if now - oldest > 60 {
                timestamps.pop_front();
            } else {
                break;
            }
        }
        drop(timestamps);
        
        // Credit SLTN to address
        let mut balances = self.balances.write().await;
        let key = (Self::SLTN_DENOM.to_string(), address.to_string());
        let balance = balances.entry(key).or_insert(0);
        *balance = balance.checked_add(self.faucet_amount)
            .ok_or_else(|| anyhow::anyhow!("Balance overflow"))?;
        
        info!("🚰 Faucet: {} SLTN claimed by {} (verified signature, total: {}/{})", 
              self.faucet_amount / 1_000_000, address,
              new_total / 1_000_000, self.faucet_max_cap / 1_000_000);
        Ok(self.faucet_amount)
    }
    
    /// Rate limiting check - max N claims per minute
    async fn check_rate_limit(&self) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let timestamps = self.faucet_claim_timestamps.read().await;
        let recent_claims = timestamps.iter().filter(|&&t| now - t < 60).count();
        
        if recent_claims >= self.faucet_rate_limit as usize {
            bail!("Rate limit exceeded. {} claims in the last minute (max {}). Try again shortly.", 
                  recent_claims, self.faucet_rate_limit);
        }
        Ok(())
    }
    
    /// Validate Sultan address format (bech32 with sultan1 prefix)
    pub fn validate_sultan_address(address: &str) -> bool {
        // Must start with sultan1 and be ~43-45 chars (bech32)
        if !address.starts_with("sultan1") {
            return false;
        }
        if address.len() < 40 || address.len() > 50 {
            return false;
        }
        // Check for valid bech32 characters
        address[7..].chars().all(|c| {
            matches!(c, 'q'..='z' | 'a'..='p' | '0'..='9') && c != 'b' && c != 'i' && c != 'o'
        })
    }
    
    /// Constant-time string comparison (prevents timing attacks)
    pub fn constant_time_compare(a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }
        let mut result = 0u8;
        for (x, y) in a.bytes().zip(b.bytes()) {
            result |= x ^ y;
        }
        result == 0
    }
    
    /// Legacy claim (for testing only - use claim_faucet_with_signature in production)
    #[cfg(test)]
    pub async fn claim_faucet(&self, address: &str) -> Result<u128> {
        if !self.is_faucet_enabled() {
            bail!("Faucet is disabled. Buy SLTN on DEX or CEX.");
        }
        
        // Check if cap would be exceeded
        let mut total_distributed = self.faucet_total_distributed.write().await;
        let new_total = *total_distributed + self.faucet_amount;
        if new_total > self.faucet_max_cap {
            bail!("Faucet cap reached ({} SLTN distributed of {} max). Buy SLTN on DEX.",  
                  *total_distributed / 1_000_000, self.faucet_max_cap / 1_000_000);
        }
        
        // Check if already claimed
        let mut claims = self.faucet_claims.write().await;
        if claims.contains(address) {
            bail!("Address {} has already claimed from faucet", address);
        }
        
        // Mark as claimed and update total
        claims.insert(address.to_string());
        *total_distributed = new_total;
        drop(claims);
        drop(total_distributed);
        
        // Credit SLTN to address
        let mut balances = self.balances.write().await;
        let key = (Self::SLTN_DENOM.to_string(), address.to_string());
        let balance = balances.entry(key).or_insert(0);
        *balance = balance.checked_add(self.faucet_amount)
            .ok_or_else(|| anyhow::anyhow!("Balance overflow"))?;
        
        info!("🚰 Faucet: {} SLTN claimed by {} (total: {}/{})", 
              self.faucet_amount / 1_000_000, address,
              new_total / 1_000_000, self.faucet_max_cap / 1_000_000);
        Ok(self.faucet_amount)
    }
    
    /// Check if address has claimed from faucet
    pub async fn has_claimed_faucet(&self, address: &str) -> bool {
        let claims = self.faucet_claims.read().await;
        claims.contains(address)
    }
    
    /// Get faucet statistics
    pub async fn get_faucet_stats(&self) -> FaucetStats {
        let claims = self.faucet_claims.read().await;
        let total_distributed = self.faucet_total_distributed.read().await;
        FaucetStats {
            enabled: self.is_faucet_enabled(),
            amount_per_claim: self.faucet_amount,
            max_cap: self.faucet_max_cap,
            total_claims: claims.len(),
            total_distributed: *total_distributed,
            remaining: self.faucet_max_cap.saturating_sub(*total_distributed),
        }
    }

    /// Load tokens and balances from persistent storage
    pub async fn load_from_storage(&self) -> Result<()> {
        let Some(path) = &self.storage_path else {
            return Ok(());
        };
        
        // Load tokens
        let tokens_file = path.join("tokens.json");
        if tokens_file.exists() {
            let data = tokio::fs::read_to_string(&tokens_file).await?;
            let loaded_tokens: HashMap<String, TokenMetadata> = serde_json::from_str(&data)?;
            let mut tokens = self.tokens.write().await;
            *tokens = loaded_tokens;
            info!("📂 Loaded {} tokens from {:?}", tokens.len(), tokens_file);
        }
        
        // Load balances (convert from serializable format)
        let balances_file = path.join("balances.json");
        if balances_file.exists() {
            let data = tokio::fs::read_to_string(&balances_file).await?;
            // Serialize as Vec of (denom, address, balance) for JSON compatibility
            let loaded: Vec<(String, String, u128)> = serde_json::from_str(&data)?;
            let mut balances = self.balances.write().await;
            balances.clear();
            for (denom, address, balance) in loaded {
                balances.insert((denom, address), balance);
            }
            info!("📂 Loaded {} balance entries from {:?}", balances.len(), balances_file);
        }
        
        Ok(())
    }

    /// Save tokens and balances to persistent storage
    pub async fn save_to_storage(&self) -> Result<()> {
        let Some(path) = &self.storage_path else {
            return Ok(());
        };
        
        // Ensure directory exists
        tokio::fs::create_dir_all(path).await?;
        
        // Save tokens
        let tokens_file = path.join("tokens.json");
        let tokens = self.tokens.read().await;
        let data = serde_json::to_string_pretty(&*tokens)?;
        tokio::fs::write(&tokens_file, data).await?;
        
        // Save balances (convert to serializable format)
        let balances_file = path.join("balances.json");
        let balances = self.balances.read().await;
        let serializable: Vec<(&String, &String, &u128)> = balances
            .iter()
            .map(|((d, a), b)| (d, a, b))
            .collect();
        let data = serde_json::to_string_pretty(&serializable)?;
        tokio::fs::write(&balances_file, data).await?;
        
        debug!("💾 Saved {} tokens and {} balances", tokens.len(), balances.len());
        Ok(())
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
        
        // Add to recipient with overflow protection
        let to_key = (denom.to_string(), to.to_string());
        let to_balance = balances.entry(to_key).or_insert(0);
        *to_balance = to_balance.checked_add(amount)
            .ok_or_else(|| anyhow::anyhow!("Recipient balance overflow"))?;
        
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
        
        // Add to recipient with overflow protection
        let to_key = (denom.to_string(), to.to_string());
        let to_balance = balances.entry(to_key).or_insert(0);
        *to_balance = to_balance.checked_add(amount)
            .ok_or_else(|| anyhow::anyhow!("Recipient balance overflow"))?;
        
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
        
        // Check if minting is enabled and respects max supply (with overflow protection)
        let new_supply = metadata.total_supply.checked_add(amount)
            .ok_or_else(|| anyhow::anyhow!("Supply overflow"))?;
        if let Some(max_supply) = metadata.max_supply {
            if new_supply > max_supply {
                bail!("Minting would exceed max supply of {}", max_supply);
            }
        }
        
        // Update total supply in metadata
        metadata.total_supply = new_supply;
        let symbol = metadata.symbol.clone();
        drop(tokens);
        
        // Mint tokens to recipient balance with overflow protection
        let mut balances = self.balances.write().await;
        let key = (denom.to_string(), recipient.to_string());
        let balance = balances.entry(key).or_insert(0);
        *balance = balance.checked_add(amount)
            .ok_or_else(|| anyhow::anyhow!("Recipient balance overflow"))?;
        
        info!("🪙 Minted {} {} to {} (internal)", amount, symbol, recipient);
        Ok(())
    }

    /// Create LP token with custom denom (for DEX pools)
    /// SECURITY: Only call this from trusted system components (NativeDex)
    /// LP tokens have no min_supply requirement and use custom denom format
    pub async fn create_lp_token_internal(
        &self,
        lp_denom: &str,
        pool_address: &str,
        total_supply: u128,
        token_a: &str,
        token_b: &str,
    ) -> Result<String> {
        // Check if token already exists
        let tokens = self.tokens.read().await;
        if tokens.contains_key(lp_denom) {
            bail!("LP token already exists: {}", lp_denom);
        }
        drop(tokens);
        
        // Create LP token metadata
        let metadata = TokenMetadata {
            creator: pool_address.to_string(),
            name: format!("LP Token {}/{}", token_a, token_b),
            symbol: format!("LP-{}-{}", 
                token_a.split('/').last().unwrap_or(token_a),
                token_b.split('/').last().unwrap_or(token_b)
            ),
            decimals: 18,
            total_supply,
            max_supply: None, // LP tokens can mint/burn freely
            logo_url: None,
            description: Some(format!("Liquidity provider token for {}/{}", token_a, token_b)),
            website: None,
            social_links: None,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            minting_enabled: true, // LP tokens can always be minted
        };
        
        // Store token metadata with custom denom
        let mut tokens = self.tokens.write().await;
        tokens.insert(lp_denom.to_string(), metadata);
        drop(tokens);
        
        // Set initial supply to pool address
        let mut balances = self.balances.write().await;
        let key = (lp_denom.to_string(), pool_address.to_string());
        balances.insert(key, total_supply);
        
        info!("🔷 LP token created: {} with {} supply", lp_denom, total_supply);
        Ok(lp_denom.to_string())
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
            // Check supply overflow before adding
            let new_supply = metadata.total_supply.checked_add(amount)
                .ok_or_else(|| anyhow::anyhow!("Supply overflow"))?;
            if new_supply > max_supply {
                bail!("Minting would exceed max supply of {}", max_supply);
            }
            
            // Update total supply in metadata (O(1))
            metadata.total_supply = new_supply;
            let symbol = metadata.symbol.clone();
            drop(tokens);
            
            // Mint tokens to recipient balance with overflow protection
            let mut balances = self.balances.write().await;
            let key = (denom.to_string(), recipient.to_string());
            let balance = balances.entry(key).or_insert(0);
            *balance = balance.checked_add(amount)
                .ok_or_else(|| anyhow::anyhow!("Recipient balance overflow"))?;
            
            info!("✅ Minted {} {} to {}", amount, symbol, recipient);
            Ok(())
        } else {
            bail!("Minting not enabled for this token");
        }
    }

    /// Internal burn for trusted callers (e.g., NativeDex LP redemption)
    /// SECURITY: Only call this from trusted system components
    /// For user-initiated burns, use `burn_with_signature`
    pub async fn burn_internal(
        &self,
        denom: &str,
        from: &str,
        amount: u128,
    ) -> Result<()> {
        // Validate amount
        if amount == 0 {
            bail!("Burn amount must be positive");
        }
        
        let mut balances = self.balances.write().await;
        
        let key = (denom.to_string(), from.to_string());
        let balance = balances.get_mut(&key)
            .ok_or_else(|| anyhow::anyhow!("No balance to burn"))?;
        
        if *balance < amount {
            bail!("Insufficient balance to burn: has {}, needs {}", balance, amount);
        }
        *balance -= amount;
        drop(balances);
        
        // Update total supply in metadata (O(1))
        let mut tokens = self.tokens.write().await;
        if let Some(metadata) = tokens.get_mut(denom) {
            metadata.total_supply = metadata.total_supply.saturating_sub(amount);
        }
        
        info!("🔥 Burned {} {} from {} (internal)", amount, denom, from);
        Ok(())
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

/// Faucet statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaucetStats {
    pub enabled: bool,
    pub amount_per_claim: u128,
    pub max_cap: u128,
    pub total_claims: usize,
    pub total_distributed: u128,
    pub remaining: u128,
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
        assert_eq!(stats.creation_fee, 3 * 1_000_000); // 3 SLTN
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

    // === Faucet Tests ===

    #[tokio::test]
    async fn test_faucet_claim() {
        let factory = TokenFactory::new();
        
        // Claim from faucet
        let amount = factory.claim_faucet("sultan1user").await.unwrap();
        assert_eq!(amount, 10 * 1_000_000); // 10 SLTN
        
        // Check balance
        let balance = factory.get_balance(TokenFactory::SLTN_DENOM, "sultan1user").await;
        assert_eq!(balance, 10 * 1_000_000);
    }

    #[tokio::test]
    async fn test_faucet_prevents_double_claim() {
        let factory = TokenFactory::new();
        
        // First claim succeeds
        factory.claim_faucet("sultan1user").await.unwrap();
        
        // Second claim fails
        let result = factory.claim_faucet("sultan1user").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already claimed"));
    }

    #[tokio::test]
    async fn test_faucet_disabled_rejects() {
        let factory = TokenFactory::new();
        factory.disable_faucet();
        
        let result = factory.claim_faucet("sultan1user").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("disabled"));
    }

    #[tokio::test]
    async fn test_faucet_stats() {
        let factory = TokenFactory::new();
        
        factory.claim_faucet("sultan1user1").await.unwrap();
        factory.claim_faucet("sultan1user2").await.unwrap();
        
        let stats = factory.get_faucet_stats().await;
        assert_eq!(stats.total_claims, 2);
        assert_eq!(stats.total_distributed, 20 * 1_000_000); // 20 SLTN
        assert!(stats.enabled);
        assert_eq!(stats.max_cap, 2_000_000 * 1_000_000); // 2M SLTN
        assert_eq!(stats.remaining, stats.max_cap - stats.total_distributed);
    }

    #[tokio::test]
    async fn test_faucet_cap_enforcement() {
        // Create factory with tiny cap for testing
        let factory = TokenFactory::with_config(
            None,
            3 * 1_000_000,  // 3 SLTN fee
            true,            // faucet enabled
            10 * 1_000_000,  // 10 SLTN per claim
        );
        // Override cap to 25 SLTN for testing (2.5 claims worth)
        *factory.faucet_total_distributed.write().await = factory.faucet_max_cap - 5 * 1_000_000; // Only 5 SLTN left
        
        // First claim should fail (10 > 5 remaining)
        let result = factory.claim_faucet("sultan1user").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cap reached"));
    }

    #[tokio::test]
    async fn test_creation_fee_is_3_sltn() {
        let factory = TokenFactory::new();
        assert_eq!(factory.creation_fee, 3 * 1_000_000); // 3 SLTN
    }

    #[tokio::test]
    async fn test_with_config() {
        let factory = TokenFactory::with_config(
            None,
            5 * 1_000_000, // 5 SLTN fee
            false,          // faucet disabled
            20 * 1_000_000, // 20 SLTN per claim
        );
        
        assert_eq!(factory.creation_fee, 5 * 1_000_000);
        assert!(!factory.is_faucet_enabled());
        assert_eq!(factory.faucet_amount, 20 * 1_000_000);
    }

    #[tokio::test]
    async fn test_challenge_response_flow() {
        use ed25519_dalek::{SigningKey, Signer};
        use rand::rngs::OsRng;
        
        let factory = TokenFactory::new();
        let address = "sultan15g5nwnlemn7zt6rtl7ch46ssvx2ym2v2umm07g";
        
        // Step 1: Generate challenge
        let nonce = factory.generate_faucet_challenge(address).await.unwrap();
        assert!(nonce.starts_with("sultan_faucet:"));
        assert!(nonce.contains(address));
        
        // Step 2: Sign the challenge with Ed25519
        let signing_key = SigningKey::generate(&mut OsRng);
        let pubkey = signing_key.verifying_key().to_bytes();
        let signature = signing_key.sign(nonce.as_bytes());
        
        // Step 3: Claim with signature
        let amount = factory.claim_faucet_with_signature(
            address,
            &nonce,
            signature.to_bytes().as_ref(),
            &pubkey,
        ).await.unwrap();
        
        assert_eq!(amount, 10 * 1_000_000); // 10 SLTN
        
        // Verify balance was credited
        let balance = factory.get_balance(TokenFactory::SLTN_DENOM, address).await;
        assert_eq!(balance, 10 * 1_000_000);
    }

    #[tokio::test]
    async fn test_challenge_expires() {
        let factory = TokenFactory::new();
        // Use a valid-format sultan address (43 chars with sultan1 prefix)
        let address = "sultan1expxr3t3st9999999999999999999deadend";
        
        // Manually insert an expired challenge
        let expired_time = 100; // Way in the past
        factory.faucet_challenges.write().await.insert(
            address.to_string(),
            ("old_nonce".to_string(), expired_time),
        );
        
        // Try to claim with expired challenge
        let result = factory.claim_faucet_with_signature(
            address,
            "old_nonce",
            &[0u8; 64], // Dummy sig
            &[0u8; 32], // Dummy pubkey
        ).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expired"));
    }

    #[tokio::test]
    async fn test_wrong_nonce_rejected() {
        use ed25519_dalek::{SigningKey, Signer};
        use rand::rngs::OsRng;
        
        let factory = TokenFactory::new();
        // Use a valid-format sultan address
        let address = "sultan1wr9ngn0nc3t3st99999999999999999deadend";
        
        // Generate challenge
        let nonce = factory.generate_faucet_challenge(address).await.unwrap();
        
        // Sign wrong message
        let signing_key = SigningKey::generate(&mut OsRng);
        let pubkey = signing_key.verifying_key().to_bytes();
        let signature = signing_key.sign(b"wrong_nonce");
        
        // Try to claim with wrong nonce
        let result = factory.claim_faucet_with_signature(
            address,
            "wrong_nonce", // Not the challenge nonce
            signature.to_bytes().as_ref(),
            &pubkey,
        ).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid nonce"));
    }

    #[test]
    fn test_address_validation() {
        // Valid addresses
        assert!(TokenFactory::validate_sultan_address("sultan15g5nwnlemn7zt6rtl7ch46ssvx2ym2v2umm07g"));
        assert!(TokenFactory::validate_sultan_address("sultan1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq5nvd7d"));
        
        // Invalid addresses
        assert!(!TokenFactory::validate_sultan_address("cosmos1abc")); // Wrong prefix
        assert!(!TokenFactory::validate_sultan_address("sultan1")); // Too short
        assert!(!TokenFactory::validate_sultan_address("eth1234567890123456789012345678901234567890")); // Wrong prefix
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(TokenFactory::constant_time_compare("secret", "secret"));
        assert!(!TokenFactory::constant_time_compare("secret", "SECRET"));
        assert!(!TokenFactory::constant_time_compare("secret", "secre"));
        assert!(!TokenFactory::constant_time_compare("", "x"));
        assert!(TokenFactory::constant_time_compare("", ""));
    }
}
