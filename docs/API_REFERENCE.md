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
8. [NFT Endpoints](#nft-endpoints)
9. [DEX Endpoints](#dex-endpoints)
10. [Bridge Endpoints](#bridge-endpoints)
11. [Error Handling](#error-handling)
12. [Code Examples](#code-examples)
13. [End-to-End Examples](#end-to-end-examples)
14. [WebSocket API](#websocket-api)

---

# Core Endpoints

## GET /status

Get current node and network status.

**Response:**
```json
{
  "node_id": "sultan-validator-1",
  "block_height": 125000,
  "validators": 6,
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
| Parameter | Type | Default | Max | Description |
|-----------|------|---------|-----|-------------|
| `limit` | integer | 50 | 100 | Max transactions to return |
| `offset` | integer | 0 | - | Skip first N transactions |
| `order` | string | `desc` | - | Sort order: `asc` or `desc` |

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

**Query Parameters:**
| Parameter | Type | Default | Max | Description |
|-----------|------|---------|-----|-------------|
| `limit` | integer | 100 | 100 | Max validators to return |
| `offset` | integer | 0 | - | Skip first N validators |
| `status` | string | `active` | - | Filter: `active`, `inactive`, `jailed`, `all` |

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
  "total_validators": 6,
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
  "validator_count": 6,
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
  "description": "This proposal increases the active validator set from 6 to 21...",
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

**Query Parameters:**
| Parameter | Type | Default | Max | Description |
|-----------|------|---------|-----|-------------|
| `limit` | integer | 50 | 100 | Max proposals to return |
| `offset` | integer | 0 | - | Skip first N proposals |
| `status` | string | `all` | - | Filter: `voting`, `passed`, `rejected`, `all` |

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

**Query Parameters:**
| Parameter | Type | Default | Max | Description |
|-----------|------|---------|-----|-------------|
| `limit` | integer | 50 | 100 | Max tokens to return |
| `offset` | integer | 0 | - | Skip first N tokens |
| `creator` | string | - | - | Filter by creator address |

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

# NFT Endpoints

Native NFT support for collections, minting, and marketplaces.

## POST /nft/collection/create

Create a new NFT collection.

**Request Body:**
```json
{
  "creator": "sultan15g5e8...",
  "name": "Sultan Punks",
  "symbol": "SPUNK",
  "description": "10,000 unique punks on Sultan L1",
  "max_supply": 10000,
  "royalty_bps": 500,
  "base_uri": "ipfs://QmXyz.../",
  "signature": "base64_encoded_signature",
  "public_key": "base64_encoded_pubkey"
}
```

**Response:**
```json
{
  "collection_id": "nft/sultan15g5e8.../SPUNK",
  "name": "Sultan Punks",
  "symbol": "SPUNK",
  "creator": "sultan15g5e8...",
  "max_supply": 10000,
  "total_minted": 0,
  "royalty_bps": 500
}
```

> **Note:** `royalty_bps` is in basis points (500 = 5% royalty on secondary sales).

---

## POST /nft/mint

Mint a new NFT in a collection.

**Request Body:**
```json
{
  "collection_id": "nft/sultan15g5e8.../SPUNK",
  "minter": "sultan15g5e8...",
  "recipient": "sultan1abc123...",
  "token_id": "1",
  "name": "Sultan Punk #1",
  "description": "The first Sultan Punk ever minted",
  "image_uri": "ipfs://QmXyz.../1.png",
  "metadata_uri": "ipfs://QmXyz.../1.json",
  "attributes": [
    { "trait_type": "Background", "value": "Blue" },
    { "trait_type": "Eyes", "value": "Laser" },
    { "trait_type": "Rarity", "value": "Legendary" }
  ],
  "signature": "base64_encoded_signature",
  "public_key": "base64_encoded_pubkey"
}
```

**Response:**
```json
{
  "collection_id": "nft/sultan15g5e8.../SPUNK",
  "token_id": "1",
  "owner": "sultan1abc123...",
  "name": "Sultan Punk #1",
  "image_uri": "ipfs://QmXyz.../1.png",
  "tx_hash": "abc123..."
}
```

> **Permissions:** Only collection creator can mint (or approved minters).

---

## POST /nft/transfer

Transfer an NFT to another address.

**Request Body:**
```json
{
  "collection_id": "nft/sultan15g5e8.../SPUNK",
  "token_id": "1",
  "from": "sultan1abc123...",
  "to": "sultan1def456...",
  "signature": "base64_encoded_signature",
  "public_key": "base64_encoded_pubkey"
}
```

**Response:**
```json
{
  "collection_id": "nft/sultan15g5e8.../SPUNK",
  "token_id": "1",
  "from": "sultan1abc123...",
  "to": "sultan1def456...",
  "tx_hash": "def456..."
}
```

---

## POST /nft/metadata/update

Update NFT metadata (creator only, if mutable).

**Request Body:**
```json
{
  "collection_id": "nft/sultan15g5e8.../SPUNK",
  "token_id": "1",
  "updater": "sultan15g5e8...",
  "name": "Sultan Punk #1 (Evolved)",
  "description": "This punk has evolved!",
  "image_uri": "ipfs://QmNewImage.../1.png",
  "attributes": [
    { "trait_type": "Background", "value": "Gold" },
    { "trait_type": "Eyes", "value": "Diamond" },
    { "trait_type": "Rarity", "value": "Mythic" }
  ],
  "signature": "base64_encoded_signature",
  "public_key": "base64_encoded_pubkey"
}
```

**Response:**
```json
{
  "collection_id": "nft/sultan15g5e8.../SPUNK",
  "token_id": "1",
  "updated_fields": ["name", "description", "image_uri", "attributes"],
  "tx_hash": "ghi789..."
}
```

---

## GET /nft/collection/{collection_id}

Get collection details.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `collection_id` | string | Collection ID (URL encoded) |

**Response:**
```json
{
  "collection_id": "nft/sultan15g5e8.../SPUNK",
  "name": "Sultan Punks",
  "symbol": "SPUNK",
  "description": "10,000 unique punks on Sultan L1",
  "creator": "sultan15g5e8...",
  "max_supply": 10000,
  "total_minted": 2500,
  "royalty_bps": 500,
  "base_uri": "ipfs://QmXyz.../",
  "floor_price": 50000000000
}
```

---

## GET /nft/{collection_id}/{token_id}

Get NFT details.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `collection_id` | string | Collection ID (URL encoded) |
| `token_id` | string | Token ID within collection |

**Response:**
```json
{
  "collection_id": "nft/sultan15g5e8.../SPUNK",
  "token_id": "1",
  "name": "Sultan Punk #1",
  "description": "The first Sultan Punk ever minted",
  "image_uri": "ipfs://QmXyz.../1.png",
  "metadata_uri": "ipfs://QmXyz.../1.json",
  "owner": "sultan1def456...",
  "creator": "sultan15g5e8...",
  "attributes": [
    { "trait_type": "Background", "value": "Blue" },
    { "trait_type": "Eyes", "value": "Laser" },
    { "trait_type": "Rarity", "value": "Legendary" }
  ],
  "royalty_bps": 500,
  "minted_at": 1735689600,
  "last_sale_price": 75000000000
}
```

---

## GET /nft/owner/{address}

Get all NFTs owned by an address.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `address` | string | Owner address |

**Query Parameters:**
| Parameter | Type | Default | Max | Description |
|-----------|------|---------|-----|-------------|
| `limit` | integer | 50 | 100 | Max NFTs to return |
| `offset` | integer | 0 | - | Skip first N NFTs |
| `collection` | string | - | - | Filter by collection ID |

**Response:**
```json
{
  "owner": "sultan1def456...",
  "nfts": [
    {
      "collection_id": "nft/sultan15g5e8.../SPUNK",
      "token_id": "1",
      "name": "Sultan Punk #1",
      "image_uri": "ipfs://QmXyz.../1.png"
    },
    {
      "collection_id": "nft/sultan15g5e8.../SPUNK",
      "token_id": "42",
      "name": "Sultan Punk #42",
      "image_uri": "ipfs://QmXyz.../42.png"
    }
  ],
  "total": 2
}
```

---

## GET /nft/collections

List all NFT collections.

**Query Parameters:**
| Parameter | Type | Default | Max | Description |
|-----------|------|---------|-----|-------------|
| `limit` | integer | 50 | 100 | Max collections to return |
| `offset` | integer | 0 | - | Skip first N collections |
| `creator` | string | - | - | Filter by creator address |
| `sort_by` | string | `created_at` | - | Sort: `created_at`, `floor_price`, `volume` |

**Response:**
```json
{
  "collections": [
    {
      "collection_id": "nft/sultan15g5e8.../SPUNK",
      "name": "Sultan Punks",
      "symbol": "SPUNK",
      "total_minted": 2500,
      "floor_price": 50000000000,
      "volume_24h": 1500000000000
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

**Query Parameters:**
| Parameter | Type | Default | Max | Description |
|-----------|------|---------|-----|-------------|
| `limit` | integer | 50 | 100 | Max pools to return |
| `offset` | integer | 0 | - | Skip first N pools |
| `sort_by` | string | `volume_24h` | - | Sort: `volume_24h`, `reserve_a`, `created_at` |

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

# End-to-End Examples

Complete workflows for common integration scenarios.

## E2E: Wallet Creation → Fund → Send

```javascript
import * as ed25519 from '@noble/ed25519';
import { bech32 } from 'bech32';

const RPC = 'https://rpc.sltn.io';

// Step 1: Create wallet
async function createWallet() {
  const privateKey = ed25519.utils.randomPrivateKey();
  const publicKey = await ed25519.getPublicKey(privateKey);
  
  const hash = await crypto.subtle.digest('SHA-256', publicKey);
  const addressBytes = new Uint8Array(hash).slice(0, 20);
  const words = bech32.toWords(addressBytes);
  const address = bech32.encode('sultan', words);
  
  return { privateKey, publicKey, address };
}

// Step 2: Check balance and nonce
async function getAccountInfo(address) {
  const res = await fetch(`${RPC}/balance/${address}`);
  return res.json();
}

// Step 3: Send transaction
async function sendTransaction(wallet, to, amountSltn) {
  const { balance, nonce } = await getAccountInfo(wallet.address);
  const amountAtomic = Math.floor(amountSltn * 1e9);
  
  if (balance < amountAtomic) {
    throw new Error(`Insufficient balance: ${balance / 1e9} SLTN`);
  }
  
  const tx = {
    from: wallet.address,
    to,
    amount: amountAtomic,
    timestamp: Math.floor(Date.now() / 1000),
    nonce
  };
  
  const message = new TextEncoder().encode(JSON.stringify(tx));
  const signature = await ed25519.sign(message, wallet.privateKey);
  
  const res = await fetch(`${RPC}/tx`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      tx,
      signature: btoa(String.fromCharCode(...signature)),
      public_key: btoa(String.fromCharCode(...wallet.publicKey))
    })
  });
  
  return res.json();
}

// Step 4: Wait for confirmation
async function waitForConfirmation(txHash, maxRetries = 30) {
  for (let i = 0; i < maxRetries; i++) {
    const res = await fetch(`${RPC}/tx/${txHash}`);
    if (res.ok) {
      const tx = await res.json();
      if (tx.status === 'confirmed') return tx;
    }
    await new Promise(r => setTimeout(r, 2000)); // 2s blocks
  }
  throw new Error('Transaction not confirmed');
}

// Full flow
async function main() {
  const wallet = await createWallet();
  console.log('Created wallet:', wallet.address);
  
  // Wait for funding (faucet or external transfer)
  console.log('Fund this address and press enter...');
  
  const recipient = 'sultan1abc123...';
  const { hash } = await sendTransaction(wallet, recipient, 10); // 10 SLTN
  console.log('Submitted:', hash);
  
  const confirmed = await waitForConfirmation(hash);
  console.log('Confirmed at block:', confirmed.block_height);
}
```

---

## E2E: Create Token → Mint → Transfer → Swap

```javascript
// Assumes wallet already created (see above)

// Step 1: Create token
async function createToken(wallet) {
  const req = {
    creator: wallet.address,
    name: 'My Token',
    symbol: 'MTK',
    decimals: 6,
    total_supply: 1000000_000000 // 1M tokens
  };
  
  const signed = await signRequest(wallet, req);
  const res = await fetch(`${RPC}/tokens/create`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(signed)
  });
  return res.json();
}

// Step 2: Create DEX pair
async function createPair(wallet, denom) {
  const req = {
    creator: wallet.address,
    token_a: 'sltn',
    token_b: denom,
    initial_a: 100000_000000000, // 100K SLTN
    initial_b: 500000_000000     // 500K MTK
  };
  
  const signed = await signRequest(wallet, req);
  const res = await fetch(`${RPC}/dex/create_pair`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(signed)
  });
  return res.json();
}

// Step 3: Swap tokens
async function swap(wallet, inputDenom, outputDenom, amount) {
  // Get quote first
  const pool = await fetch(`${RPC}/dex/pool/sltn-MTK`).then(r => r.json());
  const expectedOut = calculateOutput(amount, pool);
  const minOut = Math.floor(expectedOut * 0.99); // 1% slippage
  
  const req = {
    user: wallet.address,
    input_denom: inputDenom,
    output_denom: outputDenom,
    input_amount: amount,
    min_output: minOut
  };
  
  const signed = await signRequest(wallet, req);
  const res = await fetch(`${RPC}/dex/swap`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(signed)
  });
  return res.json();
}

// Helper: AMM output calculation (x * y = k)
function calculateOutput(amountIn, pool) {
  const k = pool.reserve_a * pool.reserve_b;
  const newReserveA = pool.reserve_a + amountIn;
  const newReserveB = k / newReserveA;
  return Math.floor(pool.reserve_b - newReserveB);
}
```

---

## E2E: NFT Collection → Mint → Transfer

```javascript
// Step 1: Create collection
async function createCollection(wallet) {
  const req = {
    creator: wallet.address,
    name: 'Sultan Punks',
    symbol: 'SPUNK',
    description: '10,000 unique punks',
    max_supply: 10000,
    royalty_bps: 500, // 5%
    base_uri: 'ipfs://QmXyz.../'
  };
  
  const signed = await signRequest(wallet, req);
  const res = await fetch(`${RPC}/nft/collection/create`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(signed)
  });
  return res.json();
}

// Step 2: Mint NFT
async function mintNFT(wallet, collectionId, tokenId, recipient) {
  const req = {
    collection_id: collectionId,
    minter: wallet.address,
    recipient,
    token_id: tokenId,
    name: `Sultan Punk #${tokenId}`,
    description: 'A unique Sultan Punk',
    image_uri: `ipfs://QmXyz.../${tokenId}.png`,
    attributes: [
      { trait_type: 'Background', value: 'Blue' },
      { trait_type: 'Eyes', value: 'Laser' }
    ]
  };
  
  const signed = await signRequest(wallet, req);
  const res = await fetch(`${RPC}/nft/mint`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(signed)
  });
  return res.json();
}

// Step 3: Transfer NFT
async function transferNFT(wallet, collectionId, tokenId, to) {
  const req = {
    collection_id: collectionId,
    token_id: tokenId,
    from: wallet.address,
    to
  };
  
  const signed = await signRequest(wallet, req);
  const res = await fetch(`${RPC}/nft/transfer`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(signed)
  });
  return res.json();
}

// Helper: Sign any request
async function signRequest(wallet, req) {
  const message = new TextEncoder().encode(JSON.stringify(req));
  const signature = await ed25519.sign(message, wallet.privateKey);
  return {
    ...req,
    signature: btoa(String.fromCharCode(...signature)),
    public_key: btoa(String.fromCharCode(...wallet.publicKey))
  };
}
```

---

## SDK Availability

| Language | Package | Status | Install |
|----------|---------|--------|---------|  
| **Rust** | `sultan-sdk` | ✅ Stable | `cargo add sultan-sdk` |
| **TypeScript** | `@sultan/sdk` | 🧪 Beta | `npm install @sultan/sdk@beta` |
| **Python** | `sultan-py` | 📋 Planned Q2 2026 | - |

```bash
# TypeScript (beta)
npm install @sultan/sdk@beta

# Rust
cargo add sultan-sdk
```

For SDK updates and full documentation: https://docs.sltn.io/sdk

---

# WebSocket API

Real-time streaming for blocks, transactions, and DEX updates.

### Connection URLs

| Environment | URL |
|-------------|-----|
| **Mainnet** | `wss://rpc.sltn.io/ws` |
| **Testnet** | `wss://testnet.sltn.io/ws` |

### Subscribe to Events

```javascript
const ws = new WebSocket('wss://rpc.sltn.io/ws');

ws.onopen = () => {
  // Subscribe to new blocks
  ws.send(JSON.stringify({
    jsonrpc: '2.0',
    id: 1,
    method: 'subscribe',
    params: { channel: 'blocks' }
  }));
  
  // Subscribe to address transactions
  ws.send(JSON.stringify({
    jsonrpc: '2.0',
    id: 2,
    method: 'subscribe',
    params: { channel: 'txs', address: 'sultan15g5e8...' }
  }));
  
  // Subscribe to DEX pair updates
  ws.send(JSON.stringify({
    jsonrpc: '2.0',
    id: 3,
    method: 'subscribe',
    params: { channel: 'dex', pair_id: 'sltn-MTK' }
  }));
};

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  if (msg.method === 'subscription') {
    console.log(`${msg.params.channel}:`, msg.params.data);
  }
};

// Keep-alive (required every 30s)
setInterval(() => ws.send('ping'), 30000);
```

### Event Types

| Channel | Event Data |
|---------|------------|
| `blocks` | `{ height, hash, timestamp, tx_count, proposer }` |
| `txs` | `{ hash, from, to, amount, status, block_height }` |
| `dex` | `{ pair_id, price_a_to_b, reserve_a, reserve_b, last_trade }` |
| `validators` | `{ address, status, stake, voting_power }` |

### Connection Limits

| Parameter | Value |
|-----------|-------|
| Ping Interval | 30 seconds |
| Max Subscriptions | 100 per connection |
| Reconnect Backoff | 1s, 2s, 4s, 8s... max 60s |

For full WebSocket specification, see [RPC_SPECIFICATION.md](RPC_SPECIFICATION.md#websocket-api-v21---q1-2026).

---

**Document Version:** 2.1  
**Last Updated:** January 1, 2026  
**Total Endpoints:** 46 (38 REST + 8 NFT)
