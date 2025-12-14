//! FFI-safe type definitions
//!
//! C-compatible structs for cross-language communication.


/// Opaque handle to Blockchain instance
#[repr(C)]
pub struct BlockchainHandle {
    _private: [u8; 0],
}

/// Opaque handle to ConsensusEngine instance
#[repr(C)]
pub struct ConsensusHandle {
    _private: [u8; 0],
}

/// FFI-safe transaction structure
#[repr(C)]
#[derive(Debug, Clone)]
pub struct CTransaction {
    pub from: *const libc::c_char,
    pub to: *const libc::c_char,
    pub amount: u64,
    pub gas_fee: u64,
    pub timestamp: u64,
    pub nonce: u64,
    pub signature: *const libc::c_char,
}

/// FFI-safe block structure
#[repr(C)]
#[derive(Debug, Clone)]
pub struct CBlock {
    pub index: u64,
    pub timestamp: u64,
    pub transactions_ptr: *const CTransaction,
    pub transactions_len: usize,
    pub prev_hash: *const libc::c_char,
    pub hash: *const libc::c_char,
    pub nonce: u64,
    pub validator: *const libc::c_char,
    pub state_root: *const libc::c_char,
}

/// FFI-safe account structure
#[repr(C)]
#[derive(Debug, Clone)]
pub struct CAccount {
    pub address: *const libc::c_char,
    pub balance: u64,
    pub nonce: u64,
}

/// FFI-safe validator structure
#[repr(C)]
#[derive(Debug, Clone)]
pub struct CValidator {
    pub address: *const libc::c_char,
    pub stake: u64,
    pub voting_power: u64,
    pub is_active: bool,
    pub blocks_proposed: u64,
    pub blocks_signed: u64,
}

/// Node status information
#[repr(C)]
#[derive(Debug, Clone)]
pub struct CNodeStatus {
    pub height: u64,
    pub latest_hash: *const libc::c_char,
    pub validator_count: u64,
    pub total_accounts: u64,
    pub pending_txs: u64,
}

/// Serialized data buffer for complex types
#[repr(C)]
pub struct CByteArray {
    pub data: *const u8,
    pub len: usize,
}

impl CByteArray {
    pub fn new(data: Vec<u8>) -> Self {
        let len = data.len();
        let ptr = data.as_ptr();
        std::mem::forget(data);  // Prevent deallocation
        CByteArray { data: ptr, len }
    }

    pub fn null() -> Self {
        CByteArray {
            data: std::ptr::null(),
            len: 0,
        }
    }
}

/// Free byte array memory
#[no_mangle]
pub extern "C" fn sultan_bridge_free_bytes(bytes: CByteArray) {
    if !bytes.data.is_null() && bytes.len > 0 {
        unsafe {
            let _ = Vec::from_raw_parts(bytes.data as *mut u8, bytes.len, bytes.len);
        }
    }
}
