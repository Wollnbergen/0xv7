#!/bin/bash
# Sultan L1 Network Monitor - Simple Version
# Usage: ./network_monitor.sh [--watch] [--json]

# Production validator IPs
declare -a VALIDATORS=(
    "206.189.224.142|NYC"
    "24.144.94.23|SFO"
    "46.101.122.13|LON"
    "142.93.238.33|FRA"
    "143.198.205.21|AMS"
    "134.122.96.36|SGP"
)

RPC_PORT=26657

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Parse arguments
WATCH_MODE=false
JSON_MODE=false
INTERVAL=10

for arg in "$@"; do
    case $arg in
        --watch|-w) WATCH_MODE=true ;;
        --json|-j) JSON_MODE=true ;;
    esac
done

collect_stats() {
    local json_output="["
    local first=true
    local max_height=0
    local online_count=0
    
    for entry in "${VALIDATORS[@]}"; do
        IFS='|' read -r ip location <<< "$entry"
        
        local status_json=$(curl -s --connect-timeout 3 "http://$ip:$RPC_PORT/status" 2>/dev/null)
        
        if [[ -n "$status_json" && "$status_json" != *"error"* ]]; then
            local height=$(echo "$status_json" | jq -r '.height // 0')
            local shards=$(echo "$status_json" | jq -r '.shard_count // 0')
            local pending=$(echo "$status_json" | jq -r '.pending_txs // 0')
            
            [[ $height -gt $max_height ]] && max_height=$height
            ((online_count++)) || true
            
            [[ "$first" == "true" ]] && first=false || json_output+=","
            json_output+="{\"ip\":\"$ip\",\"location\":\"$location\",\"status\":\"online\",\"height\":$height,\"shards\":$shards,\"pending_txs\":$pending}"
        else
            [[ "$first" == "true" ]] && first=false || json_output+=","
            json_output+="{\"ip\":\"$ip\",\"location\":\"$location\",\"status\":\"offline\",\"height\":0}"
        fi
    done
    
    json_output+="]"
    
    # Add lag to each validator
    local final_json=$(echo "$json_output" | jq --arg max "$max_height" '
        map(. + {lag: (if .height > 0 then ($max | tonumber) - .height else null end)})
    ')
    
    local health="healthy"
    [[ $online_count -lt 6 ]] && health="degraded"
    [[ $online_count -lt 4 ]] && health="critical"
    [[ $online_count -eq 0 ]] && health="offline"
    
    echo "{
        \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
        \"network_health\": \"$health\",
        \"online_validators\": $online_count,
        \"total_validators\": 6,
        \"max_height\": $max_height,
        \"validators\": $final_json
    }"
}

print_table() {
    local data="$1"
    
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}                    SULTAN L1 NETWORK MONITOR${NC}"
    echo -e "${CYAN}                    $(date '+%Y-%m-%d %H:%M:%S UTC')${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════════════════${NC}"
    echo ""
    
    local health=$(echo "$data" | jq -r '.network_health')
    local online=$(echo "$data" | jq -r '.online_validators')
    local max_height=$(echo "$data" | jq -r '.max_height')
    
    local health_color=$GREEN
    [[ "$health" == "degraded" ]] && health_color=$YELLOW
    [[ "$health" == "critical" || "$health" == "offline" ]] && health_color=$RED
    
    echo -e "Network Health: ${health_color}${health^^}${NC}    Validators: ${GREEN}$online/6${NC}    Block Height: ${BLUE}$max_height${NC}"
    echo ""
    echo -e "${CYAN}───────────────────────────────────────────────────────────────────────────${NC}"
    printf "%-18s %-8s %-10s %-8s %-8s %-10s %s\n" "IP Address" "Location" "Height" "Lag" "Shards" "Pending" "Status"
    echo -e "${CYAN}───────────────────────────────────────────────────────────────────────────${NC}"
    
    echo "$data" | jq -r '.validators[] | "\(.ip)|\(.location)|\(.height)|\(.lag // "N/A")|\(.shards // "N/A")|\(.pending_txs // "N/A")|\(.status)"' | \
    while IFS='|' read -r ip loc height lag shards pending status; do
        local status_icon="${GREEN}✅${NC}"
        
        if [[ "$status" == "offline" ]]; then
            status_icon="${RED}❌${NC}"
            height="N/A"
            lag="N/A"
            shards="N/A"
            pending="N/A"
        else
            if [[ "$lag" != "null" && "$lag" != "N/A" && "$lag" -gt 5 ]] 2>/dev/null; then
                lag="${YELLOW}${lag}${NC}"
            fi
            if [[ "$lag" != "null" && "$lag" != "N/A" && "$lag" -gt 20 ]] 2>/dev/null; then
                lag="${RED}${lag}${NC}"
            fi
        fi
        
        printf "%-18s %-8s %-10s %-8b %-8s %-10s %b\n" "$ip" "$loc" "$height" "$lag" "$shards" "$pending" "$status_icon"
    done
    
    echo ""
    
    # Network metrics from first validator
    local first_ip=$(echo "$data" | jq -r '.validators[0].ip')
    local details=$(curl -s --connect-timeout 3 "http://$first_ip:$RPC_PORT/status" 2>/dev/null)
    if [[ -n "$details" ]]; then
        local inflation=$(echo "$details" | jq -r '(.inflation_rate * 100) | floor')
        local apy=$(echo "$details" | jq -r '(.validator_apy * 100) | floor')
        local accounts=$(echo "$details" | jq -r '.total_accounts')
        
        echo -e "${BLUE}📊 Network Metrics${NC}"
        echo "   Inflation Rate: ${inflation}%"
        echo "   Validator APY:  ${apy}%"
        echo "   Total Accounts: $accounts"
        echo "   TPS Capacity:   64,000 (16 shards × 4K)"
    fi
    echo ""
}

# Main
if $JSON_MODE; then
    collect_stats | jq '.'
elif $WATCH_MODE; then
    while true; do
        clear
        data=$(collect_stats)
        print_table "$data"
        echo -e "${CYAN}Refreshing every ${INTERVAL}s... Press Ctrl+C to exit${NC}"
        sleep $INTERVAL
    done
else
    data=$(collect_stats)
    print_table "$data"
fi
