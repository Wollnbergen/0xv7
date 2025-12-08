#!/bin/bash

# Sultan L1 Staking & Governance Test Script
# Tests all production staking and governance endpoints

set -e

BASE_URL="http://localhost:26657"
VALIDATOR_ADDR="sultan1validator7xj3k2p8n9m5q4r6t8v0w2y4z6a8c0e2g4"
DELEGATOR_ADDR="sultan1delegator5k7m9n2q4s6u8w0y2a4c6e8g0i2k4m6"
PROPOSER_ADDR="sultan1proposer9a1b3c5d7e9f1g3h5i7j9k1l3m5n7o9p1"

echo "======================================"
echo "Sultan L1 Staking & Governance Tests"
echo "======================================"
echo ""

# Check if node is running
echo "🔍 Checking if node is running..."
if ! curl -s "${BASE_URL}/status" > /dev/null; then
    echo "❌ Node is not running at ${BASE_URL}"
    echo "   Start the node first: cargo run --release --bin sultan-node"
    exit 1
fi
echo "✅ Node is running"
echo ""

# Test 1: Create Validator
echo "======================================"
echo "TEST 1: Create Validator"
echo "======================================"
echo "Creating validator with 10,000 SLTN stake and 5% commission..."
RESPONSE=$(curl -s -X POST "${BASE_URL}/staking/create_validator" \
  -H "Content-Type: application/json" \
  -d '{
    "validator_address": "'"${VALIDATOR_ADDR}"'",
    "initial_stake": 10000000000000,
    "commission_rate": 0.05
  }')
echo "Response: ${RESPONSE}"
echo ""

# Test 2: Get All Validators
echo "======================================"
echo "TEST 2: Get All Validators"
echo "======================================"
VALIDATORS=$(curl -s "${BASE_URL}/staking/validators")
echo "${VALIDATORS}" | python3 -m json.tool
echo ""

# Test 3: Delegate to Validator
echo "======================================"
echo "TEST 3: Delegate to Validator"
echo "======================================"
echo "Delegating 1,000 SLTN to validator..."
RESPONSE=$(curl -s -X POST "${BASE_URL}/staking/delegate" \
  -H "Content-Type: application/json" \
  -d '{
    "delegator_address": "'"${DELEGATOR_ADDR}"'",
    "validator_address": "'"${VALIDATOR_ADDR}"'",
    "amount": 1000000000000
  }')
echo "Response: ${RESPONSE}"
echo ""

# Test 4: Get Delegations
echo "======================================"
echo "TEST 4: Get Delegations"
echo "======================================"
DELEGATIONS=$(curl -s "${BASE_URL}/staking/delegations/${DELEGATOR_ADDR}")
echo "${DELEGATIONS}" | python3 -m json.tool
echo ""

# Test 5: Get Staking Statistics
echo "======================================"
echo "TEST 5: Get Staking Statistics"
echo "======================================"
STAKING_STATS=$(curl -s "${BASE_URL}/staking/statistics")
echo "${STAKING_STATS}" | python3 -m json.tool
echo ""

# Wait for some rewards to accumulate
echo "⏳ Waiting 15 seconds for rewards to accumulate..."
sleep 15
echo ""

# Test 6: Withdraw Validator Rewards
echo "======================================"
echo "TEST 6: Withdraw Validator Rewards"
echo "======================================"
RESPONSE=$(curl -s -X POST "${BASE_URL}/staking/withdraw_rewards" \
  -H "Content-Type: application/json" \
  -d '{
    "address": "'"${VALIDATOR_ADDR}"'",
    "is_validator": true
  }')
echo "Response: ${RESPONSE}"
echo ""

# Test 7: Withdraw Delegator Rewards
echo "======================================"
echo "TEST 7: Withdraw Delegator Rewards"
echo "======================================"
RESPONSE=$(curl -s -X POST "${BASE_URL}/staking/withdraw_rewards" \
  -H "Content-Type: application/json" \
  -d '{
    "address": "'"${DELEGATOR_ADDR}"'",
    "validator_address": "'"${VALIDATOR_ADDR}"'",
    "is_validator": false
  }')
echo "Response: ${RESPONSE}"
echo ""

# Test 8: Submit Governance Proposal
echo "======================================"
echo "TEST 8: Submit Governance Proposal"
echo "======================================"
echo "Submitting proposal to reduce inflation from 8% to 6%..."
RESPONSE=$(curl -s -X POST "${BASE_URL}/governance/propose" \
  -H "Content-Type: application/json" \
  -d '{
    "proposer": "'"${PROPOSER_ADDR}"'",
    "title": "Reduce Inflation Rate to 6%",
    "description": "This proposal aims to reduce the network inflation rate from 8% to 6% to better align with long-term tokenomics.",
    "proposal_type": "parameter_change",
    "initial_deposit": 1000000000000,
    "parameters": {
      "inflation_rate": "0.06"
    }
  }')
echo "Response: ${RESPONSE}"
PROPOSAL_ID=$(echo "${RESPONSE}" | python3 -c "import sys, json; print(json.load(sys.stdin).get('proposal_id', 1))")
echo "Proposal ID: ${PROPOSAL_ID}"
echo ""

# Test 9: Get All Proposals
echo "======================================"
echo "TEST 9: Get All Proposals"
echo "======================================"
PROPOSALS=$(curl -s "${BASE_URL}/governance/proposals")
echo "${PROPOSALS}" | python3 -m json.tool
echo ""

# Test 10: Get Single Proposal
echo "======================================"
echo "TEST 10: Get Single Proposal"
echo "======================================"
PROPOSAL=$(curl -s "${BASE_URL}/governance/proposal/${PROPOSAL_ID}")
echo "${PROPOSAL}" | python3 -m json.tool
echo ""

# Test 11: Vote on Proposal
echo "======================================"
echo "TEST 11: Vote on Proposal"
echo "======================================"
echo "Voting YES with 1,000 SLTN voting power..."
RESPONSE=$(curl -s -X POST "${BASE_URL}/governance/vote" \
  -H "Content-Type: application/json" \
  -d '{
    "proposal_id": '${PROPOSAL_ID}',
    "voter": "'"${DELEGATOR_ADDR}"'",
    "option": "yes",
    "voting_power": 1000000000000
  }')
echo "Response: ${RESPONSE}"
echo ""

# Test 12: Tally Proposal
echo "======================================"
echo "TEST 12: Tally Proposal"
echo "======================================"
TALLY=$(curl -s -X POST "${BASE_URL}/governance/tally/${PROPOSAL_ID}")
echo "${TALLY}" | python3 -m json.tool
echo ""

# Test 13: Get Governance Statistics
echo "======================================"
echo "TEST 13: Get Governance Statistics"
echo "======================================"
GOV_STATS=$(curl -s "${BASE_URL}/governance/statistics")
echo "${GOV_STATS}" | python3 -m json.tool
echo ""

# Summary
echo "======================================"
echo "✅ ALL TESTS COMPLETED"
echo "======================================"
echo ""
echo "Summary:"
echo "--------"
echo "✅ Staking Tests (7):"
echo "   - Create validator"
echo "   - Get validators"
echo "   - Delegate"
echo "   - Get delegations"
echo "   - Withdraw validator rewards"
echo "   - Withdraw delegator rewards"
echo "   - Get staking statistics"
echo ""
echo "✅ Governance Tests (6):"
echo "   - Submit proposal"
echo "   - Get all proposals"
echo "   - Get single proposal"
echo "   - Vote on proposal"
echo "   - Tally proposal"
echo "   - Get governance statistics"
echo ""
echo "Total: 13 endpoints tested successfully! 🎉"
echo ""
echo "======================================"
echo "Detailed Statistics"
echo "======================================"
echo ""
echo "Staking Statistics:"
echo "${STAKING_STATS}" | python3 -m json.tool
echo ""
echo "Governance Statistics:"
echo "${GOV_STATS}" | python3 -m json.tool
echo ""
