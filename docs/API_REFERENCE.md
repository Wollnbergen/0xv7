# Sultan L1 API Reference

**Version:** 1.2  
**Updated:** December 27, 2025

## Base URL

**Production RPC:** `https://rpc.sltn.io`

## Authentication

All API calls are **FREE** - no API keys required (zero fees!)

---

## Core Endpoints

### Node Status

Get current node and network status.

```http
GET /status
```

**Response:**
```json
{
  "node_id": "sultan-validator-1",
  "block_height": 25000,
  "validators": 6,
  "uptime_seconds": 86400,
  "version": "1.0.0"
}
```

---

### Health Check

Simple health probe for load balancers.

```http
GET /health
```

**Response:**
```json
{
  "status": "ok"
}
```

---

### Total Supply

Get total and circulating supply (for block explorers).

```http
GET /supply/total
```

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

> **Note:** All `*_sltn` fields are in human-readable SLTN (divided by 10^9). Raw fields are in atomic units.

---

### Economics

Get tokenomics and staking information.

```http
GET /economics
```

**Response:**
```json
{
  "total_supply": 500000000000000000,
  "circulating_supply": 500000000000000000,
  "inflation_rate": 0.04,
  "staking_apy": 0.1333,
  "total_staked": 60000000000000
}
```

---

## Account Endpoints

### Get Balance


Get account balance by address.

```http
GET /balance/{address}
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `address` | string | Sultan address (sultan1...) |

**Response:**
```json
{
  "address": "sultan15g5e8....",
  "balance": 500000000000000000,
  "nonce": 0
}
```

---

## Transaction Endpoints

### Submit Transaction

Submit a signed transaction.

```http
POST /tx
Content-Type: application/json
```

**Request Body:**
```json
{
  "from": "sultan15g5e8...",
  "to": "sultan1abc123...",
  "amount": 1000000000,
  "memo": "Payment for services",
  "nonce": 0,
  "signature": "base64_encoded_signature"
}
```

**Response:**
```json
{
  "hash": "abc123def456...",
  "status": "pending"
}
```

---

### Get Transaction by Hash

Retrieve transaction details by hash.

```http
GET /tx/{hash}
```

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
  "timestamp": 1735300000,
  "block_height": 12345,
  "status": "confirmed"
}
```

---

### Get Transaction History

Retrieve transaction history for an address.

```http
GET /transactions/{address}?limit=50
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `address` | string | Sultan address (sultan1...) |
| `limit` | integer | Max transactions to return (default: 50) |

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
      "memo": "Payment",
      "nonce": 0,
      "timestamp": 1735300000,
      "block_height": 12345,
      "status": "confirmed"
    }
  ],
  "count": 1
}
```

---

## Block Endpoints

### Get Block by Height

Retrieve block details by height.

```http
GET /block/{height}
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `height` | integer | Block height |

**Response:**
```json
{
  "height": 12345,
  "hash": "blockhash123...",
  "timestamp": 1735300000,
  "transactions": 5,
  "proposer": "sultan_validator_1"
}
```

---

## Validator Endpoints

### List Validators

Get the active validator set.

```http
GET /validators
```

**Response:**
```json
{
  "validators": [
    {
      "address": "sultan_validator_1",
      "stake": 10000000000000,
      "voting_power": 16.67,
      "status": "active"
    }
  ],
  "total_validators": 6,
  "total_stake": 60000000000000
}
```

---

## Shard Endpoints

### Get Shard Info

Get information about active shards.

```http
GET /shards
```

**Response:**
```json
{
  "active_shards": 16,
  "tps_per_shard": 4000,
  "total_capacity": 64000
}
```

---

## Rate Limits

**No rate limits** - Sultan L1 is designed for maximum throughput.

- 64,000+ TPS capacity (16 shards × 4,000 TPS)
- All endpoints: **$0.00 forever**

---

## Error Codes

| Code | Description |
|------|-------------|
| 400 | Bad Request - Invalid parameters |
| 404 | Not Found - Resource doesn't exist |
| 500 | Internal Error - Server issue |

**Error Response Format:**
```json
{
  "error": "Description of the error",
  "code": 400
}
```

---

## WebSocket (Coming Soon)

Real-time block and transaction streaming via WebSocket will be available at:

```
wss://rpc.sltn.io/ws
```

---

## SDK Support

| Language | Status |
|----------|--------|
| Rust | ✅ Native integration |
| TypeScript | 🔄 In development |
| Python | 📋 Planned Q2 2026 |

---

**Document Version:** 1.2  
**Last Updated:** December 27, 2025
