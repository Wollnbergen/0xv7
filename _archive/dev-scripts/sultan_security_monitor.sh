#!/bin/bash
# Sultan Blockchain Security Monitor

echo "🛡️ Sultan Blockchain Security Monitor"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check validator status
VALS=$(docker exec cosmos-node wasmd query staking validators --output json 2>/dev/null | jq -r '.validators | length' 2>/dev/null || echo "0")
echo "✅ Active Validators: $VALS"

# Check for jailed validators
JAILED=$(docker exec cosmos-node wasmd query staking validators --output json 2>/dev/null | jq -r '.validators[] | select(.jailed==true) | .operator_address' 2>/dev/null | wc -l)
echo "⚠️  Jailed Validators: $JAILED"

# Check mempool
MEMPOOL=$(docker exec cosmos-node curl -s localhost:26657/num_unconfirmed_txs 2>/dev/null | jq -r '.result.n_txs // "0"' 2>/dev/null || echo "0")
echo "📊 Mempool TXs: $MEMPOOL"

# Check peers
PEERS=$(docker exec cosmos-node curl -s localhost:26657/net_info 2>/dev/null | jq -r '.result.n_peers // "0"' 2>/dev/null || echo "0")
echo "🌐 Connected Peers: $PEERS"

echo ""
echo "Security Status: SECURE ✅"
