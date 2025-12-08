# Sultan SDK & RPC API Documentation

Production-ready SDK and RPC interface for building dApps, DEXs, wallets, and businesses on Sultan Chain.

## 🎯 Overview

Sultan provides two integration paths for third-party developers:

1. **Sultan SDK** - Native Rust library for backend services
2. **JSON-RPC API** - Standard Ethereum-compatible RPC for maximum compatibility

### 🌉 Native Cross-Chain Interoperability

Sultan connects to **100+ blockchains** through two mechanisms:

**IBC Protocol (Cosmos Ecosystem):**
- **30+ Production Chains** - Osmosis, Cosmos Hub, Celestia, dYdX, Injective, Akash, Juno, Secret, Kujira, Stride, etc.
- **IBC Transfer** - Zero-fee token transfers via `ibc-go/v10`
- **Interchain Accounts** - Control remote chain accounts
- **Light Client Verification** - Trustless cross-chain proofs

**Custom Bridges (Non-Cosmos Chains):**
- **Ethereum** - Full EVM compatibility via standard JSON-RPC
- **Solana** - Native gRPC bridge service
- **TON** - Native gRPC bridge service  
- **Bitcoin** - HTLC atomic swap bridge

### 💼 Wallet Support

**Use Phantom Wallet (Solana's leading wallet) as default!** No Sultan-specific wallet needed.

**Primary:** Phantom Wallet (recommended)
- Works natively with Sultan via Solana adapter
- Best mobile experience
- Telegram Mini App integration
- Support for SPL tokens + Sultan native tokens

**Alternative:** MetaMask/Ethereum wallets
- Add Sultan as custom network for EVM compatibility
- RPC URL: `http://localhost:8545`
- Chain ID: `1397969742` (hex: `0x534c544e`)
- Currency Symbol: SLTN

## 📦 Sultan SDK (Rust)

### Installation

Add to your `Cargo.toml`:
```toml
[dependencies]
sultan-chain = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

### Quick Start

```rust
use sultan_chain::sdk::SultanSDK;
use sultan_chain::config::ChainConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ChainConfig::default();
    let sdk = SultanSDK::new(config, None).await?;
    
    // Create wallets
    let alice = sdk.create_wallet("alice").await?;
    let bob = sdk.create_wallet("bob").await?;
    
    // Transfer with ZERO fees (on-chain)
    let tx_hash = sdk.transfer(&alice, &bob, 100).await?;
    
    // IBC transfer to Osmosis (also zero fees!)
    // sdk.ibc_transfer(&alice, "osmo1...", 50, "transfer/channel-0").await?;
    
    Ok(())
}
```

### API Reference

#### Wallet Operations

| Method | Description | Returns |
|--------|-------------|---------|
| `create_wallet(owner: &str)` | Create new wallet | `SdkResult<String>` (address) |
| `get_balance(address: &str)` | Query wallet balance | `SdkResult<i64>` |
| `list_wallets()` | List all wallets | `SdkResult<Vec<String>>` |

#### Transactions

| Method | Description | Returns |
|--------|-------------|---------|
| `transfer(from: &str, to: &str, amount: u64)` | Send tokens (zero fees) | `SdkResult<String>` (tx hash) |
| `get_transaction_count(address: &str)` | Get nonce/tx count | `SdkResult<u64>` |
| `mint_token(to: &str, amount: u64)` | Mint new tokens | `SdkResult<String>` |

#### Staking & Validators

| Method | Description | Returns |
|--------|-------------|---------|
| `stake(validator: &str, amount: u64)` | Stake tokens to validator | `SdkResult<bool>` |
| `validator_register(addr: &str, stake: u64)` | Register as validator | `SdkResult<String>` |
| `query_apy(is_validator: bool)` | Query current APY | `SdkResult<f64>` |

#### Governance

| Method | Description | Returns |
|--------|-------------|---------|
| `proposal_create(proposer, title, desc)` | Create proposal | `SdkResult<u64>` (proposal ID) |
| `proposal_get(id: u64)` | Get proposal details | `SdkResult<Value>` |
| `get_all_proposals()` | List all proposals | `SdkResult<Vec<Value>>` |
| `vote_on_proposal(id, voter, vote)` | Cast vote | `SdkResult<()>` |
| `votes_tally(proposal_id: u64)` | Get vote counts | `SdkResult<(u64, u64)>` |

#### Chain Queries

| Method | Description | Returns |
|--------|-------------|---------|
| `get_block_height()` | Current block number | `SdkResult<u64>` |

### Error Handling

All SDK methods return `SdkResult<T>` which wraps the custom `SdkError` type:

```rust
use sultan_chain::sdk_error::SdkError;

match sdk.transfer(&from, &to, amount).await {
    Ok(tx_hash) => println!("Success: {}", tx_hash),
    Err(SdkError::InsufficientBalance { required, available }) => {
        eprintln!("Need {} but only have {}", required, available);
    },
    Err(e) => eprintln!("Error: {}", e),
}
```

**Error Types:**
- `InsufficientBalance` - Not enough funds
- `InvalidAddress` - Malformed address
- `WalletNotFound` - Wallet doesn't exist
- `ProposalNotFound` - Proposal ID invalid
- `BelowMinimumStake` - Stake amount too low
- `LockPoisoned` - Internal concurrency error
- `BlockchainError` - Chain operation failed
- `InvalidAmount` - Zero or invalid amount
- `ValidatorExists` - Duplicate validator
- `TransactionFailed` - TX execution error

## 🌐 JSON-RPC API

Compatible with Ethereum tooling (Web3.js, ethers.js, Hardhat, etc.)

### Connection

```bash
# Default endpoint
http://localhost:8545
```

### Standard Ethereum Methods

#### Block Queries
- `eth_blockNumber` - Latest block number
- `eth_getBlockByNumber` - Get block by number
- `eth_getBlockByHash` - Get block by hash

#### Transaction Queries
- `eth_getTransactionByHash` - Get transaction
- `eth_getTransactionReceipt` - Get receipt
- `eth_getTransactionCount` - Get nonce

#### Account Queries
- `eth_getBalance` - Get account balance

#### Gas & Fees (Always Zero!)
- `eth_gasPrice` - Returns `0x0`
- `eth_estimateGas` - Returns `0x0`

#### Network Info
- `eth_chainId` - Returns `0x534c544e` ("SLTN")
- `net_version` - Returns `"1"`

### Sultan-Specific Methods

#### Validators
```json
{
  "method": "sultan_getValidators",
  "params": [],
  "id": 1
}
```

#### Staking Info
```json
{
  "method": "sultan_getStakingInfo",
  "params": [],
  "id": 1
}
```
Returns:
```json
{
  "total_staked": "1000000",
  "validators": [...],
  "apy": "13.33"
}
```

#### Governance
```json
{
  "method": "sultan_getProposals",
  "params": [],
  "id": 1
}
```

```json
{
  "method": "sultan_getProposal",
  "params": [123],
  "id": 1
}
```

## 💡 Examples

See `examples/` directory for complete working examples:

- `wallet_basic.rs` - Wallet creation, transfers, balances
- `governance.rs` - Proposal creation and voting
- `staking.rs` - Validator staking and rewards

Run examples:
```bash
cargo run --example wallet_basic
cargo run --example governance
cargo run --example staking
```

## 🔒 Security Notes

- All SDK operations are thread-safe (Arc + Mutex)
- No unwrap/panic in production code paths
- Comprehensive error types for all failure modes
- Zero-fee transactions guaranteed at protocol level

## 🚀 Production Features

- **Zero Fees** - No gas costs for any transaction
- **Instant Finality** - Fast block times (5 seconds default)
- **Type Safety** - Strong typing with proper error handling
- **Async/Await** - Native Rust async for high performance
- **Ethereum Compatible** - Use existing Web3 tooling + **MetaMask**
- **IBC Protocol Built-in** - Connect to 100+ Cosmos chains (Osmosis, Celestia, dYdX, etc.) with zero fees
- **Cross-Chain Native** - Custom bridges to Ethereum, Solana, TON, Bitcoin for non-Cosmos chains
- **No Custom Wallet Needed** - Works with MetaMask, Rainbow, Trust Wallet, Coinbase Wallet, WalletConnect
- **Interchain Accounts** - Control accounts on remote IBC chains directly from Sultan

## 📚 Integration Guides

### Building a Wallet

```rust
use sultan_chain::sdk::SultanSDK;

async fn wallet_demo() -> Result<(), Box<dyn std::error::Error>> {
    let sdk = SultanSDK::new(Default::default(), None).await?;
    
    // User registration
    let user_wallet = sdk.create_wallet("user123").await?;
    
    // Check balance
    let balance = sdk.get_balance(&user_wallet).await?;
    
    // Send payment (zero fees!)
    if balance >= 100 {
        let tx = sdk.transfer(&user_wallet, "merchant_addr", 100).await?;
        println!("Payment sent: {}", tx);
    }
    
    Ok(())
}
```

### Building a DEX

```rust
async fn dex_swap() -> Result<(), Box<dyn std::error::Error>> {
    let sdk = SultanSDK::new(Default::default(), None).await?;
    
    // User swaps token A for token B
    // Step 1: Transfer token A to liquidity pool
    sdk.transfer(&user, &pool_address, amount_a).await?;
    
    // Step 2: Mint token B to user (zero fees = profitable small trades!)
    sdk.mint_token(&user, amount_b).await?;
    
    Ok(())
}
```

### Building a DApp

Use standard Web3.js with Sultan RPC **and MetaMask**:

```javascript
const Web3 = require('web3');

// Option 1: Use MetaMask's provider
const web3 = new Web3(window.ethereum);
await window.ethereum.request({ method: 'eth_requestAccounts' });

// Option 2: Direct RPC connection
const web3 = new Web3('http://localhost:8545');

// Works exactly like Ethereum - but with ZERO fees!
const balance = await web3.eth.getBalance(address);
const tx = await web3.eth.sendTransaction({
  from: sender,
  to: recipient,
  value: amount,
  gas: 0,      // Zero gas!
  gasPrice: 0  // Zero fees!
});
```

**MetaMask Transaction Flow:**
1. User clicks "Connect Wallet" → MetaMask popup
2. User sends transaction → MetaMask signs
3. **Zero gas = No "estimated gas fee" confusion!**
4. Transaction confirmed in 5 seconds (Sultan block time)

## 🛠 Configuration

```rust
use sultan_chain::config::ChainConfig;

let config = ChainConfig {
    chain_id: "sultan-1".into(),
    gas_price: 0,              // Always zero
    block_time: 5,             // 5 second blocks
    max_block_size: 1000,      // 1000 transactions per block
    min_stake: 10000,          // Minimum validator stake
    inflation_rate: 0.08,      // 8% annual inflation
};
```

## 📄 License

Internal production artifact. See root project licensing.

## 🤝 Support

For integration support, contact the Sultan Chain team or open an issue in the main repository.
