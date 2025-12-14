//! Sultan Cosmos Bridge - FFI Layer
//!
//! Production-grade Foreign Function Interface between Sultan Core (Rust) 
//! and Cosmos SDK (Go). This bridge enables Sultan's high-performance 
//! blockchain core to integrate seamlessly with the Cosmos ecosystem.
//!
//! Architecture:
//! - Rust (Sultan Core) ←→ C FFI ←→ CGo ←→ Go (Cosmos SDK)
//!
//! Safety: All FFI functions use proper error handling, null checks,
//! and memory management to prevent undefined behavior.

pub mod ffi;
pub mod types;
pub mod error;
pub mod state;
pub mod abci;

// Re-export for convenience
pub use ffi::*;
pub use types::*;
pub use error::*;
