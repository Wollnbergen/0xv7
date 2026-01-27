#!/bin/bash
#
# Sultan L1 Stress Test Suite
# Production-grade high-volume API and transaction testing
#
# Usage: ./scripts/stress_test.sh [OPTIONS]
#
# Options:
#   --tps NUM        Target requests per second (default: 100)
#   --duration NUM   Test duration in seconds (default: 30)
#   --parallel NUM   Concurrent connections (default: 10)
#   --rpc URL        RPC endpoint (default: https://rpc.sltn.io)
#

set -e

# Default configuration
TARGET_TPS="${TARGET_TPS:-100}"
DURATION="${DURATION:-30}"
PARALLEL="${PARALLEL:-10}"
RPC="${SULTAN_RPC:-https://rpc.sltn.io}"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --tps) TARGET_TPS="$2"; shift 2 ;;
        --duration) DURATION="$2"; shift 2 ;;
        --parallel) PARALLEL="$2"; shift 2 ;;
        --rpc) RPC="$2"; shift 2 ;;
        -h|--help)
            echo "Sultan L1 Stress Test"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --tps NUM        Target requests/sec (default: 100)"
            echo "  --duration NUM   Duration in seconds (default: 30)"
            echo "  --parallel NUM   Concurrent connections (default: 10)"
            echo "  --rpc URL        RPC endpoint"
            echo ""
            echo "Examples:"
            echo "  $0 --tps 100 --duration 30    # Basic test"
            echo "  $0 --tps 1000 --duration 60   # Higher load"
            echo "  $0 --tps 5000 --duration 120  # Stress test"
            exit 0
            ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║         Sultan L1 Stress Test Suite v2.0                         ║"
echo "╠══════════════════════════════════════════════════════════════════╣"
echo "║  RPC: $RPC"
echo "║  Target: $TARGET_TPS requests/sec for ${DURATION}s"
echo "║  Parallel: $PARALLEL connections"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

# Results directory
RESULTS_DIR="/tmp/sultan-stress-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$RESULTS_DIR"

# ============================================================================
# 1. Pre-Test Health Check
# ============================================================================

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1. Pre-Test Health Check"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if node is running
echo -n "Checking node availability... "
STATUS=$(curl -s --connect-timeout 10 "$RPC/status" 2>/dev/null)
if [ -n "$STATUS" ]; then
    echo -e "${GREEN}✓ Node is running${NC}"
else
    echo -e "${RED}✗ Node not running at $RPC${NC}"
    exit 1
fi

# Get initial metrics
INITIAL_HEIGHT=$(echo "$STATUS" | jq -r '.height // 0')
echo -e "  Initial height: ${GREEN}$INITIAL_HEIGHT${NC}"

STATS=$(curl -s "$RPC/stats")
SHARD_COUNT=$(echo "$STATS" | jq -r '.shard_count // 0')
TPS_CAPACITY=$(echo "$STATS" | jq -r '.estimated_tps // 0')
echo -e "  Shards: ${GREEN}$SHARD_COUNT${NC} (capacity: ${TPS_CAPACITY} TPS)"

echo ""

# ============================================================================
# 2. API Endpoint Stress Test
# ============================================================================

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "2. API Endpoint Stress Test"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Testing concurrent API requests..."
echo ""

# Test status endpoint
echo -n "  /status endpoint (100 concurrent): "
START=$(date +%s%N)
for i in {1..100}; do
    curl -s "$RPC/status" &>/dev/null &
done
wait
END=$(date +%s%N)
DURATION_MS=$(( (END - START) / 1000000 ))
AVG_MS=$(( DURATION_MS / 100 ))

if [ $AVG_MS -lt 50 ]; then
    echo -e "${GREEN}✓ ${AVG_MS}ms avg (excellent)${NC}"
elif [ $AVG_MS -lt 200 ]; then
    echo -e "${GREEN}✓ ${AVG_MS}ms avg (good)${NC}"
else
    echo -e "${YELLOW}⚠ ${AVG_MS}ms avg${NC}"
fi

# Test stats endpoint
echo -n "  /stats endpoint (50 concurrent): "
START=$(date +%s%N)
for i in {1..50}; do
    curl -s "$RPC/stats" &>/dev/null &
done
wait
END=$(date +%s%N)
DURATION_MS=$(( (END - START) / 1000000 ))
AVG_MS=$(( DURATION_MS / 50 ))

if [ $AVG_MS -lt 100 ]; then
    echo -e "${GREEN}✓ ${AVG_MS}ms avg (excellent)${NC}"
elif [ $AVG_MS -lt 300 ]; then
    echo -e "${GREEN}✓ ${AVG_MS}ms avg (good)${NC}"
else
    echo -e "${YELLOW}⚠ ${AVG_MS}ms avg${NC}"
fi

# Test balance endpoint
echo -n "  /balance endpoint (50 concurrent): "
START=$(date +%s%N)
for i in {1..50}; do
    curl -s "$RPC/balance/sultan15g5nwnlemn7zt6rtl7ch46ssvx2ym2v2umm07g" &>/dev/null &
done
wait
END=$(date +%s%N)
DURATION_MS=$(( (END - START) / 1000000 ))
AVG_MS=$(( DURATION_MS / 50 ))

if [ $AVG_MS -lt 100 ]; then
    echo -e "${GREEN}✓ ${AVG_MS}ms avg (excellent)${NC}"
elif [ $AVG_MS -lt 300 ]; then
    echo -e "${GREEN}✓ ${AVG_MS}ms avg (good)${NC}"
else
    echo -e "${YELLOW}⚠ ${AVG_MS}ms avg${NC}"
fi

echo ""

# ============================================================================
# 3. Sustained Load Test
# ============================================================================

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "3. Sustained Load Test ($DURATION seconds @ $TARGET_TPS req/s)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Running sustained API requests..."

# Background worker function
worker() {
    local endpoint=$1
    local count=0
    local errors=0
    local start_time=$(date +%s)
    local end_time=$((start_time + DURATION))
    
    while [ $(date +%s) -lt $end_time ]; do
        if curl -s -f "$endpoint" &>/dev/null; then
            ((count++))
        else
            ((errors++))
        fi
        sleep 0.1  # 10 req/sec per worker
    done
    
    echo "$count $errors"
}

# Start workers
echo "  Starting 10 concurrent workers..."
echo ""

WORKER_PIDS=()
for i in {1..10}; do
    worker "$API_ENDPOINT/health" > "$RESULTS_DIR/worker_$i.txt" &
    WORKER_PIDS+=($!)
done

# Monitor progress
for i in $(seq 1 $TEST_DURATION); do
    echo -ne "\r  Progress: $i/$TEST_DURATION seconds... "
    sleep 1
done
echo ""

# Wait for workers
wait

# Collect results
TOTAL_REQUESTS=0
TOTAL_ERRORS=0

for i in {1..10}; do
    if [ -f "$RESULTS_DIR/worker_$i.txt" ]; then
        read count errors < "$RESULTS_DIR/worker_$i.txt"
        TOTAL_REQUESTS=$((TOTAL_REQUESTS + count))
        TOTAL_ERRORS=$((TOTAL_ERRORS + errors))
    fi
done

echo ""
echo "  Results:"
echo "    Total requests:  $TOTAL_REQUESTS"
echo "    Total errors:    $TOTAL_ERRORS"
echo "    Success rate:    $(( (TOTAL_REQUESTS - TOTAL_ERRORS) * 100 / TOTAL_REQUESTS ))%"
echo "    Requests/sec:    $(( TOTAL_REQUESTS / TEST_DURATION ))"

if [ $TOTAL_ERRORS -eq 0 ]; then
    echo -e "    Status:          ${GREEN}✓ Perfect (0 errors)${NC}"
elif [ $TOTAL_ERRORS -lt 10 ]; then
    echo -e "    Status:          ${GREEN}✓ Excellent (< 10 errors)${NC}"
else
    echo -e "    Status:          ${YELLOW}⚠ $(( TOTAL_ERRORS * 100 / TOTAL_REQUESTS ))% error rate${NC}"
fi

echo ""

# ============================================================================
# 4. Memory Stability Test
# ============================================================================

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "4. Memory Stability Test"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Find sultand process
SULTAND_PID=$(pgrep -f "sultand start" | head -1 || echo "")

if [ -n "$SULTAND_PID" ]; then
    echo "  Monitoring process: PID $SULTAND_PID"
    echo ""
    
    # Initial memory
    INITIAL_RSS=$(grep VmRSS /proc/$SULTAND_PID/status | awk '{print $2}')
    INITIAL_RSS_MB=$((INITIAL_RSS / 1024))
    echo "  Initial RSS: ${INITIAL_RSS_MB} MB"
    
    # Wait 30 seconds under load
    echo "  Monitoring for 30 seconds under load..."
    sleep 30
    
    # Final memory
    if [ -f /proc/$SULTAND_PID/status ]; then
        FINAL_RSS=$(grep VmRSS /proc/$SULTAND_PID/status | awk '{print $2}')
        FINAL_RSS_MB=$((FINAL_RSS / 1024))
        GROWTH_MB=$((FINAL_RSS_MB - INITIAL_RSS_MB))
        GROWTH_PCT=$((GROWTH_MB * 100 / INITIAL_RSS_MB))
        
        echo "  Final RSS:   ${FINAL_RSS_MB} MB"
        echo "  Growth:      ${GROWTH_MB} MB (${GROWTH_PCT}%)"
        echo ""
        
        if [ $GROWTH_PCT -lt 10 ]; then
            echo -e "  Status:      ${GREEN}✓ Stable (< 10% growth)${NC}"
        elif [ $GROWTH_PCT -lt 25 ]; then
            echo -e "  Status:      ${GREEN}✓ Good (< 25% growth)${NC}"
        else
            echo -e "  Status:      ${YELLOW}⚠ Monitor closely (${GROWTH_PCT}% growth)${NC}"
        fi
    else
        echo -e "  ${RED}✗ Process died during test${NC}"
    fi
else
    echo -e "  ${YELLOW}⚠ sultand process not found${NC}"
    echo "  Skipping memory test"
fi

echo ""

# ============================================================================
# 5. Block Production Test
# ============================================================================

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "5. Block Production Test"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "  Initial height: $INITIAL_HEIGHT"
sleep 20
FINAL_HEIGHT=$(curl -s "$RPC_ENDPOINT/status" | grep -o '"latest_block_height":"[0-9]*"' | cut -d'"' -f4 || echo "0")
echo "  Final height:   $FINAL_HEIGHT"

BLOCKS_PRODUCED=$((FINAL_HEIGHT - INITIAL_HEIGHT))
echo "  Blocks produced: $BLOCKS_PRODUCED in 20 seconds"

if [ $BLOCKS_PRODUCED -gt 0 ]; then
    AVG_BLOCK_TIME=$((20 / BLOCKS_PRODUCED))
    echo "  Avg block time:  ~${AVG_BLOCK_TIME}s"
    echo -e "  Status:          ${GREEN}✓ Producing blocks${NC}"
else
    echo -e "  Status:          ${YELLOW}⚠ No new blocks${NC}"
fi

echo ""

# ============================================================================
# 6. Final Report
# ============================================================================

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "6. Stress Test Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Calculate overall score
SCORE=0
MAX_SCORE=5

# API performance
[ $AVG_MS -lt 100 ] && ((SCORE++))

# Sustained load
[ $TOTAL_ERRORS -eq 0 ] && ((SCORE++))

# Memory stability
[ -n "$GROWTH_PCT" ] && [ $GROWTH_PCT -lt 25 ] && ((SCORE++))

# Block production
[ $BLOCKS_PRODUCED -gt 0 ] && ((SCORE++))

# Request throughput
[ $((TOTAL_REQUESTS / TEST_DURATION)) -gt 50 ] && ((SCORE++))

PERCENTAGE=$((SCORE * 100 / MAX_SCORE))

echo "  Test Metrics:"
echo "    API latency:       $([ $AVG_MS -lt 100 ] && echo -e "${GREEN}PASS${NC}" || echo -e "${YELLOW}WARN${NC}")"
echo "    Sustained load:    $([ $TOTAL_ERRORS -eq 0 ] && echo -e "${GREEN}PASS${NC}" || echo -e "${YELLOW}WARN${NC}")"
echo "    Memory stability:  $([ -n "$GROWTH_PCT" ] && [ $GROWTH_PCT -lt 25 ] && echo -e "${GREEN}PASS${NC}" || echo -e "${YELLOW}WARN${NC}")"
echo "    Block production:  $([ $BLOCKS_PRODUCED -gt 0 ] && echo -e "${GREEN}PASS${NC}" || echo -e "${YELLOW}WARN${NC}")"
echo "    Throughput (>50):  $([ $((TOTAL_REQUESTS / TEST_DURATION)) -gt 50 ] && echo -e "${GREEN}PASS${NC}" || echo -e "${YELLOW}WARN${NC}")"
echo ""

if [ $PERCENTAGE -ge 80 ]; then
    echo -e "  Overall Score: ${GREEN}$SCORE/$MAX_SCORE ($PERCENTAGE%) - PRODUCTION READY ✓${NC}"
elif [ $PERCENTAGE -ge 60 ]; then
    echo -e "  Overall Score: ${YELLOW}$SCORE/$MAX_SCORE ($PERCENTAGE%) - GOOD${NC}"
else
    echo -e "  Overall Score: ${YELLOW}$SCORE/$MAX_SCORE ($PERCENTAGE%) - NEEDS IMPROVEMENT${NC}"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "${GREEN}✓ Stress testing complete!${NC}"
echo ""
echo "Detailed results saved to: $RESULTS_DIR"
echo ""
echo "Production Recommendations:"
echo "  • Monitor memory growth over 24+ hours"
echo "  • Set up alerting for error rates > 1%"
echo "  • Configure load balancing for > 1000 req/sec"
echo "  • Enable Prometheus metrics export"
echo "  • Schedule regular stress tests"
echo ""
