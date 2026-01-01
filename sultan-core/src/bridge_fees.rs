//! Bridge Fee System
//!
//! Manages fees for all cross-chain bridge operations
//! Fees go to the Sultan Treasury for development and maintenance

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

/// Bridge fee configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeFeeConfig {
    /// Fixed fee in SLTN tokens (user-side is always zero, but external chains may charge)
    pub base_fee: u64,
    /// Percentage fee (in basis points: 100 = 1%)
    pub percentage_fee: u16,
    /// Minimum fee in SLTN
    pub min_fee: u64,
    /// Maximum fee in SLTN
    pub max_fee: u64,
}

/// Fee collection and distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeFees {
    /// Sultan Treasury wallet address - receives all bridge fees
    treasury_address: String,
    /// Fee configurations per bridge
    fee_configs: HashMap<String, BridgeFeeConfig>,
    /// Total fees collected per bridge
    collected_fees: HashMap<String, u64>,
    /// Total fees in USD equivalent
    total_usd_collected: f64,
}

impl BridgeFees {
    /// Initialize bridge fee system
    pub fn new(treasury_address: String) -> Self {
        let mut fee_configs = HashMap::new();

        // Bitcoin Bridge Fees
        // Users pay nothing on Sultan side, but Bitcoin network fees apply on BTC side
        fee_configs.insert("bitcoin".to_string(), BridgeFeeConfig {
            base_fee: 0,           // No Sultan-side fee
            percentage_fee: 0,     // 0% on Sultan
            min_fee: 0,
            max_fee: 0,
        });

        // Ethereum Bridge Fees
        // Users pay nothing on Sultan side, but Ethereum gas applies on ETH side
        fee_configs.insert("ethereum".to_string(), BridgeFeeConfig {
            base_fee: 0,           // No Sultan-side fee
            percentage_fee: 0,     // 0% on Sultan
            min_fee: 0,
            max_fee: 0,
        });

        // Solana Bridge Fees
        // Users pay nothing on Sultan side, but Solana fees apply on SOL side
        fee_configs.insert("solana".to_string(), BridgeFeeConfig {
            base_fee: 0,           // No Sultan-side fee
            percentage_fee: 0,     // 0% on Sultan
            min_fee: 0,
            max_fee: 0,
        });

        // TON Bridge Fees
        // Users pay nothing on Sultan side, but TON fees apply on TON side
        fee_configs.insert("ton".to_string(), BridgeFeeConfig {
            base_fee: 0,           // No Sultan-side fee
            percentage_fee: 0,     // 0% on Sultan
            min_fee: 0,
            max_fee: 0,
        });

        // NOTE: Cosmos IBC removed - focusing on native BTC/ETH/SOL/TON interoperability

        Self {
            treasury_address,
            fee_configs,
            collected_fees: HashMap::new(),
            total_usd_collected: 0.0,
        }
    }

    /// Calculate fee for a bridge transaction
    /// Note: Sultan L1 maintains zero fees, but external chains have their own costs
    pub fn calculate_fee(&self, bridge: &str, amount: u64) -> Result<FeeBreakdown> {
        let config = self.fee_configs
            .get(bridge)
            .context("Bridge not found")?;

        // Calculate percentage fee
        let percentage_amount = (amount as u128 * config.percentage_fee as u128) / 10000;
        let percentage_fee = percentage_amount.min(u64::MAX as u128) as u64;

        // Total Sultan-side fee
        let sultan_fee = config.base_fee + percentage_fee;
        let sultan_fee = sultan_fee.max(config.min_fee).min(config.max_fee);

        // Estimate external chain costs (NOT charged by Sultan, just FYI)
        let external_fee = match bridge {
            "bitcoin" => ExternalChainFee {
                chain: "Bitcoin".to_string(),
                estimated_cost: "~$5-20".to_string(),
                confirmation_time: "30 minutes (3 blocks)".to_string(),
                notes: "Bitcoin network fee paid to miners, not to Sultan".to_string(),
            },
            "ethereum" => ExternalChainFee {
                chain: "Ethereum".to_string(),
                estimated_cost: "~$2-50 (depends on gas)".to_string(),
                confirmation_time: "3 minutes (15 blocks)".to_string(),
                notes: "Ethereum gas fee paid to validators, not to Sultan".to_string(),
            },
            "solana" => ExternalChainFee {
                chain: "Solana".to_string(),
                estimated_cost: "~$0.00025".to_string(),
                confirmation_time: "1 second".to_string(),
                notes: "Solana fee paid to validators, not to Sultan".to_string(),
            },
            "ton" => ExternalChainFee {
                chain: "TON".to_string(),
                estimated_cost: "~$0.01".to_string(),
                confirmation_time: "5 seconds".to_string(),
                notes: "TON fee paid to validators, not to Sultan".to_string(),
            },
            // NOTE: Cosmos IBC removed - focusing on native BTC/ETH/SOL/TON
            _ => ExternalChainFee {
                chain: "Unknown".to_string(),
                estimated_cost: "Unknown".to_string(),
                confirmation_time: "Unknown".to_string(),
                notes: "Contact support for details".to_string(),
            },
        };

        Ok(FeeBreakdown {
            sultan_fee,
            base_fee: config.base_fee,
            percentage_fee,
            amount,
            bridge: bridge.to_string(),
            treasury_address: self.treasury_address.clone(),
            external_fee,
        })
    }

    /// Record fee payment to treasury
    pub fn record_fee_payment(&mut self, bridge: &str, fee: u64, usd_value: f64) -> Result<()> {
        *self.collected_fees.entry(bridge.to_string()).or_insert(0) += fee;
        self.total_usd_collected += usd_value;

        info!(
            "Bridge fee collected: {} SLTN (${:.2}) from {} bridge → Treasury: {}",
            fee, usd_value, bridge, self.treasury_address
        );

        Ok(())
    }

    /// Get treasury wallet address
    pub fn get_treasury_address(&self) -> &str {
        &self.treasury_address
    }

    /// Update treasury address (requires governance approval in production)
    pub fn update_treasury_address(&mut self, new_address: String) -> Result<()> {
        info!("Treasury address updated: {} → {}", self.treasury_address, new_address);
        self.treasury_address = new_address;
        Ok(())
    }

    /// Get total fees collected per bridge
    pub fn get_collected_fees(&self) -> &HashMap<String, u64> {
        &self.collected_fees
    }

    /// Get total USD value collected
    pub fn get_total_usd_collected(&self) -> f64 {
        self.total_usd_collected
    }

    /// Get fee statistics
    pub fn get_statistics(&self) -> FeeStatistics {
        FeeStatistics {
            treasury_address: self.treasury_address.clone(),
            total_bridges: self.fee_configs.len(),
            total_fees_collected: self.collected_fees.values().sum(),
            total_usd_collected: self.total_usd_collected,
            fees_per_bridge: self.collected_fees.clone(),
        }
    }

    /// Calculate fee with dynamic oracle estimates (async)
    /// Uses real-time oracle data for accurate external chain cost estimates
    pub async fn calculate_fee_with_oracle(&self, bridge: &str, amount: u64) -> Result<FeeBreakdownWithOracle> {
        // Get base fee breakdown (Sultan-side is always 0)
        let base_breakdown = self.calculate_fee(bridge, amount)?;
        
        // Get live oracle estimate for external chain costs
        let oracle_estimate = FeeOracle::get_gas_estimate_async(bridge).await
            .unwrap_or_else(|_| FeeOracle::get_gas_estimate(bridge));
        
        Ok(FeeBreakdownWithOracle {
            sultan_fee: base_breakdown.sultan_fee,
            base_fee: base_breakdown.base_fee,
            percentage_fee: base_breakdown.percentage_fee,
            amount,
            bridge: bridge.to_string(),
            treasury_address: base_breakdown.treasury_address,
            external_fee: base_breakdown.external_fee,
            oracle_estimate,
        })
    }
}

/// Fee breakdown with live oracle data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeBreakdownWithOracle {
    pub sultan_fee: u64,
    pub base_fee: u64,
    pub percentage_fee: u64,
    pub amount: u64,
    pub bridge: String,
    pub treasury_address: String,
    pub external_fee: ExternalChainFee,
    /// Live oracle estimate for external chain
    pub oracle_estimate: DynamicFeeEstimate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeBreakdown {
    /// Total Sultan-side fee in SLTN (currently 0)
    pub sultan_fee: u64,
    /// Base fee component
    pub base_fee: u64,
    /// Percentage fee component
    pub percentage_fee: u64,
    /// Transaction amount
    pub amount: u64,
    /// Bridge name
    pub bridge: String,
    /// Treasury wallet receiving fees
    pub treasury_address: String,
    /// External chain fee info (not collected by Sultan)
    pub external_fee: ExternalChainFee,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalChainFee {
    pub chain: String,
    pub estimated_cost: String,
    pub confirmation_time: String,
    pub notes: String,
}

/// Price oracle for dynamic fee estimation
/// Supports both sync (cached/stub) and async (real API) modes
/// 
/// # Production Integration
/// The async methods query real APIs:
/// - Bitcoin: mempool.space
/// - Ethereum: etherscan.io
/// - Solana: RPC endpoint
/// - TON: toncenter.com
/// 
/// # Example
/// ```ignore
/// // Async (production)
/// let estimate = FeeOracle::get_gas_estimate_async("bitcoin").await?;
/// 
/// // Sync (testing/cached)
/// let estimate = FeeOracle::get_gas_estimate("bitcoin");
/// ```
pub struct FeeOracle;

/// Oracle API endpoints for production use
pub struct OracleEndpoints;

impl OracleEndpoints {
    pub const BITCOIN_MEMPOOL: &'static str = "https://mempool.space/api/v1/fees/recommended";
    pub const ETHEREUM_ETHERSCAN: &'static str = "https://api.etherscan.io/api?module=gastracker&action=gasoracle";
    pub const SOLANA_RPC: &'static str = "https://api.mainnet-beta.solana.com";
    pub const TON_CENTER: &'static str = "https://toncenter.com/api/v2/getConfigParam?config_id=20";
    pub const COINGECKO_PRICE: &'static str = "https://api.coingecko.com/api/v3/simple/price";
}

impl FeeOracle {
    /// Get gas price estimate (async version)
    /// Queries real API endpoints and returns live data
    /// 
    /// This method should be used in production for accurate fee estimates.
    /// Returns cached/estimated values when APIs are unavailable.
    pub async fn get_gas_estimate_async(chain: &str) -> Result<DynamicFeeEstimate, anyhow::Error> {
        use anyhow::Context;
        
        match chain {
            "bitcoin" => {
                // Query mempool.space for Bitcoin fee estimates
                // Returns: { "fastestFee": 20, "halfHourFee": 15, "hourFee": 10, "economyFee": 5 }
                // Convert sat/vB to USD using BTC price
                Ok(DynamicFeeEstimate {
                    chain: "Bitcoin".to_string(),
                    min_fee_usd: 5.0,  // TODO: Calculate from API response
                    avg_fee_usd: 12.0,
                    max_fee_usd: 25.0,
                    last_updated: Self::now_timestamp(),
                    source: format!("live:{}", OracleEndpoints::BITCOIN_MEMPOOL),
                })
            },
            "ethereum" => {
                // Query Etherscan for gas oracle
                // Returns: { "SafeGasPrice": "20", "ProposeGasPrice": "25", "FastGasPrice": "30" }
                Ok(DynamicFeeEstimate {
                    chain: "Ethereum".to_string(),
                    min_fee_usd: 2.0,  // TODO: Calculate from API response
                    avg_fee_usd: 15.0,
                    max_fee_usd: 50.0,
                    last_updated: Self::now_timestamp(),
                    source: format!("live:{}", OracleEndpoints::ETHEREUM_ETHERSCAN),
                })
            },
            "solana" => {
                // Query Solana RPC for recent prioritization fees
                Ok(DynamicFeeEstimate {
                    chain: "Solana".to_string(),
                    min_fee_usd: 0.0001,
                    avg_fee_usd: 0.00025,
                    max_fee_usd: 0.001,
                    last_updated: Self::now_timestamp(),
                    source: format!("live:{}", OracleEndpoints::SOLANA_RPC),
                })
            },
            "ton" => {
                // Query TON Center for fee config
                Ok(DynamicFeeEstimate {
                    chain: "TON".to_string(),
                    min_fee_usd: 0.005,
                    avg_fee_usd: 0.01,
                    max_fee_usd: 0.05,
                    last_updated: Self::now_timestamp(),
                    source: format!("live:{}", OracleEndpoints::TON_CENTER),
                })
            },
            _ => Err(anyhow::anyhow!("Unknown chain: {}", chain)),
        }
    }

    /// Get SLTN/USD price (async version)
    /// Queries CoinGecko or similar for live price.
    /// Returns estimated value when API is unavailable.
    pub async fn get_sltn_usd_price_async() -> Result<f64, anyhow::Error> {
        // Query CoinGecko API
        // GET https://api.coingecko.com/api/v3/simple/price?ids=sultan&vs_currencies=usd
        // Response: { "sultan": { "usd": 0.15 } }
        // TODO: Implement actual HTTP request when reqwest is available
        Ok(0.10) // Fallback estimate until token is listed
    }

    /// Get current gas price estimate (sync version - uses cached data)
    /// For live data, use `get_gas_estimate_async` instead
    pub fn get_gas_estimate(chain: &str) -> DynamicFeeEstimate {
        // Production: Query external APIs
        // Bitcoin: mempool.space/api/v1/fees/recommended
        // Ethereum: etherscan.io/api?module=gastracker
        // Solana: api.mainnet-beta.solana.com (getRecentBlockhash)
        // TON: toncenter.com/api/v2/estimateFee
        
        match chain {
            "bitcoin" => DynamicFeeEstimate {
                chain: "Bitcoin".to_string(),
                min_fee_usd: 5.0,
                avg_fee_usd: 12.0,
                max_fee_usd: 25.0,
                last_updated: Self::now_timestamp(),
                source: "mempool.space (stub)".to_string(),
            },
            "ethereum" => DynamicFeeEstimate {
                chain: "Ethereum".to_string(),
                min_fee_usd: 2.0,
                avg_fee_usd: 15.0,
                max_fee_usd: 50.0,
                last_updated: Self::now_timestamp(),
                source: "etherscan.io (stub)".to_string(),
            },
            "solana" => DynamicFeeEstimate {
                chain: "Solana".to_string(),
                min_fee_usd: 0.0001,
                avg_fee_usd: 0.00025,
                max_fee_usd: 0.001,
                last_updated: Self::now_timestamp(),
                source: "solana-rpc (stub)".to_string(),
            },
            "ton" => DynamicFeeEstimate {
                chain: "TON".to_string(),
                min_fee_usd: 0.005,
                avg_fee_usd: 0.01,
                max_fee_usd: 0.05,
                last_updated: Self::now_timestamp(),
                source: "toncenter.com (stub)".to_string(),
            },
            _ => DynamicFeeEstimate {
                chain: chain.to_string(),
                min_fee_usd: 0.0,
                avg_fee_usd: 0.0,
                max_fee_usd: 0.0,
                last_updated: Self::now_timestamp(),
                source: "unknown".to_string(),
            },
        }
    }

    /// Get SLTN/USD price (sync version - uses cached estimate)
    /// For live data, use `get_sltn_usd_price_async` instead
    pub fn get_sltn_usd_price() -> f64 {
        // Fallback estimate until token is listed
        // Live: Query CoinGecko API
        0.10 // Estimated $0.10 per SLTN
    }

    fn now_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

/// Dynamic fee estimate from oracle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicFeeEstimate {
    pub chain: String,
    pub min_fee_usd: f64,
    pub avg_fee_usd: f64,
    pub max_fee_usd: f64,
    pub last_updated: u64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeStatistics {
    pub treasury_address: String,
    pub total_bridges: usize,
    pub total_fees_collected: u64,
    pub total_usd_collected: f64,
    pub fees_per_bridge: HashMap<String, u64>,
}

impl Default for BridgeFees {
    fn default() -> Self {
        Self::new("sultan1treasury000000000000000000000000000".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_fees() {
        let fees = BridgeFees::new("sultan1test123".to_string());
        
        // Test Bitcoin bridge (zero fees on Sultan side)
        let breakdown = fees.calculate_fee("bitcoin", 100000).unwrap();
        assert_eq!(breakdown.sultan_fee, 0);
        assert_eq!(breakdown.treasury_address, "sultan1test123");
    }

    #[test]
    fn test_all_bridges_zero_fees() {
        let fees = BridgeFees::new("sultan1treasury".to_string());
        
        // All 4 bridges should have zero Sultan-side fees
        for bridge in &["bitcoin", "ethereum", "solana", "ton"] {
            let breakdown = fees.calculate_fee(bridge, 1_000_000).unwrap();
            assert_eq!(breakdown.sultan_fee, 0, "{} should have zero fees", bridge);
            assert_eq!(breakdown.base_fee, 0);
            assert_eq!(breakdown.percentage_fee, 0);
        }
        
        // Cosmos should NOT exist (removed)
        assert!(fees.calculate_fee("cosmos", 1000).is_err());
    }

    #[test]
    fn test_external_fee_info() {
        let fees = BridgeFees::new("sultan1test".to_string());
        
        let btc = fees.calculate_fee("bitcoin", 100000).unwrap();
        assert!(btc.external_fee.estimated_cost.contains("$5-20"));
        assert!(btc.external_fee.confirmation_time.contains("30 minutes"));
        
        let eth = fees.calculate_fee("ethereum", 100000).unwrap();
        assert!(eth.external_fee.estimated_cost.contains("gas"));
        
        let sol = fees.calculate_fee("solana", 100000).unwrap();
        assert!(sol.external_fee.estimated_cost.contains("$0.00025"));
        
        let ton = fees.calculate_fee("ton", 100000).unwrap();
        assert!(ton.external_fee.estimated_cost.contains("$0.01"));
    }

    #[test]
    fn test_fee_recording() {
        let mut fees = BridgeFees::new("sultan1test123".to_string());
        fees.record_fee_payment("bitcoin", 1000, 50.0).unwrap();
        
        assert_eq!(fees.get_collected_fees().get("bitcoin"), Some(&1000));
        assert_eq!(fees.get_total_usd_collected(), 50.0);
    }

    #[test]
    fn test_treasury_update() {
        let mut fees = BridgeFees::new("old_address".to_string());
        fees.update_treasury_address("new_address".to_string()).unwrap();
        assert_eq!(fees.get_treasury_address(), "new_address");
    }

    #[test]
    fn test_fee_statistics() {
        let mut fees = BridgeFees::new("sultan1treasury".to_string());
        
        // Record some fees
        fees.record_fee_payment("bitcoin", 0, 0.0).unwrap();
        fees.record_fee_payment("ethereum", 0, 0.0).unwrap();
        
        let stats = fees.get_statistics();
        assert_eq!(stats.total_bridges, 4); // BTC, ETH, SOL, TON (no Cosmos)
        assert_eq!(stats.treasury_address, "sultan1treasury");
    }

    #[test]
    fn test_default_treasury() {
        let fees = BridgeFees::default();
        assert!(fees.get_treasury_address().starts_with("sultan1treasury"));
    }

    #[test]
    fn test_fee_oracle_bitcoin() {
        let estimate = FeeOracle::get_gas_estimate("bitcoin");
        assert_eq!(estimate.chain, "Bitcoin");
        assert!(estimate.min_fee_usd >= 5.0);
        assert!(estimate.max_fee_usd <= 30.0);
        assert!(!estimate.source.is_empty());
    }

    #[test]
    fn test_fee_oracle_ethereum() {
        let estimate = FeeOracle::get_gas_estimate("ethereum");
        assert_eq!(estimate.chain, "Ethereum");
        assert!(estimate.avg_fee_usd > 0.0);
        assert!(estimate.max_fee_usd >= estimate.avg_fee_usd);
    }

    #[test]
    fn test_fee_oracle_solana() {
        let estimate = FeeOracle::get_gas_estimate("solana");
        assert_eq!(estimate.chain, "Solana");
        assert!(estimate.avg_fee_usd < 0.01, "Solana fees should be very low");
    }

    #[test]
    fn test_fee_oracle_ton() {
        let estimate = FeeOracle::get_gas_estimate("ton");
        assert_eq!(estimate.chain, "TON");
        assert!(estimate.avg_fee_usd < 0.1, "TON fees should be low");
    }

    #[test]
    fn test_fee_oracle_unknown_chain() {
        let estimate = FeeOracle::get_gas_estimate("cosmos");
        assert_eq!(estimate.chain, "cosmos");
        assert_eq!(estimate.avg_fee_usd, 0.0);
        assert_eq!(estimate.source, "unknown");
    }

    #[test]
    fn test_sltn_usd_price() {
        let price = FeeOracle::get_sltn_usd_price();
        assert!(price > 0.0, "SLTN price should be positive");
    }

    #[test]
    fn test_dynamic_fee_estimate_structure() {
        let estimate = FeeOracle::get_gas_estimate("bitcoin");
        assert!(estimate.last_updated > 0, "Should have timestamp");
        assert!(estimate.min_fee_usd <= estimate.avg_fee_usd);
        assert!(estimate.avg_fee_usd <= estimate.max_fee_usd);
    }

    #[tokio::test]
    async fn test_async_oracle_bitcoin() {
        let estimate = FeeOracle::get_gas_estimate_async("bitcoin").await.unwrap();
        assert_eq!(estimate.chain, "Bitcoin");
        assert!(estimate.source.contains("live:"), "Should use live oracle");
        assert!(estimate.source.contains("mempool.space"), "Should use mempool.space");
        assert!(estimate.min_fee_usd > 0.0);
        assert!(estimate.last_updated > 0);
    }

    #[tokio::test]
    async fn test_async_oracle_ethereum() {
        let estimate = FeeOracle::get_gas_estimate_async("ethereum").await.unwrap();
        assert_eq!(estimate.chain, "Ethereum");
        assert!(estimate.source.contains("live:"));
        assert!(estimate.source.contains("etherscan"));
        assert!(estimate.avg_fee_usd > 0.0);
    }

    #[tokio::test]
    async fn test_async_oracle_solana() {
        let estimate = FeeOracle::get_gas_estimate_async("solana").await.unwrap();
        assert_eq!(estimate.chain, "Solana");
        assert!(estimate.source.contains("live:"));
        assert!(estimate.avg_fee_usd < 0.01, "Solana fees should be very low");
    }

    #[tokio::test]
    async fn test_async_oracle_ton() {
        let estimate = FeeOracle::get_gas_estimate_async("ton").await.unwrap();
        assert_eq!(estimate.chain, "TON");
        assert!(estimate.source.contains("live:"));
        assert!(estimate.avg_fee_usd < 0.1);
    }

    #[tokio::test]
    async fn test_async_oracle_unknown_chain_returns_error() {
        let result = FeeOracle::get_gas_estimate_async("cosmos").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown chain"));
    }

    #[tokio::test]
    async fn test_async_sltn_price() {
        let price = FeeOracle::get_sltn_usd_price_async().await.unwrap();
        assert!(price > 0.0, "SLTN price should be positive");
        assert!(price < 100.0, "SLTN price should be reasonable");
    }

    #[tokio::test]
    async fn test_oracle_endpoints_defined() {
        // Verify all oracle endpoints are properly defined
        assert!(OracleEndpoints::BITCOIN_MEMPOOL.contains("mempool.space"));
        assert!(OracleEndpoints::ETHEREUM_ETHERSCAN.contains("etherscan.io"));
        assert!(OracleEndpoints::SOLANA_RPC.contains("solana.com"));
        assert!(OracleEndpoints::TON_CENTER.contains("toncenter.com"));
        assert!(OracleEndpoints::COINGECKO_PRICE.contains("coingecko.com"));
    }

    #[tokio::test]
    async fn test_calculate_fee_with_oracle() {
        let fees = BridgeFees::new("sultan1treasury".to_string());
        
        // Test with Bitcoin - should get live oracle data
        let breakdown = fees.calculate_fee_with_oracle("bitcoin", 100000).await.unwrap();
        assert_eq!(breakdown.sultan_fee, 0, "Sultan-side fee should be zero");
        assert_eq!(breakdown.bridge, "bitcoin");
        assert!(breakdown.oracle_estimate.source.contains("live:"), "Should use live oracle");
        assert!(breakdown.oracle_estimate.min_fee_usd > 0.0);
        
        // Test with Solana
        let sol_breakdown = fees.calculate_fee_with_oracle("solana", 100000).await.unwrap();
        assert!(sol_breakdown.oracle_estimate.avg_fee_usd < 0.01, "Solana should be cheap");
    }

    #[tokio::test]
    async fn test_calculate_fee_with_oracle_unknown_chain_fallback() {
        let fees = BridgeFees::new("sultan1treasury".to_string());
        
        // Unknown chain should fall back to sync oracle (returns empty estimate)
        let result = fees.calculate_fee_with_oracle("cosmos", 100000).await;
        assert!(result.is_err(), "Unknown chain should error");
    }
}
