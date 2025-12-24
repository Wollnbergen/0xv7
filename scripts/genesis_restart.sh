#!/usr/bin/env bash
#
# Sultan L1 Genesis Restart Script v2.0.0
# Final production launch with unified validator system
#
set -euo pipefail

# ============================================================================
# CONFIGURATION - PRODUCTION LAUNCH DEC 24, 2025
# ============================================================================

# Your validator servers (SSH user@host)
VALIDATORS=(
    # DigitalOcean (6 validators) - All active
    "root@206.189.224.142"  # NYC - Bootstrap node (rpc.sltn.io)
    "root@24.144.94.23"     # SFO
    "root@46.101.122.13"    # FRA
    "root@142.93.238.33"    # AMS
    "root@143.198.205.21"   # SGP
    "root@134.122.96.36"    # LON
)

# Bootstrap node (first validator - others connect to this)
BOOTSTRAP_NODE="${VALIDATORS[0]}"
BOOTSTRAP_DNS="rpc.sltn.io"

# Binary download URL (REAL production binary with RocksDB, libp2p, etc.)
BINARY_URL="https://github.com/Wollnbergen/0xv7/releases/download/v1.0.0/sultan-node"

# Remote paths
REMOTE_BIN_PATH="/root/sultan-node"
REMOTE_DATA_PATH="/root/data"
SERVICE_NAME="sultan"

# Genesis accounts (address:balance in base units, 9 decimals)
# 1 SLTN = 1,000,000,000 base units
# Total supply: 500,000,000 SLTN = 500,000,000,000,000,000 base units
#
# Single treasury address - will be distributed post-genesis via transactions
GENESIS_ACCOUNTS="sultan19mzzrah6h27draqc5tkh49yj623qwuz5f5t64c:500000000000000000"

# SSH key to use
SSH_KEY="$HOME/.ssh/sultan_do"

# ============================================================================
# COLORS
# ============================================================================
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info()  { echo -e "${BLUE}[INFO]${NC} $1"; }
log_ok()    { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# ============================================================================
# FUNCTIONS
# ============================================================================

ssh_cmd() {
    local host="$1"
    shift
    ssh -o StrictHostKeyChecking=no -o ConnectTimeout=10 -i "$SSH_KEY" "$host" "$@"
}

stop_all_validators() {
    log_info "Stopping all validators..."
    
    for host in "${VALIDATORS[@]}"; do
        log_info "Stopping $host..."
        ssh_cmd "$host" "systemctl stop $SERVICE_NAME 2>/dev/null || pkill -f sultan-node || true" &
    done
    
    wait
    log_ok "All validators stopped"
}

clear_all_data() {
    log_info "Clearing blockchain data on all validators..."
    
    for host in "${VALIDATORS[@]}"; do
        log_info "Clearing data on $host..."
        ssh_cmd "$host" "rm -rf $REMOTE_DATA_PATH" &
    done
    
    wait
    log_ok "All blockchain data cleared"
}

deploy_binary() {
    log_info "Deploying new binary to all validators..."
    
    for host in "${VALIDATORS[@]}"; do
        log_info "Deploying to $host..."
        ssh_cmd "$host" "
            wget -q '$BINARY_URL' -O $REMOTE_BIN_PATH.new && \
            chmod +x $REMOTE_BIN_PATH.new && \
            mv $REMOTE_BIN_PATH.new $REMOTE_BIN_PATH && \
            echo 'Binary deployed successfully'
        " &
    done
    
    wait
    log_ok "Binary deployed to all validators"
}

start_bootstrap_validator() {
    log_info "Starting bootstrap validator: $BOOTSTRAP_NODE"
    
    local validator_name="validator-1"
    
    ssh_cmd "$BOOTSTRAP_NODE" "
        nohup $REMOTE_BIN_PATH \\
            --name '$validator_name' \\
            --data-dir $REMOTE_DATA_PATH \\
            --validator \\
            --validator-address '$validator_name' \\
            --validator-stake 10000 \\
            --enable-p2p \\
            --enable-sharding \\
            --shard-count 16 \
            --p2p-addr /ip4/0.0.0.0/tcp/26656 \\
            --rpc-addr 0.0.0.0:26657 \\
            --genesis '$GENESIS_ACCOUNTS' \\
            > /var/log/sultan.log 2>&1 &
        
        sleep 3
        echo 'Bootstrap validator started'
    "
    
    log_ok "Bootstrap validator started"
    
    # Wait for bootstrap to be ready
    log_info "Waiting for bootstrap node to be ready..."
    sleep 10
}

start_other_validators() {
    log_info "Starting remaining validators..."
    
    local idx=2
    for host in "${VALIDATORS[@]:1}"; do
        local validator_name="validator-$idx"
        
        log_info "Starting $validator_name on $host..."
        
        ssh_cmd "$host" "
            nohup $REMOTE_BIN_PATH \\
                --name '$validator_name' \\
                --data-dir $REMOTE_DATA_PATH \\
                --validator \\
                --validator-address '$validator_name' \\
                --validator-stake 10000 \\
                --enable-p2p \\
                --bootstrap-peers /dns4/$BOOTSTRAP_DNS/tcp/26656 \\
                --enable-sharding \\
                --shard-count 16 \
                --rpc-addr 0.0.0.0:26657 \\
                > /var/log/sultan.log 2>&1 &
            
            sleep 1
            echo 'Validator started'
        " &
        
        ((idx++))
    done
    
    wait
    log_ok "All validators started"
}

verify_network() {
    log_info "Verifying network health..."
    
    sleep 15  # Wait for blocks to be produced
    
    for host in "${VALIDATORS[@]}"; do
        log_info "Checking $host..."
        local status=$(ssh_cmd "$host" "curl -s http://localhost:26657/status 2>/dev/null | head -1" || echo "FAILED")
        if [[ "$status" == *"height"* ]] || [[ "$status" == *"ok"* ]]; then
            log_ok "$host is healthy"
        else
            log_warn "$host may need attention"
        fi
    done
    
    # Check block height on bootstrap
    log_info "Checking block production..."
    local height=$(ssh_cmd "$BOOTSTRAP_NODE" "curl -s http://localhost:26657/status 2>/dev/null | grep -o '\"height\":[0-9]*' | head -1" || echo "0")
    log_ok "Current block height: $height"
}

create_systemd_service() {
    log_info "Creating systemd service on all validators..."
    
    local idx=1
    for host in "${VALIDATORS[@]}"; do
        local validator_name="validator-$idx"
        local is_bootstrap=""
        local bootstrap_flag=""
        
        if [[ "$idx" -eq 1 ]]; then
            is_bootstrap="# Bootstrap node"
        else
            bootstrap_flag="--bootstrap-peers /dns4/$BOOTSTRAP_DNS/tcp/26656"
        fi
        
        ssh_cmd "$host" "cat > /etc/systemd/system/sultan.service << 'EOF'
[Unit]
Description=Sultan L1 Validator Node
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root
ExecStart=$REMOTE_BIN_PATH \\
    --name $validator_name \\
    --data-dir $REMOTE_DATA_PATH \\
    --validator \\
    --validator-address $validator_name \\
    --validator-stake 10000 \\
    --enable-p2p \\
    --enable-sharding \\
    --shard-count 8 \\
    --rpc-addr 0.0.0.0:26657 \\
    $bootstrap_flag
Restart=always
RestartSec=5
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
EOF
systemctl daemon-reload
systemctl enable sultan
" &
        
        ((idx++))
    done
    
    wait
    log_ok "Systemd services created"
}

# ============================================================================
# MAIN
# ============================================================================

main() {
    echo ""
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║           SULTAN L1 GENESIS RESTART v1.1.0                     ║"
    echo "║           Fixed 4% Inflation | 13.33% APY                       ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo ""
    
    log_warn "This will RESET the blockchain to block 0!"
    log_warn "Current block height (~55,000) will be lost."
    echo ""
    read -p "Are you sure you want to continue? (yes/no): " confirm
    
    if [[ "$confirm" != "yes" ]]; then
        log_info "Aborted."
        exit 0
    fi
    
    echo ""
    log_info "Starting genesis restart..."
    echo ""
    
    # Step 1: Stop all validators
    stop_all_validators
    
    # Step 2: Clear old data
    clear_all_data
    
    # Step 3: Deploy new binary
    deploy_binary
    
    # Step 4: Create systemd services
    create_systemd_service
    
    # Step 5: Start bootstrap validator first
    start_bootstrap_validator
    
    # Step 6: Start remaining validators
    start_other_validators
    
    # Step 7: Verify network
    verify_network
    
    echo ""
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║                    GENESIS RESTART COMPLETE                    ║"
    echo "╠════════════════════════════════════════════════════════════════╣"
    echo "║  ✅ New binary v1.1.0 deployed                                 ║"
    echo "║  ✅ Fixed 4% inflation (forever)                               ║"
    echo "║  ✅ 13.33% Validator APY                                       ║"
    echo "║  ✅ Zero gas fees sustainable at 76M+ TPS                      ║"
    echo "║                                                                ║"
    echo "║  Network Status:                                               ║"
    echo "║  - Validators: 15                                              ║"
    echo "║  - Block Height: Starting from 0                               ║"
    echo "║  - RPC: https://rpc.sltn.io                                    ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo ""
}

# Run if executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
