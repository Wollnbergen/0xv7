# Sultan L1 Smart Contract & Cross-Chain DeFi Architecture

## Document Purpose

This is the **canonical technical reference** for Sultan's smart contract system and cross-chain DeFi architecture. We (the core engineering team) will build this ourselves. This document serves as our implementation guide.

**Lead Engineers:** Core Team (building in-house, not outsourcing)

---

## Part 1: Sultan Architecture Foundation

### What Sultan Is

Sultan is a **sovereign Layer 1 blockchain** built entirely in Rust from first principles. It is NOT:
- ❌ Based on Cosmos SDK
- ❌ Based on Tendermint/CometBFT
- ❌ Based on Substrate/Polkadot
- ❌ A fork of any existing blockchain

**Sultan is:**
- ✅ 100% custom Rust implementation
- ✅ Native sharding (8 shards live, scaling to 8,000)
- ✅ Zero gas fees by design
- ✅ 2-second block finality
- ✅ Ed25519 + Dilithium3 (post-quantum) cryptography
- ✅ libp2p networking with Kademlia DHT

### Production Network (Live Since December 6, 2025)

| Specification | Value |
|---------------|-------|
| Block Time | 2.00 seconds |
| Finality | Immediate (single-block) |
| Active Shards | 8 |
| Current TPS | 64,000 (base) |
| Maximum TPS | 64,000,000 (8,000 shards) |
| Validators | 15 (globally distributed) |
| RPC Endpoint | `https://rpc.sltn.io` |
| Transaction Fees | **$0 (zero-fee)** |

### Current Native Modules (Production Code)

These are **protocol-level features**, not smart contracts:

| Module | File | Lines | Purpose |
|--------|------|-------|---------|
| Token Factory | `sultan-core/src/token_factory.rs` | 354 | Create/mint/transfer custom tokens |
| Native DEX | `sultan-core/src/native_dex.rs` | 462 | AMM with constant product formula |
| Staking | `sultan-core/src/staking.rs` | 532 | Validator staking, rewards, slashing |
| Governance | `sultan-core/src/governance.rs` | 556 | On-chain proposals and voting |
| Sharding | `sultan-core/src/sharding.rs` | — | Parallel transaction processing |
| Consensus | `sultan-core/src/consensus.rs` | — | Custom PoS block production |
| P2P | `sultan-core/src/p2p.rs` | — | libp2p networking |
| Quantum | `sultan-core/src/quantum.rs` | — | Dilithium3 post-quantum signatures |

---

## Part 2: Cross-Chain Interoperability

### The Sultan Interop Model

Sultan has **native bridges** to four major blockchains:

```
                    ┌─────────────────────────────────────────┐
                    │            SULTAN L1                    │
                    │         (Zero Gas Fees)                 │
                    │                                         │
                    │  ┌─────────┐  ┌─────────┐  ┌─────────┐ │
                    │  │sBTC Pool│  │sETH Pool│  │sSOL Pool│ │
                    │  └────┬────┘  └────┬────┘  └────┬────┘ │
                    │       │            │            │       │
                    │  ┌────▼────────────▼────────────▼────┐ │
                    │  │          NATIVE DEX               │ │
                    │  │   (All swaps: 0 gas, 0.3% LP fee) │ │
                    │  └───────────────────────────────────┘ │
                    └──────────┬──────────┬──────────┬───────┘
                               │          │          │
              ┌────────────────┼──────────┼──────────┼────────────────┐
              │                │          │          │                │
      ┌───────▼───────┐ ┌──────▼──────┐ ┌─▼────────┐ ┌▼──────────────┐
      │   BITCOIN     │ │  ETHEREUM   │ │  SOLANA  │ │     TON       │
      │   (HTLC)      │ │(Light Client)│ │  (gRPC)  │ │(Smart Contract)│
      │               │ │             │ │          │ │               │
      │ BTC → sBTC    │ │ ETH → sETH  │ │SOL→sSOL  │ │ TON → sTON    │
      │ Sultan: $0    │ │ Sultan: $0  │ │Sultan: $0│ │ Sultan: $0    │
      │ BTC fee: ~$2  │ │ ETH fee:$5+ │ │SOL:$0.001│ │ TON: ~$0.01   │
      └───────────────┘ └─────────────┘ └──────────┘ └───────────────┘
```

### Bridge Implementations

| Chain | Bridge File | Protocol | Sultan Fee | External Fee |
|-------|-------------|----------|------------|--------------|
| Bitcoin | `bridges/bitcoin/btc_bridge.rs` | HTLC + SPV proofs | **$0** | ~$2 (BTC network) |
| Ethereum | `bridges/ethereum/eth_bridge.sol` | Light client + ZK | **$0** | $5-50 (gas) |
| Solana | `bridges/solana/sol_bridge.rs` | gRPC streaming | **$0** | ~$0.001 |
| TON | `bridges/ton/ton_bridge.fc` | FunC smart contract | **$0** | ~$0.01 |

### Wrapped Asset Model

When users bridge assets to Sultan:

1. **Lock** — Original asset locked on source chain
2. **Mint** — Wrapped token minted on Sultan (sBTC, sETH, sSOL, sTON)
3. **Use** — Wrapped tokens usable in Sultan DeFi (zero fees!)
4. **Burn** — When bridging back, Sultan token burned
5. **Unlock** — Original asset released on source chain

**Example: Bitcoin User DeFi Journey**

```
User has 1 BTC on Bitcoin network
    │
    ├─ Pays ~$2 BTC network fee to lock in bridge contract
    │
    ▼
User receives 1 sBTC on Sultan (instant, 2-sec finality)
    │
    ├─ Swap sBTC → sETH on Native DEX (0.3% LP fee, $0 gas)
    ├─ Swap sETH → SLTN (0.3% LP fee, $0 gas)
    ├─ Stake SLTN for 13.33% APY ($0 gas)
    ├─ Swap SLTN → sSOL (0.3% LP fee, $0 gas)
    │
    ▼
User bridges sSOL → SOL (Sultan: $0, Solana: $0.001)
```

**Total Sultan fees: $0.00**

### How Zero-Fee Cross-Chain DeFi Works

The key insight: **Sultan subsidizes compute through inflation, not transaction fees.**

| Blockchain | Fee Model | Sustainability |
|------------|-----------|----------------|
| Ethereum | Users pay gas per operation | Gas revenue to validators |
| Solana | Users pay per compute unit | Fee revenue to validators |
| **Sultan** | Users pay $0 | 4% annual inflation funds validators |

**Sultan Validator Economics:**
- 4% annual inflation → distributed to stakers
- At 30% staked → 13.33% APY for validators
- Validators earn SLTN rewards, not transaction fees
- Users get unlimited zero-fee transactions

This is sustainable because:
1. Inflation is fixed at 4% (comparable to ETH's ~0.5-1%)
2. No compute auction → predictable validator income
3. Zero fees → maximum DeFi activity → more value locked → higher SLTN demand

---

## Part 3: What is WASM and Why We Need It

### WebAssembly (WASM) Explained

**WASM** (WebAssembly) is a binary instruction format designed for:
- Near-native execution speed
- Sandboxed, safe execution
- Language-agnostic (compile Rust, C, Go, AssemblyScript, etc.)
- Deterministic execution (critical for blockchains)

**For Sultan, WASM enables:**
- Third-party developers to deploy custom logic
- Complex DeFi protocols beyond our native modules
- Composable contracts that interact with each other
- Permissionless innovation without core protocol changes

### Do We Build Our Own WASM Runtime?

**No. We integrate an existing production-grade WASM runtime.**

Building a WASM runtime from scratch would be:
- 2-3 years of work
- Massive security risk (new code = new bugs)
- Reinventing the wheel

**What we do instead:**

We integrate one of these battle-tested Rust WASM runtimes:

| Runtime | Used By | Performance | Our Assessment |
|---------|---------|-------------|----------------|
| **wasmer** | Cosmonic, NEAR Protocol | 🔥 Very fast (LLVM/Cranelift JIT) | **Best choice** |
| **wasmtime** | Fastly, Firefox, Bytecode Alliance | 🔥 Very fast (Cranelift JIT) | Excellent alternative |
| **wasmi** | Substrate/Polkadot | 🐢 Slower (interpreter) | Too slow for us |

**Recommendation: wasmer**
- Pure Rust, no C dependencies
- Multiple compiler backends (Cranelift, LLVM, Singlepass)
- Production-proven at scale
- Excellent metering support (crucial for compute limits)

### WASM Integration Architecture

```rust
// How we'll add WASM to sultan-core/src/

// New file: wasm_runtime.rs
pub struct SultanWasmRuntime {
    engine: wasmer::Engine,
    store: wasmer::Store,
    contracts: HashMap<ContractAddress, wasmer::Module>,
}

impl SultanWasmRuntime {
    /// Execute a contract call
    pub fn execute(
        &mut self,
        contract: &ContractAddress,
        method: &str,
        args: &[u8],
        compute_limit: u64,  // Instead of gas
    ) -> Result<Vec<u8>, ContractError> {
        // 1. Load compiled module
        // 2. Create instance with host functions
        // 3. Execute with metering
        // 4. Return result or error
    }
}

// Host functions exposed to contracts
impl SultanWasmRuntime {
    /// Contracts can call Token Factory
    fn host_token_transfer(&self, denom: &str, from: &str, to: &str, amount: u128);
    
    /// Contracts can call Native DEX
    fn host_dex_swap(&self, pair: &str, token_in: &str, amount: u128, min_out: u128);
    
    /// Contracts can query chain state
    fn host_get_balance(&self, address: &str, denom: &str) -> u128;
    
    /// Contracts can query block info
    fn host_get_block_height(&self) -> u64;
    fn host_get_block_time(&self) -> u64;
}
```

### The Zero-Fee Compute Model

Since Sultan has no gas fees, we need an alternative to prevent spam:

**Option 1: Compute Credits (Recommended)**
```
- Every address gets N compute units per block (e.g., 1M units)
- Contract calls consume compute units based on WASM instruction count
- Units regenerate over time (like Solana's rate limiting)
- Staking more SLTN = more compute allocation
```

**Option 2: Stake-Weighted Access**
```
- Minimum stake required to deploy contracts (e.g., 10,000 SLTN)
- Execution allowed proportional to stake
- Slash stake if contract misbehaves
```

**Option 3: Hybrid**
```
- Free tier: N compute units per block for all users
- Premium tier: Stake SLTN for unlimited compute
- Enterprise tier: Run your own validator for priority execution
```

---

## Part 4: EVM Compatibility

### Why EVM Support?

- Millions of existing Solidity contracts
- Developer familiarity (most DeFi devs know Solidity)
- Easy migration from Ethereum/BSC/Polygon
- Tooling ecosystem (Hardhat, Foundry, Remix)

### EVM Implementation Approach

We integrate **revm** (Rust EVM):

| Library | Used By | Our Assessment |
|---------|---------|----------------|
| **revm** | Foundry, Reth, Helios | **Best choice** — pure Rust, actively maintained |
| evmodin | Research | Less mature |
| SputnikVM | OpenEthereum (deprecated) | Legacy |

**revm Integration:**

```rust
// sultan-core/src/evm_runtime.rs

use revm::{EVM, Database, AccountInfo};

pub struct SultanEvmRuntime {
    evm: EVM<SultanStateDB>,
}

/// Adapter to make Sultan state look like EVM state
struct SultanStateDB {
    // Maps EVM addresses (20 bytes) to Sultan addresses
    // Maps EVM storage slots to Sultan key-value store
}

impl Database for SultanStateDB {
    fn basic(&mut self, address: Address) -> AccountInfo {
        // Query Sultan state, return as EVM AccountInfo
    }
    
    fn storage(&mut self, address: Address, slot: U256) -> U256 {
        // Query Sultan storage, return as U256
    }
}
```

### EVM-to-Sultan Mapping

| EVM Concept | Sultan Equivalent |
|-------------|-------------------|
| `msg.sender` | Sultan transaction signer (Ed25519 → Keccak address) |
| `msg.value` | SLTN amount attached to call |
| `block.number` | Sultan block height |
| `block.timestamp` | Sultan block time |
| `gasleft()` | Compute units remaining |
| `address(this).balance` | SLTN balance query |

### eth_* JSON-RPC Compatibility

For Ethereum tooling (MetaMask, Ethers.js) to work:

```
Sultan RPC: https://evm.sltn.io (separate endpoint)

Supported methods:
- eth_chainId → Sultan EVM chain ID
- eth_getBalance → Query SLTN balance
- eth_getCode → Query contract bytecode  
- eth_call → Execute read-only contract call
- eth_sendRawTransaction → Submit signed EVM transaction
- eth_getTransactionReceipt → Get execution result
- eth_getLogs → Query contract events
```

---

## Part 5: Cross-Chain DeFi Product Architecture

### Product Suite

| Product | Technology | Status |
|---------|------------|--------|
| **Token Factory** | Native Rust module | ✅ Complete |
| **Native DEX** | Native Rust module (constant product AMM) | ✅ Complete |
| **Cross-Chain Swaps** | Native bridges + DEX | 📋 Q2 2026 |
| **Lending Protocol** | WASM contract | 📋 Q2 2026 |
| **Yield Aggregator** | WASM contract | 📋 Q3 2026 |
| **NFT Marketplace** | WASM contract | 📋 Q3 2026 |
| **Governance Extensions** | WASM contract | 📋 Q3 2026 |

### Cross-Chain Swap Flow (Zero Fees on Sultan)

**User wants to swap BTC → SOL:**

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. USER INITIATES                                               │
│    - Has 0.5 BTC on Bitcoin                                     │
│    - Wants SOL on Solana                                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│ 2. BITCOIN → SULTAN                                             │
│    - Lock 0.5 BTC in HTLC contract                             │
│    - Bitcoin network fee: ~$2                                   │
│    - Sultan mints 0.5 sBTC (instant, 2-sec)                     │
│    - Sultan fee: $0                                             │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│ 3. SWAP ON SULTAN                                               │
│    - Native DEX: 0.5 sBTC → X sSOL                             │
│    - AMM formula: constant product (x * y = k)                  │
│    - LP fee: 0.3% (goes to liquidity providers)                 │
│    - Sultan gas fee: $0                                         │
│    - Execution: ~50μs                                           │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│ 4. SULTAN → SOLANA                                              │
│    - Burn X sSOL on Sultan                                      │
│    - Sultan fee: $0                                             │
│    - Unlock X SOL on Solana                                     │
│    - Solana fee: ~$0.001                                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│ 5. RESULT                                                       │
│    - User now has X SOL on Solana                              │
│    - Total fees: ~$2.01 (all external chain fees)              │
│    - Sultan fees: $0.00                                         │
│    - Time: ~65 seconds (60s BTC confirmation + 2s Sultan + 3s SOL)│
└─────────────────────────────────────────────────────────────────┘
```

### Liquidity Incentives

How do we attract liquidity for cross-chain pools?

**Liquidity Mining Program:**
```
Pool: sBTC/SLTN
- LP deposits sBTC + SLTN
- Receives LP tokens
- Earns:
  - 0.3% swap fees (proportional to share)
  - SLTN rewards (from 4% inflation allocation)
  
Target APY: 20-50% for early LPs
Duration: 2 years liquidity mining program
```

**Bridge Incentives:**
```
- First $10M bridged: 2x SLTN rewards
- $10M-$100M bridged: 1.5x SLTN rewards  
- $100M+ bridged: Standard rewards
```

---

## Part 6: Implementation Roadmap

### Phase 1: Current (Q4 2025 - Q1 2026)
**Status: In Progress**

| Task | Owner | Status |
|------|-------|--------|
| PWA Wallet (send/receive/stake) | Core Team | 🔄 In progress |
| Block Explorer (sultanscan.io) | Core Team | 📋 Next |
| Token Factory UI | Core Team | 📋 Next |
| Native DEX UI | Core Team | 📋 Next |
| TypeScript SDK | Core Team | 📋 Q1 2026 |
| Security Audit (CertiK) | External | 📋 Q1 2026 |

### Phase 2: WASM Contracts (Q2 2026)
**Status: Planning**

| Week | Task | Details |
|------|------|---------|
| 1-2 | wasmer integration | Add wasmer to sultan-core dependencies, create wasm_runtime.rs |
| 3-4 | Host functions | Token Factory, DEX, Staking queries exposed to contracts |
| 5-6 | Compute metering | Implement compute credits system |
| 7-8 | Contract storage | Key-value store for contract state |
| 9-10 | Transaction types | `ContractDeploy`, `ContractCall`, `ContractMigrate` |
| 11-12 | Testing & audit | Fuzz testing, security review |

**Deliverables:**
- [ ] `sultan-core/src/wasm_runtime.rs` — Core WASM execution
- [ ] `sultan-core/src/contract_storage.rs` — Contract state management
- [ ] `sultan-core/src/compute_metering.rs` — Rate limiting
- [ ] `sultan-cli contract deploy` — Deployment tooling
- [ ] Contract templates (token, NFT, escrow)
- [ ] Developer documentation

### Phase 3: Bridge Deployment (Q2 2026)
**Status: Planning**

| Bridge | Protocol | Complexity | Timeline |
|--------|----------|------------|----------|
| Bitcoin | HTLC + SPV | High (BTC scripting) | 8 weeks |
| Ethereum | Light client | Medium (ZK proofs) | 6 weeks |
| Solana | gRPC | Medium | 4 weeks |
| TON | FunC contract | Medium | 4 weeks |

**Security Requirements:**
- Multi-sig validation (7-of-11 validators)
- Fraud proofs for light clients
- Rate limiting (max $10M/day per bridge initially)
- Circuit breakers (auto-pause on anomalies)

### Phase 4: EVM Compatibility (Q4 2026)
**Status: Future**

| Week | Task |
|------|------|
| 1-2 | revm integration |
| 3-4 | State mapping (Sultan ↔ EVM) |
| 5-6 | Precompile implementation |
| 7-8 | eth_* JSON-RPC endpoint |
| 9-10 | Tooling (Hardhat/Foundry plugins) |
| 11-12 | Testing & audit |

---

## Part 7: Technical Specifications

### Contract Interface Standard (Sultan-native)

```rust
// All Sultan WASM contracts implement this trait
pub trait SultanContract {
    /// Called once when contract is deployed
    fn instantiate(deps: Deps, env: Env, info: MessageInfo, msg: InstantiateMsg) -> Result<Response>;
    
    /// Called for state-changing operations
    fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response>;
    
    /// Called for read-only queries
    fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary>;
    
    /// Optional: Called during contract migration
    fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response>;
}

// Standard environment provided to contracts
pub struct Env {
    pub block: BlockInfo,
    pub transaction: Option<TransactionInfo>,
    pub contract: ContractInfo,
}

pub struct BlockInfo {
    pub height: u64,
    pub time: Timestamp,  // nanoseconds since epoch
    pub chain_id: String,
    pub shard_id: u8,     // Sultan-specific: which shard
}
```

### Compute Unit Costs

| Operation | Compute Units |
|-----------|---------------|
| WASM instruction (basic) | 1 |
| Memory read (per page) | 10 |
| Memory write (per page) | 20 |
| Storage read (per byte) | 100 |
| Storage write (per byte) | 500 |
| Crypto: SHA256 | 1,000 |
| Crypto: Ed25519 verify | 5,000 |
| Host: Token transfer | 10,000 |
| Host: DEX swap | 50,000 |

**Default Allocation:**
- Free tier: 1,000,000 compute units per block per address
- Staked (10K+ SLTN): 10,000,000 compute units per block
- Validator: Unlimited

### Cross-Shard Contract Execution

Contracts on Sultan must handle sharded state:

```
Shard 0: Accounts sultan1a... to sultan1d...
Shard 1: Accounts sultan1e... to sultan1h...
...
Shard 7: Accounts sultan1y... to sultan1z...
```

**Cross-shard contract calls:**
1. Contract on Shard 0 calls contract on Shard 3
2. Message queued for cross-shard coordinator
3. 2-Phase Commit ensures atomicity
4. Result returned in next block (async)

```rust
// Cross-shard call example
fn execute_cross_shard(
    deps: DepsMut,
    target_contract: &str,  // On different shard
    msg: Binary,
) -> Result<SubMsg> {
    Ok(SubMsg::new(CosmosMsg::CrossShard {
        target: target_contract.to_string(),
        msg,
        reply_on: ReplyOn::Success,
    }))
}
```

---

## Part 8: Security Framework

### Security Philosophy

**This is not a toy. Users will trust Sultan with real funds.**

Every component we build — bridges, WASM runtime, EVM compatibility, DEX — handles real assets. A single vulnerability could result in:
- Total loss of user funds
- Permanent reputational damage
- Legal liability
- Project death

**Our security commitment:**
1. **Defense in depth** — Multiple layers of protection, never single points of failure
2. **Assume breach** — Design systems to limit damage when (not if) something goes wrong
3. **Transparency** — Open source, public audits, bug bounties
4. **Slow and safe** — No rushing to mainnet, thorough testing before deployment

### Critical Infrastructure Components

| Component | Risk Level | Funds at Risk | Security Priority |
|-----------|------------|---------------|-------------------|
| **Bridges** | 🔴 CRITICAL | All bridged assets (sBTC, sETH, sSOL, sTON) | Highest |
| **WASM Runtime** | 🔴 CRITICAL | All contract-held funds | Highest |
| **Native DEX** | 🟠 HIGH | All liquidity pool funds | High |
| **Token Factory** | 🟡 MEDIUM | Custom token supplies | Medium |
| **Staking** | 🟠 HIGH | All staked SLTN | High |
| **Governance** | 🟡 MEDIUM | Protocol control | Medium |

### Bridge Security (Highest Priority)

Bridges are the #1 target for attackers. Historical bridge hacks:
- Ronin Bridge: $625M stolen (validator key compromise)
- Wormhole: $320M stolen (signature verification bug)
- Nomad: $190M stolen (initialization flaw)

**Sultan Bridge Security Model:**

```
┌─────────────────────────────────────────────────────────────────┐
│                    BRIDGE SECURITY LAYERS                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Layer 1: Multi-Signature Validation                            │
│  ─────────────────────────────────────                          │
│  • 7-of-11 validator signatures required                        │
│  • Geographically distributed signers                           │
│  • Hardware security modules (HSMs) for key storage             │
│  • No single point of compromise                                │
│                                                                  │
│  Layer 2: Rate Limiting                                          │
│  ──────────────────────                                         │
│  • Max $1M per transaction                                      │
│  • Max $10M per hour per bridge                                 │
│  • Max $50M per day total                                       │
│  • Cooldown periods for large withdrawals                       │
│                                                                  │
│  Layer 3: Fraud Detection                                        │
│  ────────────────────────                                       │
│  • Real-time monitoring of all bridge transactions              │
│  • Anomaly detection (unusual patterns, timing, amounts)        │
│  • Automatic pause on suspicious activity                       │
│  • 24/7 on-call security response                               │
│                                                                  │
│  Layer 4: Time Locks                                             │
│  ───────────────────                                            │
│  • Large withdrawals (>$100K): 1-hour delay                     │
│  • Massive withdrawals (>$1M): 24-hour delay                    │
│  • Users can cancel during delay if compromised                 │
│                                                                  │
│  Layer 5: Insurance & Recovery                                   │
│  ──────────────────────────                                     │
│  • Insurance fund (5% of bridged TVL)                           │
│  • Slashing of malicious validator stakes                       │
│  • Governance-controlled emergency recovery                     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Bridge Validator Requirements:**
- Minimum stake: 100,000 SLTN (substantial skin in the game)
- KYC required for bridge validators (legal accountability)
- Hardware security module (HSM) mandatory
- 99.9% uptime requirement
- Slashing: 100% stake for proven fraud

### WASM Runtime Security

Smart contract runtimes are the second highest risk:

| Vulnerability Class | Mitigation |
|--------------------|------------|
| Infinite loops | Hard compute limits, fuel metering |
| Memory exhaustion | 16MB memory cap per contract |
| Stack overflow | 1024 call depth limit |
| Reentrancy | Mutex locks on state, check-effects-interactions |
| Integer overflow | Rust's checked arithmetic, no wrapping |
| Storage spam | Deposit required (100 SLTN per MB) |
| Malicious bytecode | Bytecode validation before execution |
| Host function abuse | Rate limiting on host calls |

**Sandboxing Guarantees:**
```rust
// Every contract execution is isolated
pub struct ContractSandbox {
    // Memory: Contract cannot access host memory
    memory: WasmMemory,  // Isolated 16MB max
    
    // CPU: Contract cannot run forever  
    fuel: u64,  // Decrements per instruction, hard stop at 0
    
    // Storage: Contract cannot write unlimited data
    storage_budget: u64,  // Bytes remaining, requires deposit
    
    // Calls: Contract cannot spam host functions
    host_call_budget: u32,  // Max host calls per execution
}
```

### Threat Model

| Threat | Impact | Likelihood | Mitigation |
|--------|--------|------------|------------|
| Bridge validator collusion | 🔴 Critical | Low | 7-of-11 multi-sig, geographic distribution, HSMs |
| Bridge smart contract bug | 🔴 Critical | Medium | Multiple audits, formal verification, bug bounty |
| WASM runtime escape | 🔴 Critical | Low | Use battle-tested wasmer, extensive fuzzing |
| Malicious contract draining DEX | 🟠 High | Medium | Contract verification, rate limits |
| Oracle manipulation | 🟠 High | Medium | TWAP, multiple sources, circuit breakers |
| 51% attack on consensus | 🔴 Critical | Very Low | Stake distribution, slashing, checkpoints |
| Private key theft | 🔴 Critical | Medium | HSMs, multi-sig, social recovery |
| Governance takeover | 🟡 Medium | Low | Time-locks, quorum requirements, veto power |
| DDoS on validators | 🟡 Medium | High | Rate limiting, multiple RPC endpoints |
| Quantum computing attack | 🟡 Medium | Very Low (now) | Dilithium3 post-quantum signatures ready |

### Audit Schedule & Budget

| Audit | Scope | Auditor | Timeline | Est. Cost |
|-------|-------|---------|----------|-----------|
| Core Protocol | Consensus, staking, sharding, crypto | CertiK | Q1 2026 | $150K |
| Bridge Contracts | All four bridges + manager | Halborn | Q2 2026 | $200K |
| WASM Runtime | Contract execution, host functions, metering | Trail of Bits | Q2 2026 | $180K |
| Formal Verification | Bridge multi-sig, critical paths | Runtime Verification | Q2 2026 | $120K |
| EVM Runtime | EVM compatibility, precompiles, state mapping | Spearbit | Q4 2026 | $150K |
| **Total** | | | | **$800K** |

**Bug Bounty Program:**
- Launch: Q1 2026 (after first audit)
- Platform: Immunefi
- Maximum payout: $500K for critical bridge vulnerabilities
- Scope: All production code

### Security Testing Requirements

**Before ANY mainnet deployment:**

1. **Unit Tests**
   - 100% coverage of critical paths
   - Property-based testing (proptest/quickcheck)
   - Edge case coverage

2. **Integration Tests**
   - Multi-node test networks
   - Cross-shard transaction testing
   - Bridge end-to-end flows

3. **Fuzzing**
   - cargo-fuzz on all parsers and validators
   - Honggfuzz for WASM runtime
   - Minimum 1 week continuous fuzzing

4. **Formal Verification**
   - Bridge signature verification logic
   - Multi-sig threshold enforcement
   - Token minting/burning invariants

5. **Adversarial Testing**
   - Internal red team exercises
   - External penetration testing
   - Economic attack simulations

### Emergency Procedures

```
┌─────────────────────────────────────────────────────────────────┐
│                    INCIDENT RESPONSE PLAN                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  SEVERITY 1: Active Exploit (funds being drained)               │
│  ─────────────────────────────────────────────────              │
│  T+0:     Automatic circuit breaker triggers                    │
│  T+5min:  On-call engineer confirms, escalates                  │
│  T+15min: All bridges paused, DEX paused if needed              │
│  T+30min: Root cause identified                                 │
│  T+1hr:   Public disclosure (Twitter, Discord)                  │
│  T+24hr:  Post-mortem published                                 │
│  T+7d:    Fix deployed, audited, bridges resumed                │
│                                                                  │
│  SEVERITY 2: Vulnerability Discovered (not exploited)           │
│  ──────────────────────────────────────────────────             │
│  T+0:     Bug report received (bounty or internal)              │
│  T+4hr:   Severity assessed, fix developed                      │
│  T+24hr:  Fix audited (expedited review)                        │
│  T+48hr:  Fix deployed to mainnet                               │
│  T+72hr:  Public disclosure, bounty paid                        │
│                                                                  │
│  SEVERITY 3: Suspicious Activity                                 │
│  ────────────────────────────                                   │
│  T+0:     Alert triggered (rate limit, anomaly)                 │
│  T+15min: On-call reviews, determines if real threat            │
│  T+1hr:   If threat: escalate to Severity 1/2                   │
│           If false positive: document and tune alerts           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Circuit Breaker Triggers (Automatic):**
- Bridge: >$1M outflow in 1 hour → pause bridge
- Bridge: >3 failed verifications in 10 minutes → pause bridge
- DEX: >20% price deviation in 1 block → pause affected pool
- Contract: >100 failed executions in 1 minute → quarantine contract
- Consensus: >3 validators offline → alert (no pause)

**Recovery Keys (Last Resort):**
- 5-of-9 core team multi-sig for emergency pause
- 7-of-9 for emergency upgrade
- All signers use hardware wallets
- Signers distributed globally (no single jurisdiction)

### Transparency & Trust

**How we build user trust:**

1. **Open Source**
   - All Sultan code is open source (MIT license)
   - Bridge contracts verified on-chain
   - Anyone can audit, anyone can verify

2. **Public Audits**
   - All audit reports published in full
   - No hiding "informational" findings
   - Remediation status tracked publicly

3. **Real-Time Monitoring**
   - Public dashboard showing bridge TVL
   - Transaction history fully transparent
   - Validator set and stake visible

4. **Proof of Reserves**
   - Regular attestations of bridge collateral
   - On-chain verification of backing
   - Third-party auditors for reserves

5. **Governance**
   - Major changes require governance vote
   - 7-day time-lock on all upgrades
   - Community can veto dangerous proposals

### Security Checklist (Pre-Launch)

**Before Bridge Mainnet (Q2 2026):**
- [ ] CertiK audit complete, all criticals fixed
- [ ] Halborn bridge audit complete, all criticals fixed
- [ ] 30 days testnet operation without issues
- [ ] $500K bug bounty live for 30 days
- [ ] Insurance fund seeded (minimum $5M)
- [ ] HSMs deployed for all bridge validators
- [ ] Rate limits tested under load
- [ ] Circuit breakers tested in staging
- [ ] Incident response drill completed
- [ ] On-call rotation established

**Before WASM Mainnet (Q2 2026):**
- [ ] Trail of Bits audit complete
- [ ] Fuzzing: 0 crashes in 7 days
- [ ] Formal verification of metering
- [ ] Contract size limits enforced
- [ ] Compute limits tested at scale
- [ ] Malicious contract test suite passing

**Before EVM Mainnet (Q4 2026):**
- [ ] Spearbit audit complete
- [ ] Ethereum test suite passing (ethereum/tests)
- [ ] Precompiles verified correct
- [ ] Gas/compute mapping validated
- [ ] eth_* RPC compatibility verified

---

## Part 9: Developer Experience

### Contract Development Workflow

```bash
# 1. Create new contract project
sultan-cli contract new my-token
cd my-token

# 2. Write contract in Rust
# src/lib.rs with SultanContract implementation

# 3. Compile to WASM
sultan-cli contract build
# Output: target/wasm32-unknown-unknown/release/my_token.wasm

# 4. Optimize (reduce size)
sultan-cli contract optimize
# Output: artifacts/my_token.wasm (typically 100-300KB)

# 5. Deploy to testnet
sultan-cli contract deploy \
  --network testnet \
  --code artifacts/my_token.wasm \
  --instantiate '{"name":"My Token","symbol":"MTK"}'

# 6. Interact
sultan-cli contract execute \
  --contract sultan1abc... \
  --msg '{"transfer":{"to":"sultan1xyz...","amount":"1000"}}'
```

### SDK Examples

**TypeScript:**
```typescript
import { SultanClient, Contract } from '@sultan/sdk';

const client = new SultanClient('https://rpc.sltn.io');

// Deploy contract
const codeId = await client.upload(wasmBytes);
const contract = await client.instantiate(codeId, {
  name: "My Token",
  symbol: "MTK",
  decimals: 6,
});

// Execute
await contract.execute({ 
  transfer: { to: "sultan1xyz...", amount: "1000" } 
});

// Query
const balance = await contract.query({ 
  balance: { address: "sultan1abc..." } 
});
```

**Rust:**
```rust
use sultan_sdk::{Client, Contract};

let client = Client::new("https://rpc.sltn.io");

// Deploy
let code_id = client.upload(&wasm_bytes).await?;
let contract = client.instantiate(code_id, &InstantiateMsg {
    name: "My Token".into(),
    symbol: "MTK".into(),
}).await?;

// Execute
contract.execute(&ExecuteMsg::Transfer {
    to: "sultan1xyz...".into(),
    amount: 1000u128.into(),
}).await?;
```

---

## Part 10: File Locations & Module Map

### Production Code

```
sultan-core/src/
├── main.rs                    # Node entry point
├── blockchain.rs              # Block creation/validation
├── consensus.rs               # Custom PoS consensus
├── sharding.rs                # Shard coordination
├── p2p.rs                     # libp2p networking
├── token_factory.rs           # Native token creation (354 lines)
├── native_dex.rs              # AMM/DEX (462 lines)
├── staking.rs                 # Validator staking (532 lines)
├── governance.rs              # On-chain governance (556 lines)
├── transaction_validator.rs   # TX validation
├── storage.rs                 # RocksDB persistence
├── quantum.rs                 # Dilithium3 post-quantum
├── types.rs                   # Core data types
│
├── wasm_runtime.rs            # 📋 TO CREATE (Q2 2026)
├── evm_runtime.rs             # 📋 TO CREATE (Q4 2026)
├── compute_metering.rs        # 📋 TO CREATE (Q2 2026)
└── contract_storage.rs        # 📋 TO CREATE (Q2 2026)

bridges/
├── bridge_manager.rs          # Bridge coordination
├── bitcoin/
│   └── btc_bridge.rs          # Bitcoin HTLC bridge
├── ethereum/
│   ├── eth_bridge.sol         # Solidity lock contract
│   └── deploy/                # Deployment scripts
├── solana/
│   └── sol_bridge.rs          # Solana program bridge
└── ton/
    └── ton_bridge.fc          # FunC bridge contract
```

### ⚠️ Legacy Code — DO NOT USE FOR PRODUCTION

The following files exist in the repository but are **legacy experiments** from early development when we considered using Cosmos SDK. **These are NOT compatible with Sultan's native architecture:**

| File/Directory | Why It Exists | Status |
|----------------|---------------|--------|
| `_archive/` | Old experiments, Cosmos SDK attempts | ❌ Do not use |
| `cosmos-data/` | Legacy Cosmos data structures | ❌ Do not use |
| `scripts/build_cw_artifacts.sh` | CosmWasm build script | ❌ Reference only |
| `scripts/deploy_cw20_cw721_refs.sh` | CosmWasm deployment | ❌ Reference only |
| `scripts/test_cw20_zero_fee.sh` | CosmWasm testing | ❌ Reference only |
| `scripts/harden_sultan_node.sh` | References `.wasmd` paths | ❌ Needs rewrite |

**Important:** These scripts reference `cosmwasm/rust-optimizer`, `wasmd`, and Cosmos SDK patterns. Sultan does NOT use CosmWasm. Our WASM contracts will use a Sultan-native interface, not CosmWasm's.

### Files to Create (Implementation Order)

| Priority | File | Purpose |
|----------|------|---------|
| 1 | `sultan-core/src/wasm_runtime.rs` | wasmer integration, contract execution |
| 2 | `sultan-core/src/compute_metering.rs` | Rate limiting, compute credits |
| 3 | `sultan-core/src/contract_storage.rs` | Contract state persistence |
| 4 | `sultan-cli/src/contract.rs` | CLI tooling for contracts |
| 5 | `sultan-core/src/evm_runtime.rs` | revm integration |
| 6 | `sultan-core/src/evm_state.rs` | Sultan-to-EVM state mapping |

---

## Part 11: Token & NFT Standards

### Why Standards Matter

For Sultan to have a thriving ecosystem, third-party contracts need to interoperate. Standard interfaces enable:
- Wallets to display any token without custom integration
- DEXes to list any token automatically
- NFT marketplaces to support any collection
- Composability between DeFi protocols

### Fungible Token Standard (Sultan-20)

Our native Token Factory handles most token use cases, but WASM contracts need a standard interface for custom token logic:

```rust
// Sultan-20: Fungible Token Standard
// Compatible with CW20 patterns, adapted for Sultan

pub trait Sultan20 {
    // Queries
    fn balance(&self, address: &str) -> u128;
    fn total_supply(&self) -> u128;
    fn token_info(&self) -> TokenInfo;
    fn allowance(&self, owner: &str, spender: &str) -> u128;
    
    // Executions
    fn transfer(&mut self, recipient: &str, amount: u128) -> Result<()>;
    fn transfer_from(&mut self, owner: &str, recipient: &str, amount: u128) -> Result<()>;
    fn approve(&mut self, spender: &str, amount: u128) -> Result<()>;
    fn mint(&mut self, recipient: &str, amount: u128) -> Result<()>;  // If mintable
    fn burn(&mut self, amount: u128) -> Result<()>;
}

pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u128,
}
```

**For EVM compatibility (ERC-20):**
```solidity
// Standard ERC-20 interface - fully supported via EVM runtime
interface IERC20 {
    function totalSupply() external view returns (uint256);
    function balanceOf(address account) external view returns (uint256);
    function transfer(address to, uint256 amount) external returns (bool);
    function allowance(address owner, address spender) external view returns (uint256);
    function approve(address spender, uint256 amount) external returns (bool);
    function transferFrom(address from, address to, uint256 amount) external returns (bool);
}
```

### NFT Standard (Sultan-721)

For non-fungible tokens:

```rust
// Sultan-721: NFT Standard
// Compatible with CW721 patterns, adapted for Sultan

pub trait Sultan721 {
    // Queries
    fn owner_of(&self, token_id: &str) -> Option<String>;
    fn token_info(&self, token_id: &str) -> Option<TokenMetadata>;
    fn tokens(&self, owner: &str) -> Vec<String>;
    fn all_tokens(&self) -> Vec<String>;
    fn num_tokens(&self) -> u64;
    fn contract_info(&self) -> ContractInfo;
    fn approval(&self, token_id: &str, spender: &str) -> bool;
    
    // Executions  
    fn transfer_nft(&mut self, recipient: &str, token_id: &str) -> Result<()>;
    fn approve(&mut self, spender: &str, token_id: &str) -> Result<()>;
    fn approve_all(&mut self, operator: &str, approved: bool) -> Result<()>;
    fn mint(&mut self, owner: &str, token_id: &str, metadata: TokenMetadata) -> Result<()>;
    fn burn(&mut self, token_id: &str) -> Result<()>;
}

pub struct TokenMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,        // IPFS URI or URL
    pub animation_url: Option<String>,
    pub external_url: Option<String>,
    pub attributes: Vec<Attribute>,
}

pub struct Attribute {
    pub trait_type: String,
    pub value: String,
}
```

**For EVM compatibility (ERC-721):**
```solidity
interface IERC721 {
    function balanceOf(address owner) external view returns (uint256);
    function ownerOf(uint256 tokenId) external view returns (address);
    function safeTransferFrom(address from, address to, uint256 tokenId) external;
    function transferFrom(address from, address to, uint256 tokenId) external;
    function approve(address to, uint256 tokenId) external;
    function setApprovalForAll(address operator, bool approved) external;
    function getApproved(uint256 tokenId) external view returns (address);
    function isApprovedForAll(address owner, address operator) external view returns (bool);
}
```

### Multi-Token Standard (Sultan-1155)

For games, collectibles with multiple copies:

```rust
// Sultan-1155: Multi-Token Standard
// Compatible with ERC-1155 patterns

pub trait Sultan1155 {
    fn balance_of(&self, owner: &str, token_id: u128) -> u128;
    fn balance_of_batch(&self, owners: &[String], token_ids: &[u128]) -> Vec<u128>;
    fn is_approved_for_all(&self, owner: &str, operator: &str) -> bool;
    
    fn safe_transfer_from(
        &mut self, from: &str, to: &str, token_id: u128, amount: u128
    ) -> Result<()>;
    fn safe_batch_transfer_from(
        &mut self, from: &str, to: &str, token_ids: &[u128], amounts: &[u128]
    ) -> Result<()>;
    fn set_approval_for_all(&mut self, operator: &str, approved: bool) -> Result<()>;
}
```

### Contract Templates (To Build)

We will provide official templates for common use cases:

| Template | Purpose | Status |
|----------|---------|--------|
| `sultan-token` | Sultan-20 fungible token | 📋 Q2 2026 |
| `sultan-nft` | Sultan-721 NFT collection | 📋 Q2 2026 |
| `sultan-multi-token` | Sultan-1155 multi-token | 📋 Q3 2026 |
| `sultan-escrow` | Time-locked escrow | 📋 Q2 2026 |
| `sultan-vesting` | Token vesting schedule | 📋 Q2 2026 |
| `sultan-multisig` | Multi-signature wallet | 📋 Q2 2026 |
| `sultan-dao` | Governance extensions | 📋 Q3 2026 |
| `sultan-amm` | Custom AMM pools | 📋 Q3 2026 |

---

## Part 12: Testnet & Development Infrastructure

### Testnet Architecture

Before mainnet deployment, all features go through our testnet:

```
┌─────────────────────────────────────────────────────────────────┐
│                    SULTAN TESTNET                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Network: testnet.sltn.io                                       │
│  Chain ID: sultan-testnet-1                                     │
│  Block Time: 2 seconds (same as mainnet)                        │
│  Validators: 5 (core team operated)                             │
│                                                                  │
│  RPC Endpoints:                                                  │
│  • https://rpc.testnet.sltn.io                                  │
│  • https://api.testnet.sltn.io                                  │
│  • https://evm.testnet.sltn.io (when EVM ready)                 │
│                                                                  │
│  Faucet: https://faucet.testnet.sltn.io                         │
│  • 1000 testnet SLTN per request                                │
│  • Rate limit: 1 request per hour per address                   │
│  • Requires GitHub authentication (anti-abuse)                  │
│                                                                  │
│  Explorer: https://explorer.testnet.sltn.io                     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Development Environment Setup

For contract developers:

```bash
# 1. Install Rust with WASM target
rustup target add wasm32-unknown-unknown

# 2. Install Sultan CLI
cargo install sultan-cli

# 3. Configure for testnet
sultan-cli config set network testnet
sultan-cli config set rpc https://rpc.testnet.sltn.io

# 4. Create wallet for testing
sultan-cli wallet create --name dev-wallet
# Save the mnemonic!

# 5. Get testnet tokens from faucet
sultan-cli faucet request

# 6. Verify balance
sultan-cli wallet balance
# Should show: 1000 SLTN (testnet)

# 7. Deploy your first contract
sultan-cli contract deploy \
  --code ./artifacts/my_contract.wasm \
  --instantiate '{"name":"Test"}'
```

### Local Development (Without Testnet)

For faster iteration:

```bash
# Run a local Sultan node
sultan-node --dev --tmp

# This gives you:
# - Single validator (instant blocks)
# - Pre-funded dev accounts
# - Clean state on restart
# - RPC at localhost:26657

# Deploy to local node
sultan-cli config set rpc http://localhost:26657
sultan-cli contract deploy --code ./my_contract.wasm
```

### CI/CD for Contracts

Recommended GitHub Actions workflow:

```yaml
# .github/workflows/contract-ci.yml
name: Contract CI

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-unknown-unknown
          
      - name: Build contract
        run: cargo build --release --target wasm32-unknown-unknown
        
      - name: Run tests
        run: cargo test
        
      - name: Check size
        run: |
          SIZE=$(stat -f%z target/wasm32-unknown-unknown/release/*.wasm)
          if [ $SIZE -gt 1048576 ]; then
            echo "Contract too large: $SIZE bytes (max 1MB)"
            exit 1
          fi
          
      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: contract-wasm
          path: target/wasm32-unknown-unknown/release/*.wasm
```

---

## Part 13: Cryptographic Compatibility

### Sultan's Signature Scheme

Sultan uses **Ed25519** for all transaction signatures:

| Property | Value |
|----------|-------|
| Curve | Curve25519 |
| Signature Size | 64 bytes |
| Public Key Size | 32 bytes |
| Library | `ed25519-dalek` |
| Deterministic | Yes |
| Performance | ~15,000 verifications/sec |

**Why Ed25519?**
- Faster than ECDSA (Ethereum's secp256k1)
- Deterministic signatures (same input = same output)
- No nonce reuse vulnerabilities
- Smaller signatures (64 vs 65 bytes)

### Contract Signature Verification

Contracts can verify Ed25519 signatures via host function:

```rust
// Host function available to all contracts
fn host_verify_ed25519(
    message: &[u8],
    signature: &[u8; 64],
    public_key: &[u8; 32],
) -> bool;

// Example: Contract verifies a signed message
pub fn execute_with_signature(
    deps: DepsMut,
    info: MessageInfo,
    message: Vec<u8>,
    signature: [u8; 64],
    signer_pubkey: [u8; 32],
) -> Result<Response> {
    // Verify signature
    if !deps.api.verify_ed25519(&message, &signature, &signer_pubkey) {
        return Err(ContractError::InvalidSignature);
    }
    
    // Proceed with authenticated operation
    // ...
}
```

### EVM Address Mapping

When running EVM contracts, we need to map between Sultan addresses (Ed25519) and Ethereum addresses (secp256k1/Keccak):

```
Sultan Address:  sultan1a2b3c4d5e6f7g8h9i0j... (bech32, 32-byte pubkey)
                           │
                           ▼
                 [Keccak256 hash of pubkey]
                           │
                           ▼
EVM Address:     0x1234567890abcdef1234567890abcdef12345678 (20 bytes)
```

**Mapping Rules:**
- Each Sultan address has exactly one corresponding EVM address
- Mapping is deterministic and reversible
- Native SLTN balance visible at both addresses
- EVM contracts see the 20-byte address
- Sultan native sees the bech32 address

### Post-Quantum Readiness

Sultan validators use **Dilithium3** (NIST post-quantum standard) for:
- Validator registration
- Governance votes
- Bridge multi-sig

```rust
// Dilithium3 specifications
pub struct Dilithium3Signature {
    // Signature: 3,293 bytes (larger than Ed25519's 64 bytes)
    // Public key: 1,952 bytes
    // Security level: NIST Level 3 (~192-bit classical, ~128-bit quantum)
}
```

**Why both Ed25519 and Dilithium3?**
- Ed25519: Fast, small, used for user transactions (quantum threat is years away)
- Dilithium3: Quantum-resistant, used for high-value validator operations
- Upgrade path: Can migrate users to Dilithium3 when needed

---

## Part 14: Open Questions & Decisions

### Resolved Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| WASM runtime | wasmer | Fastest, pure Rust, production-proven |
| EVM runtime | revm | Used by Foundry/Reth, actively maintained |
| Compute model | Credits + stake-weighted | Zero fees require alternative |
| Bridge security | 7-of-11 multi-sig | Balance security vs liveness |

### Open Questions (To Resolve During Implementation)

1. **Cross-shard contract calls: sync or async?**
   - Option A: Async (message passing, eventual consistency)
   - Option B: Sync with 2PC (slower but easier to reason about)
   - **Leaning:** Async with reply callbacks

2. **Contract upgrades: immutable or migrateable?**
   - Option A: Immutable (more secure, less flexible)
   - Option B: Admin-controlled migration (like CosmWasm)
   - **Leaning:** Optional migration with admin key

3. **Storage pricing: deposit or rent?**
   - Option A: Deposit (refund on deletion)
   - Option B: Rent (pay ongoing, auto-delete if empty)
   - **Leaning:** Deposit model (simpler)

4. **Contract size limit?**
   - Ethereum: 24KB
   - NEAR: 4MB
   - Solana: 10MB
   - **Leaning:** 1MB default, configurable

---

## Summary

### Security-First Mandate

**This infrastructure handles real user funds. Security is non-negotiable.**

- $800K+ allocated to security audits
- $500K bug bounty program
- Multiple independent auditors (CertiK, Trail of Bits, Halborn, Spearbit)
- Formal verification of critical paths
- Defense in depth: multi-sig, rate limits, time-locks, circuit breakers
- No mainnet deployment without passing all security checklists

### What Sultan Is

**Sultan is a sovereign L1 with:**
- Zero gas fees (4% inflation funds validators)
- Native interop with Bitcoin, Ethereum, Solana, TON
- WASM smart contracts (Q2 2026) using wasmer
- EVM compatibility (Q4 2026) using revm
- Cross-chain DeFi where all Sultan operations are free

### Who's Building It

**We (the core team) are building this ourselves:**
- PWA Wallet → Token Factory UI → DEX UI → Block Explorer (now)
- WASM Runtime → Bridges → EVM (2026)
- No outsourcing critical infrastructure
- Security expertise in-house + external auditors

### Key Files

**Existing production code:**
- `sultan-core/src/{token_factory,native_dex,staking,governance}.rs`

**To create (Q2 2026):**
- `sultan-core/src/{wasm_runtime,compute_metering,contract_storage}.rs`

**To create (Q4 2026):**
- `sultan-core/src/{evm_runtime,evm_state}.rs`

### Dependencies

```toml
# Cargo.toml additions for Q2 2026
wasmer = "4.0"
wasmer-compiler-cranelift = "4.0"

# Cargo.toml additions for Q4 2026  
revm = "3.0"
```

### Security Budget Summary

| Category | Allocation |
|----------|------------|
| Security Audits | $800K |
| Bug Bounty (annual) | $500K max payouts |
| Insurance Fund | 5% of TVL |
| HSM Infrastructure | $50K |
| Monitoring/Alerting | $20K/year |
| **Total (Year 1)** | **$1.4M+** |

### Document Sections Quick Reference

| Part | Topic |
|------|-------|
| 1 | Sultan Architecture Foundation |
| 2 | Cross-Chain Interoperability |
| 3 | What is WASM and Why We Need It |
| 4 | EVM Compatibility |
| 5 | Cross-Chain DeFi Product Architecture |
| 6 | Implementation Roadmap |
| 7 | Technical Specifications |
| 8 | Security Framework |
| 9 | Developer Experience |
| 10 | File Locations & Legacy Code Warnings |
| 11 | Token & NFT Standards |
| 12 | Testnet & Development Infrastructure |
| 13 | Cryptographic Compatibility |
| 14 | Open Questions & Decisions |

---

*Last Updated: December 15, 2025*
*Document Version: 2.0*
*Status: Canonical Technical Reference*
*Authors: Sultan Core Engineering Team*
*Security Review: Required before implementation*
