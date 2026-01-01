use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::Result;

/// Chain configuration with feature flags for hot-upgrades
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub chain_id: String,
    pub gas_price: u64,
    pub block_time: u64,
    pub max_block_size: usize,
    pub min_stake: u64,
    pub inflation_rate: f64,
    
    /// Feature flags for governance-activated upgrades
    pub features: FeatureFlags,
}

/// Feature flags for hot-activation via governance
/// This allows adding new features without restarting the chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Sharding system (currently active)
    pub sharding_enabled: bool,
    
    /// Governance system (currently active)
    pub governance_enabled: bool,
    
    /// Cross-chain bridges (currently active)
    pub bridges_enabled: bool,
    
    /// Smart contracts - VM to be selected post-launch (future)
    /// Options: wasmer/wasmtime, Move VM, or custom
    pub wasm_contracts_enabled: bool,
    
    /// EVM compatibility layer (future)
    pub evm_contracts_enabled: bool,
    
    /// Quantum-resistant signatures using Dilithium3 (future)
    pub quantum_signatures_enabled: bool,
    
    /// Reserved for future cross-chain protocol (future)
    pub ibc_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            chain_id: String::from("sultan-mainnet-1"),
            gas_price: 0, // Zero gas fees!
            block_time: 2, // 2 second blocks
            max_block_size: 1000,
            min_stake: 10000, // 10,000 SLTN minimum stake
            inflation_rate: 0.04, // 4% annual inflation
            features: FeatureFlags::default(),
        }
    }
}

impl Default for FeatureFlags {
    fn default() -> Self {
        FeatureFlags {
            // Active at launch
            sharding_enabled: true,
            governance_enabled: true,
            bridges_enabled: true,
            
            // Disabled at launch, activated later via governance
            wasm_contracts_enabled: false,
            evm_contracts_enabled: false,
            quantum_signatures_enabled: false,
            ibc_enabled: false,
        }
    }
}

impl Config {
    /// Load configuration from JSON file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&contents)?;
        Ok(config)
    }
    
    /// Save configuration to JSON file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
    
    /// Update a feature flag (used by governance)
    pub fn update_feature(&mut self, feature: &str, enabled: bool) -> Result<()> {
        match feature {
            "wasm_contracts_enabled" | "smart_contracts_enabled" => {
                self.features.wasm_contracts_enabled = enabled;
                Ok(())
            }
            "evm_contracts_enabled" => {
                self.features.evm_contracts_enabled = enabled;
                Ok(())
            }
            "quantum_signatures_enabled" => {
                self.features.quantum_signatures_enabled = enabled;
                Ok(())
            }
            "ibc_enabled" => {
                self.features.ibc_enabled = enabled;
                Ok(())
            }
            "sharding_enabled" => {
                self.features.sharding_enabled = enabled;
                Ok(())
            }
            "governance_enabled" => {
                self.features.governance_enabled = enabled;
                Ok(())
            }
            "bridges_enabled" => {
                self.features.bridges_enabled = enabled;
                Ok(())
            }
            _ => anyhow::bail!("Unknown feature flag: {}", feature),
        }
    }
}
