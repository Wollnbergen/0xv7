# ⚡ SULTAN CHAIN - Zero Gas L1 Blockchain

## 🌐 Network Status: LIVE

**Mainnet RPC:** https://rpc.sltn.io  
**Block Time:** ~2 seconds  
**Validators:** 4 active (join with 10,000 SLTN stake)  
**Shards:** 16 active (expandable to 8,000)

---

## ✅ What Sultan Is

Sultan is a **native Rust L1 blockchain** built from scratch - NOT a fork of Cosmos SDK, Substrate, or any other framework.

### Key Features
- **Zero Gas Fees** - Transactions are free, subsidized by 4% annual inflation
- **Native Token Factory** - Create CW20/CW721 tokens without smart contracts
- **Native DEX** - Built-in AMM for token swaps at protocol level
- **Cross-Chain Bridges** - BTC, ETH, SOL, TON support
- **10,000 SLTN Minimum Stake** - Democratic validator participation
- **~13.33% APY** - Validator staking rewards
- **21-Day Unbonding** - Secure stake withdrawal period
- **On-Chain Governance** - Proposal and voting system

---

## 📁 Project Structure

\`\`\`
sultan-core/src/           # Production Rust blockchain
├── main.rs                # Node binary + RPC server (Warp)
├── blockchain.rs          # Block/Transaction core types
├── sharding_production.rs # ShardingCoordinator (production)
├── sharded_blockchain_production.rs  # SultanBlockchain
├── consensus.rs           # PoS consensus & block validation
├── staking.rs             # Validators, delegation, rewards, slashing
├── governance.rs          # On-chain proposals & voting
├── storage.rs             # RocksDB persistence layer
├── p2p.rs                 # libp2p networking
├── token_factory.rs       # Native CW20/CW721 token creation
├── native_dex.rs          # Built-in AMM/swap
├── bridge_integration.rs  # Cross-chain bridge coordination
├── bridge_fees.rs         # Bridge fee calculations
└── block_sync.rs          # Block synchronization

bridges/                   # Cross-chain bridge implementations
├── bitcoin/
├── ethereum/
├── solana/
└── ton/

scripts/                   # Deployment & maintenance tools
docs/                      # Technical documentation
_archive/                  # Legacy/experimental (NOT production)
\`\`\`

---

## 🚀 Quick Start

### Run the Node
\`\`\`bash
cd sultan-core
cargo build --release
./target/release/sultan-node --rpc-port 3030
\`\`\`

### Run Tests
\`\`\`bash
# All tests
cargo test --workspace

# Staking tests only (17 tests)
cargo test -p sultan-core staking
\`\`\`

### Check Node Status
\`\`\`bash
curl https://rpc.sltn.io/status
\`\`\`

---

## 📊 Technical Specs

| Metric | Value |
|--------|-------|
| **Consensus** | Proof of Stake (BFT) |
| **Block Time** | ~2 seconds |
| **Genesis Supply** | 500,000,000 SLTN |
| **Inflation** | 4% fixed annually |
| **Decimals** | 9 (nano units) |
| **Minimum Stake** | 10,000 SLTN |
| **Unbonding Period** | 21 days |
| **Validator APY** | ~13.33% |
| **Downtime Slash** | 0.1% (after 100 missed blocks) |
| **Double-Sign Slash** | 5% (immediate jail) |
| **Gas Fees** | Zero (subsidized by inflation) |
| **Shards** | 16 active (max 8,000) |
| **RPC Rate Limit** | 100 req/10 sec per IP |

---

## 🔐 Security Features

- **Rate Limiting** - DDoS protection on RPC endpoints
- **Signature Verification** - Ed25519 for all transactions
- **Quantum Resistance** - Dilithium post-quantum signatures (optional)
- **Slashing** - Economic penalties for validator misbehavior
- **21-Day Unbonding** - Prevents quick exit attacks

---

## 📚 Documentation

- [Technical Whitepaper](SULTAN_L1_TECHNICAL_WHITEPAPER.md)
- [Technical Deep Dive](docs/SULTAN_TECHNICAL_DEEP_DIVE.md)
- [Validator Guide](VALIDATOR_GUIDE.md)
- [Architecture](ARCHITECTURE.md)
- [API Reference](docs/API_REFERENCE.md)
- [Code Review Context](docs/CODE_REVIEW_CONTEXT.md)

---

## ⚠️ What's NOT Production

The \`_archive/\` folder contains legacy/experimental code including:
- Old Cosmos SDK experiments
- CosmWasm contract templates (reference only)
- Previous architecture iterations

**Production code is only in \`sultan-core/\`**

---

## 🔗 Links

- **RPC Endpoint:** https://rpc.sltn.io
- **GitHub:** https://github.com/Wollnbergen/0xv7

---

*Sultan L1 - The People's Blockchain*

*Last updated: December 29, 2025*
