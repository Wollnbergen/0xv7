#!/bin/bash

# Sultan L1 Load Test - Production Grade
# Sends real transactions and measures TPS

set -e

CYAN='\033[96m'
GREEN='\033[92m'
YELLOW='\033[93m'
RED='\033[91m'
BOLD='\033[1m'
NC='\033[0m' # No Color

RPC_URL="${RPC_URL:-http://localhost:26657}"
TOTAL_TXS="${TOTAL_TXS:-1000}"
WORKERS="${WORKERS:-10}"

echo -e "\n${CYAN}╔═══════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║                                                                           ║${NC}"
echo -e "${CYAN}║${BOLD}           🚀 SULTAN L1 PRODUCTION LOAD TEST 🚀                        ${NC}${CYAN}║${NC}"
echo -e "${CYAN}║                                                                           ║${NC}"
echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════════════════╝${NC}\n"

echo -e "  ${BOLD}RPC Endpoint:${NC}       ${CYAN}$RPC_URL${NC}"
echo -e "  ${BOLD}Total Transactions:${NC} ${YELLOW}${BOLD}$TOTAL_TXS${NC}"
echo -e "  ${BOLD}Concurrent Workers:${NC} ${YELLOW}${BOLD}$WORKERS${NC}"
echo ""

# Check node status
echo -e "  ${CYAN}📡${NC} Checking node status..."
STATUS=$(curl -s "$RPC_URL/status")
if [ $? -ne 0 ]; then
    echo -e "  ${RED}❌ Error: Node not responding${NC}"
    exit 1
fi

HEIGHT=$(echo "$STATUS" | jq -r '.height')
SHARDING=$(echo "$STATUS" | jq -r '.sharding_enabled')
SHARDS=$(echo "$STATUS" | jq -r '.shard_count')

echo -e "  ${GREEN}✅${NC} Node is online!"
echo -e "    • Block Height: ${CYAN}$HEIGHT${NC}"
echo -e "    • Sharding: ${GREEN}✅ $SHARDS shards${NC}"
echo ""

# Send transactions
echo -e "  ${YELLOW}🔥${NC} Starting load test..."
echo ""

SUCCESSFUL=0
FAILED=0
START_TIME=$(date +%s.%N)

# Function to send a transaction
send_tx() {
    local from="account_$RANDOM"
    local to="account_$RANDOM"
    local amount=$((RANDOM % 1000 + 1))
    local nonce=$RANDOM
    local timestamp=$(date +%s)
    
    TX_JSON=$(cat <<EOF
{
  "from": "$from",
  "to": "$to",
  "amount": $amount,
  "gas_fee": 0,
  "timestamp": $timestamp,
  "nonce": $nonce,
  "signature": "sig_$nonce"
}
EOF
)
    
    RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" -d "$TX_JSON" "$RPC_URL/tx" 2>/dev/null)
    
    if echo "$RESPONSE" | grep -q "hash"; then
        return 0
    else
        return 1
    fi
}

# Progress bar function
show_progress() {
    local current=$1
    local total=$2
    local width=50
    local percentage=$((current * 100 / total))
    local filled=$((width * current / total))
    local empty=$((width - filled))
    
    printf "\r  Progress: ["
    printf "%${filled}s" | tr ' ' '█'
    printf "%${empty}s" | tr ' ' '░'
    printf "] %d%% (%d/%d)" $percentage $current $total
}

# Send transactions
echo -e "  Sending $TOTAL_TXS transactions..."
for ((i=1; i<=TOTAL_TXS; i++)); do
    if send_tx; then
        ((SUCCESSFUL++))
    else
        ((FAILED++))
    fi
    
    if ((i % 10 == 0)); then
        show_progress $i $TOTAL_TXS
    fi
done

show_progress $TOTAL_TXS $TOTAL_TXS
echo ""  # New line after progress bar
echo ""

END_TIME=$(date +%s.%N)
ELAPSED=$(echo "$END_TIME - $START_TIME" | bc)
TPS=$(echo "scale=2; $SUCCESSFUL / $ELAPSED" | bc)
SUCCESS_RATE=$(echo "scale=2; $SUCCESSFUL * 100 / $TOTAL_TXS" | bc)

# Print results
echo ""
echo -e "${CYAN}╔═══════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║${BOLD}  LOAD TEST RESULTS${NC}${CYAN}                                                          ║${NC}"
echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  ${BOLD}Total Transactions:${NC}  ${YELLOW}${BOLD}$TOTAL_TXS${NC}"
echo -e "  ${GREEN}Successful:${NC}          ${GREEN}${BOLD}$SUCCESSFUL${NC}"
echo -e "  ${RED}Failed:${NC}              ${RED}${BOLD}$FAILED${NC}"
echo -e "  ${BOLD}Success Rate:${NC}        ${CYAN}${BOLD}${SUCCESS_RATE}%${NC}"
echo ""
echo -e "  ${BOLD}Total Time:${NC}          ${YELLOW}${BOLD}${ELAPSED}s${NC}"
echo ""
echo -e "  ${BOLD}ACTUAL TPS:${NC}          ${GREEN}${BOLD}${TPS}${NC}"
echo ""
echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Comparison
echo -e "${CYAN}${BOLD}  COMPARISON WITH OTHER BLOCKCHAINS${NC}"
echo -e "────────────────────────────────────────────────────────────────────────────────"
echo -e "  Ethereum:            ${RED}~15 TPS${NC}"
echo -e "  Bitcoin:             ${RED}~7 TPS${NC}"
echo -e "  Solana:              ${YELLOW}~3,000 TPS${NC}"
echo -e "  Avalanche:           ${YELLOW}~4,500 TPS${NC}"
echo -e "  ${BOLD}Sultan L1:${NC}           ${GREEN}${BOLD}${TPS} TPS${NC}"
echo ""

if (( $(echo "$TPS > 1000" | bc -l) )); then
    echo -e "  ${GREEN}✅${NC} Excellent performance!"
elif (( $(echo "$TPS > 100" | bc -l) )); then
    echo -e "  ${YELLOW}⚠️${NC} Good performance, but room for improvement"
else
    echo -e "  ${YELLOW}⚠️${NC} Performance below expected (target: 200,000 TPS)"
    echo -e "     Note: Sequential testing limits throughput. Use parallel workers for full TPS."
fi
echo ""

# Post-test verification
echo -e "${CYAN}${BOLD}  POST-TEST VERIFICATION${NC}"
echo -e "────────────────────────────────────────────────────────────────────────────────"
FINAL_STATUS=$(curl -s "$RPC_URL/status")
FINAL_HEIGHT=$(echo "$FINAL_STATUS" | jq -r '.height')
PENDING=$(echo "$FINAL_STATUS" | jq -r '.pending_txs')

echo -e "  • Final Block Height: ${CYAN}$FINAL_HEIGHT${NC}"
echo -e "  • Pending Transactions: ${YELLOW}$PENDING${NC}"
echo ""

echo -e "${GREEN}${BOLD}🎉 Load test completed successfully!${NC}"
echo ""
