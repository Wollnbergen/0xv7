#!/bin/bash
# Sultan Validator Deployment Script
# Deploys release build to all validators with proper P2P config

set -e

# === CONFIGURATION ===
BOOTSTRAP_IP="5.161.225.96"  # rpc.sltn.io - Hetzner
BOOTSTRAP_PEER="/ip4/${BOOTSTRAP_IP}/tcp/26656"

# All validator IPs
DO_VALIDATORS=(
    "159.65.88.145"
    "188.166.218.123"
    "188.166.102.7"
    "159.65.113.160"
    "143.198.67.237"
    "192.241.154.140"
)

HZ_VALIDATORS=(
    "49.13.26.15"
    "116.203.92.158"
)

# SSH keys (set these before running)
DO_SSH_KEY="${HOME}/.ssh/sultan_do"
HZ_SSH_PASSWORD=""  # Will prompt if not set

# Binary location
RELEASE_BINARY="/tmp/cargo-target/release/sultan-node"

# === FUNCTIONS ===

upload_binary() {
    local ip=$1
    local ssh_opts=$2
    
    echo "📦 Uploading binary to $ip..."
    scp $ssh_opts "$RELEASE_BINARY" "root@${ip}:/opt/sultan/sultan-node"
}

create_systemd_service() {
    local ip=$1
    local name=$2
    local is_bootstrap=$3
    local ssh_opts=$4
    
    local bootstrap_arg=""
    if [ "$is_bootstrap" != "true" ]; then
        bootstrap_arg="--bootstrap-peers \"${BOOTSTRAP_PEER}\""
    fi
    
    echo "⚙️  Configuring $name on $ip..."
    
    ssh $ssh_opts "root@${ip}" << EOF
# Stop existing service
systemctl stop sultan-node 2>/dev/null || true

# Create directory
mkdir -p /opt/sultan/data

# Make binary executable
chmod +x /opt/sultan/sultan-node

# Create systemd service
cat > /etc/systemd/system/sultan-node.service << 'SVCEOF'
[Unit]
Description=Sultan L1 Blockchain Node
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/sultan
ExecStart=/opt/sultan/sultan-node \\
  --name "${name}" \\
  --data-dir /opt/sultan/data \\
  --validator \\
  --validator-address "${name}" \\
  --validator-stake 100000 \\
  --enable-p2p \\
  --enable-sharding \\
  --shard-count 8 \\
  --block-time 2 \\
  --rpc-addr 0.0.0.0:26657 \\
  --p2p-addr /ip4/0.0.0.0/tcp/26656 \\
  ${bootstrap_arg}
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
SVCEOF

# Open firewall
ufw allow 26656/tcp 2>/dev/null || true
ufw allow 26657/tcp 2>/dev/null || true

# Reload and start
systemctl daemon-reload
systemctl enable sultan-node
systemctl start sultan-node

echo "✅ $name started on $ip"
EOF
}

check_status() {
    local ip=$1
    local ssh_opts=$2
    
    echo "🔍 Checking $ip..."
    ssh $ssh_opts "root@${ip}" "systemctl status sultan-node --no-pager -l | head -20"
}

# === MAIN ===

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║           SULTAN VALIDATOR DEPLOYMENT                         ║"
echo "╠════════════════════════════════════════════════════════════════╣"
echo "║  Bootstrap: ${BOOTSTRAP_IP}                                    ║"
echo "║  DO Validators: ${#DO_VALIDATORS[@]}                                          ║"
echo "║  HZ Validators: ${#HZ_VALIDATORS[@]}                                           ║"
echo "╚════════════════════════════════════════════════════════════════╝"

# Check binary exists
if [ ! -f "$RELEASE_BINARY" ]; then
    echo "❌ Release binary not found at $RELEASE_BINARY"
    echo "   Run: cd /workspaces/0xv7/sultan-core && cargo build --release"
    exit 1
fi

echo ""
echo "Binary size: $(du -h $RELEASE_BINARY | cut -f1)"
echo ""

# Prompt for action
echo "What would you like to do?"
echo "  1) Deploy to ALL validators (fresh start)"
echo "  2) Deploy to bootstrap only"
echo "  3) Deploy to DO validators only"
echo "  4) Deploy to Hetzner validators only"
echo "  5) Check status of all validators"
echo "  6) Exit"
read -p "Choice [1-6]: " choice

case $choice in
    1)
        echo ""
        echo "=== Deploying Bootstrap (${BOOTSTRAP_IP}) ==="
        upload_binary "$BOOTSTRAP_IP" "-o StrictHostKeyChecking=no"
        create_systemd_service "$BOOTSTRAP_IP" "validator-bootstrap" "true" "-o StrictHostKeyChecking=no"
        
        # Wait for bootstrap to start
        echo "⏳ Waiting 10s for bootstrap to initialize..."
        sleep 10
        
        echo ""
        echo "=== Deploying Hetzner Validators ==="
        idx=1
        for ip in "${HZ_VALIDATORS[@]}"; do
            upload_binary "$ip" "-o StrictHostKeyChecking=no"
            create_systemd_service "$ip" "validator-hz-${idx}" "false" "-o StrictHostKeyChecking=no"
            ((idx++))
        done
        
        echo ""
        echo "=== Deploying DigitalOcean Validators ==="
        idx=1
        for ip in "${DO_VALIDATORS[@]}"; do
            upload_binary "$ip" "-i $DO_SSH_KEY -o StrictHostKeyChecking=no"
            create_systemd_service "$ip" "validator-do-${idx}" "false" "-i $DO_SSH_KEY -o StrictHostKeyChecking=no"
            ((idx++))
        done
        ;;
    2)
        upload_binary "$BOOTSTRAP_IP" "-o StrictHostKeyChecking=no"
        create_systemd_service "$BOOTSTRAP_IP" "validator-bootstrap" "true" "-o StrictHostKeyChecking=no"
        ;;
    3)
        idx=1
        for ip in "${DO_VALIDATORS[@]}"; do
            upload_binary "$ip" "-i $DO_SSH_KEY -o StrictHostKeyChecking=no"
            create_systemd_service "$ip" "validator-do-${idx}" "false" "-i $DO_SSH_KEY -o StrictHostKeyChecking=no"
            ((idx++))
        done
        ;;
    4)
        idx=1
        for ip in "${HZ_VALIDATORS[@]}"; do
            upload_binary "$ip" "-o StrictHostKeyChecking=no"
            create_systemd_service "$ip" "validator-hz-${idx}" "false" "-o StrictHostKeyChecking=no"
            ((idx++))
        done
        ;;
    5)
        echo ""
        echo "=== Checking All Validators ==="
        check_status "$BOOTSTRAP_IP" "-o StrictHostKeyChecking=no"
        for ip in "${HZ_VALIDATORS[@]}"; do
            check_status "$ip" "-o StrictHostKeyChecking=no"
        done
        for ip in "${DO_VALIDATORS[@]}"; do
            check_status "$ip" "-i $DO_SSH_KEY -o StrictHostKeyChecking=no"
        done
        ;;
    6)
        echo "Exiting."
        exit 0
        ;;
    *)
        echo "Invalid choice"
        exit 1
        ;;
esac

echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║  ✅ DEPLOYMENT COMPLETE                                        ║"
echo "╠════════════════════════════════════════════════════════════════╣"
echo "║  Check status: curl https://rpc.sltn.io/status                ║"
echo "║  View logs: ssh root@IP 'journalctl -u sultan-node -f'        ║"
echo "╚════════════════════════════════════════════════════════════════╝"
