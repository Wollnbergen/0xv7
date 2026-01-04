# Sultan Core

**Layer 1: Pure Sultan Blockchain Implementation (Rust)**

This is the core blockchain implementation for Sultan Chain, written in Rust. It provides:

- ✅ Block creation and validation
- ✅ Transaction processing  
- ✅ Ed25519 cryptography
- ✅ Consensus mechanism
- ✅ P2P networking
- ✅ Persistent storage
- ✅ Economics and tokenomics

## Architecture

Sultan Core is Layer 1 of the 3-layer Sultan blockchain architecture:

```
Layer 1: Sultan Core (Rust) ← YOU ARE HERE
    ↕ FFI
Layer 2: Cosmos Bridge (Rust + Go)
    ↕
Layer 3: Cosmos Ecosystem (Go)
```

## Usage

```rust
use sultan_core::{Blockchain, Transaction};

// Initialize blockchain
let mut chain = Blockchain::new();

// Add transaction
let tx = Transaction {
    from: "alice".to_string(),
    to: "bob".to_string(),
    amount: 100,
    gas_fee: 0, // Zero fees on Sultan
    timestamp: 0,
};

chain.add_transaction(tx);

// Create block
let block = chain.create_block();
```

## Building

```bash
# Build library
cargo build --release

# Run tests
cargo test

# Build with FFI support
cargo build --release --lib
```

## FFI Support

This crate is built with C FFI support (`staticlib`, `cdylib`) for integration with Go via the Cosmos SDK bridge.

## License

MIT
