# Sultan L1 - Code Review Context

Use this summary when submitting files for external code review.

---

## What is Sultan?

Sultan is a **native Rust L1 blockchain** built from scratch - NOT a fork of Cosmos SDK, Substrate, or any other framework. It's a zero-gas, sharded Proof-of-Stake chain.

## Core Technical Parameters

| Parameter | Value | Notes |
|-----------|-------|-------|
| **Genesis Supply** | 500,000,000 SLTN | Initial supply at launch |
| **Inflation** | 4% fixed annually | Funds validator rewards (zero-gas model) |
| **Decimals** | 9 (nano units) | 1 SLTN = 1,000,000,000 nSLTN |
| **Block Time** | ~2 seconds | Target |
| **Minimum Stake** | 10,000 SLTN | To become validator |
| **Unbonding Period** | 21 days | 1,814,400 seconds |
| **Downtime Slash** | 0.1% | After 100 missed blocks |
| **Double-Sign Slash** | 5% | Immediate jail |
| **Base APY** | ~13.33% | From 4% inflation ÷ 30% staked |
| **Gas Fees** | Zero | Subsidized by inflation |

## Architecture Overview

```
sultan-core/src/
├── main.rs                         # Node entry point, RPC server (Warp)
├── blockchain.rs                   # Block/Transaction structs
├── sharding_production.rs          # ShardingCoordinator (2,244 lines)
├── sharded_blockchain_production.rs # SultanBlockchain (1,250 lines)
├── consensus.rs                    # Block validation
├── staking.rs                      # Validators, delegation, rewards
├── governance.rs                   # Proposals, voting
├── storage.rs                      # RocksDB persistence
├── p2p.rs                          # libp2p networking
├── token_factory.rs                # Native token creation (fungible & NFT)
├── native_dex.rs                   # Built-in AMM
└── bridge_integration.rs           # Cross-chain (BTC, ETH, SOL, TON)
```

## Key Design Decisions

1. **Zero Gas** - Transactions are free; inflation subsidizes validators
2. **Native Token Factory** - Create tokens without smart contracts
3. **Built-in DEX** - AMM at protocol level, not contract level
4. **Sharded** - 16 shards at launch, expandable to 8,000
5. **RocksDB** - Persistent storage for blocks, wallets, staking state
6. **Rate Limiting** - 100 requests/10 seconds per IP on RPC
7. **Transaction Memo** - Optional user notes preserved in history
8. **History Pruning** - MAX_HISTORY_PER_ADDRESS (10,000) prevents memory bloat
9. **Deterministic Ordering** - Mempool sorted by timestamp/from/nonce for consensus stability

## Current State

- **Network:** Live at `rpc.sltn.io`
- **Validators:** 4 active
- **Tests:** 141 passing (96 lib + 45 integration/stress)

## When Reviewing, Check For

1. **Security** - Input validation, overflow protection, access control
2. **Consistency** - Does code match the parameters above?
3. **Error Handling** - Proper Result/Option handling, no unwrap() in production paths
4. **Performance** - Lock contention, async patterns, memory usage
5. **Dead Code** - Unused functions, unreachable branches

---

*Last updated: December 29, 2025*
