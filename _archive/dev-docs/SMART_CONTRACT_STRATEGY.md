# Sultan L1 - Smart Contract Strategy

## 🎯 Decision: Which VM to Integrate?

### Current Status
- ✅ CosmWasm contracts exist in repo (`week2-smart-contracts/`, `third_party/cw-plus/`)
- ✅ Counter contract example implemented
- ⚠️ No VM integrated into sultan-core yet
- ⚠️ Contract deployment not enabled

---

## 📊 Option Comparison

### Option A: CosmWasm Only (Recommended for Phase 1)

**Pros:**
- ✅ Already have example contracts in repository
- ✅ Native Cosmos ecosystem compatibility
- ✅ Rust-based (same language as Sultan core)
- ✅ Battle-tested (Terra, Osmosis, Juno use it)
- ✅ IBC-compatible for cross-chain contracts
- ✅ Strong security (no reentrancy attacks)
- ✅ Deterministic gas metering

**Cons:**
- ❌ Smaller developer ecosystem than Ethereum
- ❌ Not compatible with Solidity contracts
- ❌ Can't use existing Ethereum tools (Hardhat, Remix)

**Implementation Complexity:** Medium (2-3 weeks)

**Dependencies:**
```toml
[dependencies]
cosmwasm-vm = "1.5"
cosmwasm-std = "1.5"
cosmwasm-storage = "1.5"
```

**Key Features:**
- Wasm bytecode execution
- Gas metering
- Storage abstraction
- Message passing between contracts
- Query/Execute separation

---

### Option B: EVM Only

**Pros:**
- ✅ Huge developer ecosystem (Solidity)
- ✅ Compatible with MetaMask, Remix, Hardhat
- ✅ Can deploy existing Ethereum contracts
- ✅ Familiar to most Web3 developers
- ✅ Rich tooling and libraries

**Cons:**
- ❌ No existing contracts in our repo
- ❌ Reentrancy vulnerabilities possible
- ❌ Less Cosmos-native
- ❌ Harder to integrate with IBC

**Implementation Complexity:** High (4-6 weeks)

**Dependencies:**
```toml
[dependencies]
revm = "3.5"
# or
evm = "0.41"
alloy-primitives = "0.5"
```

**Key Features:**
- Solidity contract support
- EVM opcode execution
- Ethereum RPC compatibility
- Web3.js / ethers.js support

---

### Option C: Both (CosmWasm + EVM)

**Pros:**
- ✅ Maximum flexibility
- ✅ Support both ecosystems
- ✅ Attract developers from both communities
- ✅ Cross-VM contract calls possible

**Cons:**
- ❌ Complex to maintain
- ❌ Double the security surface
- ❌ Longer development time
- ❌ More testing required

**Implementation Complexity:** Very High (8-12 weeks)

**Architecture:**
```
Sultan Core
├── CosmWasm VM (port 8081)
├── EVM (port 8082)
└── Bridge Layer (cross-VM calls)
```

---

## ✅ Recommendation: Start with CosmWasm

### Why CosmWasm First?

1. **Already Have Contracts**
   - `/workspaces/0xv7/week2-smart-contracts/contracts/counter_contract.rs`
   - `/workspaces/0xv7/third_party/cw-plus/` (battle-tested contracts)

2. **Cosmos Ecosystem Synergy**
   - Sultan already has IBC integration
   - CosmWasm contracts can use IBC
   - Natural fit for cosmos-sdk-integration branch

3. **Faster Time to Market**
   - 2-3 weeks vs 4-6 weeks for EVM
   - Can add EVM later if needed

4. **Security First**
   - No reentrancy attacks
   - Better gas metering
   - Simpler audit

---

## 🚀 CosmWasm Implementation Plan

### Phase 1: Core Integration (Week 1)

**Add Dependencies:**
```toml
# sultan-core/Cargo.toml
[dependencies]
cosmwasm-vm = "1.5"
cosmwasm-std = "1.5"
cosmwasm-storage = "1.5"
wasmer = "4.2"
schemars = "0.8"
```

**Create Wasm Runtime:**
```rust
// sultan-core/src/wasm_runtime.rs
use cosmwasm_vm::{Instance, VmError, Cache, Backend};

pub struct WasmRuntime {
    cache: Cache<DefaultApi>,
    contracts: HashMap<String, Vec<u8>>,
}

impl WasmRuntime {
    pub fn new() -> Result<Self> {
        let cache = unsafe {
            Cache::new("/var/lib/sultan/wasm-cache", 100)?
        };
        
        Ok(Self {
            cache,
            contracts: HashMap::new(),
        })
    }
    
    pub fn deploy_contract(
        &mut self,
        wasm_code: Vec<u8>,
        deployer: String,
    ) -> Result<String> {
        // Validate wasm
        // Store in contracts map
        // Return contract address
    }
    
    pub fn execute_contract(
        &self,
        contract_addr: String,
        msg: Vec<u8>,
        sender: String,
    ) -> Result<Response> {
        // Execute contract logic
    }
}
```

**Storage Layer:**
```rust
// sultan-core/src/contract_storage.rs
pub struct ContractStorage {
    db: Arc<RocksDB>,
}

impl Storage for ContractStorage {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.db.get(key).unwrap()
    }
    
    fn set(&mut self, key: &[u8], value: &[u8]) {
        self.db.put(key, value).unwrap();
    }
}
```

### Phase 2: RPC Integration (Week 2)

**Add Endpoints:**
```rust
// sultan-core/src/main.rs

// Deploy contract
let deploy_route = warp::path!("wasm" / "deploy")
    .and(warp::post())
    .and(warp::body::json())
    .map(|body: DeployRequest| {
        // Deploy wasm contract
    });

// Execute contract
let execute_route = warp::path!("wasm" / "execute" / String)
    .and(warp::post())
    .and(warp::body::json())
    .map(|addr: String, msg: ExecuteMsg| {
        // Execute contract
    });

// Query contract
let query_route = warp::path!("wasm" / "query" / String)
    .and(warp::get())
    .and(warp::query())
    .map(|addr: String, query: QueryMsg| {
        // Query contract state
    });
```

**API Endpoints:**
```bash
# Deploy contract
POST /wasm/deploy
{
  "code": "<base64-encoded-wasm>",
  "deployer": "sultan1abc...",
  "msg": {"count": 0}
}

# Execute contract
POST /wasm/execute/{contract_addr}
{
  "sender": "sultan1xyz...",
  "msg": {"increment": {}}
}

# Query contract
GET /wasm/query/{contract_addr}?msg={"get_count": {}}
```

### Phase 3: Testing & Examples (Week 3)

**Test Contracts:**
```bash
# Deploy counter contract
cd /workspaces/0xv7/week2-smart-contracts
cargo wasm

# Upload to chain
curl -X POST https://rpc.sltn.io/wasm/deploy \
  -d @counter_contract.wasm

# Execute increment
curl -X POST https://rpc.sltn.io/wasm/execute/contract1 \
  -d '{"sender":"alice","msg":{"increment":{}}}'

# Query state
curl https://rpc.sltn.io/wasm/query/contract1?msg={"get_count":{}}
# Response: {"count": 1}
```

**Example Contracts to Deploy:**
1. Counter (basic state)
2. CW20 Token (fungible tokens)
3. CW721 NFT (non-fungible tokens)
4. Simple DEX pool
5. Governance voting

---

## 📅 Implementation Timeline

| Week | Task | Deliverable |
|------|------|-------------|
| 1 | Core VM integration | WasmRuntime working |
| 1 | Storage layer | ContractStorage implemented |
| 2 | RPC endpoints | Deploy/Execute/Query working |
| 2 | Transaction integration | Contracts in blocks |
| 3 | Testing | 5 example contracts deployed |
| 3 | Documentation | Developer guide |

**Total: 3 weeks to production-ready CosmWasm support**

---

## 🔧 Integration Points

### Where to Add Wasm Support in Sultan Core

**1. Transaction Types:**
```rust
// sultan-core/src/types.rs
pub enum TransactionType {
    Transfer,
    Stake,
    Unstake,
    Governance,
    WasmDeploy,      // NEW
    WasmExecute,     // NEW
}
```

**2. Transaction Processing:**
```rust
// sultan-core/src/blockchain.rs
async fn apply_transaction(&mut self, tx: &Transaction) -> Result<()> {
    match tx.tx_type {
        TransactionType::WasmDeploy => {
            self.wasm_runtime.deploy_contract(tx.data)?;
        }
        TransactionType::WasmExecute => {
            self.wasm_runtime.execute_contract(tx.data)?;
        }
        // ... other types
    }
}
```

**3. State Management:**
```rust
// Contracts stored in separate keyspace
// rocksdb key: contract:{addr}:{key}
// Example: contract:sultan1abc123:count
```

---

## 💰 Gas & Fees

### Gas Metering
```rust
// sultan-core/src/wasm_runtime.rs
pub struct GasConfig {
    base_cost: u64,              // 1000 gas per contract call
    storage_write_cost: u64,     // 100 gas per byte
    storage_read_cost: u64,      // 10 gas per byte
    compute_cost: u64,           // 1 gas per wasm instruction
}

impl WasmRuntime {
    fn meter_gas(&self, operations: &[Operation]) -> u64 {
        // Calculate total gas used
    }
}
```

### Fee Structure
```
Sultan maintains ZERO fees for users
Gas costs are internal accounting only
No SLTN charged for contract deployment or execution
```

---

## 🔐 Security Considerations

### Wasm Validation
```rust
fn validate_wasm(code: &[u8]) -> Result<()> {
    // 1. Check wasm magic bytes
    // 2. Validate module structure
    // 3. Check for forbidden imports
    // 4. Verify memory limits
    // 5. Scan for known vulnerabilities
}
```

### Resource Limits
```rust
pub struct WasmConfig {
    max_contract_size: usize,    // 800 KB
    max_memory: usize,           // 64 MB
    max_gas: u64,                // 10M gas per transaction
    max_storage: usize,          // 100 MB per contract
}
```

### Audit Requirements
- [ ] CosmWasm VM security audit
- [ ] Example contracts audited
- [ ] Gas metering verified
- [ ] Storage limits tested
- [ ] Stress test with malicious contracts

---

## 📚 Developer Resources

### Documentation to Create
1. **Contract Development Guide**
   - How to write CosmWasm contracts
   - Sultan-specific features
   - Example templates

2. **Deployment Tutorial**
   - Step-by-step deployment
   - Testing locally
   - Deploying to mainnet

3. **API Reference**
   - All RPC endpoints
   - Request/response schemas
   - Error codes

4. **Migration Guide**
   - Porting from Ethereum
   - Porting from other Cosmos chains
   - Best practices

---

## 🎯 Success Metrics

### Phase 1 Complete When:
- [x] CosmWasm VM integrated
- [x] Storage layer working
- [x] Can deploy contracts via RPC
- [x] Can execute contract functions
- [x] Can query contract state
- [x] Counter contract example working

### Production Ready When:
- [ ] 10+ example contracts deployed
- [ ] Documentation complete
- [ ] Security audit passed
- [ ] Performance benchmarked (>1000 TPS)
- [ ] Developer tools available

---

## 🔮 Future: Adding EVM Support

Once CosmWasm is stable, add EVM as second VM:

```rust
pub enum VmType {
    CosmWasm,
    EVM,
}

pub struct MultiVmRuntime {
    cosmwasm: WasmRuntime,
    evm: EvmRuntime,
}

impl MultiVmRuntime {
    pub fn execute(&self, vm_type: VmType, tx: Transaction) {
        match vm_type {
            VmType::CosmWasm => self.cosmwasm.execute(tx),
            VmType::EVM => self.evm.execute(tx),
        }
    }
}
```

**Benefits of Multi-VM:**
- CosmWasm contracts can call EVM contracts
- EVM contracts can call CosmWasm contracts
- Best of both ecosystems

---

## ✅ Decision: CosmWasm First, EVM Later

**Immediate Action:** Integrate CosmWasm (3 weeks)
**Future Action:** Add EVM support (4-6 weeks after CosmWasm stable)

This gives us:
- ✅ Faster time to market
- ✅ Natural Cosmos integration
- ✅ Path to multi-VM future
- ✅ Leverage existing contracts in repo

Ready to start implementation after deployment is complete! 🚀
