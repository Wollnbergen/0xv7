# Sultan L1 Blockchain - Build Instructions

**For Third-Party Auditors and Developers**

This document provides complete instructions for building and auditing the Sultan L1 blockchain codebase.

## ⚠️ Important Build Configuration Notes

This project uses a **custom build configuration** that differs from standard Rust projects:

1. **Build Output Location**: `/tmp/cargo-target/` instead of `./target/`
2. **Binary Name**: `sultan-node` (not `sultan-core`)
3. **Workspace Configuration**: Custom `.cargo/config.toml` redirects build artifacts

These configurations are intentional and documented below.

---

## Prerequisites

### Required Software

- **Rust**: 1.75.0 or later
- **Cargo**: Latest stable
- **Git**: For source control
- **SSH**: For deployment (production only)
- **jq**: For JSON processing in scripts

Install Rust via rustup:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### System Requirements

- **RAM**: 8GB minimum (16GB recommended for release builds)
- **Disk**: 10GB free space
- **CPU**: 4 cores recommended
- **OS**: Linux (Ubuntu 20.04+), macOS, or WSL2 on Windows

---

## Build Configuration

### Custom Cargo Configuration

The project uses a custom build directory configuration in `.cargo/config.toml`:

```toml
[build]
target-dir = "/tmp/cargo-target"
```

**Why?** This prevents build artifacts from cluttering the workspace and improves performance in containerized environments.

### Binary Output

The main binary is named `sultan-node` and is defined in `Cargo.toml`:

```toml
[[bin]]
name = "sultan-node"
path = "src/main.rs"
```

**After building**, find the binary at:
- **Debug**: `/tmp/cargo-target/debug/sultan-node`
- **Release**: `/tmp/cargo-target/release/sultan-node`

---

## Building the Project

### Quick Build (Development)

```bash
# Clone the repository
git clone https://github.com/Wollnbergen/0xv7.git
cd 0xv7

# Build debug version
cargo build -p sultan-core

# Binary location
/tmp/cargo-target/debug/sultan-node --help
```

### Production Build (Optimized)

```bash
# Build release version (optimized)
cargo build --release -p sultan-core

# Binary location
/tmp/cargo-target/release/sultan-node --help

# Binary size: ~16MB
ls -lh /tmp/cargo-target/release/sultan-node
```

### Build Time Expectations

- **Debug build**: 2-5 minutes (first build), 10-30 seconds (incremental)
- **Release build**: 10-15 minutes (first build), 1-2 minutes (incremental)
- **Full workspace rebuild**: 15-20 minutes

---

## Automated Build Script

We provide `build-production.sh` for automated production builds:

```bash
#!/bin/bash
# Usage: ./build-production.sh

chmod +x build-production.sh
./build-production.sh
```

This script:
1. Verifies Rust installation
2. Runs cargo check for quick validation
3. Builds release binary
4. Verifies binary output
5. Reports size and location

---

## Verification Steps

### 1. Verify Binary Exists

```bash
ls -lh /tmp/cargo-target/release/sultan-node
```

Expected output: `~16MB` binary file

### 2. Test Binary Execution

```bash
/tmp/cargo-target/release/sultan-node --help
```

Expected: Help menu with command-line options

### 3. Verify Dependencies

```bash
cargo tree -p sultan-core | head -20
```

Shows dependency graph for audit

### 4. Check for Security Vulnerabilities

```bash
cargo audit
```

Scans dependencies for known CVEs

---

## Running Tests

### Unit Tests

```bash
# Run all tests
cargo test --all --all-features

# Run specific package tests
cargo test -p sultan-core --all-features

# Run with output
cargo test -- --nocapture
```

### Integration Tests

```bash
# Requires Scylla database
scripts/scylla_dev.sh start

# Run node tests
cargo test -p node --all-features
```

---

## Code Audit Guidelines

### Project Structure

```
0xv7/
├── sultan-core/                      # Main blockchain implementation
│   ├── src/
│   │   ├── main.rs                   # Entry point, RPC server
│   │   ├── blockchain.rs             # Block production logic
│   │   ├── consensus.rs              # Validator consensus (PoS)
│   │   ├── sharding_production.rs    # Production sharding
│   │   ├── sharded_blockchain_production.rs  # Sharded blockchain
│   │   ├── governance.rs             # On-chain governance
│   │   ├── bridge_integration.rs     # Cross-chain bridges
│   │   ├── bridge_fees.rs            # Bridge fee logic
│   │   ├── staking.rs                # Validator staking
│   │   ├── native_dex.rs             # Built-in DEX
│   │   ├── token_factory.rs          # Token creation
│   │   ├── config.rs                 # Feature flags
│   │   ├── economics.rs              # Economic model
│   │   ├── transaction_validator.rs  # Transaction validation
│   │   ├── database.rs               # Database layer
│   │   ├── storage.rs                # Storage backend
│   │   ├── p2p.rs                    # P2P networking
│   │   ├── quantum.rs                # Quantum-safe crypto
│   │   └── lib.rs                    # Library exports
│   └── Cargo.toml
├── Cargo.toml            # Workspace configuration
└── .cargo/config.toml    # Build configuration
```

### Critical Files for Security Audit

**Priority 1 - Consensus & Validation:**
- `sultan-core/src/consensus.rs` - Validator selection, voting power
- `sultan-core/src/blockchain.rs` - Block validation, chain state
- `sultan-core/src/staking.rs` - Stake management, slashing

**Priority 2 - Economic Security:**
- `sultan-core/src/governance.rs` - Proposal execution, parameter changes
- `sultan-core/src/bridge_integration.rs` - Cross-chain value transfer
- `sultan-core/src/bridge_fees.rs` - Bridge fee logic
- `sultan-core/src/native_dex.rs` - Automated market maker logic
- `sultan-core/src/economics.rs` - Economic model, inflation
- `sultan-core/src/token_factory.rs` - Token creation

**Priority 3 - Infrastructure:**
- `sultan-core/src/sharding_production.rs` - Production shard management
- `sultan-core/src/sharded_blockchain_production.rs` - Sharded state distribution
- `sultan-core/src/main.rs` - RPC endpoints, input validation
- `sultan-core/src/config.rs` - Feature flags, hot-upgrade logic
- `sultan-core/src/database.rs` - Database abstraction
- `sultan-core/src/storage.rs` - Storage backend
- `sultan-core/src/p2p.rs` - P2P networking
- `sultan-core/src/quantum.rs` - Quantum-safe cryptography

### Key Security Considerations

1. **Consensus Safety**
   - 2/3+1 Byzantine fault tolerance in `consensus.rs`
   - Validator voting power calculations
   - Proposer selection determinism

2. **Economic Security**
   - No integer overflows in stake calculations
   - Proper bounds checking on token amounts
   - Slashing conditions and execution

3. **Bridge Security**
   - Zero-fee bridge validation
   - Cross-chain transaction verification
   - Replay attack prevention

4. **Smart Contract Safety** (Future)
   - WASM execution sandboxing
   - Gas metering
   - Storage rent prevention

---

## Feature Flags

The blockchain uses **governance-activated feature flags** for zero-downtime upgrades.

Configuration file: `sultan-core/chain_config.json`

```json
{
  "features": {
    "sharding_enabled": true,
    "governance_enabled": true,
    "bridges_enabled": true,
    "wasm_contracts_enabled": false,  // Activate via governance
    "evm_contracts_enabled": false    // Activate via governance
  }
}
```

**Hot-Activation Code**: See `sultan-core/src/governance.rs` line 337-370

```rust
// governance.rs - execute_proposal()
if key == "feature.wasm_contracts_enabled" && value == "true" {
    warn!("⚠️  ACTIVATING WASM CONTRACTS");
    // Runtime activation without chain restart
}
```

---

## Deployment

### Local Testing

```bash
# Build release binary
cargo build --release -p sultan-core

# Run validator node
/tmp/cargo-target/release/sultan-node \
  --validator \
  --enable-sharding \
  --shard-count 8 \
  --rpc-addr 127.0.0.1:8080 \
  --block-time 2
```

### Production Deployment

See `DEPLOY_TO_HETZNER.sh` for automated production deployment.

```bash
# Deploy to production server
./DEPLOY_TO_HETZNER.sh
```

---

## Common Build Issues

### Issue: "Binary not found after build"

**Solution**: Check `/tmp/cargo-target/release/` not `./target/`

```bash
ls -la /tmp/cargo-target/release/
```

### Issue: "Disk space error during build"

**Solution**: Clean old build artifacts

```bash
cargo clean
# Or manually: rm -rf /tmp/cargo-target
```

### Issue: "Linker errors on macOS"

**Solution**: Install Xcode Command Line Tools

```bash
xcode-select --install
```

### Issue: "Out of memory during release build"

**Solution**: Reduce parallel compilation

```bash
cargo build --release -p sultan-core -j 2
```

---

## Benchmarking

### Transaction Throughput

```bash
# Build with benchmarks
cargo bench -p sultan-core

# Or use included script
./BENCHMARK_MILLION_TPS.sh
```

### Memory Profiling

```bash
# Using valgrind
cargo build --release -p sultan-core
valgrind --tool=massif /tmp/cargo-target/release/sultan-node
```

---

## Continuous Integration

### GitHub Actions (Recommended)

```yaml
# .github/workflows/build.yml
name: Build and Test

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --release -p sultan-core
      - run: cargo test --all --all-features
      - run: cargo audit
```

---

## Audit Checklist

For third-party security auditors:

- [ ] Clone repository and verify commit hash
- [ ] Review `Cargo.toml` dependencies for known vulnerabilities
- [ ] Run `cargo audit` for CVE scanning
- [ ] Build release binary: `cargo build --release -p sultan-core`
- [ ] Verify binary at `/tmp/cargo-target/release/sultan-node`
- [ ] Run full test suite: `cargo test --all --all-features`
- [ ] Review consensus logic in `consensus.rs`
- [ ] Review economic security in `staking.rs` and `governance.rs`
- [ ] Review bridge security in `bridge_integration.rs` and `bridge_fees.rs`
- [ ] Check for unsafe code: `rg "unsafe" sultan-core/src/`
- [ ] Review cryptographic implementations (Ed25519, SHA256)
- [ ] Test hot-upgrade mechanism with feature flags
- [ ] Verify no hardcoded private keys or secrets
- [ ] Check integer overflow protection
- [ ] Review RPC input validation
- [ ] Test Byzantine fault tolerance scenarios

---

## Support

- **Documentation**: See `ARCHITECTURE.md`, `DEPLOYMENT_PLAN.md`
- **Issues**: GitHub Issues
- **Security**: See `SECURITY.md` for responsible disclosure

---

## License

See LICENSE file in repository root.

---

**Last Updated**: December 6, 2025  
**Build System Version**: Cargo 1.75.0+  
**Target Binary**: sultan-node v1.0.0
