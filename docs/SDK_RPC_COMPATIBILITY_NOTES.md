# SDK & RPC Compatibility with Smart Contracts

## Document Purpose

This document analyzes whether our existing SDK and RPC work will be compatible with the upcoming WASM smart contract and EVM development. **Read this before starting Q2 2026 contract work.**

**TL;DR:** The current SDK/RPC is a foundation that we **extend**, not replace. Core functionality remains, but we need to add contract-specific methods.

---

## Part 1: Current SDK/RPC State Analysis

### What We Have Today

Based on the website (`replit-website/index.html`) and docs:

**RPC Endpoints (Production):**
| Endpoint | Purpose | Status |
|----------|---------|--------|
| `https://rpc.sltn.io/status` | Network status, block height, validators | ✅ Working |
| `https://rpc.sltn.io/balance/{address}` | Query SLTN balance | ✅ Working |
| `https://rpc.sltn.io/bridges` | Bridge status | ✅ Working |
| `https://api.sultan.network` | REST API | ✅ Working |

**SDK Methods (Rust):**
```rust
// Current SDK pattern from website
use sultan_sdk::SultanSDK;

let sdk = SultanSDK::new_mainnet().await?;
let balance = sdk.get_balance_sltn("sultan1...").await?;
```

**JavaScript/TypeScript:**
```javascript
// Current pattern from website
const balance = await fetch('https://rpc.sltn.io/balance/sultan1...').then(r => r.json());
const status = await fetch('https://rpc.sltn.io/status').then(r => r.json());
```

### What's Missing for Smart Contracts

The current SDK/RPC handles **native operations only**:
- ✅ Balance queries
- ✅ Network status
- ✅ Block information
- ✅ Validator data
- ✅ Bridge status
- ❌ Contract deployment
- ❌ Contract execution
- ❌ Contract queries
- ❌ Contract events/logs
- ❌ WASM bytecode upload
- ❌ EVM transaction handling
- ❌ eth_* JSON-RPC methods

---

## Part 2: Compatibility Assessment

### ✅ What Will Still Work (No Changes Needed)

| Current Feature | Why It's Safe |
|-----------------|---------------|
| Balance queries | Contracts don't change native balance API |
| Block/status queries | Block structure stays the same |
| Validator queries | Staking module unchanged |
| Bridge queries | Bridges are separate from contract runtime |
| Native transfers | Transfer logic stays in native module |
| Native DEX swaps | DEX remains a native module |
| Token Factory operations | Token Factory remains native |

**Conclusion:** All native operations continue to work exactly as before.

### 🔧 What Needs Extension (Add, Don't Replace)

| New Feature | SDK Method | RPC Endpoint |
|-------------|------------|--------------|
| Upload WASM code | `sdk.contract_upload(wasm_bytes)` | `POST /contract/upload` |
| Instantiate contract | `sdk.contract_instantiate(code_id, msg)` | `POST /contract/instantiate` |
| Execute contract | `sdk.contract_execute(addr, msg)` | `POST /contract/execute` |
| Query contract | `sdk.contract_query(addr, msg)` | `GET /contract/{addr}/query` |
| Get contract info | `sdk.contract_info(addr)` | `GET /contract/{addr}/info` |
| List contracts | `sdk.contracts_list()` | `GET /contracts` |
| Get contract code | `sdk.contract_code(code_id)` | `GET /contract/code/{id}` |
| Contract events | `sdk.contract_events(addr, filter)` | `GET /contract/{addr}/events` |

### 🆕 What's Entirely New (EVM Layer)

EVM needs a **separate endpoint** for Ethereum tooling compatibility:

| Feature | Endpoint | Purpose |
|---------|----------|---------|
| EVM RPC | `https://evm.sltn.io` | Separate endpoint for eth_* methods |
| Chain ID | `eth_chainId` | Sultan EVM chain ID |
| Balance | `eth_getBalance` | Query via EVM address (0x...) |
| Send TX | `eth_sendRawTransaction` | Submit signed EVM transaction |
| Call | `eth_call` | Execute read-only contract call |
| Logs | `eth_getLogs` | Query contract events |
| Receipt | `eth_getTransactionReceipt` | Get execution result |
| Code | `eth_getCode` | Query contract bytecode |
| Gas Est. | `eth_estimateGas` | Compute unit estimation |

**Why separate endpoint?**
- MetaMask, Ethers.js, Hardhat expect standard eth_* methods
- Address format is different (0x... vs sultan1...)
- Transaction signing uses secp256k1 not Ed25519
- Clean separation prevents confusion

---

## Part 3: SDK Extension Plan

### Rust SDK Extension

```rust
// sultan-sdk/src/lib.rs - CURRENT (keep as-is)
impl SultanSDK {
    pub async fn new_mainnet() -> Result<Self, Error>;
    pub async fn new_testnet() -> Result<Self, Error>;
    pub async fn get_balance_sltn(&self, address: &str) -> Result<u128, Error>;
    pub async fn get_status(&self) -> Result<NetworkStatus, Error>;
    pub async fn send_sltn(&self, to: &str, amount: u128) -> Result<TxHash, Error>;
    // ... other native methods
}

// sultan-sdk/src/contract.rs - NEW (add for Q2 2026)
impl SultanSDK {
    // WASM Contract Methods
    pub async fn contract_upload(&self, wasm: &[u8]) -> Result<CodeId, Error>;
    pub async fn contract_instantiate(
        &self,
        code_id: CodeId,
        msg: &impl Serialize,
        label: &str,
    ) -> Result<ContractAddress, Error>;
    pub async fn contract_execute(
        &self,
        contract: &str,
        msg: &impl Serialize,
    ) -> Result<TxResult, Error>;
    pub async fn contract_query<T: DeserializeOwned>(
        &self,
        contract: &str,
        msg: &impl Serialize,
    ) -> Result<T, Error>;
    pub async fn contract_info(&self, contract: &str) -> Result<ContractInfo, Error>;
    pub async fn contracts_by_code(&self, code_id: CodeId) -> Result<Vec<String>, Error>;
}

// sultan-sdk/src/evm.rs - NEW (add for Q4 2026)
impl SultanSDK {
    // EVM Methods (wraps eth_* JSON-RPC)
    pub async fn evm_balance(&self, address: H160) -> Result<U256, Error>;
    pub async fn evm_call(&self, tx: &EvmCall) -> Result<Bytes, Error>;
    pub async fn evm_send(&self, signed_tx: &[u8]) -> Result<H256, Error>;
    pub async fn evm_receipt(&self, tx_hash: H256) -> Result<Option<Receipt>, Error>;
    pub async fn evm_code(&self, address: H160) -> Result<Bytes, Error>;
}
```

### TypeScript SDK Extension

```typescript
// @sultan/sdk - CURRENT (keep as-is)
class SultanClient {
  constructor(rpcUrl: string);
  async getBalance(address: string): Promise<Balance>;
  async getStatus(): Promise<NetworkStatus>;
  async sendSltn(to: string, amount: bigint): Promise<TxResult>;
  // ... other native methods
}

// @sultan/sdk - NEW methods for contracts (Q2 2026)
class SultanClient {
  // Extend with contract methods
  async uploadContract(wasmBytes: Uint8Array): Promise<CodeId>;
  async instantiateContract(codeId: number, msg: object, label: string): Promise<ContractAddr>;
  async executeContract(contract: string, msg: object): Promise<TxResult>;
  async queryContract<T>(contract: string, msg: object): Promise<T>;
  async getContractInfo(contract: string): Promise<ContractInfo>;
}

// @sultan/evm-sdk - NEW package for EVM (Q4 2026)
// This is a thin wrapper, most users will use ethers.js directly
class SultanEvmProvider extends ethers.providers.JsonRpcProvider {
  constructor() {
    super('https://evm.sltn.io');
  }
}
```

---

## Part 4: RPC Endpoint Extension Plan

### Native RPC (extend existing)

```
https://rpc.sltn.io/

EXISTING (keep as-is):
├── GET  /status                    → Network status
├── GET  /balance/{address}         → SLTN balance
├── POST /tx                        → Submit native transaction
├── GET  /tx/{hash}                 → Get transaction
├── GET  /block/{height}            → Get block
├── GET  /validators                → List validators
└── GET  /bridges                   → Bridge status

NEW FOR CONTRACTS (Q2 2026):
├── POST /contract/upload           → Upload WASM code
│   Request: { wasm_base64: "...", sender: "sultan1..." }
│   Response: { code_id: 1, checksum: "..." }
│
├── POST /contract/instantiate      → Create contract instance
│   Request: { code_id: 1, msg: {...}, label: "my-token", sender: "..." }
│   Response: { address: "sultan1contract...", tx_hash: "..." }
│
├── POST /contract/execute          → Execute contract
│   Request: { contract: "sultan1...", msg: {...}, sender: "..." }
│   Response: { tx_hash: "...", result: {...} }
│
├── POST /contract/query            → Query contract (read-only)
│   Request: { contract: "sultan1...", msg: {...} }
│   Response: { data: {...} }
│
├── GET  /contract/{addr}           → Contract info
│   Response: { code_id: 1, creator: "...", label: "...", created: "..." }
│
├── GET  /contract/{addr}/state     → Raw contract state
│   Response: { state: {...} }
│
├── GET  /contracts                 → List all contracts
│   Query: ?code_id=1 (optional filter)
│   Response: { contracts: ["sultan1...", ...] }
│
└── GET  /code/{id}                 → Get code info
    Response: { code_id: 1, checksum: "...", creator: "..." }
```

### EVM RPC (new endpoint)

```
https://evm.sltn.io/

Standard Ethereum JSON-RPC 2.0:

{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}
{"jsonrpc":"2.0","method":"eth_getBalance","params":["0x...","latest"],"id":1}
{"jsonrpc":"2.0","method":"eth_call","params":[{...},"latest"],"id":1}
{"jsonrpc":"2.0","method":"eth_sendRawTransaction","params":["0x..."],"id":1}
{"jsonrpc":"2.0","method":"eth_getTransactionReceipt","params":["0x..."],"id":1}
{"jsonrpc":"2.0","method":"eth_getLogs","params":[{...}],"id":1}
{"jsonrpc":"2.0","method":"eth_getCode","params":["0x...","latest"],"id":1}
{"jsonrpc":"2.0","method":"eth_estimateGas","params":[{...}],"id":1}
{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}
{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["latest",true],"id":1}
... (full Ethereum JSON-RPC compatibility)
```

---

## Part 5: Website Updates Needed

### Current Website (replit-website/index.html)

The website currently shows:

```html
<div class="resource-card">
    <h3>Sultan SDK</h3>
    <p>Production-ready Rust SDK for building on Sultan L1...</p>
    <div class="code-example">
        <code>
use sultan_sdk::SultanSDK;
let sdk = SultanSDK::new_mainnet().await?;
let balance = sdk.get_balance_sltn("sultan1...").await?;
        </code>
    </div>
</div>
```

### After Contract Implementation (Update Website)

```html
<div class="resource-card">
    <h3>Sultan SDK</h3>
    <p>Production-ready SDK for building on Sultan L1. Native transactions, 
       WASM contracts, and EVM compatibility.</p>
    <div class="code-example">
        <code>
// Native operations
let balance = sdk.get_balance_sltn("sultan1...").await?;

// WASM contracts
let code_id = sdk.contract_upload(&wasm_bytes).await?;
let result = sdk.contract_execute(addr, &msg).await?;

// EVM (via ethers.js)
const provider = new ethers.JsonRpcProvider("https://evm.sltn.io");
        </code>
    </div>
</div>
```

---

## Part 6: Implementation Order

### Timeline

| Phase | When | What to Do |
|-------|------|------------|
| **Phase 1: Current** | Now | Keep SDK/RPC as-is, focus on PWA wallet |
| **Phase 2: Pre-Contract** | Q1 2026 | Document final RPC spec, design contract endpoints |
| **Phase 3: WASM** | Q2 2026 | Add contract endpoints to RPC, extend SDK |
| **Phase 4: EVM** | Q4 2026 | Add `evm.sltn.io` endpoint, add EVM SDK methods |

### Files to Create/Modify

**Q2 2026 (WASM Contracts):**
```
sultan-core/src/
├── rpc/
│   ├── mod.rs              # Add contract routes
│   ├── contract.rs         # NEW: /contract/* handlers
│   └── native.rs           # EXISTING: /balance, /status, etc.

sultan-sdk/src/
├── lib.rs                  # Keep existing
├── native.rs               # Keep existing (balance, transfer, etc.)
├── contract.rs             # NEW: contract methods
└── types.rs                # Extend with contract types
```

**Q4 2026 (EVM):**
```
sultan-core/src/
├── rpc/
│   ├── evm.rs              # NEW: eth_* JSON-RPC handlers
│   └── evm_router.rs       # NEW: route to evm.sltn.io

sultan-sdk/src/
├── evm.rs                  # NEW: EVM helper methods (optional)
```

---

## Part 7: Backward Compatibility Guarantees

### What We Promise

1. **Existing endpoints will not change**
   - `/status`, `/balance`, `/tx`, etc. remain identical
   - No breaking changes to response formats

2. **Existing SDK methods will not change**
   - `get_balance_sltn()`, `send_sltn()`, etc. remain identical
   - We only ADD new methods, never modify existing ones

3. **Existing integrations will not break**
   - PWA wallet will continue to work
   - Block explorer will continue to work
   - Any current API consumers will continue to work

4. **Versioning if needed**
   - If we ever need breaking changes: `/v2/` prefix
   - Current API becomes `/v1/` (optional, redirects work)

### Migration Path for External Developers

```
Phase 1 (Now):
  Use https://rpc.sltn.io/* for all operations

Phase 2 (Q2 2026):
  Same as Phase 1, plus:
  Use https://rpc.sltn.io/contract/* for WASM contracts

Phase 3 (Q4 2026):
  Same as Phases 1-2, plus:
  Use https://evm.sltn.io for EVM JSON-RPC
```

---

## Part 8: Checklist for Contract Implementation

### Before Starting Contract Work (Q2 2026)

- [ ] Review this document
- [ ] Finalize RPC endpoint design
- [ ] Finalize SDK method signatures
- [ ] Update website code examples (don't deploy yet)
- [ ] Prepare TypeScript SDK package structure

### During Contract Implementation

- [ ] Implement `/contract/*` RPC endpoints in `sultan-core`
- [ ] Test endpoints with curl before SDK
- [ ] Implement Rust SDK contract methods
- [ ] Implement TypeScript SDK contract methods
- [ ] Add integration tests for SDK ↔ RPC

### After Contract Implementation

- [ ] Update website with new code examples
- [ ] Update docs/sdk.md with contract methods
- [ ] Update docs/RPC_SPECIFICATION.md with contract endpoints
- [ ] Publish updated TypeScript SDK to npm
- [ ] Announce to developer community

---

## Part 9: Key Decisions Already Made

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Add, don't replace | Extend existing SDK/RPC | Backward compatibility |
| Separate EVM endpoint | `evm.sltn.io` | Clean separation, Ethereum tooling expects standard |
| JSON responses | All endpoints return JSON | Consistency with current API |
| No API keys | Keep free, no authentication | Zero-fee philosophy |
| REST for contracts | POST /contract/execute | Matches current REST pattern |
| eth_* for EVM | Standard JSON-RPC 2.0 | MetaMask/Ethers.js compatibility |

---

## Summary

### The Answer: Do We Need to Redo SDK/RPC Work?

**No. We extend it.**

| Component | Redo? | Action |
|-----------|-------|--------|
| Current RPC endpoints | ❌ No | Keep as-is |
| Current SDK methods | ❌ No | Keep as-is |
| Website code examples | 🔧 Update | Add contract examples |
| Contract endpoints | ✅ New | Add `/contract/*` routes |
| Contract SDK methods | ✅ New | Add `contract_*()` methods |
| EVM endpoint | ✅ New | New `evm.sltn.io` |
| EVM SDK | ✅ New | Thin wrapper (users use ethers.js) |

### What This Means for Current Work

1. **PWA Wallet:** Continue as planned, will still work after contracts
2. **Block Explorer:** Continue as planned, will still work after contracts
3. **SDK repo (github.com/Wollnbergen/BUILD):** Will need extension in Q2 2026
4. **Website:** Will need code example updates in Q2 2026

**No current work needs to be redone. Plan ahead for extensions in Q2 2026.**

---

*Last Updated: December 15, 2025*
*Document Version: 1.0*
*Status: Reference for Q2 2026*
*Related: SMART_CONTRACT_DEVELOPMENT_NOTES.md*
