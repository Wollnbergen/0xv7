# Third-Party Developer Guide for Sultan L1

## 🎉 BUILD Repository is Live!

**Repository:** https://github.com/Wollnbergen/BUILD

Third-party developers can now build production applications on Sultan L1 using our official SDK and RPC server.

---

## 📦 What's Available

### 1. **Sultan SDK (Rust)**
- **Location:** `sdk.rs` in BUILD repo
- **Package:** `sultan-sdk` (version 1.0.0)
- **Features:**
  - HTTP RPC client for Sultan L1
  - Support for mainnet, testnet, and local networks
  - Account balance queries
  - Transaction submission
  - Validator staking
  - Reward calculations
  - Network status queries

### 2. **RPC Server API**
- **Documentation:** `RPC_SERVER.md` in BUILD repo
- **Mainnet RPC:** https://rpc.sultan.network
- **Mainnet REST:** https://api.sultan.network
- **Endpoints:**
  - `GET /status` - Network status
  - `GET /balance/:address` - Account balance
  - `POST /tx` - Submit transaction
  - `GET /block/:height` - Block data
  - Full Cosmos SDK REST API

### 3. **Complete Documentation**
- **README.md** - Quick start guide with examples
- **RPC_SERVER.md** - Full API reference
- **Examples** in multiple languages (Rust, JavaScript, Python, cURL)

---

## 🚀 What Can Third Parties Build?

### DApps (Decentralized Applications)
```rust
use sultan_sdk::SultanSDK;

let sdk = SultanSDK::new_mainnet().await?;
let balance = sdk.get_balance_sltn(user_address).await?;
// Build your DApp logic with zero fees!
```

**Use cases:**
- DeFi protocols
- NFT marketplaces
- Gaming platforms
- Social networks
- DAOs

### DEXs (Decentralized Exchanges)
```javascript
const response = await fetch('https://rpc.sultan.network/status');
const status = await response.json();
// Build trading pairs, liquidity pools, swaps
// All with $0 gas fees
```

**Use cases:**
- Token swaps
- Liquidity pools
- Yield farming
- Staking platforms

### Wallets
```rust
let sdk = SultanSDK::new_mainnet().await?;

// Check balance
let balance = sdk.get_balance("sultan1...").await?;

// Send transaction
let hash = sdk.send_transaction(tx).await?;
```

**Wallet types:**
- Mobile wallets (iOS/Android)
- Browser extensions
- Desktop applications
- Hardware wallet integrations

### Block Explorers
```python
import requests

status = requests.get('https://rpc.sultan.network/status').json()
print(f"Block height: {status['height']}")
print(f"Validators: {status['validator_count']}")
```

**Features:**
- Transaction history
- Block details
- Validator analytics
- Account tracking
- Network statistics

### Analytics Tools
```rust
let status = sdk.status().await?;
let validator_apy = sdk.query_validator_apy().await?; // 26.67%

// Build dashboards, charts, insights
```

**Analytics types:**
- Network health monitoring
- Validator performance tracking
- Transaction volume analysis
- Staking statistics

---

## 💰 Sultan L1 Economics

### Token Details
- **Symbol:** SLTN
- **Total Supply:** 500,000,000 SLTN
- **Decimals:** 6 (1 SLTN = 1,000,000 usltn)

### Zero Fees
- **Gas Cost:** $0.00
- **Transaction Fee:** $0.00
- **No hidden costs**

### Staking Rewards
- **Validator APY:** 26.67% (fixed)
- **Delegator APY:** 10%
- **Minimum Validator Stake:** 10,000 SLTN

### Calculation Example
```rust
// Validator with 10,000 SLTN stake
let yearly = sdk.calculate_rewards(10_000_000_000_000, true);
// Returns: 2,667,000,000,000 usltn (2,667 SLTN/year)

let monthly = sdk.calculate_monthly_rewards(10_000_000_000_000, true);
// Returns: ~222 SLTN/month

let daily = sdk.calculate_daily_rewards(10_000_000_000_000, true);
// Returns: ~7.3 SLTN/day
```

---

## 🏗️ Quick Start for Third Parties

### 1. Clone BUILD Repository
```bash
git clone https://github.com/Wollnbergen/BUILD.git
cd BUILD
```

### 2. Add SDK to Your Project
```toml
[dependencies]
sultan-sdk = { git = "https://github.com/Wollnbergen/BUILD.git" }
```

Or copy `sdk.rs` directly into your project.

### 3. Connect to Mainnet
```rust
use sultan_sdk::SultanSDK;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sdk = SultanSDK::new_mainnet().await?;
    
    // You're connected! Start building.
    let status = sdk.status().await?;
    println!("Connected to Sultan L1");
    println!("Block height: {}", status.height);
    
    Ok(())
}
```

### 4. Alternative: Direct HTTP API
Don't want to use Rust? Use the HTTP API directly:

```javascript
// JavaScript/TypeScript
const balance = await fetch('https://rpc.sultan.network/balance/sultan1...')
  .then(r => r.json());
console.log('Balance:', balance.balance / 1_000_000, 'SLTN');
```

```python
# Python
import requests
balance = requests.get('https://rpc.sultan.network/balance/sultan1...').json()
print(f"Balance: {balance['balance'] / 1_000_000} SLTN")
```

```bash
# cURL
curl https://rpc.sultan.network/status
```

---

## 🌐 Network Information

### Mainnet
- **Chain ID:** `sultan-1`
- **RPC:** `https://rpc.sultan.network` (port 26657)
- **REST:** `https://api.sultan.network` (port 1317)
- **Bech32 Prefix:** `sultan`
- **Coin Type:** `118` (Cosmos standard)

### Testnet
- **Chain ID:** `sultan-testnet-1`
- **RPC:** `https://rpc-testnet.sultan.network`
- **REST:** `https://api-testnet.sultan.network`

### Local Development
- **RPC:** `http://localhost:26657`
- **REST:** `http://localhost:1317`

---

## 📚 Example Projects

### Simple Balance Checker
```rust
use sultan_sdk::SultanSDK;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sdk = SultanSDK::new_mainnet().await?;
    let address = "sultan1abcdef...";
    
    let sltn = sdk.get_balance_sltn(address).await?;
    println!("{} has {} SLTN", address, sltn);
    
    Ok(())
}
```

### Validator Dashboard
```rust
use sultan_sdk::SultanSDK;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sdk = SultanSDK::new_mainnet().await?;
    
    // Get network stats
    let status = sdk.status().await?;
    println!("Active Validators: {}", status.validator_count);
    println!("Block Height: {}", status.height);
    
    // Show APY
    let apy = sdk.query_validator_apy().await?;
    println!("Validator APY: {:.2}%", apy * 100.0);
    
    // Calculate rewards for 10K SLTN stake
    let stake = 10_000_000_000_000; // 10,000 SLTN
    let yearly = sdk.calculate_rewards(stake, true);
    println!("Yearly Rewards: {} SLTN", yearly / 1_000_000.0);
    
    Ok(())
}
```

### Transaction Sender
```rust
use sultan_sdk::{SultanSDK, Transaction};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sdk = SultanSDK::new_mainnet().await?;
    
    let tx = Transaction {
        sender: "sultan1sender...".to_string(),
        recipient: "sultan1receiver...".to_string(),
        amount: 1_000_000_000, // 1000 SLTN
        nonce: 1,
        signature: vec![], // Sign with your private key
    };
    
    let hash = sdk.send_transaction(tx).await?;
    println!("Transaction submitted: {}", hash);
    
    Ok(())
}
```

---

## 🛠️ Development Tools

### Required Dependencies
```toml
[dependencies]
sultan-sdk = { git = "https://github.com/Wollnbergen/BUILD.git" }
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
```

### Testing on Testnet
Always test on testnet before deploying to mainnet:

```rust
// Use testnet for development
let sdk = SultanSDK::new_testnet().await?;

// Test your application
// ...

// Switch to mainnet when ready
let sdk = SultanSDK::new_mainnet().await?;
```

### Local Development Node
```bash
# Run a local Sultan node for development
./sultand start --rpc.laddr tcp://0.0.0.0:26657

# Connect your app to local node
let sdk = SultanSDK::new_local().await?;
```

---

## ✅ What Third Parties Get

1. **Production-Ready SDK** ✓
   - Fully tested Rust implementation
   - HTTP RPC client
   - Type-safe API
   - Error handling

2. **Complete Documentation** ✓
   - Quick start guide
   - API reference
   - Code examples
   - Multiple language support

3. **Zero-Fee Network** ✓
   - No gas costs
   - No transaction fees
   - Build without worrying about costs

4. **High Performance** ✓
   - Sub-50ms finality
   - Instant transactions
   - Great user experience

5. **High Rewards** ✓
   - 26.67% validator APY
   - Attracts users and validators
   - Sustainable economics

6. **Cosmos Ecosystem** ✓
   - IBC compatible
   - CosmWasm support
   - Standard Cosmos SDK

---

## 🤝 Support & Community

### Get Help
- **GitHub Issues:** https://github.com/Wollnbergen/BUILD/issues
- **Discord:** Join developer channel
- **Documentation:** Full API reference in RPC_SERVER.md

### Contribute
- Submit PRs to improve SDK
- Report bugs and issues
- Share your projects
- Help other developers

### Share Your Build
Built something cool? Let us know!
- Share on Discord
- Tweet with #SultanL1
- Add to ecosystem directory

---

## 🎯 Business Opportunities

### Build and Monetize
1. **DeFi Protocols** - Trading fees, yield farming
2. **NFT Marketplaces** - Listing fees, royalties
3. **Gaming Platforms** - In-game purchases
4. **Premium Analytics** - Subscription services
5. **White-Label Solutions** - License your software

### Competitive Advantages
- **Zero fees** = Better margins
- **Fast finality** = Better UX
- **High APY** = More users
- **Simple API** = Faster development

---

## 🚀 Start Building Today!

Everything you need is now available:

1. ✅ **SDK:** Production-ready Rust SDK
2. ✅ **RPC Server:** Live at https://rpc.sultan.network
3. ✅ **Documentation:** Complete API reference
4. ✅ **Examples:** Multiple languages
5. ✅ **Network:** Mainnet running with validators
6. ✅ **Economics:** 26.67% APY, zero fees

**No barriers. No limitations. Just build.**

Visit https://github.com/Wollnbergen/BUILD and start coding!

---

## 📄 License

MIT License - Free for commercial use

Build anything, make money, no restrictions.

---

**The Sultan L1 ecosystem is open for business.** 🏰
