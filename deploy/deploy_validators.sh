#!/bin/bash
# Sultan L1 Validator Deployment Script
# Version: 3.1 - Christmas Day 2025 Launch
# Deploys sultan-node to 6 globally distributed validators

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Binary location
BINARY="$(dirname "$0")/sultan-node"
REMOTE_PATH="/root/sultan-node"
SERVICE_NAME="sultan-node"

# Genesis configuration - Christmas Day 2025 Launch
GENESIS_WALLET="sultan15g5nwnlemn7zt6rtl7ch46ssvx2ym2v2umm07g"
GENESIS_BALANCE="500000000000000"  # 500M SULTAN (with 6 decimals)
GENESIS_CONFIG="${GENESIS_WALLET}:${GENESIS_BALANCE}"

# Validator configurations
# Format: NAME:IP:STAKE:P2P_PORT:RPC_PORT
declare -a VALIDATORS=(
    "sultan-nyc:134.122.96.36:1000000:26656:8545"
    "sultan-sfo:143.198.205.21:1000000:26656:8545"
    "sultan-fra:142.93.238.33:1000000:26656:8545"
    "sultan-ams:46.101.122.13:1000000:26656:8545"
    "sultan-sgp:24.144.94.23:1000000:26656:8545"
    "sultan-lon:206.189.224.142:1000000:26656:8545"
)

# Genesis validators (for peer discovery)
GENESIS_PEERS="/ip4/134.122.96.36/tcp/26656,/ip4/143.198.205.21/tcp/26656,/ip4/142.93.238.33/tcp/26656"

print_header() {
    echo ""
    echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║${NC}  ${GREEN}Sultan L1 Validator Deployment - v3.1${NC}                      ${BLUE}║${NC}"
    echo -e "${BLUE}║${NC}  ${YELLOW}64,000 TPS | 16 Shards | 2s Block Time${NC}                    ${BLUE}║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

check_binary() {
    if [[ ! -f "$BINARY" ]]; then
        echo -e "${RED}Error: Binary not found at $BINARY${NC}"
        echo "Please build first: cargo build --release -p sultan-core"
        exit 1
    fi
    echo -e "${GREEN}✓ Binary found: $(ls -lh $BINARY | awk '{print $5}')${NC}"
}

deploy_to_validator() {
    local config="$1"
    IFS=':' read -r name host stake p2p_port rpc_port <<< "$config"
    
    echo ""
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}Deploying to: ${GREEN}$name${NC} ($host)"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    # Check SSH connectivity
    echo -n "  → Checking SSH connectivity... "
    if ! ssh -o ConnectTimeout=5 -o BatchMode=yes "root@$host" "echo ok" &>/dev/null; then
        echo -e "${RED}FAILED${NC}"
        echo -e "    ${RED}Cannot connect to $host. Skipping.${NC}"
        return 1
    fi
    echo -e "${GREEN}OK${NC}"
    
    # Stop existing service
    echo -n "  → Stopping existing service... "
    ssh "root@$host" "systemctl stop $SERVICE_NAME 2>/dev/null || true"
    echo -e "${GREEN}OK${NC}"
    
    # Upload binary
    echo -n "  → Uploading binary... "
    scp -q "$BINARY" "root@$host:$REMOTE_PATH"
    ssh "root@$host" "chmod +x $REMOTE_PATH"
    echo -e "${GREEN}OK${NC}"
    
    # Create data directory
    echo -n "  → Creating data directory... "
    ssh "root@$host" "mkdir -p /var/lib/sultan"
    echo -e "${GREEN}OK${NC}"
    
    # Create systemd service
    echo -n "  → Creating systemd service... "
    ssh "root@$host" "cat > /etc/systemd/system/$SERVICE_NAME.service << 'EOF'
[Unit]
Description=Sultan L1 Blockchain Node
After=network.target

[Service]
Type=simple
User=root
ExecStart=$REMOTE_PATH \\
    --name $name \\
    --data-dir /var/lib/sultan \\
    --block-time 2 \\
    --validator \\
    --validator-stake $stake \\
    --p2p-addr /ip4/0.0.0.0/tcp/$p2p_port \\
    --rpc-addr 0.0.0.0:$rpc_port \\
    --peers $GENESIS_PEERS \\
    --genesis $GENESIS_CONFIG \\
    --enable-sharding \\
    --shard-count 16
Restart=always
RestartSec=3
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
EOF"
    echo -e "${GREEN}OK${NC}"
    
    # Reload and start service
    echo -n "  → Starting service... "
    ssh "root@$host" "systemctl daemon-reload && systemctl enable $SERVICE_NAME && systemctl start $SERVICE_NAME"
    echo -e "${GREEN}OK${NC}"
    
    # Wait for startup
    sleep 2
    
    # Check status
    echo -n "  → Checking status... "
    if ssh "root@$host" "systemctl is-active $SERVICE_NAME" &>/dev/null; then
        echo -e "${GREEN}RUNNING${NC}"
    else
        echo -e "${RED}FAILED${NC}"
        ssh "root@$host" "journalctl -u $SERVICE_NAME -n 20 --no-pager" || true
        return 1
    fi
    
    echo -e "  ${GREEN}✓ $name deployed successfully!${NC}"
    return 0
}

check_all_validators() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}Checking All Validators${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    for config in "${VALIDATORS[@]}"; do
        IFS=':' read -r name host stake p2p_port rpc_port <<< "$config"
        
        echo -n "  $name ($host): "
        
        # Try to get status from RPC
        status=$(curl -s --connect-timeout 3 "http://$host:$rpc_port/status" 2>/dev/null || echo "")
        
        if [[ -n "$status" ]]; then
            height=$(echo "$status" | jq -r '.block_height // .height // "N/A"' 2>/dev/null || echo "N/A")
            peers=$(echo "$status" | jq -r '.peers // .peer_count // "N/A"' 2>/dev/null || echo "N/A")
            echo -e "${GREEN}ONLINE${NC} | Height: $height | Peers: $peers"
        else
            # Try SSH check
            if ssh -o ConnectTimeout=3 -o BatchMode=yes "root@$host" "systemctl is-active $SERVICE_NAME" &>/dev/null; then
                echo -e "${YELLOW}RUNNING${NC} (RPC not responding)"
            else
                echo -e "${RED}OFFLINE${NC}"
            fi
        fi
    done
}

show_usage() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  deploy    - Deploy to all validators"
    echo "  status    - Check status of all validators"
    echo "  restart   - Restart all validators"
    echo "  logs      - Show logs from all validators"
    echo "  help      - Show this help"
    echo ""
}

restart_all_validators() {
    echo ""
    echo -e "${BLUE}Restarting All Validators${NC}"
    
    for config in "${VALIDATORS[@]}"; do
        IFS=':' read -r name host stake p2p_port rpc_port <<< "$config"
        echo -n "  Restarting $name... "
        if ssh -o ConnectTimeout=5 "root@$host" "systemctl restart $SERVICE_NAME" &>/dev/null; then
            echo -e "${GREEN}OK${NC}"
        else
            echo -e "${RED}FAILED${NC}"
        fi
    done
}

show_logs() {
    echo ""
    echo -e "${BLUE}Recent Logs from All Validators${NC}"
    
    for config in "${VALIDATORS[@]}"; do
        IFS=':' read -r name host stake p2p_port rpc_port <<< "$config"
        echo ""
        echo -e "${YELLOW}━━━ $name ━━━${NC}"
        ssh -o ConnectTimeout=5 "root@$host" "journalctl -u $SERVICE_NAME -n 10 --no-pager" 2>/dev/null || echo "  Cannot connect"
    done
}

# Main
print_header

case "${1:-deploy}" in
    deploy)
        check_binary
        echo ""
        echo -e "${YELLOW}Deploying to ${#VALIDATORS[@]} validators...${NC}"
        
        success=0
        failed=0
        
        for config in "${VALIDATORS[@]}"; do
            if deploy_to_validator "$config"; then
                ((success++))
            else
                ((failed++))
            fi
        done
        
        echo ""
        echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${BLUE}Deployment Complete${NC}"
        echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "  ${GREEN}Success: $success${NC}"
        echo -e "  ${RED}Failed:  $failed${NC}"
        echo ""
        
        # Check all validators after deployment
        sleep 5
        check_all_validators
        ;;
    status)
        check_all_validators
        ;;
    restart)
        restart_all_validators
        sleep 3
        check_all_validators
        ;;
    logs)
        show_logs
        ;;
    help|--help|-h)
        show_usage
        ;;
    *)
        echo -e "${RED}Unknown command: $1${NC}"
        show_usage
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}Done!${NC}"
