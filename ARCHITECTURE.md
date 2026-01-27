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
| `staking.rs` | ~1,640 | Validator registration, delegation, rewards (with reward_wallet), uptime tracking (blocks_signed/blocks_missed), slashing with auto-persist (21 tests) |
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
| `p2p.rs` | 1,200+ | libp2p P2P networking (GossipSub, Kademlia, DoS protection, Ed25519 sig verify, persistent node keys, validator discovery, height-based sync, 16 tests) |
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

## Network Parameters (Verified January 27, 2026)

| Parameter | Value | Verified |
|-----------|-------|----------|
| Block Time | ~0.9-2.0 seconds | ✅ 0.9s measured at low load |
| TPS Capacity | 64,000 (launch) → 32M (max with 8,000 shards) | ✅ 16 shards × 4,000 TPS |
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
| Genesis Wallet | `sultan15g5nwnlemn7zt6rtl7ch46ssvx2ym2v2umm07g` (receives validator APY) |
| Binary Version | v0.2.2 |
| Binary SHA256 | `d5cc6e636059fe9bcc59ae608f163abe7517191edfdc4b446e6c1c22d3b1de18` |
| Bootstrap Peer | `/ip4/206.189.224.142/tcp/26656/p2p/12D3KooWM9Pza4nMLHapDya6ghiMNL24RFU9VRg9krRbi5kLf5L7` |

---

## Industry Comparison

### vs Blockchain Networks

| Feature | Sultan L1 | Solana | Ethereum | Cosmos |
|---------|-----------|--------|----------|--------|
| **TPS** | 64K (32M max) | 65K (400-2K real) | 15-30 | 10K |
| **Fees** | Zero | ~$0.0002 | $2-50 | ~$0.01 |
| **Finality** | <2s | 13s | 15 min | 6s |
| **Sharding** | Native | ❌ | ❌ (abandoned) | ❌ |
| **Uptime** | 100% | 90%+ (outages) | 100% | 100% |

### vs Payment Networks

| Feature | Sultan L1 | Visa | PayPal |
|---------|-----------|------|--------|
| **Peak TPS** | 64K→32M | 65K | 793 |
| **Avg TPS** | ~0 (early) | ~1,700 | ~200 |
| **Fees** | Zero | 1.5-3.5% | 2.9%+$0.30 |
| **Decentralized** | ✅ | ❌ | ❌ |
| **Permissionless** | ✅ | ❌ | ❌ |

### Stress Testing

```bash
./scripts/stress_test.sh --tps 1000 --duration 60
```

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

### Validator Registration Architecture (v0.1.6+)

Sultan implements **enterprise-grade separation** between P2P discovery and consensus membership:

| Layer | Purpose | Determines Consensus? |
|-------|---------|----------------------|
| **P2P Discovery** | Find validators, exchange pubkeys | ❌ No |
| **On-Chain Registration** | Stake tokens, join validator set | ✅ Yes |

**Why This Matters:**
- All nodes derive validator set from **blockchain state** (identical everywhere)
- P2P announcements only register pubkeys for **signature verification**
- Prevents divergent validator sets that cause chain stalls
- Provides audit trail and economic security (stake required)

**Validator Registration Flow:**
```
1. New validator sets up node → P2P connects to network
2. Node discovers other validators via ValidatorAnnounce (pubkey + current height)
3. Validator submits on-chain registration:
   POST /staking/create_validator { address, stake_amount, commission_rate }
4. Blockchain processes registration → validator added to on-chain state
5. All nodes see new validator in next block → consensus includes them
```

**Key Principle:**
> P2P is for discovery. Blockchain is for consensus.

### Byzantine Fault Tolerance
- Tolerates 33% malicious/offline validators
- Automatic recovery when validators come back online
- Missed block counts reset after successful block production

### Block Timestamp Guarantee (v0.1.7+)
Sultan enforces **strictly increasing block timestamps** to prevent consensus failures:

| Feature | Implementation |
|---------|---------------|
| Timestamp Source | `SystemTime::now()` with monotonic guarantee |
| Minimum Increment | `max(current_time, prev_timestamp + 1)` |
| Validation | Reject blocks where `timestamp <= prev_timestamp` |

**Why This Matters:**
When multiple validators operate in rapid succession (e.g., at genesis or recovery), blocks could be created within the same second. Without this protection, identical timestamps cause block validation to fail with "Block timestamp must be greater than previous block."

**Implementation (sharded_blockchain_production.rs):**
```rust
let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
let timestamp = std::cmp::max(current_time, prev_timestamp + 1);
```

See [Validator Deadlock Postmortem](docs/VALIDATOR_DEADLOCK_POSTMORTEM.md) Issue #7 for details.

### Height-Based Validator Sync (v0.1.8+)
Sultan validators broadcast their current chain height in `ValidatorAnnounce` messages, enabling automatic sync detection:

| Feature | Implementation |
|---------|---------------|
| Height Broadcast | `ValidatorAnnounce` includes `current_height: u64` field |
| Peer Height Tracking | `update_peer_height()` called on announce receive |
| Sync Detection | Validators detect peers ahead and request sync |
| Auto-Recovery | Validators automatically catch up after downtime |

**Why This Matters:**
When a validator restarts or experiences network partition, it can immediately detect if other validators are ahead by examining incoming `ValidatorAnnounce` messages. This eliminates the previous failure mode where validators at different heights couldn't detect desync.

**Implementation (p2p.rs + main.rs):**
```rust
pub struct ValidatorAnnounce {
    pub address: String,
    pub stake: u64,
    pub peer_id: String,
    pub pubkey: [u8; 32],
    pub signature: Vec<u8>,
    pub current_height: u64,  // NEW: Chain height for sync detection
}
```

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

### Production Infrastructure
| Service | URL | Hosting |
|---------|-----|---------|
| **RPC** | https://rpc.sltn.io | NYC Validator (DigitalOcean) |
| **Wallet** | https://wallet.sltn.io | Replit (Wollnbergen/PWA repo) |
| **Backup Wallet** | https://rpc.sltn.io/wallet/ | NYC Validator |

### Wallet Deployment Workflow
The Sultan Wallet PWA is developed in `wallet-extension/` but deployed via a separate repo:

```
wallet-extension/ (0xv7)  →  Wollnbergen/PWA repo  →  Replit  →  wallet.sltn.io
```

**To deploy wallet changes:**
```bash
# 1. Sync changes to PWA repo and push
./scripts/deploy_wallet.sh --push

# 2. On Replit (wallet.sltn.io project):
git pull origin main
npm install
npm run build
```

### Validators
Dynamic validator set - anyone can join with 10,000 SLTN stake.

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

## Production Validator Set (v0.2.2)

Sultan mainnet operates with **6 globally distributed validators**, each with equal voting power:

| Validator | Address | Location | Voting Power |
|-----------|---------|----------|-------------|
| NYC | `sultan1valnyc...vnyc01` | New York, USA | 16.67% |
| SFO | `sultan1valsfo...vsfo02` | San Francisco, USA | 16.67% |
| FRA | `sultan1valfra...vfra03` | Frankfurt, EU | 16.67% |
| AMS | `sultan1valams...vams04` | Amsterdam, EU | 16.67% |
| SGP | `sultan1valsgp...vsgp05` | Singapore, APAC | 16.67% |
| LON | `sultan1vallon...vlon06` | London, EU | 16.67% |

**Staking System Features:**
- Genesis validator auto-registration via `--genesis-validators` CLI flag
- Real-time uptime tracking (`blocks_signed`, `blocks_missed`, `uptime_percent`)
- Persistent staking state in RocksDB (`staking:state` key)
- `--reset-staking` flag for state recovery/rebuild
- `total_blocks_missed` lifetime counter for validator performance history

---

*Last updated: January 27, 2026 - v0.2.2 with staking system improvements, 6 validators live with equal voting power*

---

## Network Testing

Run the network test suite to verify all capabilities:

```bash
./scripts/network_test.sh
```

**Latest Test Results (January 27, 2026):**
- ✅ All 6 validators in consensus (0 block spread)
- ✅ Block production: 11 blocks in 10s (~0.9s/block)
- ✅ 16 active shards, all healthy
- ✅ TPS capacity: 64,000 (max: 32M with 8,000 shards)
- ✅ All validators have reward_wallet configured
- ✅ API latency <310ms all endpoints
- ✅ **17/17 tests passing**
