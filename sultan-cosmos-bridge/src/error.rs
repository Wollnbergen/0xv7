//! Error types for FFI bridge
//!
//! Production-grade error handling across the FFI boundary.

use std::fmt;

/// Error codes for FFI functions
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeErrorCode {
    Success = 0,
    NullPointer = 1,
    InvalidUtf8 = 2,
    SerializationError = 3,
    DeserializationError = 4,
    BlockchainError = 5,
    ConsensusError = 6,
    TransactionError = 7,
    StateError = 8,
    InvalidParameter = 9,
    InternalError = 10,
}

impl fmt::Display for BridgeErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BridgeErrorCode::Success => write!(f, "Success"),
            BridgeErrorCode::NullPointer => write!(f, "Null pointer"),
            BridgeErrorCode::InvalidUtf8 => write!(f, "Invalid UTF-8"),
            BridgeErrorCode::SerializationError => write!(f, "Serialization error"),
            BridgeErrorCode::DeserializationError => write!(f, "Deserialization error"),
            BridgeErrorCode::BlockchainError => write!(f, "Blockchain error"),
            BridgeErrorCode::ConsensusError => write!(f, "Consensus error"),
            BridgeErrorCode::TransactionError => write!(f, "Transaction error"),
            BridgeErrorCode::StateError => write!(f, "State error"),
            BridgeErrorCode::InvalidParameter => write!(f, "Invalid parameter"),
            BridgeErrorCode::InternalError => write!(f, "Internal error"),
        }
    }
}

/// FFI-safe error result
#[repr(C)]
pub struct BridgeError {
    pub code: BridgeErrorCode,
    pub message: *mut libc::c_char,
}

impl BridgeError {
    pub fn success() -> Self {
        BridgeError {
            code: BridgeErrorCode::Success,
            message: std::ptr::null_mut(),
        }
    }

    pub fn new(code: BridgeErrorCode, message: String) -> Self {
        let c_message = std::ffi::CString::new(message)
            .unwrap_or_else(|_| std::ffi::CString::new("Invalid error message").unwrap());
        
        BridgeError {
            code,
            message: c_message.into_raw(),
        }
    }

    pub fn null_pointer() -> Self {
        Self::new(BridgeErrorCode::NullPointer, "Null pointer provided".to_string())
    }

    pub fn invalid_utf8() -> Self {
        Self::new(BridgeErrorCode::InvalidUtf8, "Invalid UTF-8 string".to_string())
    }

    pub fn serialization_error(msg: &str) -> Self {
        Self::new(BridgeErrorCode::SerializationError, format!("Serialization failed: {}", msg))
    }

    pub fn blockchain_error(msg: &str) -> Self {
        Self::new(BridgeErrorCode::BlockchainError, format!("Blockchain error: {}", msg))
    }
}

/// Free error message memory (must be called from Go side)
#[no_mangle]
pub extern "C" fn sultan_bridge_free_error(error: BridgeError) {
    if !error.message.is_null() {
        unsafe {
            let _ = std::ffi::CString::from_raw(error.message);
        }
    }
}
