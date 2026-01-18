# Sultan L1 RPC Specification

**Version:** 2.1  
**Updated:** January 18, 2026  
**Protocol:** HTTP REST API (JSON-RPC planned for v2.2)

---

## Overview

Sultan L1 provides a REST API for blockchain interaction. All endpoints accept and return JSON.

### Base URLs

| Network | URL | Chain ID |
|---------|-----|----------|
| **Mainnet** | `https://rpc.sltn.io` | `sultan-1` |
| **Testnet** | `https://testnet.sltn.io` | `sultan-testnet-1` |

### Key Features

- **Zero Fees**: All transactions are FREE - no gas fees
- **No API Keys**: Open access, no authentication required
- **Ed25519 Signatures**: All signed operations use Ed25519
- **Rate Limiting**: 100 req/10s per IP (bridge: 50 req/min per pubkey)

### Authentication Notes

| Auth Type | Description |
|-----------|-------------|
| **Signature** | Standard Ed25519 signature (base64 encoded) |
| **Ed25519 Sig*** | Enhanced auth: signature + pubkey + timestamp validation (5 min window) |

---

## Quick Reference

### All Endpoints (38 total)

| Category | Method | Endpoint | Auth |
|----------|--------|----------|------|
| **Core** | GET | `/status` | No |
| **Core** | GET | `/supply/total` | No |
| **Core** | GET | `/economics` | No |
| **Account** | GET | `/balance/{address}` | No |
| **Transaction** | POST | `/tx` | Signature |
| **Transaction** | GET | `/tx/{hash}` | No |
| **Transaction** | GET | `/transactions/{address}` | No |
| **Block** | GET | `/block/{height}` | No |
| **Staking** | POST | `/staking/create_validator` | Signature |
| **Staking** | POST | `/staking/delegate` | Signature |
| **Staking** | POST | `/staking/undelegate` | Signature |
| **Staking** | POST | `/staking/withdraw_rewards` | Signature |
| **Staking** | POST | `/staking/set_reward_wallet` | Ed25519 Sig* |
| **Staking** | GET | `/staking/reward_wallet/{address}` | No |
| **Staking** | GET | `/staking/validators` | No |
| **Staking** | GET | `/staking/delegations/{address}` | No |
| **Staking** | GET | `/staking/statistics` | No |
| **Governance** | POST | `/governance/propose` | Signature |
| **Governance** | POST | `/governance/vote` | Signature |
| **Governance** | POST | `/governance/tally/{id}` | No |
| **Governance** | GET | `/governance/proposals` | No |
| **Governance** | GET | `/governance/proposal/{id}` | No |
| **Governance** | GET | `/governance/statistics` | No |
| **Tokens** | POST | `/tokens/create` | Signature |
| **Tokens** | POST | `/tokens/mint` | Signature |
| **Tokens** | POST | `/tokens/transfer` | Signature |
| **Tokens** | POST | `/tokens/burn` | Signature |
| **Tokens** | GET | `/tokens/{denom}/metadata` | No |
| **Tokens** | GET | `/tokens/{denom}/balance/{address}` | No |
| **Tokens** | GET | `/tokens/list` | No |
| **DEX** | POST | `/dex/create_pair` | Signature |
| **DEX** | POST | `/dex/swap` | Signature |
| **DEX** | POST | `/dex/add_liquidity` | Signature |
| **DEX** | POST | `/dex/remove_liquidity` | Signature |
| **DEX** | GET | `/dex/pool/{pair_id}` | No |
| **DEX** | GET | `/dex/pools` | No |
| **DEX** | GET | `/dex/price/{pair_id}` | No |
| **Bridge** | GET | `/bridges` | No |
| **Bridge** | GET | `/bridge/{chain}` | No |
| **Bridge** | POST | `/bridge/submit` | Signature |
| **Bridge** | GET | `/bridge/{chain}/fee` | No |
| **Bridge** | GET | `/bridge/fees/treasury` | No |
| **Bridge** | GET | `/bridge/fees/statistics` | No |

---

## Request/Response Format

### Request Headers

```http
Content-Type: application/json
Accept: application/json
```

### Success Response

```json
{
  "field1": "value1",
  "field2": 12345
}
```

### Error Response

```json
{
  "error": "Description of the error",
  "status": 400
}
```

---

## Authentication (Ed25519 Signatures)

All POST endpoints require Ed25519 signature authentication.

### Signature Format

1. **Message**: JSON-stringify the transaction object (without signature/public_key)
2. **Sign**: Ed25519 sign the UTF-8 bytes of the message
3. **Encode**: Base64 encode the 64-byte signature
4. **Public Key**: Base64 encode the 32-byte public key

### Example (JavaScript)

```javascript
import * as ed25519 from '@noble/ed25519';

const tx = {
  from: 'sultan1abc...',
  to: 'sultan1xyz...',
  amount: 1000000000,
  timestamp: Date.now(),
  nonce: 0
};

const message = new TextEncoder().encode(JSON.stringify(tx));
const signature = await ed25519.sign(message, privateKey);

const request = {
  tx,
  signature: btoa(String.fromCharCode(...signature)),
  public_key: btoa(String.fromCharCode(...publicKey))
};
```

### Address Derivation

Sultan addresses are derived from Ed25519 public keys:

```
1. Public Key: 32 bytes (Ed25519)
2. Hash: SHA-256(public_key)[0:20] (first 20 bytes)
3. Bech32: Encode with "sultan" prefix
4. Result: sultan1abc123def456...
```

---

## Data Types

### Amounts

All amounts are in **atomic units** (1 SLTN = 10^9 atomic units):

| Human | Atomic |
|-------|--------|
| 1 SLTN | 1,000,000,000 |
| 0.1 SLTN | 100,000,000 |
| 0.000000001 SLTN | 1 |

### Timestamps

Unix timestamps in **seconds** (not milliseconds):

```json
{
  "timestamp": 1735689600
}
```

### Addresses

| Type | Format | Example |
|------|--------|---------|
| Account | `sultan1...` | `sultan15g5e8xyz789...` |
| Validator | `sultanvaloper1...` | `sultanvaloper1abc123...` |
| Token | `factory/{creator}/{symbol}` | `factory/sultan1.../MTK` |

---

## Rate Limiting

### Default Limits

| Endpoint Type | Limit | Window |
|---------------|-------|--------|
| All endpoints | 100 requests | 10 seconds |
| Bridge submit | 50 requests | 60 seconds (per pubkey) |

### Rate Limit Response

```http
HTTP/1.1 429 Too Many Requests
```

```json
{
  "error": "Rate limit exceeded",
  "retry_after": 5
}
```

---

## CORS Configuration

The RPC server supports CORS for browser-based applications:

| Header | Value |
|--------|-------|
| `Access-Control-Allow-Methods` | GET, POST, PUT, DELETE, OPTIONS |
| `Access-Control-Allow-Headers` | Content-Type, Authorization, Accept |
| `Access-Control-Allow-Origin` | Configured per deployment |

---

## Pagination

Endpoints returning lists support pagination:

| Parameter | Type | Default | Max |
|-----------|------|---------|-----|
| `limit` | integer | 50 | 100 |
| `offset` | integer | 0 | - |

**Example:**
```http
GET /transactions/sultan1...?limit=20&offset=40
```

---

## Network Parameters

### Chain Configuration

| Parameter | Value |
|-----------|-------|
| Block Time | 2 seconds |
| Finality | Instant (single block) |
| Decimals | 9 |
| Native Denom | `sltn` |
| Address Prefix | `sultan` |

### Staking Parameters

| Parameter | Value |
|-----------|-------|
| Min Validator Stake | 10,000 SLTN |
| Unbonding Period | 21 days |
| Max Validators | 100 |
| Inflation Rate | 4% annual |

### Governance Parameters

| Parameter | Value |
|-----------|-------|
| Min Deposit | 1,000 SLTN |
| Voting Period | 7 days |
| Quorum | 33.4% |
| Pass Threshold | 50% |
| Veto Threshold | 33.4% |

---

## Bridge Specifications

### Supported Chains

| Chain | Proof Type | Confirmations | Finality |
|-------|------------|---------------|----------|
| Bitcoin | SPV Merkle | 3 blocks | ~60 min |
| Ethereum | ZK-SNARK (Groth16) | 15 blocks | ~3 min |
| Solana | gRPC Finality | 1 slot | ~400ms |
| TON | BOC Contract | 1 block | ~5 sec |

### Proof Formats

**Bitcoin SPV (Merkle Proof):**
```
Binary Layout (minimum 120 bytes):
┌────────────────────────────────────────────────────────────┐
│ tx_hash        │ 32 bytes │ SHA256d of the transaction     │
│ branch_count   │ 4 bytes  │ Little-endian uint32           │
│ merkle_branches│ 32*n     │ Sibling hashes for merkle path │
│ tx_index       │ 4 bytes  │ Transaction position in block  │
│ block_header   │ 80 bytes │ Full Bitcoin block header      │
└────────────────────────────────────────────────────────────┘

Verification:
1. Parse block_header bytes 36-68 for merkle_root
2. Compute root from tx_hash + branches using tx_index bits
3. Compare computed root == header merkle_root
```

**Ethereum ZK-SNARK (Groth16):**
```
Binary Layout (minimum 256 bytes):
┌────────────────────────────────────────────────────────────┐
│ pi_a           │ 64 bytes │ G1 point (x,y coordinates)     │
│ pi_b           │ 128 bytes│ G2 point (2x2 coordinates)     │
│ pi_c           │ 64 bytes │ G1 point (x,y coordinates)     │
│ public_inputs  │ variable │ 32 bytes per public input      │
└────────────────────────────────────────────────────────────┘

Validation Rules:
- pi_a/pi_c: Non-zero 64-byte G1 points (reject all-zeros)
- pi_b: Non-zero 128-byte G2 point (reject all-zeros)
- Minimum total: 256 bytes (empty inputs)
- Typical: 288-320 bytes (1-2 public inputs)
```

**Solana gRPC Finality:**
```
Binary Layout (73 bytes):
┌────────────────────────────────────────────────────────────┐
│ signature      │ 64 bytes │ Ed25519 transaction signature  │
│ slot           │ 8 bytes  │ Little-endian uint64 slot num  │
│ status         │ 1 byte   │ 0=failed, 1=confirmed, 2=pending│
└────────────────────────────────────────────────────────────┘

Status Codes:
- 0x00: Transaction failed (do not credit)
- 0x01: Transaction confirmed (safe to credit)
- 0x02: Transaction pending (wait for confirmation)
```

**TON BOC (Bag of Cells):**
```
Binary Layout (variable):
┌────────────────────────────────────────────────────────────┐
│ magic          │ 4 bytes  │ 0xb5ee9c72 or 0xb5ee9c73       │
│ flags_size     │ 1 byte   │ Serialization flags            │
│ cell_count     │ 1-4 bytes│ Number of cells (varint)       │
│ root_count     │ 1-4 bytes│ Number of roots (varint)       │
│ cells_data     │ variable │ Serialized cell data           │
└────────────────────────────────────────────────────────────┘

Magic Bytes:
- 0xb5ee9c72: Standard BOC
- 0xb5ee9c73: BOC with CRC32
```

### Security Features

| Feature | Description |
|---------|-------------|
| Rate Limiting | 50 req/min per pubkey |
| Multi-sig Large TX | 2-of-3 for amounts >100,000 SLTN |
| Treasury Governance | 3-of-5 multi-sig for treasury updates |
| ZK Validation | Groth16 structure + zero-element checks |

---

## WebSocket API (v2.1 - Q1 2026)

Real-time streaming for dApps, wallets, and explorers.

### Connection

```
Mainnet:  wss://rpc.sltn.io/ws
Testnet:  wss://testnet.sltn.io/ws
```

### Authentication

WebSocket connections are unauthenticated (read-only subscriptions). Write operations require REST API with signatures.

### Subscription Messages

**Subscribe to New Blocks:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "subscribe",
  "params": {
    "channel": "blocks"
  }
}
```

**Subscribe to Address Transactions:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "subscribe",
  "params": {
    "channel": "txs",
    "address": "sultan15g5e8..."
  }
}
```

**Subscribe to DEX Pool Updates:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "subscribe",
  "params": {
    "channel": "dex",
    "pair_id": "sltn-MTK"
  }
}
```

**Subscribe to Validator Set Changes:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "subscribe",
  "params": {
    "channel": "validators"
  }
}
```

### Event Formats

**New Block Event:**
```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "blocks",
    "data": {
      "height": 125000,
      "hash": "abc123...",
      "timestamp": 1735689600,
      "tx_count": 25,
      "proposer": "sultanvaloper1..."
    }
  }
}
```

**Transaction Event:**
```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "txs",
    "data": {
      "hash": "def456...",
      "from": "sultan15g5e8...",
      "to": "sultan1abc...",
      "amount": 1000000000,
      "status": "confirmed",
      "block_height": 125000
    }
  }
}
```

**DEX Price Update Event:**
```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "dex",
    "data": {
      "pair_id": "sltn-MTK",
      "price_a_to_b": 0.502,
      "reserve_a": 1500500000000,
      "reserve_b": 749750000000,
      "last_trade": {
        "type": "swap",
        "input": 500000000,
        "output": 249500000
      }
    }
  }
}
```

### Unsubscribe

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "unsubscribe",
  "params": {
    "subscription_id": 1
  }
}
```

### Connection Management

| Parameter | Value |
|-----------|-------|
| Ping Interval | 30 seconds |
| Pong Timeout | 10 seconds |
| Max Subscriptions | 100 per connection |
| Reconnect Backoff | Exponential (1s, 2s, 4s, 8s, max 60s) |

### JavaScript Example

```javascript
const ws = new WebSocket('wss://rpc.sltn.io/ws');

ws.onopen = () => {
  // Subscribe to blocks
  ws.send(JSON.stringify({
    jsonrpc: '2.0',
    id: 1,
    method: 'subscribe',
    params: { channel: 'blocks' }
  }));
  
  // Subscribe to my transactions
  ws.send(JSON.stringify({
    jsonrpc: '2.0',
    id: 2,
    method: 'subscribe',
    params: { channel: 'txs', address: 'sultan15g5e8...' }
  }));
};

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  if (msg.method === 'subscription') {
    console.log(`${msg.params.channel}:`, msg.params.data);
  }
};

// Keep-alive ping
setInterval(() => ws.send('ping'), 30000);
```

---

## Error Codes Reference

| Code | Status | Description |
|------|--------|-------------|
| 400 | Bad Request | Invalid parameters |
| 401 | Unauthorized | Invalid signature |
| 404 | Not Found | Resource not found |
| 409 | Conflict | Nonce mismatch |
| 429 | Too Many Requests | Rate limited |
| 500 | Server Error | Internal error |

### Common Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| `Invalid signature` | Ed25519 verification failed | Check keypair matches address |
| `Insufficient balance` | Balance < amount | Check balance first |
| `Invalid nonce` | Nonce != expected | Query current nonce |
| `Validator not found` | Invalid validator address | Query validator list |
| `Token not found` | Invalid denom | Query token list |
| `Pair not found` | Invalid DEX pair | Query pool list |

---

## SDK Integration

### Official SDKs

| Language | Package | Status |
|----------|---------|--------|
| Rust | `sultan-sdk` | ✅ Released |
| TypeScript | `@sultan/sdk` | 🔄 Development |
| Python | `sultan-py` | 📋 Q2 2026 |

### Community SDKs

We welcome community SDK contributions. Requirements:
- Ed25519 signature support
- Bech32 address encoding
- Full endpoint coverage
- Test suite

---

## Changelog

### v2.0 (January 1, 2026)
- Added Token Factory endpoints (7)
- Added DEX endpoints (7)
- Added Bridge endpoints (6)
- Added Governance endpoints (6)
- Added rate limiting documentation
- Added bridge security features
- Added code examples

### v1.2 (December 27, 2025)
- Initial public release
- Core, Account, Transaction, Block, Staking endpoints

---

**Full API Reference:** See [API_REFERENCE.md](API_REFERENCE.md)  
**Developer Portal:** https://docs.sltn.io

