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

        // Cosmos IBC Fees
        // Zero fees on Sultan, but destination chains may charge their own fees
        fee_configs.insert("cosmos".to_string(), BridgeFeeConfig {
            base_fee: 0,           // No Sultan-side fee
            percentage_fee: 0,     // 0% on Sultan
            min_fee: 0,
            max_fee: 0,
        });

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
            "cosmos" => ExternalChainFee {
                chain: "Cosmos IBC".to_string(),
                estimated_cost: "$0.00 - $0.10 (depends on chain)".to_string(),
                confirmation_time: "7 seconds".to_string(),
                notes: "Destination chain fee, not Sultan. Many Cosmos chains have low/zero fees".to_string(),
            },
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
}
