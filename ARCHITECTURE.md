# Sultan Chain Architecture

## Overview

Sultan is a **native Rust L1 blockchain** with every component custom-built for Sultan's specific requirements.

---

## Production Architecture

### Core Modules (sultan-core/src/)

| Module | Lines | Purpose |
|--------|-------|---------|
| `main.rs` | 4,736 | Node binary, RPC server (30+ endpoints), P2P networking, keygen CLI |
| `lib.rs` | ~100 | Library exports for all modules |
| `consensus.rs` | 1,351 | Proof of Stake consensus engine (26 tests, Ed25519, enterprise failover) |
| `staking.rs` | ~1,540 | Validator registration, delegation, rewards, slashing with auto-persist (21 tests) |
| `governance.rs` | ~1,900 | On-chain proposals, voting, slashing proposals, encrypted storage (21 tests) |
| `storage.rs` | ~1,120 | Persistent state with AES-256-GCM encryption, HKDF key derivation (14 tests) |
| `token_factory.rs` | ~880 | Native token creation with Ed25519 signatures (14 tests) |
| `native_dex.rs` | ~970 | Built-in AMM with Ed25519 signatures (13 tests) |
| `bridge_integration.rs` | ~1,965 | Cross-chain bridge with real SPV/ZK/gRPC/BOC proof verification, rate limiting, multi-sig (39 tests) |
| `bridge_fees.rs` | ~880 | Zero-fee bridge with rate limiting, treasury governance, async oracle (30 tests) |
| `sharding_production.rs` | 2,244 | **PRODUCTION** shard routing with Ed25519, 2PC, WAL recovery |
| `sharded_blockchain_production.rs` | 1,342 | **PRODUCTION** multi-shard coordinator |
| `economics.rs` | 100 | Inflation (fixed 4%), rewards, APY calculations |
| `transaction_validator.rs` | 782 | Transaction validation (18 tests, typed errors, Ed25519 sig verify) |
| `blockchain.rs` | 374 | Block/Transaction structures (with memo) |
| `p2p.rs` | 1,200+ | libp2p P2P networking (GossipSub, Kademlia, DoS protection, Ed25519 sig verify, persistent node keys, validator discovery, 16 tests) |
| `block_sync.rs` | 1,174 | Byzantine-tolerant block sync (voter verification, signature validation, 31 tests) |
| `mev_protection.rs` | ~100 | MEV resistance |
| `sharding.rs` | 362 | ⚠️ LEGACY (deprecated, tests only) |
| `sharded_blockchain.rs` | 179 | ⚠️ LEGACY (deprecated, tests only) |

**Total:** ~22,000+ lines of production Rust code (283+ tests passing)

### Cross-Chain Bridges (bridges/)

| Bridge | Status |
|--------|--------|
| Bitcoin | Active |
| Ethereum | Active |
| Solana | Active |
| TON | Active |

---

## Key Design Decisions

### Why Native Rust?

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
| TPS Capacity | 64,000 (launch) → 64M (max with 8,000 shards) |
| Minimum Validator Stake | 10,000 SLTN |
| Validator APY | ~13.33% (4% inflation ÷ 30% staked) |
| Gas Fees | Zero (subsidized by 4% fixed inflation) |
| Shards | 16 at launch (expandable to 8,000) |
| Consensus | Proof of Stake |
| Max History/Address | 10,000 entries (pruned) |
| Mempool Ordering | Deterministic (timestamp/from/nonce) |
| Signature Verification | Ed25519 STRICT mode |
| Tests | 283+ passing (lib tests) |
| DEX Swap Fee | 0.3% total (0.2% to LP reserves, 0.1% to protocol) |
| Protocol Fee Address | `sultan15g5nwnlemn7zt6rtl7ch46ssvx2ym2v2umm07g` (genesis treasury) |
| Binary Version | v0.1.4 |
| Binary SHA256 | `bd934d97e464ce083da300a7a23f838791db9869aed859a7f9e51a95c9ae01ff` |
| Bootstrap Peer | `/ip4/206.189.224.142/tcp/26656/p2p/12D3KooWM9Pza4nMLHapDya6ghiMNL24RFU9VRg9krRbi5kLf5L7` |

---

## Enterprise Consensus Features

### PoS Proposer Selection with Failover (v0.1.5)
Sultan uses an enterprise-grade Proof-of-Stake consensus with automatic failover:

| Feature | Implementation |
|---------|---------------|
| Proposer Selection | Height-based, stake-weighted deterministic selection |
| Fallback Threshold | 5 consecutive missed blocks before fallback |
| Fallback Positions | Top 3 stake-weighted validators can act as fallbacks |
| Missed Block Tracking | Height-based deduplication prevents double-counting |
| Slashing | 100 consecutive misses triggers stake slash |
| Memory Cleanup | Automatic cleanup of old missed block records (1000 block window) |

**Key Constants (consensus.rs):**
```rust
pub const MAX_MISSED_BLOCKS_BEFORE_SLASH: u64 = 100;  // Slash threshold
pub const FALLBACK_THRESHOLD_MISSED_BLOCKS: u64 = 5;  // When fallback kicks in
pub const MAX_FALLBACK_POSITIONS: usize = 3;          // Only top 3 can fallback
pub const MISSED_BLOCK_TRACKING_WINDOW: u64 = 1000;   // Memory cleanup window
```

### Proposer Failover Flow
```
Primary proposer offline for 5 consecutive blocks
    ↓
Fallback #1 (highest remaining stake) takes over
    ↓
If #1 also offline, Fallback #2 tries
    ↓
If #2 also offline, Fallback #3 tries
    ↓
Chain continues producing blocks (no single point of failure)
```

### Byzantine Fault Tolerance
- Tolerates 33% malicious/offline validators
- Automatic recovery when validators come back online
- Missed block counts reset after successful block production

---

## Security Features

### Node Storage Encryption (AES-256-GCM)
Sultan uses **production-grade authenticated encryption** for sensitive data:

| Feature | Implementation |
|---------|---------------|
| Algorithm | AES-256-GCM (NIST approved) |
| Key Derivation | HKDF-SHA256 (RFC 5869) |
| Nonce | 12-byte random per encryption |
| Authentication | Built-in integrity verification |
| Multi-tenant | Custom salt support for isolation |

### Wallet Security (PWA) - v1.0.0
The Sultan Wallet PWA has undergone comprehensive security review (January 2026):

| Feature | Implementation |
|---------|---------------|
| Storage Encryption | AES-256-GCM with PBKDF2 (600K iterations) |
| Memory Protection | SecureString (XOR encrypted) for PIN and mnemonic |
| Signature Scheme | Ed25519 with SHA-256 message hashing |
| API Security | 30s timeouts, Zod validation, retry with backoff |
| BIP39 Passphrase | Optional 25th word for plausible deniability |
| High-Value Protection | Confirmation for transactions >1000 SLTN |
| TOTP 2FA | With 8 backup codes |
| Test Coverage | 219 tests passing |

### Browser Extension Security - v1.0.0
The Sultan Wallet Chrome extension (Manifest V3) provides dApp integration:

| Feature | Implementation |
|---------|---------------|
| CSP | `script-src 'self'; object-src 'none'; frame-ancestors 'none'` |
| Rate Limiting | 60 req/min (background), 100 msg/min (content script) |
| Audit Logging | 16 security event types to chrome.storage.local |
| Phishing Detection | Pattern matching + homograph attack detection |
| Provider Security | Object.freeze, non-writable window.sultan |
| Crypto | Same as PWA (PBKDF2 600K, AES-256-GCM, Ed25519) |
| dApp API | `window.sultan.connect()`, `signAndSendTransaction()` |

See [Browser Extension Security Audit](docs/BROWSER_EXTENSION_SECURITY_AUDIT.md) for full details.

### Governance Security
| Protection | Mechanism |
|------------|-----------|
| Flash Stake Prevention | Voting power snapshot at proposal creation |
| Anti-Spam | 1,000 SLTN deposit + rate limiting |
| Slashing Proposals | Community can slash misbehaving validators |
| Emergency Pause | 67% validator multisig for critical actions |
| Encrypted Storage | Sensitive proposals can be encrypted at rest |

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

- [Technical Whitepaper](SULTAN_L1_TECHNICAL_WHITEPAPER.md) - Full technical specification (v3.5)
- [Technical Deep Dive](docs/SULTAN_TECHNICAL_DEEP_DIVE.md) - Investor-focused explanation (v3.3)
- [Code Review Context](docs/CODE_REVIEW_CONTEXT.md) - Context for external auditors
- [Validator Guide](VALIDATOR_GUIDE.md) - How to run a validator
- [API Reference](docs/API_REFERENCE.md) - RPC endpoints

---

*Last updated: January 15, 2026 - v0.1.5 with enterprise PoS failover and persistent node keys*
