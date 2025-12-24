# Sultan Chain Architecture

## Overview

Sultan is a **native Rust L1 blockchain** - NOT based on Cosmos SDK, Tendermint, Substrate, or any external framework. Every component is custom-built for Sultan's specific requirements.

---

## Production Architecture

### Core Modules (sultan-core/src/)

| Module | Purpose |
|--------|---------|
| `main.rs` | Node binary, RPC server, P2P networking |
| `lib.rs` | Library exports for all modules |
| `consensus.rs` | Proof of Stake consensus engine |
| `staking.rs` | Validator registration, delegation, rewards |
| `governance.rs` | On-chain proposals and voting |
| `token_factory.rs` | Native token creation (no smart contracts) |
| `native_dex.rs` | Built-in AMM for token swaps |
| `sharding.rs` | Horizontal scaling (16 shards at launch, expandable to 8,000) |
| `sharding_production.rs` | Production shard routing |
| `sharded_blockchain.rs` | Multi-shard block production |
| `bridge_integration.rs` | Cross-chain bridge coordinator |
| `bridge_fees.rs` | Bridge fee calculation |
| `economics.rs` | Inflation, rewards, APY calculations |
| `transaction_validator.rs` | Transaction validation (zero-fee) |
| `storage.rs` | Persistent state storage |
| `types.rs` | Core data structures |
| `config.rs` | Node configuration |
| `quantum.rs` | Post-quantum cryptography (Dilithium) |
| `mev_protection.rs` | MEV resistance |

### Cross-Chain Bridges (bridges/)

| Bridge | Status |
|--------|--------|
| Bitcoin | Active |
| Ethereum | Active |
| Solana | Active |
| TON | Active |

---

## Key Design Decisions

### Why Native Rust (Not Cosmos SDK)?

| Challenge | Our Solution |
|-----------|--------------|
| Zero gas fees | Built into protocol - no "AnteHandler" workaround needed |
| Native sharding | Custom implementation, not bolted on |
| Token factory | Protocol-level, no CosmWasm contracts |
| Full control | No framework constraints or upgrade delays |

### Token System

Sultan uses a **Native Token Factory** - tokens are created directly in the protocol without smart contracts:

```rust
// sultan-core/src/token_factory.rs
pub async fn create_token(
    &self,
    creator: &str,
    name: String,
    symbol: String,
    decimals: u8,
    total_supply: u128,
    ...
) -> Result<String>
```

**NOT using:** CosmWasm, CW20, CW721, or any smart contract system.

### DEX System

Sultan has a **Native DEX** built into the protocol:

```rust
// sultan-core/src/native_dex.rs
pub async fn swap(
    &self,
    user: &str,
    input_denom: &str,
    output_denom: &str,
    input_amount: u128,
    min_output: u128,
) -> Result<SwapResult>
```

---

## Network Parameters

| Parameter | Value |
|-----------|-------|
| Block Time | ~2 seconds |
| Minimum Validator Stake | 10,000 SLTN |
| Validator APY | ~13.33% |
| Gas Fees | Zero (subsidized by inflation) |
| Shards | 16 at launch (expandable to 8000) |
| Consensus | Proof of Stake |

---

## What's NOT Production

The `_archive/` folder contains legacy/experimental code:
- `contracts-cosmwasm-legacy/` - Old CosmWasm experiments (not used)
- `sultan-cosmos-*/` - Cosmos SDK experiments (abandoned)
- Various other prototypes

**All production code is in `sultan-core/`**

---

## Deployment

### Production
- **RPC:** https://rpc.sltn.io
- **Wallet:** https://wallet.sltn.io
- **Validators:** Dynamic (anyone can join with 10,000 SLTN)

### Development
```bash
# Build
cd sultan-core
cargo build --release

# Run node
./target/release/sultan-node

# Run tests
cargo test --workspace
```

---

## Documentation

- [Technical Whitepaper](SULTAN_L1_TECHNICAL_WHITEPAPER.md) - Full technical specification
- [Technical Deep Dive](docs/SULTAN_TECHNICAL_DEEP_DIVE.md) - Investor-focused explanation
- [Validator Guide](VALIDATOR_GUIDE.md) - How to run a validator
- [API Reference](docs/API_REFERENCE.md) - RPC endpoints
