# ⚡ SULTAN CHAIN - Zero Gas L1 Blockchain

## 🌐 Network Status: LIVE

**Mainnet RPC:** https://rpc.sltn.io  
**Block Height:** 45,000+  
**Validators:** 9 active  
**Block Time:** ~2 seconds

---

## ✅ What Sultan Is

Sultan is a **native Rust L1 blockchain** built from scratch - NOT a fork of Cosmos SDK, Substrate, or any other framework.

### Key Features
- **Zero Gas Fees** - Transactions are free, subsidized by protocol inflation
- **Native Token Factory** - Create tokens without smart contracts
- **Native DEX** - Built-in AMM for token swaps
- **Cross-Chain Bridges** - BTC, ETH, SOL, TON support
- **10,000 SLTN Minimum Stake** - Democratic validator participation
- **~13.33% APY** - Validator staking rewards

---

## 📁 Project Structure

```
/workspaces/0xv7/
├── sultan-core/         # Production Rust blockchain (22 modules)
│   └── src/
│       ├── lib.rs              # Core exports
│       ├── main.rs             # Node binary (sultan-node)
│       ├── consensus.rs        # PoS consensus engine
│       ├── staking.rs          # Validator staking
│       ├── governance.rs       # On-chain governance
│       ├── token_factory.rs    # Native token creation
│       ├── native_dex.rs       # Built-in AMM
│       ├── sharding.rs         # Horizontal scaling
│       ├── bridge_integration.rs  # Cross-chain bridges
│       └── ...
├── bridges/             # Cross-chain bridge implementations
│   ├── bitcoin/
│   ├── ethereum/
│   ├── solana/
│   └── ton/
├── api/                 # RPC server
├── scripts/             # Deployment & maintenance tools
├── docs/                # Technical documentation
└── _archive/            # Legacy/experimental code (not production)
```

---

## 🚀 Quick Start

### Run the Node
```bash
cd sultan-core
cargo build --release
./target/release/sultan-node
```

### Run Tests
```bash
cargo test --workspace
```

---

## 📊 Technical Specs

| Metric | Value |
|--------|-------|
| Consensus | Proof of Stake |
| Block Time | ~2 seconds |
| Minimum Stake | 10,000 SLTN |
| Validator APY | ~13.33% |
| Gas Fees | Zero (subsidized) |
| Shards | 8 (expandable) |

---

## 📚 Documentation

- [Technical Whitepaper](SULTAN_L1_TECHNICAL_WHITEPAPER.md)
- [Technical Deep Dive](docs/SULTAN_TECHNICAL_DEEP_DIVE.md)
- [Validator Guide](VALIDATOR_GUIDE.md)
- [Architecture](ARCHITECTURE.md)
- [API Reference](docs/API_REFERENCE.md)

---

## ⚠️ What's NOT Production

The `_archive/` folder contains legacy/experimental code including:
- Old Cosmos SDK experiments
- CosmWasm contract templates (not used)
- Previous architecture iterations

**Production code is only in `sultan-core/`**

---

## 🔗 Links

- **RPC Endpoint:** https://rpc.sltn.io
- **GitHub:** https://github.com/Wollnbergen/0xv7

---

*Sultan L1 - The People's Blockchain*
