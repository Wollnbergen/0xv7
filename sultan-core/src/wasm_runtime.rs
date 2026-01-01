//! WASM Runtime for CosmWasm Smart Contracts
//!
//! This module provides the runtime for executing CosmWasm smart contracts.
//! It is activated via governance when the `wasm_contracts_enabled` feature flag is set.
//!
//! # Hot-Activation
//! 
//! The WASM runtime can be enabled/disabled at runtime via governance proposals:
//! ```json
//! {
//!   "title": "Enable CosmWasm Smart Contracts",
//!   "proposal_type": "ParameterChange",
//!   "parameters": {
//!     "features.wasm_contracts_enabled": "true"
//!   }
//! }
//! ```

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Represents a deployed CosmWasm contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    /// Unique contract address (generated from code_id + salt)
    pub address: String,
    /// The code ID this contract was instantiated from
    pub code_id: u64,
    /// Contract admin (can migrate/update)
    pub admin: Option<String>,
    /// Contract label for identification
    pub label: String,
    /// Creator address
    pub creator: String,
    /// Block height when created
    pub created_at: u64,
}

/// Represents uploaded WASM code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeInfo {
    /// Unique code ID
    pub code_id: u64,
    /// SHA256 hash of the WASM bytecode
    pub code_hash: String,
    /// Address that uploaded the code
    pub creator: String,
    /// Block height when uploaded
    pub created_at: u64,
    /// Size of the WASM bytecode in bytes
    pub size: usize,
}

/// WASM execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Return data (if any)
    pub data: Option<Vec<u8>>,
    /// Emitted events
    pub events: Vec<ContractEvent>,
    /// Gas used
    pub gas_used: u64,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Contract event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEvent {
    /// Event type
    pub r#type: String,
    /// Event attributes
    pub attributes: Vec<(String, String)>,
}

/// WASM Runtime state and configuration
/// 
/// This runtime manages:
/// - Code uploads (store WASM bytecode)
/// - Contract instantiation
/// - Contract execution
/// - Contract queries
/// - Contract migration
pub struct WasmRuntime {
    /// Whether the runtime is active
    enabled: bool,
    
    /// Next code ID to assign
    next_code_id: u64,
    
    /// Stored WASM code (code_id -> bytecode)
    /// In production, this would use persistent storage
    code_storage: HashMap<u64, Vec<u8>>,
    
    /// Code metadata
    code_info: HashMap<u64, CodeInfo>,
    
    /// Deployed contracts
    contracts: HashMap<String, Contract>,
    
    /// Contract state storage (address -> key -> value)
    /// In production, this would use persistent storage with Merkle proofs
    contract_state: HashMap<String, HashMap<Vec<u8>, Vec<u8>>>,
    
    /// Gas limits
    #[allow(dead_code)]
    max_gas_per_tx: u64,
    max_code_size: usize,
}

impl WasmRuntime {
    /// Create a new WASM runtime (disabled by default)
    pub fn new() -> Self {
        info!("📦 Initializing WASM runtime (disabled until governance activation)");
        Self {
            enabled: false,
            next_code_id: 1,
            code_storage: HashMap::new(),
            code_info: HashMap::new(),
            contracts: HashMap::new(),
            contract_state: HashMap::new(),
            max_gas_per_tx: 10_000_000, // 10M gas limit
            max_code_size: 1024 * 1024, // 1MB max code size
        }
    }
    
    /// Enable or disable the runtime
    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled && !self.enabled {
            info!("🚀 WASM runtime ACTIVATED - CosmWasm smart contracts are now available");
        } else if !enabled && self.enabled {
            warn!("⚠️  WASM runtime DEACTIVATED - Smart contract operations will be rejected");
        }
        self.enabled = enabled;
    }
    
    /// Check if the runtime is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Store WASM code (returns code_id)
    pub fn store_code(
        &mut self,
        creator: String,
        wasm_bytecode: Vec<u8>,
        current_height: u64,
    ) -> Result<u64> {
        if !self.enabled {
            bail!("WASM runtime is not enabled. Submit a governance proposal to enable it.");
        }
        
        // Validate code size
        if wasm_bytecode.len() > self.max_code_size {
            bail!("WASM code exceeds maximum size of {} bytes", self.max_code_size);
        }
        
        // Validate WASM magic bytes
        if wasm_bytecode.len() < 4 || &wasm_bytecode[0..4] != b"\0asm" {
            bail!("Invalid WASM bytecode (missing magic header)");
        }
        
        // Calculate code hash
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&wasm_bytecode);
        let code_hash = hex::encode(hasher.finalize());
        
        let code_id = self.next_code_id;
        self.next_code_id += 1;
        
        // Store the code
        self.code_storage.insert(code_id, wasm_bytecode.clone());
        self.code_info.insert(code_id, CodeInfo {
            code_id,
            code_hash: code_hash.clone(),
            creator,
            created_at: current_height,
            size: wasm_bytecode.len(),
        });
        
        info!("📝 WASM code stored: code_id={}, hash={}, size={} bytes", 
              code_id, &code_hash[..16], wasm_bytecode.len());
        
        Ok(code_id)
    }
    
    /// Instantiate a contract from stored code
    pub fn instantiate(
        &mut self,
        sender: String,
        code_id: u64,
        _init_msg: Vec<u8>,
        label: String,
        admin: Option<String>,
        current_height: u64,
    ) -> Result<String> {
        if !self.enabled {
            bail!("WASM runtime is not enabled");
        }
        
        // Verify code exists
        if !self.code_storage.contains_key(&code_id) {
            bail!("Code ID {} not found", code_id);
        }
        
        // Generate contract address
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(format!("{}_{}_{}_{}", code_id, sender, current_height, label));
        let addr_bytes = hasher.finalize();
        let contract_address = format!("sultan1{}", hex::encode(&addr_bytes[..20]));
        
        // TODO: In production, execute the instantiate entry point
        // For now, we create the contract record and empty state
        let contract = Contract {
            address: contract_address.clone(),
            code_id,
            admin,
            label: label.clone(),
            creator: sender.clone(),
            created_at: current_height,
        };
        
        self.contracts.insert(contract_address.clone(), contract);
        self.contract_state.insert(contract_address.clone(), HashMap::new());
        
        info!("🎉 Contract instantiated: {} (code_id={}, label={})", 
              contract_address, code_id, label);
        
        Ok(contract_address)
    }
    
    /// Execute a contract
    pub fn execute(
        &mut self,
        sender: String,
        contract_address: String,
        msg: Vec<u8>,
        _funds: Vec<(String, u64)>, // (denom, amount)
    ) -> Result<ExecutionResult> {
        if !self.enabled {
            bail!("WASM runtime is not enabled");
        }
        
        // Verify contract exists
        if !self.contracts.contains_key(&contract_address) {
            bail!("Contract {} not found", contract_address);
        }
        
        // TODO: In production, execute the contract with CosmWasm VM
        // For now, return a placeholder result
        info!("⚡ Contract executed: {} by {}", contract_address, sender);
        
        Ok(ExecutionResult {
            success: true,
            data: Some(msg), // Echo back the message for now
            events: vec![ContractEvent {
                r#type: "execute".to_string(),
                attributes: vec![
                    ("_contract_address".to_string(), contract_address),
                    ("sender".to_string(), sender),
                ],
            }],
            gas_used: 1000, // Placeholder
            error: None,
        })
    }
    
    /// Query a contract
    pub fn query(
        &self,
        contract_address: String,
        msg: Vec<u8>,
    ) -> Result<Vec<u8>> {
        if !self.enabled {
            bail!("WASM runtime is not enabled");
        }
        
        // Verify contract exists
        if !self.contracts.contains_key(&contract_address) {
            bail!("Contract {} not found", contract_address);
        }
        
        // TODO: In production, query the contract
        // For now, return the query message as acknowledgment
        Ok(msg)
    }
    
    /// Get contract info
    pub fn get_contract(&self, address: &str) -> Option<&Contract> {
        self.contracts.get(address)
    }
    
    /// Get code info
    pub fn get_code_info(&self, code_id: u64) -> Option<&CodeInfo> {
        self.code_info.get(&code_id)
    }
    
    /// List all contracts
    pub fn list_contracts(&self) -> Vec<&Contract> {
        self.contracts.values().collect()
    }
    
    /// List all code
    pub fn list_code(&self) -> Vec<&CodeInfo> {
        self.code_info.values().collect()
    }
    
    /// Get runtime statistics
    pub fn get_stats(&self) -> WasmStats {
        WasmStats {
            enabled: self.enabled,
            total_codes: self.code_storage.len(),
            total_contracts: self.contracts.len(),
            next_code_id: self.next_code_id,
        }
    }
}

impl Default for WasmRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// WASM runtime statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmStats {
    pub enabled: bool,
    pub total_codes: usize,
    pub total_contracts: usize,
    pub next_code_id: u64,
}

/// Thread-safe WASM runtime handle
pub type SharedWasmRuntime = Arc<RwLock<WasmRuntime>>;

/// Create a shared WASM runtime
pub fn create_shared_runtime() -> SharedWasmRuntime {
    Arc::new(RwLock::new(WasmRuntime::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_runtime_disabled_by_default() {
        let runtime = WasmRuntime::new();
        assert!(!runtime.is_enabled());
    }
    
    #[test]
    fn test_store_code_requires_enabled() {
        let mut runtime = WasmRuntime::new();
        let result = runtime.store_code(
            "creator".to_string(),
            vec![0, 97, 115, 109], // WASM magic bytes
            1,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not enabled"));
    }
    
    #[test]
    fn test_enable_runtime() {
        let mut runtime = WasmRuntime::new();
        runtime.set_enabled(true);
        assert!(runtime.is_enabled());
    }
    
    #[test]
    fn test_store_valid_wasm() {
        let mut runtime = WasmRuntime::new();
        runtime.set_enabled(true);
        
        // Minimal valid WASM module (magic + version + empty sections)
        let wasm = vec![
            0x00, 0x61, 0x73, 0x6d, // \0asm magic
            0x01, 0x00, 0x00, 0x00, // version 1
        ];
        
        let result = runtime.store_code("creator".to_string(), wasm, 1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }
    
    #[test]
    fn test_invalid_wasm_rejected() {
        let mut runtime = WasmRuntime::new();
        runtime.set_enabled(true);
        
        let result = runtime.store_code(
            "creator".to_string(),
            vec![0, 1, 2, 3], // Not valid WASM
            1,
        );
        assert!(result.is_err());
    }
}
