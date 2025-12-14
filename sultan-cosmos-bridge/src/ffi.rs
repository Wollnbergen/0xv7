//! FFI functions for Sultan-Cosmos bridge
//!
//! Production-grade C-compatible API for Go integration.
//! All functions follow these safety principles:
//! - Null pointer checks
//! - Panic catching (convert to error codes)
//! - Proper memory management
//! - Thread-safe global state

use crate::error::{BridgeError, BridgeErrorCode};
use crate::types::*;
use crate::state::get_state;
use sultan_core::{Blockchain, Transaction, ConsensusEngine};
use std::ffi::{CStr, CString};
use std::panic;

// ============================================================================
// Initialization & Cleanup
// ============================================================================

/// Initialize the Sultan bridge (call once at startup)
#[no_mangle]
pub extern "C" fn sultan_bridge_init() -> BridgeError {
    panic::catch_unwind(|| {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .try_init()
            .ok(); // Ignore if already initialized
        
        BridgeError::success()
    }).unwrap_or_else(|_| {
        BridgeError::new(BridgeErrorCode::InternalError, "Panic during init".to_string())
    })
}

/// Shutdown the bridge (cleanup resources)
#[no_mangle]
pub extern "C" fn sultan_bridge_shutdown() -> BridgeError {
    BridgeError::success()
}

// ============================================================================
// Blockchain Management
// ============================================================================

/// Create new blockchain instance
/// Returns: handle ID (> 0) on success, 0 on error
#[no_mangle]
pub extern "C" fn sultan_blockchain_new(error: *mut BridgeError) -> usize {
    panic::catch_unwind(|| {
        let blockchain = Blockchain::new();
        let id = get_state().write().add_blockchain(blockchain);
        
        if !error.is_null() {
            unsafe { *error = BridgeError::success(); }
        }
        
        id
    }).unwrap_or_else(|_| {
        if !error.is_null() {
            unsafe {
                *error = BridgeError::new(
                    BridgeErrorCode::InternalError,
                    "Panic creating blockchain".to_string()
                );
            }
        }
        0
    })
}

/// Destroy blockchain instance
#[no_mangle]
pub extern "C" fn sultan_blockchain_destroy(handle: usize) -> BridgeError {
    panic::catch_unwind(|| {
        if handle == 0 {
            return BridgeError::new(
                BridgeErrorCode::InvalidParameter,
                "Invalid handle (0)".to_string()
            );
        }

        let removed = get_state().write().remove_blockchain(handle);
        if removed.is_some() {
            BridgeError::success()
        } else {
            BridgeError::new(
                BridgeErrorCode::InvalidParameter,
                format!("Blockchain handle {} not found", handle)
            )
        }
    }).unwrap_or_else(|_| {
        BridgeError::new(BridgeErrorCode::InternalError, "Panic destroying blockchain".to_string())
    })
}

/// Get blockchain height
#[no_mangle]
pub extern "C" fn sultan_blockchain_height(handle: usize, error: *mut BridgeError) -> u64 {
    panic::catch_unwind(|| {
        if handle == 0 {
            if !error.is_null() {
                unsafe {
                    *error = BridgeError::new(
                        BridgeErrorCode::InvalidParameter,
                        "Invalid handle".to_string()
                    );
                }
            }
            return 0;
        }

        let state = get_state().read();
        if let Some(blockchain) = state.get_blockchain(handle) {
            if !error.is_null() {
                unsafe { *error = BridgeError::success(); }
            }
            blockchain.height()
        } else {
            if !error.is_null() {
                unsafe {
                    *error = BridgeError::new(
                        BridgeErrorCode::InvalidParameter,
                        "Blockchain not found".to_string()
                    );
                }
            }
            0
        }
    }).unwrap_or(0)
}

/// Get latest block hash
#[no_mangle]
pub extern "C" fn sultan_blockchain_latest_hash(
    handle: usize,
    error: *mut BridgeError
) -> *mut libc::c_char {
    panic::catch_unwind(|| {
        if handle == 0 {
            if !error.is_null() {
                unsafe { *error = BridgeError::new(BridgeErrorCode::InvalidParameter, "Invalid handle".to_string()); }
            }
            return std::ptr::null_mut();
        }

        let state = get_state().read();
        if let Some(blockchain) = state.get_blockchain(handle) {
            let latest = blockchain.get_latest_block();
            let hash_cstr = CString::new(latest.hash.clone()).unwrap();
            
            if !error.is_null() {
                unsafe { *error = BridgeError::success(); }
            }
            
            hash_cstr.into_raw()
        } else {
            if !error.is_null() {
                unsafe { *error = BridgeError::new(BridgeErrorCode::InvalidParameter, "Blockchain not found".to_string()); }
            }
            std::ptr::null_mut()
        }
    }).unwrap_or(std::ptr::null_mut())
}

/// Free string memory
#[no_mangle]
pub extern "C" fn sultan_bridge_free_string(s: *mut libc::c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

// ============================================================================
// Transaction Management
// ============================================================================

/// Add transaction to blockchain
#[no_mangle]
pub extern "C" fn sultan_blockchain_add_transaction(
    handle: usize,
    tx: CTransaction,
    error: *mut BridgeError
) -> bool {
    panic::catch_unwind(|| {
        if handle == 0 {
            if !error.is_null() {
                unsafe { *error = BridgeError::new(BridgeErrorCode::InvalidParameter, "Invalid handle".to_string()); }
            }
            return false;
        }

        // Convert C transaction to Rust
        let from = unsafe {
            if tx.from.is_null() {
                if !error.is_null() {
                    *error = BridgeError::null_pointer();
                }
                return false;
            }
            match CStr::from_ptr(tx.from).to_str() {
                Ok(s) => s.to_string(),
                Err(_) => {
                    if !error.is_null() {
                        *error = BridgeError::invalid_utf8();
                    }
                    return false;
                }
            }
        };

        let to = unsafe {
            if tx.to.is_null() {
                if !error.is_null() {
                    *error = BridgeError::null_pointer();
                }
                return false;
            }
            match CStr::from_ptr(tx.to).to_str() {
                Ok(s) => s.to_string(),
                Err(_) => {
                    if !error.is_null() {
                        *error = BridgeError::invalid_utf8();
                    }
                    return false;
                }
            }
        };

        let signature = if tx.signature.is_null() {
            None
        } else {
            unsafe {
                CStr::from_ptr(tx.signature).to_str().ok().map(|s| s.to_string())
            }
        };

        let rust_tx = Transaction {
            from,
            to,
            amount: tx.amount,
            gas_fee: tx.gas_fee,
            timestamp: tx.timestamp,
            nonce: tx.nonce,
            signature,
        };

        let mut state = get_state().write();
        if let Some(blockchain) = state.get_blockchain_mut(handle) {
            match blockchain.add_transaction(rust_tx) {
                Ok(_) => {
                    if !error.is_null() {
                        unsafe { *error = BridgeError::success(); }
                    }
                    true
                }
                Err(e) => {
                    if !error.is_null() {
                        unsafe { *error = BridgeError::blockchain_error(&e.to_string()); }
                    }
                    false
                }
            }
        } else {
            if !error.is_null() {
                unsafe { *error = BridgeError::new(BridgeErrorCode::InvalidParameter, "Blockchain not found".to_string()); }
            }
            false
        }
    }).unwrap_or(false)
}

/// Get account balance
#[no_mangle]
pub extern "C" fn sultan_blockchain_get_balance(
    handle: usize,
    address: *const libc::c_char,
    error: *mut BridgeError
) -> u64 {
    panic::catch_unwind(|| {
        if handle == 0 || address.is_null() {
            if !error.is_null() {
                unsafe { *error = BridgeError::null_pointer(); }
            }
            return 0;
        }

        let addr = unsafe {
            match CStr::from_ptr(address).to_str() {
                Ok(s) => s,
                Err(_) => {
                    if !error.is_null() {
                        *error = BridgeError::invalid_utf8();
                    }
                    return 0;
                }
            }
        };

        let state = get_state().read();
        if let Some(blockchain) = state.get_blockchain(handle) {
            let balance = blockchain.get_balance(addr);
            if !error.is_null() {
                unsafe { *error = BridgeError::success(); }
            }
            balance
        } else {
            if !error.is_null() {
                unsafe { *error = BridgeError::new(BridgeErrorCode::InvalidParameter, "Blockchain not found".to_string()); }
            }
            0
        }
    }).unwrap_or(0)
}

/// Initialize account
#[no_mangle]
pub extern "C" fn sultan_blockchain_init_account(
    handle: usize,
    address: *const libc::c_char,
    balance: u64,
    error: *mut BridgeError
) -> bool {
    panic::catch_unwind(|| {
        if handle == 0 || address.is_null() {
            if !error.is_null() {
                unsafe { *error = BridgeError::null_pointer(); }
            }
            return false;
        }

        let addr = unsafe {
            match CStr::from_ptr(address).to_str() {
                Ok(s) => s.to_string(),
                Err(_) => {
                    if !error.is_null() {
                        *error = BridgeError::invalid_utf8();
                    }
                    return false;
                }
            }
        };

        let mut state = get_state().write();
        if let Some(blockchain) = state.get_blockchain_mut(handle) {
            blockchain.init_account(addr, balance);
            if !error.is_null() {
                unsafe { *error = BridgeError::success(); }
            }
            true
        } else {
            if !error.is_null() {
                unsafe { *error = BridgeError::new(BridgeErrorCode::InvalidParameter, "Blockchain not found".to_string()); }
            }
            false
        }
    }).unwrap_or(false)
}

// ============================================================================
// Block Production
// ============================================================================

/// Create new block
#[no_mangle]
pub extern "C" fn sultan_blockchain_create_block(
    handle: usize,
    validator: *const libc::c_char,
    error: *mut BridgeError
) -> bool {
    panic::catch_unwind(|| {
        if handle == 0 || validator.is_null() {
            if !error.is_null() {
                unsafe { *error = BridgeError::null_pointer(); }
            }
            return false;
        }

        let val = unsafe {
            match CStr::from_ptr(validator).to_str() {
                Ok(s) => s.to_string(),
                Err(_) => {
                    if !error.is_null() {
                        *error = BridgeError::invalid_utf8();
                    }
                    return false;
                }
            }
        };

        let mut state = get_state().write();
        if let Some(blockchain) = state.get_blockchain_mut(handle) {
            match blockchain.create_block(val) {
                Ok(block) => {
                    // Block created but not yet added to chain
                    // Caller must validate and add
                    blockchain.chain.push(block);
                    if !error.is_null() {
                        unsafe { *error = BridgeError::success(); }
                    }
                    true
                }
                Err(e) => {
                    if !error.is_null() {
                        unsafe { *error = BridgeError::blockchain_error(&e.to_string()); }
                    }
                    false
                }
            }
        } else {
            if !error.is_null() {
                unsafe { *error = BridgeError::new(BridgeErrorCode::InvalidParameter, "Blockchain not found".to_string()); }
            }
            false
        }
    }).unwrap_or(false)
}

// ============================================================================
// Consensus Management
// ============================================================================

/// Create new consensus engine
#[no_mangle]
pub extern "C" fn sultan_consensus_new(error: *mut BridgeError) -> usize {
    panic::catch_unwind(|| {
        let consensus = ConsensusEngine::new();
        let id = get_state().write().add_consensus(consensus);
        
        if !error.is_null() {
            unsafe { *error = BridgeError::success(); }
        }
        
        id
    }).unwrap_or_else(|_| {
        if !error.is_null() {
            unsafe {
                *error = BridgeError::new(
                    BridgeErrorCode::InternalError,
                    "Panic creating consensus".to_string()
                );
            }
        }
        0
    })
}

/// Add validator
#[no_mangle]
pub extern "C" fn sultan_consensus_add_validator(
    handle: usize,
    address: *const libc::c_char,
    stake: u64,
    error: *mut BridgeError
) -> bool {
    panic::catch_unwind(|| {
        if handle == 0 || address.is_null() {
            if !error.is_null() {
                unsafe { *error = BridgeError::null_pointer(); }
            }
            return false;
        }

        let addr = unsafe {
            match CStr::from_ptr(address).to_str() {
                Ok(s) => s.to_string(),
                Err(_) => {
                    if !error.is_null() {
                        *error = BridgeError::invalid_utf8();
                    }
                    return false;
                }
            }
        };

        let mut state = get_state().write();
        if let Some(consensus) = state.get_consensus_mut(handle) {
            match consensus.add_validator(addr, stake) {
                Ok(_) => {
                    if !error.is_null() {
                        unsafe { *error = BridgeError::success(); }
                    }
                    true
                }
                Err(e) => {
                    if !error.is_null() {
                        unsafe { *error = BridgeError::new(BridgeErrorCode::ConsensusError, e.to_string()); }
                    }
                    false
                }
            }
        } else {
            if !error.is_null() {
                unsafe { *error = BridgeError::new(BridgeErrorCode::InvalidParameter, "Consensus not found".to_string()); }
            }
            false
        }
    }).unwrap_or(false)
}

/// Select next proposer
#[no_mangle]
pub extern "C" fn sultan_consensus_select_proposer(
    handle: usize,
    error: *mut BridgeError
) -> *mut libc::c_char {
    panic::catch_unwind(|| {
        if handle == 0 {
            if !error.is_null() {
                unsafe { *error = BridgeError::new(BridgeErrorCode::InvalidParameter, "Invalid handle".to_string()); }
            }
            return std::ptr::null_mut();
        }

        let mut state = get_state().write();
        if let Some(consensus) = state.get_consensus_mut(handle) {
            if let Some(proposer) = consensus.select_proposer() {
                let proposer_cstr = CString::new(proposer).unwrap();
                if !error.is_null() {
                    unsafe { *error = BridgeError::success(); }
                }
                proposer_cstr.into_raw()
            } else {
                if !error.is_null() {
                    unsafe { *error = BridgeError::new(BridgeErrorCode::ConsensusError, "No active validators".to_string()); }
                }
                std::ptr::null_mut()
            }
        } else {
            if !error.is_null() {
                unsafe { *error = BridgeError::new(BridgeErrorCode::InvalidParameter, "Consensus not found".to_string()); }
            }
            std::ptr::null_mut()
        }
    }).unwrap_or(std::ptr::null_mut())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_lifecycle() {
        let mut error = BridgeError::success();
        
        // Create blockchain
        let handle = sultan_blockchain_new(&mut error);
        assert_eq!(error.code, BridgeErrorCode::Success);
        assert!(handle > 0);
        
        // Get height
        let height = sultan_blockchain_height(handle, &mut error);
        assert_eq!(error.code, BridgeErrorCode::Success);
        assert_eq!(height, 0);
        
        // Destroy
        let destroy_error = sultan_blockchain_destroy(handle);
        assert_eq!(destroy_error.code, BridgeErrorCode::Success);
    }
}
