# Sultan Block Explorer API Reference

**Last Updated:** December 27, 2025

## RPC Endpoints

**Primary RPC:** `http://206.189.224.142:8545` (NYC)  
**Backup RPCs:**
- London: `http://134.122.96.36:8545`
- Singapore: `http://143.198.205.21:8545`
- Amsterdam: `http://142.93.238.33:8545`
- Frankfurt: `http://46.101.122.13:8545`
- San Francisco: `http://24.144.94.23:8545`

---

## Key Endpoints for Block Explorer

### 1. Total Supply (for market cap display)

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

> **Note:** `*_sltn` fields are human-readable (500M SLTN). Raw fields are in atomic units (divide by 10^9).

---

### 2. Network Status

```http
GET /status
```

**Response:**
```json
{
  "height": 31666,
  "validator_count": 6,
  "shard_count": 16,
  "sharding_enabled": true,
  "validator_apy": 0.1333,
  "node_name": "sultan-nyc"
}
```

---

### 3. Economics/Tokenomics

```http
GET /economics
```

**Response:**
```json
{
  "total_supply": 500000000000000000,
  "total_supply_formatted": "500000000",
  "circulating_supply": 500000000000000000,
  "genesis_supply": 500000000000000000,
  "current_inflation_rate": 0.04,
  "inflation_percentage": "4.0%",
  "current_burn_rate": 0.01,
  "burn_percentage": "1.0%",
  "validator_apy": 0.1333,
  "apy_percentage": "13.33%",
  "total_burned": 0,
  "is_deflationary": false,
  "years_since_genesis": 0,
  "inflation_policy": "Fixed 4% annual inflation guarantees zero gas fees sustainable at 76M+ TPS",
  "inflation_rate": "4.0% (fixed forever)"
}
```

---

### 4. Block Information

```http
GET /block/{height}
```

**Response:**
```json
{
  "index": 31666,
  "hash": "abc123...",
  "previous_hash": "def456...",
  "timestamp": 1735300000,
  "transactions": [...],
  "validator": "sultanval6newyork"
}
```

---

### 5. Transaction by Hash

```http
GET /tx/{hash}
```

**Response:**
```json
{
  "hash": "stake_353132e353a0022d...",
  "from": "sultan1testhistory",
  "to": "sultanval6newyork",
  "amount": 5000000000,
  "memo": "Stake delegation",
  "nonce": 0,
  "timestamp": 1766844085,
  "block_height": 31553,
  "status": "confirmed"
}
```

---

### 6. Transaction History by Address

```http
GET /transactions/{address}?limit=50
```

**Response:**
```json
{
  "address": "sultan1testhistory",
  "count": 2,
  "transactions": [
    {
      "hash": "stake_f9c1d5805f0623fb...",
      "from": "sultan1testhistory",
      "to": "sultanval6newyork",
      "amount": 5000000000,
      "memo": "Stake delegation",
      "nonce": 0,
      "timestamp": 1766844121,
      "block_height": 31571,
      "status": "confirmed"
    }
  ]
}
```

---

### 7. Validators List

```http
GET /staking/validators
```

**Response:**
```json
[
  {
    "validator_address": "sultanval6newyork",
    "self_stake": 10000000000000,
    "delegated_stake": 10000000000,
    "total_stake": 10010000000000,
    "commission_rate": 0.05,
    "rewards_accumulated": 0,
    "blocks_signed": 1000,
    "blocks_missed": 0,
    "jailed": false,
    "jailed_until": 0,
    "created_at": 0,
    "last_reward_height": 0
  }
]
```

---

### 8. Account Balance

```http
GET /balance/{address}
```

**Response:**
```json
{
  "address": "sultan15g5nwnlemn7zt6rtl7ch46ssvx2ym2v2umm07g",
  "balance": 500000000000000000,
  "nonce": 0
}
```

---

## Key Facts

| Property | Value |
|----------|-------|
| **Token Symbol** | SLTN |
| **Decimals** | 9 |
| **Genesis Supply** | 500,000,000 SLTN |
| **Block Time** | 2 seconds |
| **Shards** | 16 |
| **Gas Fees** | ZERO (free forever) |
| **Staking APY** | 13.33% |
| **Inflation Rate** | 4% annual (fixed) |
| **Burn Rate** | 1% of inflation |

---

## Live Test Commands

```bash
# Check supply
curl -s http://206.189.224.142:8545/supply/total | jq .

# Check status
curl -s http://206.189.224.142:8545/status | jq .

# Get validators
curl -s http://206.189.224.142:8545/staking/validators | jq .

# Get transaction history
curl -s http://206.189.224.142:8545/transactions/{address} | jq .
```
