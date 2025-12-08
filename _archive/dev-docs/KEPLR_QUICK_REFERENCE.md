# 🦊 Keplr + Sultan L1 - Quick Reference

## Adding Sultan to Keplr

### One-Click Method
```
Visit: https://sultanchain.io/add-to-keplr.html
Click: "Add Sultan L1 to Keplr"
```

### Programmatic Method
```javascript
await window.keplr.experimentalSuggestChain({
  chainId: "sultan-1",
  chainName: "Sultan L1",
  rpc: "https://rpc.sultanchain.io",
  rest: "https://api.sultanchain.io",
  bip44: { coinType: 118 },
  currencies: [{
    coinDenom: "SLTN",
    coinMinimalDenom: "usltn",
    coinDecimals: 9
  }],
  stakeCurrency: {
    coinDenom: "SLTN",
    coinMinimalDenom: "usltn",
    coinDecimals: 9
  },
  features: ["staking", "governance", "ibc-transfer"]
});
```

---

## Staking in Keplr

### Delegate
```javascript
const msg = {
  typeUrl: "/cosmos.staking.v1beta1.MsgDelegate",
  value: {
    delegatorAddress: "sultan1...",
    validatorAddress: "sultanvaloper...",
    amount: { denom: "usltn", amount: "5000000000000" } // 5,000 SLTN
  }
};
```

### Claim Rewards
```javascript
const msg = {
  typeUrl: "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward",
  value: {
    delegatorAddress: "sultan1...",
    validatorAddress: "sultanvaloper..."
  }
};
```

### Undelegate
```javascript
const msg = {
  typeUrl: "/cosmos.staking.v1beta1.MsgUndelegate",
  value: {
    delegatorAddress: "sultan1...",
    validatorAddress: "sultanvaloper...",
    amount: { denom: "usltn", amount: "1000000000000" }
  }
};
```

---

## Governance in Keplr

### Vote on Proposal
```javascript
const msg = {
  typeUrl: "/cosmos.gov.v1beta1.MsgVote",
  value: {
    proposalId: "1",
    voter: "sultan1...",
    option: 1 // 1=Yes, 2=Abstain, 3=No, 4=NoWithVeto
  }
};
```

### Submit Proposal
```javascript
const msg = {
  typeUrl: "/cosmos.gov.v1beta1.MsgSubmitProposal",
  value: {
    content: {
      typeUrl: "/cosmos.gov.v1beta1.TextProposal",
      value: {
        title: "My Proposal",
        description: "Proposal details..."
      }
    },
    initialDeposit: [{ denom: "usltn", amount: "1000000000000" }], // 1,000 SLTN
    proposer: "sultan1..."
  }
};
```

---

## Common Keplr Functions

### Connect Wallet
```javascript
await window.keplr.enable("sultan-1");
const signer = window.keplr.getOfflineSigner("sultan-1");
const accounts = await signer.getAccounts();
const address = accounts[0].address; // sultan1...
```

### Get Balance
```javascript
const response = await fetch(
  `https://api.sultanchain.io/cosmos/bank/v1beta1/balances/${address}`
);
const data = await response.json();
const balance = data.balances.find(b => b.denom === 'usltn');
const sltn = parseInt(balance.amount) / 1e9;
```

### Sign and Broadcast
```javascript
const result = await window.keplr.signAndBroadcast(
  "sultan-1",
  address,
  [msg],
  {
    amount: [{ denom: "usltn", amount: "5000" }],
    gas: "200000"
  }
);
```

---

## Key Parameters

| Parameter | Value |
|-----------|-------|
| **Chain ID** | sultan-1 |
| **Prefix** | sultan |
| **Token** | SLTN |
| **Decimals** | 9 |
| **Min Denom** | usltn |
| **Coin Type** | 118 |
| **RPC** | https://rpc.sultanchain.io |
| **API** | https://api.sultanchain.io |

---

## Address Formats

| Type | Prefix | Example |
|------|--------|---------|
| **User** | sultan | sultan1abc...xyz |
| **Validator** | sultanvaloper | sultanvaloper1abc...xyz |
| **Consensus** | sultanvalcons | sultanvalcons1abc...xyz |

---

## Transaction Fees

```javascript
{
  amount: [{ denom: "usltn", amount: "5000" }], // 0.000005 SLTN
  gas: "200000"
}
```

**Gas Price Steps:**
- Low: 0.01 usltn
- Average: 0.025 usltn
- High: 0.04 usltn

---

## Vote Options

```javascript
// Governance vote options
const VoteOption = {
  YES: 1,
  ABSTAIN: 2,
  NO: 3,
  NO_WITH_VETO: 4
};
```

---

## Quick Links

- **Add to Keplr**: https://sultanchain.io/add-to-keplr.html
- **Explorer**: https://explorer.sultanchain.io
- **Staking**: https://wallet.sultanchain.io/staking
- **Governance**: https://wallet.sultanchain.io/governance
- **Full Guide**: /KEPLR_INTEGRATION_GUIDE.md
