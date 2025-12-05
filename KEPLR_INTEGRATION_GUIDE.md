# 🦊 Keplr Wallet Integration Guide for Sultan L1

## Overview

Sultan L1 is **fully compatible with Keplr wallet**, the most popular wallet for Cosmos-based chains. Users can stake, vote on governance, and manage their SLTN tokens directly through Keplr.

---

## 🎯 What Works in Keplr

### ✅ Full Feature Support

1. **Wallet Management**
   - Create/import Sultan addresses
   - View SLTN balance
   - Send/receive transactions
   - Transaction history

2. **Staking** (26.67% APY)
   - View all validators
   - Delegate SLTN to validators
   - Redelegate between validators
   - Undelegate tokens
   - Claim staking rewards
   - View delegation history

3. **Governance**
   - View active proposals
   - Vote on proposals (Yes/No/Abstain/NoWithVeto)
   - View voting power
   - Track proposal status
   - See voting results

4. **IBC Transfers**
   - Send SLTN to other Cosmos chains
   - Receive tokens from other chains
   - Bridge between Sultan and Cosmos ecosystem

---

## 📱 How to Add Sultan to Keplr

### Method 1: Web Integration (Recommended)

Visit our integration page and click "Add to Keplr":

```
https://sultanchain.io/add-to-keplr.html
```

### Method 2: Manual Configuration

1. Open Keplr extension
2. Click settings (⚙️)
3. Select "Manage Chain Visibility"
4. Click "Add Custom Chain"
5. Paste the configuration below

### Method 3: Programmatic (For Developers)

```javascript
// Add Sultan L1 to Keplr
async function addSultanToKeplr() {
    if (!window.keplr) {
        alert('Please install Keplr extension');
        return;
    }

    const chainConfig = {
        chainId: "sultan-1",
        chainName: "Sultan L1",
        rpc: "https://rpc.sultanchain.io",
        rest: "https://api.sultanchain.io",
        bip44: {
            coinType: 118,
        },
        bech32Config: {
            bech32PrefixAccAddr: "sultan",
            bech32PrefixAccPub: "sultanpub",
            bech32PrefixValAddr: "sultanvaloper",
            bech32PrefixValPub: "sultanvaloperpub",
            bech32PrefixConsAddr: "sultanvalcons",
            bech32PrefixConsPub: "sultanvalconspub"
        },
        currencies: [{
            coinDenom: "SLTN",
            coinMinimalDenom: "usltn",
            coinDecimals: 9,
            coinGeckoId: "sultan"
        }],
        feeCurrencies: [{
            coinDenom: "SLTN",
            coinMinimalDenom: "usltn",
            coinDecimals: 9,
            coinGeckoId: "sultan",
            gasPriceStep: {
                low: 0.01,
                average: 0.025,
                high: 0.04
            }
        }],
        stakeCurrency: {
            coinDenom: "SLTN",
            coinMinimalDenom: "usltn",
            coinDecimals: 9,
            coinGeckoId: "sultan"
        },
        features: ["ibc-transfer", "ibc-go", "staking", "governance"]
    };

    await window.keplr.experimentalSuggestChain(chainConfig);
    console.log('Sultan L1 added to Keplr!');
}

// Call the function
addSultanToKeplr();
```

---

## 💰 Staking in Keplr

### Step 1: Connect to Sultan

1. Open Keplr extension
2. Select "Sultan L1" from chain dropdown
3. Your address will be displayed (starts with `sultan`)

### Step 2: View Validators

1. Click "Stake" button in Keplr
2. Browse list of Sultan validators
3. View validator details:
   - Commission rate
   - Total stake
   - Uptime
   - Voting power

### Step 3: Delegate

1. Select a validator
2. Click "Delegate"
3. Enter amount to stake (minimum 5,000 SLTN for validators)
4. Approve transaction
5. Wait for confirmation

**Rewards are distributed automatically every block!**

### Step 4: Claim Rewards

1. Go to "Stake" tab
2. Click "Claim Rewards"
3. Select validators to claim from (or "Claim All")
4. Approve transaction
5. Rewards added to your balance

### Staking Rewards Calculation

```
Your Annual Rewards = Your Stake × 26.67% APY
Daily Rewards = Annual Rewards / 365
Per Block Rewards = Daily Rewards / 17,280 blocks
```

**Example:**
- Stake: 10,000 SLTN
- APY: 26.67%
- Annual Rewards: 2,667 SLTN
- Daily Rewards: 7.31 SLTN
- Per Block Rewards: 0.000423 SLTN

---

## 🗳️ Governance in Keplr

### Step 1: View Proposals

1. Open Keplr
2. Select "Sultan L1"
3. Click "Governance" tab
4. View all active proposals

### Step 2: Read Proposal Details

Each proposal shows:
- Title and description
- Proposal type (ParameterChange, SoftwareUpgrade, etc.)
- Voting period end time
- Current vote tally
- Your voting power

### Step 3: Cast Your Vote

1. Click on a proposal
2. Select your vote:
   - **Yes**: Support the proposal
   - **No**: Oppose the proposal
   - **Abstain**: Participate in quorum without opinion
   - **NoWithVeto**: Strong opposition (can veto if >33.4%)
3. Review your voting power
4. Click "Vote"
5. Approve transaction

### Voting Power

Your voting power equals your total staked SLTN:

```
Voting Power = Self-Staked + Delegated to Validators
```

**Example:**
- Delegated to Validator A: 5,000 SLTN
- Delegated to Validator B: 3,000 SLTN
- **Total Voting Power: 8,000 SLTN**

### Proposal Outcomes

- **Passed**: Quorum reached (33.4%), >50% Yes votes, <33.4% NoWithVeto
- **Rejected**: Quorum reached but failed to pass
- **Failed**: Quorum not reached
- **Vetoed**: >33.4% voted NoWithVeto

---

## 🔗 Connecting to Sultan dApps

### Web3 Integration Example

```javascript
// Connect to Sultan via Keplr
async function connectToSultan() {
    // Enable Sultan chain
    await window.keplr.enable("sultan-1");
    
    // Get offline signer
    const offlineSigner = window.keplr.getOfflineSigner("sultan-1");
    
    // Get accounts
    const accounts = await offlineSigner.getAccounts();
    const address = accounts[0].address;
    
    console.log("Connected:", address);
    return address;
}

// Get balance
async function getBalance(address) {
    const response = await fetch(
        `https://api.sultanchain.io/cosmos/bank/v1beta1/balances/${address}`
    );
    const data = await response.json();
    const balance = data.balances.find(b => b.denom === 'usltn');
    return balance ? parseInt(balance.amount) / 1e9 : 0;
}

// Sign transaction
async function signTransaction(tx) {
    const offlineSigner = window.keplr.getOfflineSigner("sultan-1");
    const result = await offlineSigner.signDirect(
        tx.signerAddress,
        tx.signDoc
    );
    return result;
}
```

---

## 🌉 IBC Transfers via Keplr

### Sending SLTN to Other Chains

1. Open Keplr
2. Select "Sultan L1"
3. Click "IBC Transfer"
4. Select destination chain (e.g., Cosmos Hub, Osmosis)
5. Enter recipient address
6. Enter amount
7. Review fees
8. Approve transaction

### Receiving Tokens from Other Chains

1. Get your Sultan address from Keplr (starts with `sultan`)
2. On source chain, initiate IBC transfer
3. Enter your Sultan address
4. Wait for transfer completion (~7 seconds)
5. Check balance in Keplr

---

## 💻 Developer Integration

### React Example

```javascript
import { useEffect, useState } from 'react';

function SultanWallet() {
    const [address, setAddress] = useState('');
    const [balance, setBalance] = useState(0);

    async function connectKeplr() {
        if (!window.keplr) {
            alert('Install Keplr!');
            return;
        }

        await window.keplr.enable("sultan-1");
        const offlineSigner = window.keplr.getOfflineSigner("sultan-1");
        const accounts = await offlineSigner.getAccounts();
        
        setAddress(accounts[0].address);
        
        // Fetch balance
        const response = await fetch(
            `https://api.sultanchain.io/cosmos/bank/v1beta1/balances/${accounts[0].address}`
        );
        const data = await response.json();
        const bal = data.balances.find(b => b.denom === 'usltn');
        setBalance(bal ? parseInt(bal.amount) / 1e9 : 0);
    }

    return (
        <div>
            <button onClick={connectKeplr}>Connect Keplr</button>
            {address && (
                <div>
                    <p>Address: {address}</p>
                    <p>Balance: {balance} SLTN</p>
                </div>
            )}
        </div>
    );
}
```

### Staking Integration

```javascript
async function delegateToValidator(validatorAddress, amount) {
    const offlineSigner = window.keplr.getOfflineSigner("sultan-1");
    const accounts = await offlineSigner.getAccounts();
    const delegatorAddress = accounts[0].address;

    const msg = {
        typeUrl: "/cosmos.staking.v1beta1.MsgDelegate",
        value: {
            delegatorAddress: delegatorAddress,
            validatorAddress: validatorAddress,
            amount: {
                denom: "usltn",
                amount: (amount * 1e9).toString() // Convert to usltn
            }
        }
    };

    // Sign and broadcast
    const result = await window.keplr.signAndBroadcast(
        "sultan-1",
        delegatorAddress,
        [msg],
        {
            amount: [{ denom: "usltn", amount: "5000" }],
            gas: "200000",
        }
    );

    return result;
}
```

### Governance Integration

```javascript
async function voteOnProposal(proposalId, vote) {
    const offlineSigner = window.keplr.getOfflineSigner("sultan-1");
    const accounts = await offlineSigner.getAccounts();
    const voter = accounts[0].address;

    // vote: 1=Yes, 2=Abstain, 3=No, 4=NoWithVeto
    const msg = {
        typeUrl: "/cosmos.gov.v1beta1.MsgVote",
        value: {
            proposalId: proposalId.toString(),
            voter: voter,
            option: vote
        }
    };

    const result = await window.keplr.signAndBroadcast(
        "sultan-1",
        voter,
        [msg],
        {
            amount: [{ denom: "usltn", amount: "5000" }],
            gas: "200000",
        }
    );

    return result;
}
```

---

## 📊 Keplr Dashboard Features

### Staking Dashboard
- **My Validators**: List of validators you've delegated to
- **Available Rewards**: Total claimable rewards across all validators
- **Total Staked**: Sum of all your delegations
- **Unbonding**: Tokens currently unbonding (if unbonding period enabled)

### Governance Dashboard
- **Active Proposals**: Currently voting proposals
- **My Votes**: Proposals you've voted on
- **Voting Power**: Your total voting power
- **Participation Rate**: % of proposals you've voted on

---

## 🎓 Best Practices

### Security
1. Never share your seed phrase
2. Verify transaction details before signing
3. Use hardware wallet for large amounts
4. Enable Keplr lock when not in use

### Staking
1. Research validators before delegating
2. Diversify across multiple validators
3. Claim rewards regularly
4. Monitor validator performance

### Governance
1. Read proposals thoroughly
2. Participate in discussions
3. Vote on all proposals
4. Use NoWithVeto sparingly (only for malicious proposals)

---

## 🔧 Troubleshooting

### Keplr Not Detecting Sultan

**Solution:**
1. Refresh the page
2. Make sure Keplr is unlocked
3. Try adding chain manually
4. Clear browser cache

### Transaction Fails

**Solution:**
1. Check you have enough SLTN for fees
2. Increase gas limit
3. Wait a few seconds and retry
4. Check if chain is congested

### Balance Not Showing

**Solution:**
1. Wait for block confirmation (~5 seconds)
2. Refresh Keplr
3. Check transaction on explorer
4. Verify correct network selected

---

## 📞 Resources

### Official Links
- **Add to Keplr**: https://sultanchain.io/add-to-keplr.html
- **Explorer**: https://explorer.sultanchain.io
- **Staking Dashboard**: https://wallet.sultanchain.io
- **Governance**: https://governance.sultanchain.io

### Documentation
- **Staking Guide**: /STAKING_GUIDE.md
- **Governance Guide**: /GOVERNANCE_GUIDE.md
- **API Docs**: https://docs.sultanchain.io

### Support
- **Discord**: https://discord.gg/sultan
- **Telegram**: https://t.me/sultanchain
- **Twitter**: https://twitter.com/sultanchain

---

## 🎉 Quick Start Summary

1. **Install Keplr**: https://www.keplr.app/
2. **Add Sultan**: Visit add-to-keplr.html
3. **Get SLTN**: Bridge from other chains or buy on DEX
4. **Stake**: Delegate to validators for 26.67% APY
5. **Vote**: Participate in governance
6. **Earn**: Claim rewards regularly

**Sultan L1 × Keplr - Seamless Cosmos Integration** 🚀
