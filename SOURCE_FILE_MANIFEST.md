# Sultan L1 - Actual Source File Manifest

**Generated**: December 6, 2025  
**Purpose**: Authoritative list of production source files for security auditors

---

## Core Source Files (sultan-core/src/)

### Critical Security Files (Priority 1)

1. **consensus.rs** (252 lines)
   - Validator selection and rotation
   - Voting power calculations
   - Byzantine fault tolerance (2/3+1)
   - Proposer selection (deterministic, SHA256-based)

2. **blockchain.rs**
   - Block production and validation
   - State transitions
   - Chain state management

3. **staking.rs**
   - Stake delegation and unbonding
   - Validator registration
   - Slashing conditions and execution

4. **transaction_validator.rs**
   - Transaction validation logic
   - Signature verification
   - Nonce checking

### Economic Security Files (Priority 2)

5. **governance.rs** (525 lines)
   - Proposal creation and execution
   - Voting mechanism
   - **HOT-ACTIVATION CODE** (lines 337-370)
   - Parameter change validation

6. **bridge_integration.rs**
   - Cross-chain bridge logic
   - Bridge validation
   - Asset transfer mechanisms

7. **bridge_fees.rs**
   - Bridge fee calculation
   - Zero-fee bridge configuration

8. **native_dex.rs**
   - Automated market maker (AMM)
   - Liquidity pool management
   - Swap execution
   - Price calculations

9. **token_factory.rs**
   - Token creation
   - Token metadata management

10. **economics.rs**
    - Inflation model (8% annual)
    - Fee distribution
    - APY calculations

### Infrastructure Files (Priority 3)

11. **sharding_production.rs**
    - Production sharding configuration
    - Shard count management (8 → 8,000)
    - Shard expansion logic
    - **TPS Calculation**: 8 shards × 8,000 TPS = 64,000 TPS initially
    - **Maximum TPS**: 8,000 shards × 8,000 TPS = 64M TPS

12. **sharded_blockchain_production.rs**
    - Sharded state management
    - Cross-shard operations
    - Shard assignment

13. **main.rs** (1584 lines)
    - Entry point
    - RPC server (warp-based)
    - **All RPC endpoint definitions**
    - Route handling
    - Server initialization

14. **config.rs** (50 lines)
    - Chain configuration
    - **FeatureFlags struct**
    - Default settings

15. **database.rs**
    - Database abstraction layer
    - ScyllaDB integration

16. **storage.rs**
    - Storage backend
    - Data persistence

17. **p2p.rs**
    - P2P networking (libp2p)
    - Peer discovery
    - Message propagation

18. **quantum.rs**
    - Quantum-safe cryptography
    - Dilithium3 signatures
    - Post-quantum algorithms

### Supporting Files

19. **types.rs**
    - Core type definitions
    - Transaction types
    - Block structures
    - Account types

20. **lib.rs**
    - Module exports
    - Public API surface

---

## ⚠️ Files NOT to Audit

These are development/test files, not production:

- **sharding.rs** - OLD development version (use sharding_production.rs)
- **sharded_blockchain.rs** - OLD version (use sharded_blockchain_production.rs)
- Any file in `tests/` directory
- Any file with `_test.rs` suffix

---

## Build Configuration

### Non-Standard Build Locations

**CRITICAL**: This project uses custom build configuration!

**Configuration file**: `.cargo/config.toml`
```toml
[build]
target-dir = "/tmp/cargo-target"
```

**Binary output**:
- Location: `/tmp/cargo-target/release/sultan-node`
- NOT in `./target/` (standard Rust location)
- Binary name: `sultan-node` (not `sultan-core`)

### Build Command

```bash
cargo build --release -p sultan-core
```

**Output**:
```
Compiling sultan-core v0.1.0
Finished release [optimized] target(s)
Binary: /tmp/cargo-target/release/sultan-node
```

---

## Line Count Reference

For audit scoping:

```bash
# Total production code
wc -l sultan-core/src/*.rs

# Key files
252   sultan-core/src/consensus.rs
525   sultan-core/src/governance.rs
1584  sultan-core/src/main.rs
```

---

## Module Dependencies

From `lib.rs`:

```rust
pub mod blockchain;
pub mod consensus;
pub mod p2p;
pub mod quantum;
pub mod database;
pub mod storage;
pub mod types;
pub mod config;
pub mod economics;
pub mod transaction_validator;
pub mod sharding;                         // Old version
pub mod sharding_production;              // PRODUCTION VERSION ✓
pub mod sharded_blockchain;               // Old version
pub mod sharded_blockchain_production;    // PRODUCTION VERSION ✓
pub mod bridge_integration;
pub mod bridge_fees;
pub mod staking;
pub mod governance;
pub mod token_factory;
pub mod native_dex;
```

---

## Critical Code Locations

### Feature Flag Hot-Activation

**File**: `governance.rs`  
**Lines**: 337-370  
**Function**: `execute_proposal()`

```rust
ProposalType::ParameterChange => {
    if let Some(params) = &proposal.parameters {
        for (key, value) in params {
            if key == "feature.wasm_contracts_enabled" && value == "true" {
                warn!("⚠️  ACTIVATING WASM CONTRACTS");
                // Runtime activation without chain restart
            }
            // ...
        }
    }
}
```

### Validator Selection

**File**: `consensus.rs`  
**Lines**: 105-144  
**Function**: `select_proposer()`

Deterministic weighted selection based on:
- Round number
- Total stake
- SHA256 hashing

### Bridge Validation

**File**: `bridge_integration.rs`  
**Search for**: `validate`, `verify`, `confirm`

### DEX Pricing

**File**: `native_dex.rs`  
**Search for**: `price`, `swap`, `liquidity`

---

## Audit Workflow

1. **Clone and verify**:
   ```bash
   git clone https://github.com/Wollnbergen/0xv7.git
   git log -1 --format="%H"  # Note commit hash
   ```

2. **Build**:
   ```bash
   cargo build --release -p sultan-core
   ls -lh /tmp/cargo-target/release/sultan-node
   ```

3. **Run tests**:
   ```bash
   cargo test --all --all-features
   ```

4. **Scan for issues**:
   ```bash
   cargo audit
   cargo clippy --all --all-features -- -D warnings
   ```

5. **Review files in order**:
   - Start with Priority 1 (consensus, staking)
   - Then Priority 2 (economics, governance)
   - Then Priority 3 (infrastructure)

---

## Common Audit Questions

**Q: Why two sharding files?**  
A: `sharding.rs` is development version. **PRODUCTION uses `sharding_production.rs`**. Check `main.rs` imports.

**Q: Where's the binary after build?**  
A: `/tmp/cargo-target/release/sultan-node` (NOT `./target/`)

**Q: Which bridge file to audit?**  
A: **Both** `bridge_integration.rs` AND `bridge_fees.rs`

**Q: Where's the DEX code?**  
A: `native_dex.rs` (NOT `dex.rs`)

**Q: Total lines of code?**  
A: Run `tokei sultan-core/src/` for accurate count

---

## Verification Checklist

Before starting audit:

- [ ] Confirmed commit hash matches audit scope
- [ ] Build completes successfully
- [ ] Binary found at `/tmp/cargo-target/release/sultan-node`
- [ ] All tests pass (`cargo test --all`)
- [ ] No CVEs (`cargo audit`)
- [ ] Have access to all 20 source files listed above
- [ ] Understand which files are production vs. development

---

## Contact

For questions about file structure or discrepancies:
- Open GitHub issue with `[AUDIT]` prefix
- Reference this manifest document
- Include actual file paths and line numbers

---

**This manifest is authoritative. If any documentation conflicts with this file, trust this manifest.**

**Last Updated**: December 6, 2025  
**Verified Against**: Commit hash on production deployment
