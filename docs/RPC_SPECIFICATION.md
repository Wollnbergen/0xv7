# Sultan L1 RPC Specification

**Version:** 2.0  
**Updated:** January 1, 2026  
**Protocol:** HTTP REST API (JSON-RPC planned for v2.1)

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

**Bitcoin SPV:**
```
[tx_hash:32][branch_count:4][branches:32*n][tx_index:4][header:80]
```

**Ethereum ZK-SNARK (Groth16):**
```
[pi_a:64][pi_b:128][pi_c:64][public_inputs:variable]
```
Minimum: 256 bytes

**Solana gRPC:**
```
[signature:64][slot:8][status:1]
```
Status: 0=failed, 1=confirmed, 2=pending

**TON BOC:**
```
Magic: 0xb5ee9c72 or 0xb5ee9c73
```

### Security Features

| Feature | Description |
|---------|-------------|
| Rate Limiting | 50 req/min per pubkey |
| Multi-sig Large TX | 2-of-3 for amounts >100,000 SLTN |
| Treasury Governance | 3-of-5 multi-sig for treasury updates |
| ZK Validation | Groth16 structure + zero-element checks |

---

## WebSocket API (Planned v2.1)

### Connection

```
wss://rpc.sltn.io/ws
```

### Subscriptions

| Event | Message |
|-------|---------|
| New Block | `{"subscribe": "blocks"}` |
| Transactions | `{"subscribe": "txs", "address": "sultan1..."}` |
| Validator Updates | `{"subscribe": "validators"}` |

### Event Format

```json
{
  "type": "new_block",
  "data": {
    "height": 12345,
    "hash": "abc123...",
    "timestamp": 1735689600,
    "tx_count": 25
  }
}
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

