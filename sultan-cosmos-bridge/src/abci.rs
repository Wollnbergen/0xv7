//! ABCI (Application Blockchain Interface) Adapter
//!
//! Bridges Sultan Core to Cosmos SDK's CometBFT consensus engine.
//! Implements the ABCI protocol for seamless integration.

use crate::error::{BridgeError, BridgeErrorCode};
use crate::types::CByteArray;
use sultan_core::Transaction;
use serde::{Deserialize, Deserializer, Serialize};
use base64::{Engine as _, engine::general_purpose};

/// Custom deserializer for base64-encoded byte arrays
fn deserialize_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    general_purpose::STANDARD.decode(s.as_bytes())
        .map_err(serde::de::Error::custom)
}

/// Custom deserializer for optional base64-encoded byte arrays (with #[serde(default)])
fn deserialize_base64_opt<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(vec![]);
    }
    general_purpose::STANDARD.decode(s.as_bytes())
        .map_err(serde::de::Error::custom)
}

/// Genesis account for InitChain
#[derive(Debug, Serialize, Deserialize)]
pub struct GenesisAccount {
    pub address: String,
    pub balance: u64,
}

/// ABCI Request types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ABCIRequest {
    Info,
    InitChain {
        #[serde(default)]
        validators: Vec<String>,
        genesis_accounts: Vec<GenesisAccount>,
    },
    BeginBlock { height: u64, proposer: String },
    DeliverTx { 
        #[serde(deserialize_with = "deserialize_base64")]
        tx_data: Vec<u8> 
    },
    EndBlock { height: u64 },
    Commit,
    Query {
        path: String,
        #[serde(default, deserialize_with = "deserialize_base64_opt")]
        data: Vec<u8>,
    },
}

/// ABCI Response types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ABCIResponse {
    Info { height: u64, app_hash: String },
    InitChainOk,
    BeginBlockOk,
    DeliverTx { code: u32, log: String },
    EndBlockOk { validator_updates: Vec<String> },
    Commit { data: Vec<u8> },
    Query { code: u32, value: Vec<u8>, log: String },
}

/// Process ABCI request
#[no_mangle]
pub extern "C" fn sultan_abci_process(
    blockchain_handle: usize,
    request_bytes: CByteArray,
    error: *mut BridgeError
) -> CByteArray {
    let result = std::panic::catch_unwind(|| {
        if blockchain_handle == 0 {
            if !error.is_null() {
                unsafe {
                    *error = BridgeError::new(
                        BridgeErrorCode::InvalidParameter,
                        "Invalid blockchain handle".to_string()
                    );
                }
            }
            return CByteArray::null();
        }

        if request_bytes.data.is_null() || request_bytes.len == 0 {
            if !error.is_null() {
                unsafe { *error = BridgeError::null_pointer(); }
            }
            return CByteArray::null();
        }

        // Deserialize request
        let request_data = unsafe {
            std::slice::from_raw_parts(request_bytes.data, request_bytes.len)
        };

        let request: ABCIRequest = match serde_json::from_slice(request_data) {
            Ok(req) => req,
            Err(e) => {
                if !error.is_null() {
                    unsafe {
                        *error = BridgeError::serialization_error(&e.to_string());
                    }
                }
                return CByteArray::null();
            }
        };

        // Process request
        let response = process_abci_request(blockchain_handle, request);

        // Serialize response
        match serde_json::to_vec(&response) {
            Ok(data) => {
                if !error.is_null() {
                    unsafe { *error = BridgeError::success(); }
                }
                CByteArray::new(data)
            }
            Err(e) => {
                if !error.is_null() {
                    unsafe {
                        *error = BridgeError::serialization_error(&e.to_string());
                    }
                }
                CByteArray::null()
            }
        }
    });

    result.unwrap_or_else(|_| {
        if !error.is_null() {
            unsafe {
                *error = BridgeError::new(
                    BridgeErrorCode::InternalError,
                    "Panic in ABCI processing".to_string()
                );
            }
        }
        CByteArray::null()
    })
}

/// Internal ABCI request processing
fn process_abci_request(blockchain_handle: usize, request: ABCIRequest) -> ABCIResponse {
    use crate::state::get_state;

    match request {
        ABCIRequest::Info => {
            let state = get_state().read();
            if let Some(blockchain) = state.get_blockchain(blockchain_handle) {
                let latest = blockchain.get_latest_block();
                ABCIResponse::Info {
                    height: blockchain.height(),
                    app_hash: latest.state_root.clone(),
                }
            } else {
                ABCIResponse::Info {
                    height: 0,
                    app_hash: String::new(),
                }
            }
        }

        ABCIRequest::InitChain { validators: _, genesis_accounts } => {
            let mut state = get_state().write();
            if let Some(blockchain) = state.get_blockchain_mut(blockchain_handle) {
                for account in genesis_accounts {
                    blockchain.init_account(account.address, account.balance);
                }
            }
            ABCIResponse::InitChainOk
        }

        ABCIRequest::BeginBlock { height: _, proposer: _ } => {
            // Prepare for new block
            ABCIResponse::BeginBlockOk
        }

        ABCIRequest::DeliverTx { tx_data } => {
            let mut state = get_state().write();
            if let Some(blockchain) = state.get_blockchain_mut(blockchain_handle) {
                // tx_data is already base64-decoded by custom deserializer
                match serde_json::from_slice::<Transaction>(&tx_data) {
                    Ok(tx) => {
                        match blockchain.add_transaction(tx) {
                            Ok(_) => ABCIResponse::DeliverTx {
                                code: 0,
                                log: "Transaction accepted".to_string(),
                            },
                            Err(e) => ABCIResponse::DeliverTx {
                                code: 1,
                                log: format!("Transaction rejected: {}", e),
                            },
                        }
                    }
                    Err(e) => ABCIResponse::DeliverTx {
                        code: 2,
                        log: format!("Invalid transaction format: {}", e),
                    },
                }
            } else {
                ABCIResponse::DeliverTx {
                    code: 3,
                    log: "Blockchain not found".to_string(),
                }
            }
        }

        ABCIRequest::EndBlock { height: _ } => {
            // Finalize block
            ABCIResponse::EndBlockOk {
                validator_updates: vec![],
            }
        }

        ABCIRequest::Commit => {
            let state = get_state().read();
            if let Some(blockchain) = state.get_blockchain(blockchain_handle) {
                let latest = blockchain.get_latest_block();
                let state_root = latest.state_root.clone();
                ABCIResponse::Commit {
                    data: state_root.into_bytes(),
                }
            } else {
                ABCIResponse::Commit {
                    data: vec![],
                }
            }
        }

        ABCIRequest::Query { path, data: _ } => {
            let state = get_state().read();
            if let Some(blockchain) = state.get_blockchain(blockchain_handle) {
                // Parse path - supports /balance/{address}
                if path.starts_with("/balance/") {
                    let address = &path[9..]; // Skip "/balance/"
                    let balance = blockchain.get_balance(address);
                    ABCIResponse::Query {
                        code: 0,
                        value: balance.to_string().into_bytes(),
                        log: format!("Balance for {}: {}", address, balance),
                    }
                } else if path == "/height" {
                    let height = blockchain.height();
                    ABCIResponse::Query {
                        code: 0,
                        value: height.to_string().into_bytes(),
                        log: "Height query".to_string(),
                    }
                } else {
                    ABCIResponse::Query {
                        code: 1,
                        value: vec![],
                        log: format!("Unknown query path: {}", path),
                    }
                }
            } else {
                ABCIResponse::Query {
                    code: 2,
                    value: vec![],
                    log: "Blockchain not found".to_string(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abci_info() {
        let request = ABCIRequest::Info;
        let response = process_abci_request(0, request);
        
        match response {
            ABCIResponse::Info { height, app_hash: _ } => {
                assert_eq!(height, 0); // No blockchain
            }
            _ => panic!("Unexpected response"),
        }
    }

    #[test]
    fn test_abci_serialization() {
        let request = ABCIRequest::Info;
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ABCIRequest = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            ABCIRequest::Info => (),
            _ => panic!("Deserialization failed"),
        }
    }
}
