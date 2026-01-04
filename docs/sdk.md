# Sultan L1 SDK Documentation

**Version:** 2.0  
**Updated:** January 1, 2026

---

## Overview

Build on Sultan L1 - a zero-fee, high-throughput blockchain with native token factory, DEX, governance, and cross-chain bridges.

### Why Sultan L1?

| Feature | Benefit |
|---------|---------|
| **Zero Fees** | All transactions FREE forever |
| **64,000+ TPS** | 16 shards × 4,000 TPS each |
| **2-Second Blocks** | Instant finality |
| **Native DEX** | Built-in AMM, no smart contracts needed |
| **Token Factory** | Create tokens in one API call |
| **Cross-Chain** | BTC, ETH, SOL, TON bridges |

---

## TypeScript Interfaces

All types for Sultan L1 integration. Copy these to your project for full type safety.

```typescript
// Core Types
export interface Wallet {
  privateKey: Uint8Array;
  publicKey: Uint8Array;
  address: string; // Bech32 "sultan1..." format
}

export interface Transaction {
  hash: string;
  from: string;
  to: string;
  amount: number; // atomic units (1 SLTN = 10^9)
  timestamp: number;
  nonce: number;
  signature?: string; // base64 encoded
  public_key?: string; // base64 encoded
  status?: 'pending' | 'confirmed' | 'failed';
  block_height?: number;
  tx_type?: 'transfer' | 'stake' | 'vote' | 'token_create' | 'swap' | 'bridge';
}

export interface TransactionRequest {
  tx: {
    from: string;
    to: string;
    amount: number;
    timestamp: number;
    nonce: number;
  };
  signature: string; // base64 encoded Ed25519 signature
  public_key: string; // base64 encoded Ed25519 public key
}

export interface Balance {
  address: string;
  balance: number; // atomic units
  nonce: number;
  tokens?: TokenBalance[];
}

export interface TokenBalance {
  token_id: string;
  symbol: string;
  balance: number;
  decimals: number;
}

// Staking Types
export interface Validator {
  operator_address: string; // "sultanvaloper1..."
  moniker: string;
  status: 'bonded' | 'unbonding' | 'unbonded';
  tokens: number; // total stake
  delegator_shares: number;
  commission_rate: number; // 0.0 - 1.0
  uptime: number; // percentage
  voting_power: number;
}

export interface Delegation {
  delegator: string;
  validator: string;
  amount: number;
  shares: number;
  rewards_pending: number;
}

export interface UnbondingEntry {
  creation_height: number;
  completion_time: number; // Unix timestamp
  amount: number;
}

export interface StakingReward {
  validator: string;
  amount: number;
  claimed_at?: number;
}

// Governance Types
export interface Proposal {
  id: number;
  title: string;
  description: string;
  proposer: string;
  status: 'voting' | 'passed' | 'rejected' | 'deposit_period';
  submit_time: number;
  voting_end_time: number;
  total_deposit: number;
  yes_votes: number;
  no_votes: number;
  abstain_votes: number;
  veto_votes: number;
}

export interface Vote {
  proposal_id: number;
  voter: string;
  option: 'yes' | 'no' | 'abstain' | 'no_with_veto';
  weight: number;
  timestamp: number;
}

// Token Factory Types
export interface Token {
  token_id: string;
  name: string;
  symbol: string;
  decimals: number;
  total_supply: number;
  max_supply?: number;
  creator: string;
  mintable: boolean;
  burnable: boolean;
  metadata_uri?: string;
}

export interface TokenCreateRequest {
  name: string;
  symbol: string;
  decimals: number;
  initial_supply: number;
  max_supply?: number;
  mintable?: boolean;
  burnable?: boolean;
  metadata_uri?: string;
}

// NFT Types
export interface NFT {
  token_id: string;
  collection: string;
  name: string;
  description?: string;
  image_uri: string;
  metadata_uri?: string;
  owner: string;
  creator: string;
  royalty_bps?: number; // basis points (100 = 1%)
  attributes?: NFTAttribute[];
}

export interface NFTAttribute {
  trait_type: string;
  value: string | number;
}

export interface NFTCollection {
  collection_id: string;
  name: string;
  symbol: string;
  creator: string;
  total_supply: number;
  max_supply?: number;
  royalty_bps: number;
}

// DEX Types
export interface LiquidityPool {
  pair_id: string; // e.g., "sltn-MTK"
  token_a: string;
  token_b: string;
  reserve_a: number;
  reserve_b: number;
  total_lp_tokens: number;
  fee_bps: number; // basis points (30 = 0.3%)
  volume_24h?: number;
}

export interface SwapQuote {
  input_token: string;
  output_token: string;
  input_amount: number;
  output_amount: number;
  price_impact: number; // percentage
  route: string[];
  minimum_received?: number;
}

export interface SwapRequest {
  pair_id: string;
  direction: 'a_to_b' | 'b_to_a';
  amount_in: number;
  min_amount_out: number;
  deadline?: number;
}

export interface LiquidityPosition {
  pair_id: string;
  owner: string;
  lp_tokens: number;
  share_of_pool: number; // percentage
  token_a_value: number;
  token_b_value: number;
}

// Bridge Types
export interface BridgeTransaction {
  tx_id: string;
  source_chain: 'bitcoin' | 'ethereum' | 'solana' | 'ton';
  source_tx: string;
  destination: string; // Sultan address
  amount: number;
  status: 'pending' | 'confirming' | 'completed' | 'failed';
  confirmations: number;
  required_confirmations: number;
  created_at: number;
  completed_at?: number;
}

export interface BridgeQuote {
  source_chain: string;
  source_amount: number;
  destination_amount: number;
  fee: number;
  estimated_time: number; // seconds
  rate: number;
}

export interface BridgeProof {
  proof_type: 'spv' | 'groth16' | 'solana' | 'ton';
  proof_data: string; // base64 encoded
  block_height: number;
  merkle_root: string;
}

// API Response Types
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: ApiError;
}

export interface ApiError {
  code: number;
  message: string;
  details?: string;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  limit: number;
  offset: number;
  has_more: boolean;
}

// WebSocket Types
export interface WSSubscription {
  jsonrpc: '2.0';
  id: number;
  method: 'subscribe' | 'unsubscribe';
  params: {
    channel: 'blocks' | 'txs' | 'dex' | 'validators';
    address?: string;
    pair_id?: string;
  };
}

export interface WSEvent<T = unknown> {
  jsonrpc: '2.0';
  method: 'subscription';
  params: {
    channel: string;
    data: T;
  };
}

export interface BlockEvent {
  height: number;
  hash: string;
  timestamp: number;
  tx_count: number;
  proposer: string;
}

export interface TxEvent {
  hash: string;
  from: string;
  to: string;
  amount: number;
  status: 'confirmed' | 'failed';
  block_height: number;
}
```

---

## Quick Start

### JavaScript/TypeScript

```bash
npm install @noble/ed25519 bech32
```

```typescript
import * as ed25519 from '@noble/ed25519';
import { bech32 } from 'bech32';

const RPC_URL = 'https://rpc.sltn.io';

// Generate a new wallet
async function createWallet() {
  const privateKey = ed25519.utils.randomPrivateKey();
  const publicKey = await ed25519.getPublicKey(privateKey);
  
  // Derive address from public key
  const hash = await crypto.subtle.digest('SHA-256', publicKey);
  const addressBytes = new Uint8Array(hash).slice(0, 20);
  const words = bech32.toWords(addressBytes);
  const address = bech32.encode('sultan', words);
  
  return { privateKey, publicKey, address };
}

// Get balance
async function getBalance(address: string) {
  const res = await fetch(`${RPC_URL}/balance/${address}`);
  const data = await res.json();
  return {
    balance: data.balance / 1e9, // Convert to SLTN
    nonce: data.nonce
  };
}

// Send transaction
async function sendTransaction(
  privateKey: Uint8Array,
  publicKey: Uint8Array,
  from: string,
  to: string,
  amount: number, // in SLTN
  nonce: number
) {
  const tx = {
    from,
    to,
    amount: Math.floor(amount * 1e9), // Convert to atomic
    timestamp: Math.floor(Date.now() / 1000),
    nonce
  };
  
  const message = new TextEncoder().encode(JSON.stringify(tx));
  const signature = await ed25519.sign(message, privateKey);
  
  const res = await fetch(`${RPC_URL}/tx`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      tx,
      signature: btoa(String.fromCharCode(...signature)),
      public_key: btoa(String.fromCharCode(...publicKey))
    })
  });
  
  return res.json();
}

// Example usage
async function main() {
  const wallet = await createWallet();
  console.log('Address:', wallet.address);
  
  const { balance, nonce } = await getBalance(wallet.address);
  console.log('Balance:', balance, 'SLTN');
  
  // Send 10 SLTN to another address
  const result = await sendTransaction(
    wallet.privateKey,
    wallet.publicKey,
    wallet.address,
    'sultan1recipient...',
    10,
    nonce
  );
  console.log('TX Hash:', result.hash);
}
```

---

## Core API Reference

### Network Status

```typescript
// GET /status
const status = await fetch(`${RPC_URL}/status`).then(r => r.json());
// {
//   "node_id": "sultan-validator-1",
//   "block_height": 125000,
//   "validators": 6,
//   "shard_count": 16,
//   "tps_capacity": 64000
// }
```

### Economics

```typescript
// GET /economics
const economics = await fetch(`${RPC_URL}/economics`).then(r => r.json());
// {
//   "total_supply": 500000000000000000,
//   "inflation_rate": 0.04,
//   "staking_apy": 0.1333,
//   "total_staked": 300000000000000000
// }
```

### Total Supply (for explorers)

```typescript
// GET /supply/total
const supply = await fetch(`${RPC_URL}/supply/total`).then(r => r.json());
// {
//   "total_supply_sltn": 500000000.0,
//   "circulating_supply_sltn": 500000000.0,
//   "decimals": 9,
//   "denom": "sltn"
// }
```

---

## Transaction API

### Get Transaction by Hash

```typescript
// GET /tx/{hash}
const tx = await fetch(`${RPC_URL}/tx/abc123...`).then(r => r.json());
// {
//   "hash": "abc123...",
//   "from": "sultan1...",
//   "to": "sultan1...",
//   "amount": 1000000000,
//   "block_height": 12345,
//   "status": "confirmed"
// }
```

### Get Transaction History

```typescript
// GET /transactions/{address}?limit=20
const history = await fetch(
  `${RPC_URL}/transactions/sultan1...?limit=20`
).then(r => r.json());
// {
//   "address": "sultan1...",
//   "transactions": [...],
//   "count": 20
// }
```

---

## Staking API

### List Validators

```typescript
// GET /staking/validators
const validators = await fetch(`${RPC_URL}/staking/validators`).then(r => r.json());
// {
//   "validators": [
//     {
//       "address": "sultanvaloper1...",
//       "moniker": "Validator One",
//       "stake": 50000000000000,
//       "voting_power": 16.67,
//       "commission": 0.05,
//       "status": "active"
//     }
//   ],
//   "total_validators": 6,
//   "total_stake": 300000000000000
// }
```

### Delegate to Validator

```typescript
async function delegate(
  privateKey: Uint8Array,
  publicKey: Uint8Array,
  delegator: string,
  validator: string,
  amount: number // in SLTN
) {
  const request = {
    delegator,
    validator,
    amount: Math.floor(amount * 1e9)
  };
  
  const message = new TextEncoder().encode(JSON.stringify(request));
  const signature = await ed25519.sign(message, privateKey);
  
  return fetch(`${RPC_URL}/staking/delegate`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      ...request,
      signature: btoa(String.fromCharCode(...signature)),
      public_key: btoa(String.fromCharCode(...publicKey))
    })
  }).then(r => r.json());
}

// Delegate 1000 SLTN to a validator
const result = await delegate(
  privateKey,
  publicKey,
  'sultan1myaddress...',
  'sultanvaloper1validator...',
  1000
);
```

### Get My Delegations

```typescript
// GET /staking/delegations/{address}
const delegations = await fetch(
  `${RPC_URL}/staking/delegations/sultan1...`
).then(r => r.json());
// {
//   "delegator": "sultan1...",
//   "delegations": [
//     {
//       "validator": "sultanvaloper1...",
//       "amount": 1000000000000,
//       "rewards_pending": 5000000000
//     }
//   ],
//   "total_delegated": 1000000000000
// }
```

### Claim Rewards

```typescript
async function claimRewards(
  privateKey: Uint8Array,
  publicKey: Uint8Array,
  delegator: string,
  validator: string
) {
  const request = { delegator, validator };
  const message = new TextEncoder().encode(JSON.stringify(request));
  const signature = await ed25519.sign(message, privateKey);
  
  return fetch(`${RPC_URL}/staking/withdraw_rewards`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      ...request,
      signature: btoa(String.fromCharCode(...signature)),
      public_key: btoa(String.fromCharCode(...publicKey))
    })
  }).then(r => r.json());
}
```

---

## Token Factory API

Create and manage custom tokens without smart contracts!

### Create a Token

```typescript
async function createToken(
  privateKey: Uint8Array,
  publicKey: Uint8Array,
  creator: string,
  name: string,
  symbol: string,
  decimals: number,
  totalSupply: number // in human-readable units
) {
  const request = {
    creator,
    name,
    symbol,
    decimals,
    total_supply: Math.floor(totalSupply * Math.pow(10, decimals))
  };
  
  const message = new TextEncoder().encode(JSON.stringify(request));
  const signature = await ed25519.sign(message, privateKey);
  
  return fetch(`${RPC_URL}/tokens/create`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      ...request,
      signature: btoa(String.fromCharCode(...signature)),
      public_key: btoa(String.fromCharCode(...publicKey))
    })
  }).then(r => r.json());
}

// Create "MyToken" with 1 million supply
const token = await createToken(
  privateKey,
  publicKey,
  'sultan1myaddress...',
  'My Token',
  'MTK',
  6,
  1000000
);
console.log('Token denom:', token.denom);
// factory/sultan1myaddress.../MTK
```

### Transfer Tokens

```typescript
async function transferToken(
  privateKey: Uint8Array,
  publicKey: Uint8Array,
  denom: string,
  from: string,
  to: string,
  amount: number,
  decimals: number
) {
  const request = {
    denom,
    from,
    to,
    amount: Math.floor(amount * Math.pow(10, decimals))
  };
  
  const message = new TextEncoder().encode(JSON.stringify(request));
  const signature = await ed25519.sign(message, privateKey);
  
  return fetch(`${RPC_URL}/tokens/transfer`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      ...request,
      signature: btoa(String.fromCharCode(...signature)),
      public_key: btoa(String.fromCharCode(...publicKey))
    })
  }).then(r => r.json());
}
```

### Get Token Balance

```typescript
// GET /tokens/{denom}/balance/{address}
// Note: denom must be URL-encoded
const denom = encodeURIComponent('factory/sultan1.../MTK');
const balance = await fetch(
  `${RPC_URL}/tokens/${denom}/balance/sultan1...`
).then(r => r.json());
// {
//   "denom": "factory/sultan1.../MTK",
//   "address": "sultan1...",
//   "balance": 100000000
// }
```

### List All Tokens

```typescript
// GET /tokens/list
const tokens = await fetch(`${RPC_URL}/tokens/list`).then(r => r.json());
// {
//   "tokens": [
//     { "denom": "factory/sultan1.../MTK", "name": "My Token", "symbol": "MTK" }
//   ],
//   "total": 156
// }
```

---

## DEX API

Sultan has a native AMM DEX with **zero fees**!

### Swap Tokens

```typescript
async function swap(
  privateKey: Uint8Array,
  publicKey: Uint8Array,
  user: string,
  inputDenom: string,
  outputDenom: string,
  inputAmount: number, // in atomic units
  minOutput: number    // slippage protection
) {
  const request = {
    user,
    input_denom: inputDenom,
    output_denom: outputDenom,
    input_amount: inputAmount,
    min_output: minOutput
  };
  
  const message = new TextEncoder().encode(JSON.stringify(request));
  const signature = await ed25519.sign(message, privateKey);
  
  return fetch(`${RPC_URL}/dex/swap`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      ...request,
      signature: btoa(String.fromCharCode(...signature)),
      public_key: btoa(String.fromCharCode(...publicKey))
    })
  }).then(r => r.json());
}

// Swap 100 SLTN for MTK with 5% slippage tolerance
const result = await swap(
  privateKey,
  publicKey,
  'sultan1myaddress...',
  'sltn',
  'factory/sultan1.../MTK',
  100_000_000_000, // 100 SLTN
  47_500_000_000   // Min 47.5 MTK (5% slippage)
);
console.log('Received:', result.output_amount);
```

### Get Pool Price

```typescript
// GET /dex/price/{pair_id}
const price = await fetch(`${RPC_URL}/dex/price/sltn-MTK`).then(r => r.json());
// {
//   "pair_id": "sltn-MTK",
//   "price_a_to_b": 0.5,  // 1 SLTN = 0.5 MTK
//   "price_b_to_a": 2.0,  // 1 MTK = 2 SLTN
//   "reserve_a": 1500000000000,
//   "reserve_b": 750000000000
// }
```

### List All Pools

```typescript
// GET /dex/pools
const pools = await fetch(`${RPC_URL}/dex/pools`).then(r => r.json());
// {
//   "pools": [
//     { "pair_id": "sltn-MTK", "reserve_a": 1500000000000, "volume_24h": 50000000000000 }
//   ],
//   "total": 25
// }
```

### Add Liquidity

```typescript
async function addLiquidity(
  privateKey: Uint8Array,
  publicKey: Uint8Array,
  user: string,
  pairId: string,
  amountA: number,
  amountB: number,
  minLpTokens: number
) {
  const request = {
    user,
    pair_id: pairId,
    amount_a: amountA,
    amount_b: amountB,
    min_lp_tokens: minLpTokens
  };
  
  const message = new TextEncoder().encode(JSON.stringify(request));
  const signature = await ed25519.sign(message, privateKey);
  
  return fetch(`${RPC_URL}/dex/add_liquidity`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      ...request,
      signature: btoa(String.fromCharCode(...signature)),
      public_key: btoa(String.fromCharCode(...publicKey))
    })
  }).then(r => r.json());
}
```

---

## Governance API

### List Proposals

```typescript
// GET /governance/proposals
const proposals = await fetch(`${RPC_URL}/governance/proposals`).then(r => r.json());
// {
//   "proposals": [
//     {
//       "id": 42,
//       "title": "Increase validator set",
//       "status": "voting",
//       "yes_votes": 150000000000000,
//       "no_votes": 20000000000000,
//       "voting_end": 1736294400
//     }
//   ],
//   "total": 42
// }
```

### Vote on Proposal

```typescript
async function vote(
  privateKey: Uint8Array,
  publicKey: Uint8Array,
  voter: string,
  proposalId: number,
  voteOption: 'yes' | 'no' | 'abstain' | 'no_with_veto'
) {
  const request = {
    voter,
    proposal_id: proposalId,
    vote: voteOption
  };
  
  const message = new TextEncoder().encode(JSON.stringify(request));
  const signature = await ed25519.sign(message, privateKey);
  
  return fetch(`${RPC_URL}/governance/vote`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      ...request,
      signature: btoa(String.fromCharCode(...signature)),
      public_key: btoa(String.fromCharCode(...publicKey))
    })
  }).then(r => r.json());
}

// Vote yes on proposal #42
await vote(privateKey, publicKey, 'sultan1...', 42, 'yes');
```

---

## Bridge API

### Check Bridge Status

```typescript
// GET /bridges
const bridges = await fetch(`${RPC_URL}/bridges`).then(r => r.json());
// {
//   "bridges": [
//     { "chain": "bitcoin", "status": "active", "pending_transactions": 3 },
//     { "chain": "ethereum", "status": "active", "pending_transactions": 12 },
//     { "chain": "solana", "status": "active", "pending_transactions": 5 },
//     { "chain": "ton", "status": "active", "pending_transactions": 2 }
//   ]
// }
```

### Get Bridge Fee Estimate

```typescript
// GET /bridge/{chain}/fee?amount=X
const fee = await fetch(
  `${RPC_URL}/bridge/ethereum/fee?amount=1000000000000000000`
).then(r => r.json());
// {
//   "chain": "ethereum",
//   "sultan_fee": 0,           // FREE on Sultan!
//   "external_fee_estimate": 25000000000000000,
//   "external_fee_usd": 5.50
// }
```

---

## Python SDK

```python
import requests
from nacl.signing import SigningKey
from nacl.encoding import RawEncoder
import json
import hashlib
import base64

RPC_URL = "https://rpc.sltn.io"

class SultanWallet:
    def __init__(self, private_key_hex: str = None):
        if private_key_hex:
            self.signing_key = SigningKey(bytes.fromhex(private_key_hex))
        else:
            self.signing_key = SigningKey.generate()
        
        self.public_key = self.signing_key.verify_key.encode()
        self.address = self._derive_address()
    
    def _derive_address(self) -> str:
        # SHA-256 hash of public key, take first 20 bytes
        hash_bytes = hashlib.sha256(self.public_key).digest()[:20]
        # Bech32 encode with "sultan" prefix
        from bech32 import bech32_encode, convertbits
        data = convertbits(list(hash_bytes), 8, 5)
        return bech32_encode("sultan", data)
    
    def sign(self, message: dict) -> tuple[str, str]:
        msg_bytes = json.dumps(message).encode()
        signed = self.signing_key.sign(msg_bytes, encoder=RawEncoder)
        signature = base64.b64encode(signed.signature).decode()
        pubkey = base64.b64encode(self.public_key).decode()
        return signature, pubkey

def get_balance(address: str) -> dict:
    res = requests.get(f"{RPC_URL}/balance/{address}")
    data = res.json()
    return {
        "balance": data["balance"] / 1e9,
        "nonce": data["nonce"]
    }

def send_transaction(wallet: SultanWallet, to: str, amount: float, nonce: int) -> dict:
    import time
    tx = {
        "from": wallet.address,
        "to": to,
        "amount": int(amount * 1e9),
        "timestamp": int(time.time()),
        "nonce": nonce
    }
    
    signature, pubkey = wallet.sign(tx)
    
    res = requests.post(f"{RPC_URL}/tx", json={
        "tx": tx,
        "signature": signature,
        "public_key": pubkey
    })
    return res.json()

# Example usage
if __name__ == "__main__":
    wallet = SultanWallet()
    print(f"Address: {wallet.address}")
    
    balance = get_balance(wallet.address)
    print(f"Balance: {balance['balance']} SLTN")
```

---

## Rust SDK

```rust
use ed25519_dalek::{SigningKey, Signer, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use bech32::{self, ToBase32, Variant};

const RPC_URL: &str = "https://rpc.sltn.io";

#[derive(Debug)]
pub struct Wallet {
    signing_key: SigningKey,
    pub public_key: VerifyingKey,
    pub address: String,
}

impl Wallet {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let signing_key = SigningKey::generate(&mut rng);
        let public_key = signing_key.verifying_key();
        
        // Derive address
        let mut hasher = Sha256::new();
        hasher.update(public_key.as_bytes());
        let hash = hasher.finalize();
        let addr_bytes = &hash[..20];
        let address = bech32::encode("sultan", addr_bytes.to_base32(), Variant::Bech32)
            .expect("bech32 encode failed");
        
        Self { signing_key, public_key, address }
    }
    
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.signing_key.sign(message).to_bytes().to_vec()
    }
}

#[derive(Serialize, Deserialize)]
pub struct BalanceResponse {
    pub address: String,
    pub balance: u128,
    pub nonce: u64,
}

pub async fn get_balance(address: &str) -> Result<BalanceResponse, reqwest::Error> {
    let url = format!("{}/balance/{}", RPC_URL, address);
    reqwest::get(&url).await?.json().await
}

#[derive(Serialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u128,
    pub timestamp: u64,
    pub nonce: u64,
}

pub async fn send_transaction(
    wallet: &Wallet,
    to: String,
    amount: u128,
    nonce: u64,
) -> Result<serde_json::Value, reqwest::Error> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let tx = Transaction {
        from: wallet.address.clone(),
        to,
        amount,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        nonce,
    };
    
    let tx_json = serde_json::to_string(&tx).unwrap();
    let signature = wallet.sign(tx_json.as_bytes());
    
    let client = reqwest::Client::new();
    client
        .post(format!("{}/tx", RPC_URL))
        .json(&serde_json::json!({
            "tx": tx,
            "signature": base64::encode(&signature),
            "public_key": base64::encode(wallet.public_key.as_bytes())
        }))
        .send()
        .await?
        .json()
        .await
}

#[tokio::main]
async fn main() {
    let wallet = Wallet::new();
    println!("Address: {}", wallet.address);
    
    let balance = get_balance(&wallet.address).await.unwrap();
    println!("Balance: {} SLTN", balance.balance as f64 / 1e9);
}
```

---

## Error Handling

All endpoints return errors in this format:

```json
{
  "error": "Description of the error",
  "status": 400
}
```

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `Invalid signature` | Signature verification failed | Ensure correct keypair |
| `Insufficient balance` | Not enough SLTN | Check balance first |
| `Invalid nonce` | Nonce mismatch | Query current nonce |
| `Rate limit exceeded` | Too many requests | Wait and retry |
| `Validator not found` | Invalid validator | Check validator list |
| `Token not found` | Invalid denom | Check token list |
| `Pair not found` | Invalid DEX pair | Check pool list |

### Error Handling Example

```typescript
async function safeRequest(url: string, options?: RequestInit) {
  const res = await fetch(url, options);
  const data = await res.json();
  
  if (data.error) {
    throw new Error(`Sultan API Error: ${data.error} (${data.status})`);
  }
  
  return data;
}

try {
  const result = await safeRequest(`${RPC_URL}/tx`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(txRequest)
  });
  console.log('Success:', result);
} catch (error) {
  console.error('Failed:', error.message);
}
```

---

## Best Practices

### 1. Always Check Nonce

```typescript
// Before sending a transaction, get the current nonce
const { nonce } = await getBalance(address);
// Use this nonce in your transaction
```

### 2. Handle Rate Limits

```typescript
async function withRetry(fn: () => Promise<any>, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await fn();
    } catch (e) {
      if (e.message.includes('Rate limit') && i < maxRetries - 1) {
        await new Promise(r => setTimeout(r, 5000));
        continue;
      }
      throw e;
    }
  }
}
```

### 3. Use Slippage Protection

```typescript
// Always set min_output for swaps
const price = await getPrice('sltn-MTK');
const expectedOutput = inputAmount * price.price_a_to_b;
const minOutput = expectedOutput * 0.95; // 5% slippage tolerance
```

### 4. Verify Transaction Confirmation

```typescript
async function waitForConfirmation(hash: string, timeout = 30000) {
  const start = Date.now();
  while (Date.now() - start < timeout) {
    const tx = await fetch(`${RPC_URL}/tx/${hash}`).then(r => r.json());
    if (tx.status === 'confirmed') return tx;
    await new Promise(r => setTimeout(r, 2000));
  }
  throw new Error('Transaction confirmation timeout');
}
```

---

## Support

- **Documentation:** https://docs.sltn.io
- **API Reference:** [API_REFERENCE.md](API_REFERENCE.md)
- **RPC Specification:** [RPC_SPECIFICATION.md](RPC_SPECIFICATION.md)
- **Discord:** https://discord.gg/sultanchain
- **GitHub:** https://github.com/sultanchain

---

**SDK Version:** 2.0  
**Last Updated:** January 1, 2026
