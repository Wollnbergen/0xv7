# 🔧 How to Update Token Distribution

## Current vs New Distribution

### Currently in genesis.json:
```
Total Supply: 500M SLTN (500000000000000000 usltn)
└─ 5 validators × 50M each = 250M SLTN
   (This leaves 250M unallocated)
```

### Recommended New Distribution:
```
Total Supply: 500M SLTN

1. Validator Rewards Pool:    150M (30%) - sultan1validatorrewards...
2. Ecosystem Development:      100M (20%) - sultan1ecosystem...
3. Seed Round (locked):         25M (5%)  - sultan1seedround...
4. Private Round (locked):      50M (10%) - sultan1privateround...
5. Development Fund:            75M (15%) - sultan1devfund...
6. Team (4-year vesting):       50M (10%) - sultan1team...
7. Liquidity (CEX/DEX):         50M (10%) - sultan1liquidity...

TOTAL: 500M SLTN
```

---

## 📍 Where to Change It

### File: `/workspaces/0xv7/genesis-validators/genesis.json`

**Section to modify: `app_state.bank.balances`**

Currently (line ~131):
```json
"bank": {
  "total_supply": "500000000000000000",
  "balances": [
    {
      "address": "sultan1validator1...",
      "coins": [{"denom": "usltn", "amount": "50000000000000000"}]
    },
    // ... 4 more validators with 50M each
  ]
}
```

**Replace with new allocation:**

```json
"bank": {
  "total_supply": "500000000000000000",
  "balances": [
    {
      "address": "sultan1validatorrewardspool000000000000000",
      "coins": [{"denom": "usltn", "amount": "150000000000000000"}]
    },
    {
      "address": "sultan1ecosystem00000000000000000000000",
      "coins": [{"denom": "usltn", "amount": "100000000000000000"}]
    },
    {
      "address": "sultan1seedround00000000000000000000000",
      "coins": [{"denom": "usltn", "amount": "25000000000000000"}]
    },
    {
      "address": "sultan1privateround000000000000000000000",
      "coins": [{"denom": "usltn", "amount": "50000000000000000"}]
    },
    {
      "address": "sultan1devfund000000000000000000000000",
      "coins": [{"denom": "usltn", "amount": "75000000000000000"}]
    },
    {
      "address": "sultan1team0000000000000000000000000000",
      "coins": [{"denom": "usltn", "amount": "50000000000000000"}]
    },
    {
      "address": "sultan1liquidity00000000000000000000000",
      "coins": [{"denom": "usltn", "amount": "50000000000000000"}]
    }
  ]
}
```

---

## 🔑 Generate Real Addresses

**Before mainnet, generate proper addresses:**

```bash
# Generate wallet for each allocation
sultan-node keys add validator-rewards-pool
sultan-node keys add ecosystem-dev
sultan-node keys add seed-round
sultan-node keys add private-round
sultan-node keys add dev-fund
sultan-node keys add team-wallet
sultan-node keys add liquidity-provision

# Each will output an address like:
# sultan1abc123def456...
```

**Then update genesis.json with real addresses**

---

## 🔒 Vesting Contracts (Important!)

For Team, Seed, and Private allocations, use **time-locked contracts**:

### Team Wallet (4-year vesting)
```rust
// Vesting schedule:
- Cliff: 12 months (no unlock)
- After cliff: Linear unlock over 36 months
- Total: 48 months to full unlock
- Monthly unlock: 50M / 36 = 1,388,888 SLTN/month (after cliff)
```

### Seed Round (36-month total)
```rust
- Cliff: 12 months
- Linear: 24 months
- Monthly: 25M / 24 = 1,041,666 SLTN/month (after cliff)
```

### Private Round (24-month total)
```rust
- Cliff: 6 months
- Linear: 18 months
- Monthly: 50M / 18 = 2,777,777 SLTN/month (after cliff)
```

**Implementation options:**
1. Use CosmWasm vesting contracts (when smart contracts enabled)
2. Use Cosmos SDK periodic vesting accounts
3. Manual multi-sig releases (less ideal)

---

## 🛡️ Security Best Practices

### Multi-Signature Wallets
Use multi-sig for large allocations:
```bash
# Create 3-of-5 multi-sig
sultan-node keys add multisig-name \
  --multisig=addr1,addr2,addr3,addr4,addr5 \
  --multisig-threshold=3
```

**Recommended multi-sigs:**
- ✅ Team Wallet (3-of-5)
- ✅ Development Fund (2-of-3)
- ✅ Ecosystem Fund (2-of-3)
- ✅ Liquidity (2-of-3)

### Hardware Wallet Support
- Store private keys on Ledger/Trezor
- Use Keplr with hardware wallet integration
- Never store keys in plain text

---

## 📊 Transparency & Tracking

### Public Dashboard
Create transparency page showing:
- Real-time balances of all wallets
- Vesting schedules with countdown
- Monthly unlock amounts
- Transaction history

### Example Code for Website:
```javascript
// Add to index.html
const wallets = {
  validatorRewards: 'sultan1validatorrewards...',
  ecosystem: 'sultan1ecosystem...',
  seedRound: 'sultan1seedround...',
  privateRound: 'sultan1privateround...',
  devFund: 'sultan1devfund...',
  team: 'sultan1team...',
  liquidity: 'sultan1liquidity...'
};

async function fetchAllocation(address) {
  const response = await fetch(`${API_BASE}/cosmos/bank/v1beta1/balances/${address}`);
  const data = await response.json();
  return data.balances;
}

// Display on transparency page
```

---

## ✅ Verification Checklist

Before mainnet launch:

- [ ] Total supply = 500,000,000 SLTN (500000000000000000 usltn)
- [ ] All 7 wallets created with proper addresses
- [ ] Balances add up to exactly 500M
- [ ] Vesting contracts deployed (Team, Seed, Private)
- [ ] Multi-sig wallets configured
- [ ] Private keys backed up securely (hardware wallet)
- [ ] Genesis file validated (no JSON errors)
- [ ] Transparency page ready
- [ ] Team committed to 4-year vesting
- [ ] Legal docs match token allocation (SAFT, etc.)

---

## 🚀 Quick Update Command

Want to update genesis right now?

```bash
# Backup current genesis
cp genesis-validators/genesis.json genesis-validators/genesis.json.backup

# Edit with your favorite editor
nano genesis-validators/genesis.json

# Or use this script (generates placeholder addresses)
bash update-token-distribution.sh
```

---

## 💡 Key Takeaway

**You have 100% control over token distribution!**

It's just a JSON configuration file. You can:
- ✅ Change allocations anytime before mainnet
- ✅ Add/remove wallets
- ✅ Adjust percentages
- ✅ Create new categories
- ❌ Cannot change AFTER mainnet launches (immutable)

**Decision deadline:** Before you run the genesis validators for mainnet.

**Current status:** You're still in dev/testnet, so fully flexible!

---

**File created:** November 28, 2025  
**Ready to update when you decide on final distribution**
