#!/bin/bash
# Sultan Testnet Testing Script
# Tests all functionality of a deployed testnet node

set -e

BOLD='\033[1m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

RPC_ENDPOINT="${RPC_ENDPOINT:-http://localhost:26657}"
API_ENDPOINT="${API_ENDPOINT:-http://localhost:1317}"
GRPC_ENDPOINT="${GRPC_ENDPOINT:-localhost:9090}"

passed=0
failed=0

print_header() {
    echo -e "${BOLD}$1${NC}"
    echo "$(printf '=%.0s' {1..60})"
}

print_test() {
    echo -e "${YELLOW}Testing:${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
    ((passed++))
}

print_failure() {
    echo -e "${RED}✗${NC} $1"
    ((failed++))
}

test_endpoint() {
    local name=$1
    local url=$2
    
    print_test "$name connectivity"
    if curl -sf "$url" > /dev/null 2>&1; then
        print_success "$name is reachable"
        return 0
    else
        print_failure "$name is not reachable"
        return 1
    fi
}

test_json_endpoint() {
    local name=$1
    local url=$2
    local expected_field=$3
    
    print_test "$name response"
    response=$(curl -sf "$url" 2>&1)
    if echo "$response" | jq -e "$expected_field" > /dev/null 2>&1; then
        print_success "$name returns valid JSON with $expected_field"
        return 0
    else
        print_failure "$name does not return expected data"
        return 1
    fi
}

echo ""
print_header "🧪 Sultan Testnet Testing Suite"
echo ""

# Test 1: RPC Health
print_header "1. RPC Endpoint Tests"
test_endpoint "RPC health" "$RPC_ENDPOINT/health"
test_json_endpoint "RPC status" "$RPC_ENDPOINT/status" ".result.node_info.network"
test_json_endpoint "RPC sync info" "$RPC_ENDPOINT/status" ".result.sync_info.latest_block_height"
echo ""

# Test 2: Get chain info
print_header "2. Chain Information"
print_test "Chain ID"
chain_id=$(curl -sf "$RPC_ENDPOINT/status" | jq -r '.result.node_info.network')
if [ -n "$chain_id" ]; then
    print_success "Chain ID: $chain_id"
else
    print_failure "Could not retrieve chain ID"
fi

print_test "Latest block height"
height=$(curl -sf "$RPC_ENDPOINT/status" | jq -r '.result.sync_info.latest_block_height')
if [ "$height" -gt 0 ] 2>/dev/null; then
    print_success "Current height: $height"
else
    print_failure "Invalid block height"
fi

print_test "Catching up status"
catching_up=$(curl -sf "$RPC_ENDPOINT/status" | jq -r '.result.sync_info.catching_up')
if [ "$catching_up" == "false" ]; then
    print_success "Node is fully synced"
else
    print_failure "Node is still catching up"
fi
echo ""

# Test 3: Validator info
print_header "3. Validator Tests"
print_test "Validator info"
validator_addr=$(curl -sf "$RPC_ENDPOINT/status" | jq -r '.result.validator_info.address')
if [ -n "$validator_addr" ] && [ "$validator_addr" != "null" ]; then
    print_success "Validator address: $validator_addr"
else
    print_failure "Node is not a validator"
fi

print_test "Voting power"
voting_power=$(curl -sf "$RPC_ENDPOINT/status" | jq -r '.result.validator_info.voting_power')
if [ "$voting_power" -gt 0 ] 2>/dev/null; then
    print_success "Voting power: $voting_power"
else
    print_failure "No voting power"
fi
echo ""

# Test 4: Network connectivity
print_header "4. P2P Network Tests"
print_test "Peer connections"
num_peers=$(curl -sf "$RPC_ENDPOINT/net_info" | jq -r '.result.n_peers')
if [ -n "$num_peers" ]; then
    print_success "Connected peers: $num_peers"
else
    print_failure "Could not get peer count"
fi

print_test "Node is listening"
listening=$(curl -sf "$RPC_ENDPOINT/net_info" | jq -r '.result.listening')
if [ "$listening" == "true" ]; then
    print_success "P2P is listening"
else
    print_failure "P2P is not listening"
fi
echo ""

# Test 5: Block production
print_header "5. Block Production Tests"
print_test "Block production rate"
height1=$(curl -sf "$RPC_ENDPOINT/status" | jq -r '.result.sync_info.latest_block_height')
sleep 6
height2=$(curl -sf "$RPC_ENDPOINT/status" | jq -r '.result.sync_info.latest_block_height')
blocks_produced=$((height2 - height1))
if [ "$blocks_produced" -gt 0 ]; then
    print_success "Produced $blocks_produced blocks in 6 seconds"
else
    print_failure "No blocks produced in 6 seconds"
fi

print_test "Block time"
block_time=$(curl -sf "$RPC_ENDPOINT/status" | jq -r '.result.sync_info.latest_block_time')
if [ -n "$block_time" ] && [ "$block_time" != "null" ]; then
    print_success "Latest block time: $block_time"
else
    print_failure "Could not get block time"
fi
echo ""

# Test 6: API endpoint
print_header "6. REST API Tests"
test_endpoint "API health" "$API_ENDPOINT/cosmos/base/tendermint/v1beta1/node_info"

print_test "API node info"
if curl -sf "$API_ENDPOINT/cosmos/base/tendermint/v1beta1/node_info" | jq -e '.default_node_info.network' > /dev/null 2>&1; then
    print_success "API returns node info"
else
    print_failure "API node info failed"
fi
echo ""

# Test 7: Module queries
print_header "7. Module Query Tests"

print_test "Auth module"
if curl -sf "$API_ENDPOINT/cosmos/auth/v1beta1/params" | jq -e '.params' > /dev/null 2>&1; then
    print_success "Auth module is accessible"
else
    print_failure "Auth module query failed"
fi

print_test "Bank module"
if curl -sf "$API_ENDPOINT/cosmos/bank/v1beta1/params" | jq -e '.params' > /dev/null 2>&1; then
    print_success "Bank module is accessible"
else
    print_failure "Bank module query failed"
fi

print_test "Staking module"
if curl -sf "$API_ENDPOINT/cosmos/staking/v1beta1/params" | jq -e '.params' > /dev/null 2>&1; then
    print_success "Staking module is accessible"
else
    print_failure "Staking module query failed"
fi

print_test "Validators list"
validators=$(curl -sf "$API_ENDPOINT/cosmos/staking/v1beta1/validators" | jq -r '.validators | length')
if [ "$validators" -gt 0 ] 2>/dev/null; then
    print_success "Found $validators validator(s)"
else
    print_failure "No validators found"
fi
echo ""

# Test 8: Consensus params
print_header "8. Consensus Parameters"
print_test "Consensus params"
if curl -sf "$RPC_ENDPOINT/consensus_params" | jq -e '.result.consensus_params' > /dev/null 2>&1; then
    print_success "Consensus params accessible"
    max_gas=$(curl -sf "$RPC_ENDPOINT/consensus_params" | jq -r '.result.consensus_params.block.max_gas')
    print_success "Max block gas: $max_gas"
else
    print_failure "Consensus params query failed"
fi
echo ""

# Test 9: Genesis
print_header "9. Genesis Tests"
print_test "Genesis hash"
genesis_hash=$(curl -sf "$RPC_ENDPOINT/genesis" | jq -r '.result.genesis.consensus_params.block.max_bytes')
if [ -n "$genesis_hash" ] && [ "$genesis_hash" != "null" ]; then
    print_success "Genesis is accessible"
else
    print_failure "Genesis query failed"
fi
echo ""

# Test 10: Mempool
print_header "10. Mempool Tests"
print_test "Mempool size"
mempool_size=$(curl -sf "$RPC_ENDPOINT/num_unconfirmed_txs" | jq -r '.result.total')
if [ -n "$mempool_size" ]; then
    print_success "Mempool size: $mempool_size"
else
    print_failure "Mempool query failed"
fi
echo ""

# Test 11: ABCI Info
print_header "11. ABCI Application Tests"
print_test "ABCI info"
app_version=$(curl -sf "$RPC_ENDPOINT/abci_info" | jq -r '.result.response.version')
if [ -n "$app_version" ]; then
    print_success "App version: $app_version"
else
    print_failure "ABCI info failed"
fi
echo ""

# Test 12: Port accessibility
print_header "12. Port Accessibility Tests"

check_port() {
    local port=$1
    local name=$2
    print_test "Port $port ($name)"
    if nc -z localhost $port 2>/dev/null; then
        print_success "Port $port is open"
    else
        print_failure "Port $port is not accessible"
    fi
}

check_port 26656 "P2P"
check_port 26657 "RPC"
check_port 9090 "gRPC"
check_port 1317 "REST API"
echo ""

# Summary
print_header "📊 Test Summary"
total=$((passed + failed))
pass_rate=0
if [ $total -gt 0 ]; then
    pass_rate=$((passed * 100 / total))
fi

echo ""
echo -e "${GREEN}Passed:${NC} $passed"
echo -e "${RED}Failed:${NC} $failed"
echo -e "${BOLD}Total:${NC}  $total"
echo -e "${BOLD}Pass Rate:${NC} $pass_rate%"
echo ""

if [ $failed -eq 0 ]; then
    echo -e "${GREEN}${BOLD}🎉 All tests passed! Testnet is fully functional.${NC}"
    exit 0
elif [ $pass_rate -ge 80 ]; then
    echo -e "${YELLOW}${BOLD}⚠️  Most tests passed, but some issues detected.${NC}"
    exit 0
else
    echo -e "${RED}${BOLD}❌ Critical failures detected. Please check your configuration.${NC}"
    exit 1
fi
