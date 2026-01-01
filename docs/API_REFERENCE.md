# Sultan L1 API Reference

**Version:** 2.0  
**Updated:** January 1, 2026

## Base URL

| Environment | URL |
|-------------|-----|
| **Production** | `https://rpc.sltn.io` |
| **Testnet** | `https://testnet.sltn.io` |

## Authentication

**No API keys required!** Sultan L1 has zero fees and open access.

## Rate Limiting

- **Default:** 100 requests per 10 seconds per IP
- **Bridge endpoints:** 50 requests per minute per pubkey (anti-spam)

---

# Table of Contents

1. [Core Endpoints](#core-endpoints)
2. [Account Endpoints](#account-endpoints)
3. [Transaction Endpoints](#transaction-endpoints)
4. [Block Endpoints](#block-endpoints)
5. [Staking Endpoints](#staking-endpoints)
6. [Governance Endpoints](#governance-endpoints)
7. [Token Factory Endpoints](#token-factory-endpoints)
8. [DEX Endpoints](#dex-endpoints)
9. [Bridge Endpoints](#bridge-endpoints)
10. [Error Handling](#error-handling)
11. [Code Examples](#code-examples)

---

# Core Endpoints

## GET /status

Get current node and network status.

**Response:**
```json
{
  "node_id": "sultan-validator-1",
  "block_height": 125000,
  "validators": 12,
  "uptime_seconds": 864000,
  "version": "1.0.0",
  "shard_count": 16,
  "tps_capacity": 64000
}
```

---

## GET /supply/total

Get total and circulating supply (for block explorers like CoinGecko/CoinMarketCap).

**Response:**
```json
{
  "total_supply": 500000000000000000,
  "total_supply_sltn": 500000000.0,
  "circulating_supply": 500000000000000000,
  "circulating_supply_sltn": 500000000.0,
  "genesis_supply": 500000000000000000,
  "genesis_supply_sltn": 500000000.0,
  "total_burned": 0,
  "decimals": 9,
  "denom": "sltn"
}
```

> **Note:** `*_sltn` fields are human-readable (divided by 10^9). Raw fields are in atomic units (1 SLTN = 1,000,000,000 atomic units).

---

## GET /economics

Get tokenomics and staking economics.

**Response:**
```json
{
  "total_supply": 500000000000000000,
  "circulating_supply": 500000000000000000,
  "inflation_rate": 0.04,
  "staking_apy": 0.1333,
  "total_staked": 60000000000000000
}
```

---

# Account Endpoints

## GET /balance/{address}

Get account balance and nonce.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `address` | string | Sultan address (sultan1...) |

**Response:**
```json
{
  "address": "sultan15g5e8xyz...",
  "balance": 500000000000000000,
  "nonce": 5
}
```

---

# Transaction Endpoints

## POST /tx

Submit a signed transaction.

**Request Body (Wallet Format - Recommended):**
```json
{
  "tx": {
    "from": "sultan15g5e8...",
    "to": "sultan1abc123...",
    "amount": 1000000000,
    "timestamp": 1735689600,
    "nonce": 0
  },
  "signature": "base64_encoded_ed25519_signature",
  "public_key": "base64_encoded_ed25519_pubkey"
}
```

**Request Body (Simple Format):**
```json
{
  "from": "sultan15g5e8...",
  "to": "sultan1abc123...",
  "amount": 1000000000,
  "gas_fee": 0,
  "timestamp": 1735689600,
  "nonce": 0,
  "signature": "base64_encoded_signature"
}
```

**Response:**
```json
{
  "hash": "abc123def456..."
}
```

**Error Response:**
```json
{
  "error": "Invalid signature",
  "status": 400
}
```

---

## GET /tx/{hash}

Get transaction by hash.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `hash` | string | Transaction hash |

**Response:**
```json
{
  "hash": "abc123def456...",
  "from": "sultan15g5e8...",
  "to": "sultan1abc123...",
  "amount": 1000000000,
  "memo": "Payment for services",
  "nonce": 0,
  "timestamp": 1735689600,
  "block_height": 12345,
  "status": "confirmed"
}
```

---

## GET /transactions/{address}

Get transaction history for an address.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `address` | string | Sultan address |

**Query Parameters:**
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | integer | 50 | Max transactions to return |

**Response:**
```json
{
  "address": "sultan15g5e8...",
  "transactions": [
    {
      "hash": "abc123def456...",
      "from": "sultan15g5e8...",
      "to": "sultan1abc123...",
      "amount": 1000000000,
      "nonce": 0,
      "timestamp": 1735689600,
      "block_height": 12345,
      "status": "confirmed"
    }
  ],
  "count": 1
}
```

---

# Block Endpoints

## GET /block/{height}

Get block by height.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `height` | integer | Block height |

**Response:**
```json
{
  "height": 12345,
  "hash": "blockhash123...",
  "timestamp": 1735689600,
  "transactions": 25,
  "proposer": "sultan_validator_1",
  "shard_id": 0
}
```

---

# Staking Endpoints

## POST /staking/create_validator

Register as a validator. Requires minimum 10,000 SLTN stake.

**Request Body:**
```json
{
  "moniker": "My Validator",
  "pubkey": "base64_encoded_ed25519_pubkey",
  "stake_amount": 10000000000000,
  "commission_rate": 0.05,
  "signature": "base64_encoded_signature"
}
```

**Response:**
```json
{
  "validator_address": "sultanvaloper1...",
  "status": "active",
  "stake": 10000000000000
}
```

---

## POST /staking/delegate

Delegate SLTN to a validator.

**Request Body:**
```json
{
  "delegator": "sultan15g5e8...",
  "validator": "sultanvaloper1...",
  "amount": 1000000000000,
  "signature": "base64_encoded_signature",
  "public_key": "base64_encoded_pubkey"
}
```

**Response:**
```json
{
  "delegator": "sultan15g5e8...",
  "validator": "sultanvaloper1...",
  "amount": 1000000000000,
  "status": "delegated"
}
```

---

## POST /staking/undelegate

Undelegate (unstake) SLTN. Starts 21-day unbonding period.

**Request Body:**
```json
{
  "delegator": "sultan15g5e8...",
  "validator": "sultanvaloper1...",
  "amount": 500000000000,
  "signature": "base64_encoded_signature",
  "public_key": "base64_encoded_pubkey"
}
```

**Response:**
```json
{
  "delegator": "sultan15g5e8...",
  "validator": "sultanvaloper1...",
  "amount": 500000000000,
  "completion_time": 1737504000
}
```

---

## POST /staking/withdraw_rewards

Claim staking rewards.

**Request Body:**
```json
{
  "delegator": "sultan15g5e8...",
  "validator": "sultanvaloper1...",
  "signature": "base64_encoded_signature",
  "public_key": "base64_encoded_pubkey"
}
```

**Response:**
```json
{
  "delegator": "sultan15g5e8...",
  "rewards_claimed": 50000000000,
  "status": "success"
}
```

---

## GET /staking/validators

List all active validators.

**Response:**
```json
{
  "validators": [
    {
      "address": "sultanvaloper1...",
      "moniker": "Validator One",
      "stake": 50000000000000,
      "voting_power": 16.67,
      "commission": 0.05,
      "status": "active",
      "delegator_count": 150
    }
  ],
  "total_validators": 12,
  "total_stake": 300000000000000
}
```

---

## GET /staking/delegations/{address}

Get delegations for an address.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `address` | string | Delegator address |

**Response:**
```json
{
  "delegator": "sultan15g5e8...",
  "delegations": [
    {
      "validator": "sultanvaloper1...",
      "amount": 1000000000000,
      "rewards_pending": 5000000000
    }
  ],
  "total_delegated": 1000000000000
}
```

---

## GET /staking/statistics

Get network-wide staking statistics.

**Response:**
```json
{
  "total_staked": 300000000000000,
  "total_delegators": 5000,
  "validator_count": 12,
  "average_commission": 0.05,
  "current_apy": 0.1333,
  "unbonding_period_days": 21
}
```

---

# Governance Endpoints

## POST /governance/propose

Submit a governance proposal. Requires 1,000 SLTN deposit.

**Request Body:**
```json
{
  "proposer": "sultan15g5e8...",
  "title": "Increase validator set to 21",
  "description": "This proposal increases the active validator set from 12 to 21...",
  "proposal_type": "parameter_change",
  "deposit": 1000000000000,
  "signature": "base64_encoded_signature",
  "public_key": "base64_encoded_pubkey"
}
```

**Proposal Types:**
- `parameter_change` - Change network parameters
- `text` - Signaling proposal (no on-chain effect)
- `spend` - Treasury spend proposal
- `slash` - Validator slashing proposal

**Response:**
```json
{
  "proposal_id": 42,
  "status": "voting",
  "voting_end": 1736294400
}
```

---

## POST /governance/vote

Vote on an active proposal.

**Request Body:**
```json
{
  "voter": "sultan15g5e8...",
  "proposal_id": 42,
  "vote": "yes",
  "signature": "base64_encoded_signature",
  "public_key": "base64_encoded_pubkey"
}
```

**Vote Options:** `yes`, `no`, `abstain`, `no_with_veto`

**Response:**
```json
{
  "voter": "sultan15g5e8...",
  "proposal_id": 42,
  "vote": "yes",
  "voting_power": 1000000000000
}
```

---

## GET /governance/proposals

List all proposals.

**Response:**
```json
{
  "proposals": [
    {
      "id": 42,
      "title": "Increase validator set to 21",
      "proposer": "sultan15g5e8...",
      "status": "voting",
      "yes_votes": 150000000000000,
      "no_votes": 20000000000000,
      "abstain_votes": 5000000000000,
      "veto_votes": 0,
      "voting_end": 1736294400
    }
  ],
  "total": 42
}
```

---

## GET /governance/proposal/{id}

Get proposal details.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | integer | Proposal ID |

**Response:**
```json
{
  "id": 42,
  "title": "Increase validator set to 21",
  "description": "This proposal increases the active validator set...",
  "proposer": "sultan15g5e8...",
  "proposal_type": "parameter_change",
  "status": "voting",
  "deposit": 1000000000000,
  "yes_votes": 150000000000000,
  "no_votes": 20000000000000,
  "abstain_votes": 5000000000000,
  "veto_votes": 0,
  "submit_time": 1735689600,
  "voting_start": 1735689600,
  "voting_end": 1736294400
}
```

---

## POST /governance/tally/{id}

Tally votes and execute proposal (if passed).

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | integer | Proposal ID |

**Response:**
```json
{
  "proposal_id": 42,
  "result": "passed",
  "executed": true,
  "final_yes": 150000000000000,
  "final_no": 20000000000000,
  "turnout": 0.583
}
```

---

## GET /governance/statistics

Get governance statistics.

**Response:**
```json
{
  "total_proposals": 42,
  "active_proposals": 3,
  "passed_proposals": 35,
  "rejected_proposals": 4,
  "average_turnout": 0.65,
  "total_deposits": 42000000000000
}
```

---

# Token Factory Endpoints

Create custom tokens directly on Sultan L1 - no smart contracts needed!

## POST /tokens/create

Create a new token.

**Request Body:**
```json
{
  "creator": "sultan15g5e8...",
  "name": "My Token",
  "symbol": "MTK",
  "decimals": 6,
  "total_supply": 1000000000000,
  "signature": "base64_encoded_signature",
  "public_key": "base64_encoded_pubkey"
}
```

**Response:**
```json
{
  "denom": "factory/sultan15g5e8.../MTK",
  "name": "My Token",
  "symbol": "MTK",
  "decimals": 6,
  "total_supply": 1000000000000,
  "creator": "sultan15g5e8..."
}
```

---

## POST /tokens/mint

Mint additional tokens (creator only).

**Request Body:**
```json
{
  "denom": "factory/sultan15g5e8.../MTK",
  "amount": 500000000000,
  "recipient": "sultan1abc123...",
  "signature": "base64_encoded_signature",
  "public_key": "base64_encoded_pubkey"
}
```

**Response:**
```json
{
  "denom": "factory/sultan15g5e8.../MTK",
  "amount_minted": 500000000000,
  "recipient": "sultan1abc123...",
  "new_total_supply": 1500000000000
}
```

---

## POST /tokens/transfer

Transfer tokens.

**Request Body:**
```json
{
  "denom": "factory/sultan15g5e8.../MTK",
  "from": "sultan15g5e8...",
  "to": "sultan1abc123...",
  "amount": 100000000,
  "signature": "base64_encoded_signature",
  "public_key": "base64_encoded_pubkey"
}
```

**Response:**
```json
{
  "tx_hash": "abc123...",
  "denom": "factory/sultan15g5e8.../MTK",
  "from": "sultan15g5e8...",
  "to": "sultan1abc123...",
  "amount": 100000000
}
```

---

## POST /tokens/burn

Burn tokens (reduce supply).

**Request Body:**
```json
{
  "denom": "factory/sultan15g5e8.../MTK",
  "amount": 50000000,
  "burner": "sultan15g5e8...",
  "signature": "base64_encoded_signature",
  "public_key": "base64_encoded_pubkey"
}
```

**Response:**
```json
{
  "denom": "factory/sultan15g5e8.../MTK",
  "amount_burned": 50000000,
  "new_total_supply": 1450000000000
}
```

---

## GET /tokens/{denom}/metadata

Get token metadata.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `denom` | string | Token denomination (URL encoded) |

**Response:**
```json
{
  "denom": "factory/sultan15g5e8.../MTK",
  "name": "My Token",
  "symbol": "MTK",
  "decimals": 6,
  "total_supply": 1450000000000,
  "creator": "sultan15g5e8...",
  "created_at": 1735689600
}
```

---

## GET /tokens/{denom}/balance/{address}

Get token balance for an address.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `denom` | string | Token denomination (URL encoded) |
| `address` | string | Account address |

**Response:**
```json
{
  "denom": "factory/sultan15g5e8.../MTK",
  "address": "sultan1abc123...",
  "balance": 100000000
}
```

---

## GET /tokens/list

List all tokens.

**Response:**
```json
{
  "tokens": [
    {
      "denom": "factory/sultan15g5e8.../MTK",
      "name": "My Token",
      "symbol": "MTK",
      "total_supply": 1450000000000
    }
  ],
  "total": 156
}
```

---

# DEX Endpoints

Sultan L1 has a native AMM DEX built into the protocol.

## POST /dex/create_pair

Create a new trading pair.

**Request Body:**
```json
{
  "creator": "sultan15g5e8...",
  "token_a": "sltn",
  "token_b": "factory/sultan15g5e8.../MTK",
  "initial_a": 1000000000000,
  "initial_b": 500000000000,
  "signature": "base64_encoded_signature",
  "public_key": "base64_encoded_pubkey"
}
```

**Response:**
```json
{
  "pair_id": "sltn-MTK",
  "token_a": "sltn",
  "token_b": "factory/sultan15g5e8.../MTK",
  "reserve_a": 1000000000000,
  "reserve_b": 500000000000,
  "lp_token": "lp-sltn-MTK"
}
```

---

## POST /dex/swap

Execute a swap.

**Request Body:**
```json
{
  "user": "sultan15g5e8...",
  "input_denom": "sltn",
  "output_denom": "factory/sultan15g5e8.../MTK",
  "input_amount": 100000000000,
  "min_output": 45000000000,
  "signature": "base64_encoded_signature",
  "public_key": "base64_encoded_pubkey"
}
```

**Response:**
```json
{
  "tx_hash": "abc123...",
  "input_denom": "sltn",
  "output_denom": "factory/sultan15g5e8.../MTK",
  "input_amount": 100000000000,
  "output_amount": 48500000000,
  "price_impact": 0.015,
  "fee": 0
}
```

> **Note:** Sultan DEX has **zero fees** - no swap fees!

---

## POST /dex/add_liquidity

Add liquidity to a pool.

**Request Body:**
```json
{
  "user": "sultan15g5e8...",
  "pair_id": "sltn-MTK",
  "amount_a": 500000000000,
  "amount_b": 250000000000,
  "min_lp_tokens": 350000000000,
  "signature": "base64_encoded_signature",
  "public_key": "base64_encoded_pubkey"
}
```

**Response:**
```json
{
  "pair_id": "sltn-MTK",
  "deposited_a": 500000000000,
  "deposited_b": 250000000000,
  "lp_tokens_minted": 353553390593,
  "share_of_pool": 0.0353
}
```

---

## POST /dex/remove_liquidity

Remove liquidity from a pool.

**Request Body:**
```json
{
  "user": "sultan15g5e8...",
  "pair_id": "sltn-MTK",
  "lp_tokens": 100000000000,
  "min_a": 140000000000,
  "min_b": 70000000000,
  "signature": "base64_encoded_signature",
  "public_key": "base64_encoded_pubkey"
}
```

**Response:**
```json
{
  "pair_id": "sltn-MTK",
  "lp_tokens_burned": 100000000000,
  "received_a": 141421356237,
  "received_b": 70710678118
}
```

---

## GET /dex/pool/{pair_id}

Get pool information.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `pair_id` | string | Trading pair ID (e.g., "sltn-MTK") |

**Response:**
```json
{
  "pair_id": "sltn-MTK",
  "token_a": "sltn",
  "token_b": "factory/sultan15g5e8.../MTK",
  "reserve_a": 1500000000000,
  "reserve_b": 750000000000,
  "total_lp_tokens": 1060660171780,
  "volume_24h": 50000000000000,
  "fee_rate": 0
}
```

---

## GET /dex/pools

List all trading pools.

**Response:**
```json
{
  "pools": [
    {
      "pair_id": "sltn-MTK",
      "reserve_a": 1500000000000,
      "reserve_b": 750000000000,
      "volume_24h": 50000000000000
    }
  ],
  "total": 25
}
```

---

## GET /dex/price/{pair_id}

Get current price for a trading pair.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `pair_id` | string | Trading pair ID |

**Response:**
```json
{
  "pair_id": "sltn-MTK",
  "price_a_to_b": 0.5,
  "price_b_to_a": 2.0,
  "reserve_a": 1500000000000,
  "reserve_b": 750000000000
}
```

---

# Bridge Endpoints

Cross-chain bridges for Bitcoin, Ethereum, Solana, and TON.

## GET /bridges

List all bridge statuses.

**Response:**
```json
{
  "bridges": [
    {
      "chain": "bitcoin",
      "status": "active",
      "pending_transactions": 3,
      "total_locked": 15000000000
    },
    {
      "chain": "ethereum",
      "status": "active",
      "pending_transactions": 12,
      "total_locked": 250000000000000
    },
    {
      "chain": "solana",
      "status": "active",
      "pending_transactions": 5,
      "total_locked": 1000000000000
    },
    {
      "chain": "ton",
      "status": "active",
      "pending_transactions": 2,
      "total_locked": 50000000000
    }
  ]
}
```

---

## GET /bridge/{chain}

Get bridge status for a specific chain.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `chain` | string | Chain name: `bitcoin`, `ethereum`, `solana`, `ton` |

**Response:**
```json
{
  "chain": "ethereum",
  "status": "active",
  "pending_transactions": 12,
  "total_locked": 250000000000000,
  "proof_type": "zk-snark",
  "confirmations_required": 15,
  "avg_finality_seconds": 180
}
```

---

## POST /bridge/submit

Submit a bridge transaction.

**Request Body:**
```json
{
  "source_chain": "ethereum",
  "destination_chain": "sultan",
  "source_tx_hash": "0xabc123...",
  "amount": 1000000000000000000,
  "recipient": "sultan15g5e8...",
  "proof": "base64_encoded_proof",
  "signature": "base64_encoded_signature",
  "public_key": "base64_encoded_pubkey"
}
```

**Response:**
```json
{
  "bridge_tx_id": "bridge-123...",
  "status": "pending_confirmation",
  "source_chain": "ethereum",
  "source_tx_hash": "0xabc123...",
  "estimated_completion": 1735693200
}
```

> **Security:** Transactions >100,000 SLTN require multi-signature approval (2-of-3).

---

## GET /bridge/{chain}/fee

Get estimated bridge fee (external chain fees only - Sultan side is FREE).

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `chain` | string | Chain name |

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `amount` | integer | Amount to bridge |

**Response:**
```json
{
  "chain": "ethereum",
  "sultan_fee": 0,
  "external_fee_estimate": 25000000000000000,
  "external_fee_usd": 5.50,
  "total_fee": 25000000000000000
}
```

---

## GET /bridge/fees/treasury

Get bridge fee treasury information.

**Response:**
```json
{
  "treasury_address": "sultan_treasury...",
  "total_collected": 0,
  "governance_required": true,
  "multi_sig_threshold": "3-of-5"
}
```

---

## GET /bridge/fees/statistics

Get bridge fee statistics.

**Response:**
```json
{
  "total_bridges": 15420,
  "total_volume": 5000000000000000,
  "fees_collected": 0,
  "bridges_by_chain": {
    "bitcoin": 2500,
    "ethereum": 8000,
    "solana": 3500,
    "ton": 1420
  }
}
```

---

# Error Handling

## Error Response Format

All errors follow this format:

```json
{
  "error": "Description of the error",
  "status": 400
}
```

## HTTP Status Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 400 | Bad Request - Invalid parameters or signature |
| 404 | Not Found - Resource doesn't exist |
| 429 | Rate Limited - Too many requests |
| 500 | Internal Server Error |

## Common Errors

| Error Message | Cause | Solution |
|---------------|-------|----------|
| `Invalid signature` | Ed25519 signature verification failed | Check signing key matches from address |
| `Insufficient balance` | Not enough SLTN for transaction | Ensure balance > amount |
| `Invalid nonce` | Nonce doesn't match expected | Query `/balance/{address}` for current nonce |
| `Rate limit exceeded` | Too many requests | Wait and retry |
| `Validator not found` | Invalid validator address | Check `/staking/validators` for valid addresses |

---

# Code Examples

## JavaScript/TypeScript

### Query Balance
```javascript
const response = await fetch('https://rpc.sltn.io/balance/sultan15g5e8...');
const { balance, nonce } = await response.json();
console.log(`Balance: ${balance / 1e9} SLTN, Nonce: ${nonce}`);
```

### Submit Transaction
```javascript
import * as ed25519 from '@noble/ed25519';

const tx = {
  from: 'sultan15g5e8...',
  to: 'sultan1abc123...',
  amount: 1000000000, // 1 SLTN
  timestamp: Date.now(),
  nonce: 0
};

const message = JSON.stringify(tx);
const signature = await ed25519.sign(message, privateKey);

const response = await fetch('https://rpc.sltn.io/tx', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    tx,
    signature: btoa(String.fromCharCode(...signature)),
    public_key: btoa(String.fromCharCode(...publicKey))
  })
});

const { hash } = await response.json();
console.log(`Transaction hash: ${hash}`);
```

### Create Token
```javascript
const tokenRequest = {
  creator: 'sultan15g5e8...',
  name: 'My Token',
  symbol: 'MTK',
  decimals: 6,
  total_supply: 1000000000000,
  signature: '...',
  public_key: '...'
};

const response = await fetch('https://rpc.sltn.io/tokens/create', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify(tokenRequest)
});

const { denom } = await response.json();
console.log(`Token created: ${denom}`);
```

### DEX Swap
```javascript
const swapRequest = {
  user: 'sultan15g5e8...',
  input_denom: 'sltn',
  output_denom: 'factory/sultan15g5e8.../MTK',
  input_amount: 100000000000, // 100 SLTN
  min_output: 45000000000,    // 45 MTK minimum
  signature: '...',
  public_key: '...'
};

const response = await fetch('https://rpc.sltn.io/dex/swap', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify(swapRequest)
});

const { output_amount, price_impact } = await response.json();
console.log(`Received: ${output_amount}, Impact: ${price_impact * 100}%`);
```

## Python

### Query Status
```python
import requests

response = requests.get('https://rpc.sltn.io/status')
status = response.json()
print(f"Block height: {status['block_height']}")
print(f"Validators: {status['validators']}")
```

### Get Validators
```python
response = requests.get('https://rpc.sltn.io/staking/validators')
data = response.json()
for v in data['validators']:
    print(f"{v['moniker']}: {v['stake'] / 1e9} SLTN staked")
```

## Rust

### Using reqwest
```rust
use reqwest;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    // Query balance
    let resp: Value = client
        .get("https://rpc.sltn.io/balance/sultan15g5e8...")
        .send()
        .await?
        .json()
        .await?;
    
    println!("Balance: {} SLTN", resp["balance"].as_u64().unwrap() / 1_000_000_000);
    Ok(())
}
```

---

## SDK Availability

| Language | Package | Status |
|----------|---------|--------|
| **Rust** | `sultan-sdk` | ✅ Available (native) |
| **TypeScript** | `@sultan/sdk` | 🔄 In development |
| **Python** | `sultan-py` | 📋 Planned Q2 2026 |

For SDK updates, visit: https://docs.sltn.io/sdk

---

## WebSocket (Coming Soon)

Real-time streaming at `wss://rpc.sltn.io/ws`:

```javascript
const ws = new WebSocket('wss://rpc.sltn.io/ws');
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('New block:', data.height);
};
ws.send(JSON.stringify({ subscribe: 'blocks' }));
```

---

**Document Version:** 2.0  
**Last Updated:** January 1, 2026  
**Total Endpoints:** 38
