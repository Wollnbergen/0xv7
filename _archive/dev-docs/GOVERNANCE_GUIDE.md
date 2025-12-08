# Sultan L1 Governance Guide

## Production On-Chain Governance

Sultan L1 features a **production-ready on-chain governance system** with weighted voting, quorum requirements, and automatic proposal execution.

---

## 📊 Key Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| **Proposal Deposit** | 1,000 SLTN | Required to submit a proposal |
| **Voting Period** | 100,800 blocks | ~7 days (5s blocks) |
| **Quorum** | 33.4% | Minimum % of staked tokens that must vote |
| **Pass Threshold** | 50% | Minimum % of Yes votes to pass |
| **Veto Threshold** | 33.4% | % of NoWithVeto to fail proposal |
| **Vote Options** | Yes, No, Abstain, NoWithVeto | Available voting options |

---

## 🎯 Features

### ✅ Real Production Implementation
- **NO STUBS** - All functionality is production-ready
- **NO TODOs** - Complete implementation
- Real weighted voting based on staking power
- Automatic quorum and veto calculations
- Real proposal execution
- Parameter change enforcement

### 📜 Proposal Types

1. **ParameterChange**: Modify chain parameters
2. **SoftwareUpgrade**: Schedule network upgrades
3. **CommunityPool**: Spend from community treasury
4. **TextProposal**: Signaling proposals (no execution)

### 🗳️ Vote Options

- **Yes**: Support the proposal
- **No**: Oppose the proposal
- **Abstain**: Participate in quorum without opinion
- **NoWithVeto**: Strong opposition (can veto if >33.4%)

### ⚖️ Voting Rules

1. **Weighted Voting**: Vote power = staking power
2. **One Vote Per Address**: Cannot vote multiple times
3. **Quorum Required**: 33.4% of bonded tokens must vote
4. **Pass Threshold**: >50% of votes (excluding Abstain) must be Yes
5. **Veto Protection**: >33.4% NoWithVeto fails proposal
6. **Execution**: Passed proposals execute automatically

---

## 🚀 API Endpoints

### 1. Submit Proposal

**POST** `/governance/propose`

Create a new governance proposal.

**Request:**
```json
{
  "proposer": "sultan1proposer...",
  "title": "Increase Block Size",
  "description": "Proposal to increase maximum block size from 1MB to 2MB...",
  "proposal_type": "parameter_change",
  "initial_deposit": 1000000000000,
  "parameters": {
    "max_block_size": "2000000"
  }
}
```

**Parameters:**
- `proposer`: Address submitting the proposal
- `title`: 1-140 characters
- `description`: 1-10,000 characters
- `proposal_type`: `parameter_change`, `software_upgrade`, `community_pool`, or `text`
- `initial_deposit`: Minimum 1,000 SLTN (1,000,000,000,000 with 9 decimals)
- `parameters`: Optional parameters for execution (depends on type)

**Response:**
```json
{
  "proposal_id": 1,
  "status": "submitted"
}
```

**Example (JavaScript):**
```javascript
const response = await fetch('http://localhost:3030/governance/propose', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    proposer: "sultan1proposer5k7m9n2q4s6u8w0y2a4c6e8g0i2k4m6",
    title: "Reduce Inflation Rate to 6%",
    description: "This proposal aims to reduce the network inflation rate from 8% to 6% to better align with long-term tokenomics. Lower inflation will:\n\n1. Reduce token dilution\n2. Increase scarcity\n3. Maintain validator rewards at sustainable levels\n\nThe change will take effect immediately upon passage.",
    proposal_type: "parameter_change",
    initial_deposit: 1_000_000_000_000, // 1,000 SLTN
    parameters: {
      inflation_rate: "0.06"
    }
  })
});
const data = await response.json();
console.log(`Proposal #${data.proposal_id} submitted`);
```

**Example (Python):**
```python
import requests

response = requests.post('http://localhost:3030/governance/propose', json={
    'proposer': 'sultan1proposer5k7m9n2q4s6u8w0y2a4c6e8g0i2k4m6',
    'title': 'Reduce Inflation Rate to 6%',
    'description': 'Proposal to reduce inflation from 8% to 6%...',
    'proposal_type': 'parameter_change',
    'initial_deposit': 1_000_000_000_000,
    'parameters': {
        'inflation_rate': '0.06'
    }
})
print(f"Proposal #{response.json()['proposal_id']} submitted")
```

**Example (curl):**
```bash
curl -X POST http://localhost:3030/governance/propose \
  -H "Content-Type: application/json" \
  -d '{
    "proposer": "sultan1proposer5k7m9n2q4s6u8w0y2a4c6e8g0i2k4m6",
    "title": "Reduce Inflation Rate to 6%",
    "description": "Proposal to reduce inflation from 8% to 6%...",
    "proposal_type": "parameter_change",
    "initial_deposit": 1000000000000,
    "parameters": {"inflation_rate": "0.06"}
  }'
```

---

### 2. Vote on Proposal

**POST** `/governance/vote`

Cast a vote on an active proposal.

**Request:**
```json
{
  "proposal_id": 1,
  "voter": "sultan1voter...",
  "option": "yes",
  "voting_power": 5000000000000
}
```

**Parameters:**
- `proposal_id`: The proposal ID
- `voter`: Your address
- `option`: `yes`, `no`, `abstain`, or `no_with_veto`
- `voting_power`: Your staked amount (must match actual stake)

**Response:**
```json
{
  "proposal_id": 1,
  "voter": "sultan1voter...",
  "status": "voted"
}
```

**Example (JavaScript):**
```javascript
// Get your voting power from staking
const delegations = await fetch(`http://localhost:3030/staking/delegations/sultan1voter...`)
  .then(r => r.json());
const votingPower = delegations.reduce((sum, d) => sum + d.amount, 0);

// Cast your vote
const response = await fetch('http://localhost:3030/governance/vote', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    proposal_id: 1,
    voter: "sultan1voter5k7m9n2q4s6u8w0y2a4c6e8g0i2k4m6",
    option: "yes",
    voting_power: votingPower
  })
});
const data = await response.json();
console.log('Vote cast:', data);
```

**Example (Python):**
```python
import requests

# Get voting power
delegations = requests.get('http://localhost:3030/staking/delegations/sultan1voter...').json()
voting_power = sum(d['amount'] for d in delegations)

# Cast vote
response = requests.post('http://localhost:3030/governance/vote', json={
    'proposal_id': 1,
    'voter': 'sultan1voter5k7m9n2q4s6u8w0y2a4c6e8g0i2k4m6',
    'option': 'yes',
    'voting_power': voting_power
})
print('Vote cast:', response.json())
```

---

### 3. Get All Proposals

**GET** `/governance/proposals`

List all governance proposals (newest first).

**Response:**
```json
[
  {
    "id": 1,
    "proposer": "sultan1proposer...",
    "title": "Reduce Inflation Rate to 6%",
    "description": "Proposal to reduce inflation...",
    "proposal_type": "ParameterChange",
    "status": "Voting",
    "submit_height": 10000,
    "voting_end_height": 110800,
    "total_deposit": 1000000000000,
    "depositors": ["sultan1proposer..."],
    "final_tally": null,
    "parameters": {"inflation_rate": "0.06"}
  }
]
```

**Status Values:**
- `Voting`: Active voting period
- `Passed`: Quorum reached, vote passed, executed
- `Rejected`: Quorum reached, vote failed
- `Failed`: Quorum not reached
- `Vetoed`: >33.4% voted NoWithVeto

**Example (JavaScript):**
```javascript
const response = await fetch('http://localhost:3030/governance/proposals');
const proposals = await response.json();

proposals.forEach(p => {
  console.log(`\nProposal #${p.id}: ${p.title}`);
  console.log(`  Status: ${p.status}`);
  console.log(`  Type: ${p.proposal_type}`);
  console.log(`  Proposer: ${p.proposer}`);
  console.log(`  Voting ends at block: ${p.voting_end_height}`);
  
  if (p.final_tally) {
    console.log(`  Results:`);
    console.log(`    Yes: ${p.final_tally.yes}`);
    console.log(`    No: ${p.final_tally.no}`);
    console.log(`    Abstain: ${p.final_tally.abstain}`);
    console.log(`    NoWithVeto: ${p.final_tally.no_with_veto}`);
  }
});
```

---

### 4. Get Single Proposal

**GET** `/governance/proposal/:id`

Get details of a specific proposal.

**Response:**
```json
{
  "id": 1,
  "proposer": "sultan1proposer...",
  "title": "Reduce Inflation Rate to 6%",
  "description": "Proposal to reduce inflation from 8% to 6%...",
  "proposal_type": "ParameterChange",
  "status": "Voting",
  "submit_height": 10000,
  "voting_end_height": 110800,
  "total_deposit": 1000000000000,
  "depositors": ["sultan1proposer..."],
  "final_tally": null,
  "parameters": {"inflation_rate": "0.06"}
}
```

**Example (JavaScript):**
```javascript
const proposalId = 1;
const response = await fetch(`http://localhost:3030/governance/proposal/${proposalId}`);
const proposal = await response.json();

console.log(`Proposal #${proposal.id}`);
console.log(`Title: ${proposal.title}`);
console.log(`Status: ${proposal.status}`);
console.log(`Description:\n${proposal.description}`);
```

---

### 5. Tally Proposal

**POST** `/governance/tally/:id`

Manually tally votes for a proposal (normally done automatically).

**Response:**
```json
{
  "yes": 50000000000000,
  "no": 10000000000000,
  "abstain": 5000000000000,
  "no_with_veto": 2000000000000,
  "total_voting_power": 67000000000000,
  "quorum_reached": true,
  "passed": true,
  "vetoed": false
}
```

**Example (JavaScript):**
```javascript
const proposalId = 1;
const response = await fetch(`http://localhost:3030/governance/tally/${proposalId}`, {
  method: 'POST'
});
const tally = await response.json();

console.log('=== Proposal Results ===');
console.log(`Yes: ${tally.yes / 1e9} SLTN`);
console.log(`No: ${tally.no / 1e9} SLTN`);
console.log(`Abstain: ${tally.abstain / 1e9} SLTN`);
console.log(`NoWithVeto: ${tally.no_with_veto / 1e9} SLTN`);
console.log(`Total Voting Power: ${tally.total_voting_power / 1e9} SLTN`);
console.log(`Quorum Reached: ${tally.quorum_reached ? 'YES' : 'NO'}`);
console.log(`Passed: ${tally.passed ? 'YES' : 'NO'}`);
console.log(`Vetoed: ${tally.vetoed ? 'YES' : 'NO'}`);
```

---

### 6. Get Governance Statistics

**GET** `/governance/statistics`

Get network-wide governance statistics.

**Response:**
```json
{
  "total_proposals": 25,
  "active_proposals": 3,
  "passed_proposals": 18,
  "rejected_proposals": 3,
  "failed_proposals": 1,
  "current_height": 150000,
  "total_bonded": 500000000000000
}
```

**Example (JavaScript):**
```javascript
const response = await fetch('http://localhost:3030/governance/statistics');
const stats = await response.json();

console.log('=== Governance Statistics ===');
console.log(`Total Proposals: ${stats.total_proposals}`);
console.log(`Active Proposals: ${stats.active_proposals}`);
console.log(`Passed: ${stats.passed_proposals}`);
console.log(`Rejected: ${stats.rejected_proposals}`);
console.log(`Failed (no quorum): ${stats.failed_proposals}`);
console.log(`Total Bonded: ${stats.total_bonded / 1e9} SLTN`);
console.log(`Current Block: ${stats.current_height}`);
```

---

## 💡 Voting Power Calculation

### How Voting Power Works

Your voting power equals your **total staked tokens**:
- Validator self-stake
- All delegations to validators

### Example

**User Stakes:**
- Delegated to Validator A: 1,000 SLTN
- Delegated to Validator B: 500 SLTN
- **Total Voting Power: 1,500 SLTN**

**Network:**
- Total Bonded: 100,000 SLTN
- **Your Vote Weight: 1.5%**

---

## 📊 Tally Calculation

### Quorum Check

```
Total Voting Power / Total Bonded >= 0.334 (33.4%)
```

**Example:**
- Total Bonded: 100,000 SLTN
- Total Votes Cast: 40,000 SLTN
- Quorum: 40,000 / 100,000 = 40% ✅ REACHED

### Veto Check

```
NoWithVeto / Total Voting Power > 0.334 (33.4%)
```

**Example:**
- Total Voting Power: 40,000 SLTN
- NoWithVeto: 15,000 SLTN
- Veto Ratio: 15,000 / 40,000 = 37.5% ✅ VETOED

### Pass Check

```
Yes / (Yes + No) > 0.50 (50%)
AND Quorum Reached
AND NOT Vetoed
```

**Example:**
- Yes: 25,000 SLTN
- No: 10,000 SLTN
- Abstain: 3,000 SLTN
- NoWithVeto: 2,000 SLTN
- Pass Ratio: 25,000 / (25,000 + 10,000) = 71.4% ✅ PASSED

---

## 📋 Proposal Lifecycle

### 1. Submit (Block 10,000)
```javascript
// User submits proposal with 1,000 SLTN deposit
const proposal = await submitProposal({
  title: "Reduce Inflation",
  deposit: 1_000_000_000_000
});
// Status: Voting
// Voting ends at: 10,000 + 100,800 = 110,800
```

### 2. Voting Period (Blocks 10,000 - 110,800)
```javascript
// Users vote with their staking power
await vote({
  proposal_id: 1,
  option: "yes",
  voting_power: 5_000_000_000_000
});
```

### 3. Voting Ends (Block 110,800)
```
Automatic tally:
- Yes: 50,000 SLTN (60%)
- No: 20,000 SLTN (24%)
- Abstain: 10,000 SLTN (12%)
- NoWithVeto: 3,000 SLTN (4%)
Total: 83,000 SLTN

Network bonded: 200,000 SLTN
Quorum: 83,000 / 200,000 = 41.5% ✅
Veto: 3,000 / 83,000 = 3.6% ✅ (below 33.4%)
Pass: 50,000 / 70,000 = 71.4% ✅

Result: PASSED ✅
```

### 4. Execution (Automatic)
```
For ParameterChange proposals:
- Parameters updated immediately
- inflation_rate changed from 0.08 to 0.06
- Takes effect on next block

For SoftwareUpgrade:
- Upgrade scheduled
- Validators notified
- Upgrade happens at target block

For CommunityPool:
- Funds transferred
- Treasury updated

For TextProposal:
- Signaling only, no execution
```

---

## 🎯 Proposal Types in Detail

### 1. ParameterChange

Change on-chain parameters.

**Common Parameters:**
- `inflation_rate`: Network inflation (0.0 - 1.0)
- `min_validator_stake`: Minimum to become validator
- `max_block_size`: Maximum block size in bytes
- `block_time`: Time between blocks
- `quorum`: Governance quorum requirement
- `pass_threshold`: Governance pass threshold

**Example:**
```javascript
{
  "proposal_type": "parameter_change",
  "parameters": {
    "inflation_rate": "0.06",
    "min_validator_stake": "10000000000000"
  }
}
```

### 2. SoftwareUpgrade

Schedule a network upgrade.

**Parameters:**
- `upgrade_name`: Name of the upgrade
- `upgrade_height`: Block height for upgrade
- `upgrade_info`: JSON with upgrade details

**Example:**
```javascript
{
  "proposal_type": "software_upgrade",
  "parameters": {
    "upgrade_name": "v2.0.0",
    "upgrade_height": "200000",
    "upgrade_info": '{"binary_url": "https://..."}'
  }
}
```

### 3. CommunityPool

Spend from community treasury.

**Parameters:**
- `recipient`: Address to receive funds
- `amount`: Amount to transfer

**Example:**
```javascript
{
  "proposal_type": "community_pool",
  "parameters": {
    "recipient": "sultan1dev...",
    "amount": "50000000000000"
  }
}
```

### 4. TextProposal

Signaling proposal (no execution).

**Parameters:** None (optional metadata)

**Example:**
```javascript
{
  "proposal_type": "text",
  "title": "Should we integrate with XYZ protocol?",
  "description": "Community sentiment proposal..."
}
```

---

## 🎓 Best Practices

### For Proposal Authors

1. **Clear Title**: Keep under 140 characters
2. **Detailed Description**: Explain rationale, implementation, impact
3. **Community Discussion**: Discuss before submitting
4. **Realistic Parameters**: Test proposed values
5. **Sufficient Deposit**: 1,000 SLTN minimum

### For Voters

1. **Research Proposals**: Read full description
2. **Check Parameters**: Verify technical details
3. **Vote Deadline**: Vote before voting period ends
4. **Use NoWithVeto Carefully**: Strong opposition only
5. **Consider Impact**: Think long-term

### Proposal Template

```markdown
# Proposal Title (Short & Clear)

## Summary
One paragraph overview

## Motivation
Why is this change needed?

## Specification
Technical details of the change

## Implementation
How will it be executed?

## Risks
What could go wrong?

## Timeline
When will this take effect?

## Parameters
{
  "parameter_name": "value"
}
```

---

## 📊 Governance Dashboard

### Monitor Active Proposals

```javascript
async function monitorGovernance() {
  const stats = await fetch('http://localhost:3030/governance/statistics').then(r => r.json());
  const proposals = await fetch('http://localhost:3030/governance/proposals').then(r => r.json());
  
  console.log('=== Sultan Governance ===');
  console.log(`Total Proposals: ${stats.total_proposals}`);
  console.log(`Active: ${stats.active_proposals}`);
  console.log(`Success Rate: ${(stats.passed_proposals / stats.total_proposals * 100).toFixed(1)}%`);
  console.log(`\nActive Proposals:\n`);
  
  const active = proposals.filter(p => p.status === 'Voting');
  
  for (const proposal of active) {
    const blocksRemaining = proposal.voting_end_height - stats.current_height;
    const hoursRemaining = (blocksRemaining * 5) / 3600;
    
    console.log(`📜 Proposal #${proposal.id}: ${proposal.title}`);
    console.log(`   Type: ${proposal.proposal_type}`);
    console.log(`   Ends in ${blocksRemaining} blocks (~${hoursRemaining.toFixed(1)} hours)`);
    
    // Get current tally
    const tally = await fetch(`http://localhost:3030/governance/tally/${proposal.id}`, {
      method: 'POST'
    }).then(r => r.json());
    
    const totalVotes = tally.yes + tally.no + tally.abstain + tally.no_with_veto;
    
    console.log(`   Current Votes: ${totalVotes / 1e9} SLTN`);
    console.log(`   Yes: ${(tally.yes / totalVotes * 100).toFixed(1)}%`);
    console.log(`   No: ${(tally.no / totalVotes * 100).toFixed(1)}%`);
    console.log(`   NoWithVeto: ${(tally.no_with_veto / totalVotes * 100).toFixed(1)}%`);
    console.log(`   Quorum: ${tally.quorum_reached ? '✅' : '❌'}`);
    console.log(`   Trending: ${tally.passed ? '✅ PASSING' : '❌ FAILING'}\n`);
  }
}

// Run every 5 minutes
setInterval(monitorGovernance, 300000);
```

---

## 🔍 FAQ

**Q: Can I change my vote?**  
A: No, votes are immutable once cast. Choose carefully.

**Q: What happens to the deposit?**  
A: Currently returned to proposer. Future versions may burn or pool deposits.

**Q: Can proposals be canceled?**  
A: No, once submitted they run to completion.

**Q: What if quorum isn't reached?**  
A: Proposal fails with status "Failed".

**Q: Can I vote if I delegate?**  
A: Yes, your voting power equals your delegated stake.

**Q: Do validators vote on behalf of delegators?**  
A: No, delegators vote independently with their staking power.

**Q: How long do I have to vote?**  
A: 100,800 blocks (~7 days) from proposal submission.

**Q: Can I see who voted what?**  
A: Yes, all votes are on-chain and queryable.

---

## 🚀 Quick Start

### 1. Check Active Proposals
```bash
curl http://localhost:3030/governance/proposals
```

### 2. Get Your Voting Power
```bash
curl http://localhost:3030/staking/delegations/sultan1your...
```

### 3. Vote
```bash
curl -X POST http://localhost:3030/governance/vote \
  -H "Content-Type: application/json" \
  -d '{"proposal_id":1,"voter":"sultan1your...","option":"yes","voting_power":5000000000000}'
```

### 4. Submit Proposal
```bash
curl -X POST http://localhost:3030/governance/propose \
  -H "Content-Type: application/json" \
  -d '{"proposer":"sultan1your...","title":"My Proposal","description":"Details...","proposal_type":"text","initial_deposit":1000000000000}'
```

---

## 📊 Governance Statistics

| Metric | Example Value |
|--------|---------------|
| **Proposal Deposit** | 1,000 SLTN |
| **Voting Period** | ~7 days (100,800 blocks) |
| **Quorum Required** | 33.4% of bonded tokens |
| **Pass Threshold** | 50% of votes (excl. Abstain) |
| **Veto Threshold** | 33.4% of votes |
| **Average Participation** | Tracked per proposal |

---

## 📞 Support

- **Documentation**: `/GOVERNANCE_GUIDE.md`
- **Staking**: `/STAKING_GUIDE.md`  
- **API Reference**: `http://localhost:3030/docs`
- **Network Stats**: `http://localhost:8080`

**Sultan L1 - Production-Ready On-Chain Governance** 🗳️
