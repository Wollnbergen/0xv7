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
├── storage.rs                      # RocksDB persistence + AES-256-GCM encryption
├── p2p.rs                          # libp2p networking (GossipSub, Kademlia, DoS, Ed25519)
├── block_sync.rs                   # Byzantine-tolerant sync (voter verify, sig validation)
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
- **Tests:** 274 passing (lib tests + integration/stress)
- **Code Review:** Phase 5 Complete (10/10 ratings on all modules)

## Phase 4 Review Summary (p2p.rs & block_sync.rs)

**p2p.rs (1,025 lines, 16 tests) - 10/10 Enterprise-Grade:**
- libp2p with GossipSub (strict validation, max 1MB, max_ihave 5000, max_messages_per_rpc 100)
- Kademlia DHT for peer discovery
- DoS protection: rate limiting (1000/min), peer banning (600s), message size caps
- Ed25519 signatures on all message types (proposals, votes, announcements)
- Validator pubkey registry with minimum stake verification (10T SULTAN)
- Signature verification in event loop (rejects invalid proposals/announcements)

**block_sync.rs (1,174 lines, 31 tests) - 10/10 Enterprise-Grade:**
- Byzantine-tolerant block synchronization
- Voter verification against consensus validators
- Signature validation with VoteRejection::InvalidSignature
- SyncConfig with DoS limits (max_pending 100, max_seen 10K)
- Proposer verification in validate_block_full()
- Statistics tracking (blocks_synced, votes_recorded, votes_rejected)
- Sync request/response helpers for P2P integration

## Phase 5 Review Summary (Bridge & DeFi Modules)

**bridge_integration.rs (~1,600 lines, 32 tests) - 10/10 Enterprise-Grade:**
- Real SPV proof parsing with merkle root verification
- ZK-SNARK structure validation (Groth16, 256+ bytes)
- Solana gRPC finality with status codes (0=failed, 1=confirmed, 2=pending)
- TON BOC magic byte validation (0xb5ee9c72, 0xb5ee9c73)
- Wrapped token minting/burning for sBTC, sETH, sSOL, sTON

**bridge_fees.rs (~680 lines, 23 tests) - 10/10 Enterprise-Grade:**
- Zero Sultan-side fees (external chain fees only)
- Async oracle integration (mempool.space, etherscan, solana RPC, toncenter)
- USD conversion via CoinGecko API
- FeeBreakdownWithOracle combined response

**token_factory.rs (~880 lines, 14 tests) - 10/10 Enterprise-Grade:**
- Ed25519 signatures on all public APIs (*_with_signature methods)
- Internal methods restricted to pub(crate) for tests/native_dex
- 1000 SLTN creation fee, 1M minimum supply

**native_dex.rs (~970 lines, 13 tests) - 10/10 Enterprise-Grade:**
- Ed25519 signatures on swap, create_pair, add/remove_liquidity
- Constant product AMM with 0.3% fee (30 basis points)
- DexStatistics tracking (total_pools, total_volume, total_liquidity)

## When Reviewing, Check For

1. **Security** - Input validation, overflow protection, access control
2. **Consistency** - Does code match the parameters above?
3. **Error Handling** - Proper Result/Option handling, no unwrap() in production paths
4. **Performance** - Lock contention, async patterns, memory usage
5. **Dead Code** - Unused functions, unreachable branches

---

*Last updated: December 30, 2025 - Code Review Phase 5 Complete (274 tests)*
