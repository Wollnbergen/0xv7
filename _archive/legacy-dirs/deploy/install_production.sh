#!/bin/bash

# Sultan L1 Production Deployment Script
# This script sets up a full production node with monitoring and security

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SULTAN_USER="${SULTAN_USER:-sultan}"
INSTALL_DIR="${INSTALL_DIR:-/opt/sultan}"
DATA_DIR="${DATA_DIR:-/var/lib/sultan}"
LOG_DIR="${LOG_DIR:-/var/log/sultan}"
DOMAIN="${DOMAIN:-rpc.sultanl1.com}"
EMAIL="${EMAIL:-admin@sultanl1.com}"

echo -e "${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                                                               ║${NC}"
echo -e "${BLUE}║         🚀 SULTAN L1 PRODUCTION DEPLOYMENT 🚀                ║${NC}"
echo -e "${BLUE}║                                                               ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}❌ This script must be run as root${NC}" 
   exit 1
fi

echo -e "${GREEN}✅ Running as root${NC}"
echo ""

# Step 1: Create Sultan user
echo -e "${BLUE}📝 Step 1/10: Creating system user...${NC}"
if id "$SULTAN_USER" &>/dev/null; then
    echo -e "${YELLOW}⚠️  User $SULTAN_USER already exists${NC}"
else
    useradd -r -s /bin/bash -d "$INSTALL_DIR" -m "$SULTAN_USER"
    echo -e "${GREEN}✅ User $SULTAN_USER created${NC}"
fi
echo ""

# Step 2: Create directories
echo -e "${BLUE}📁 Step 2/10: Creating directories...${NC}"
mkdir -p "$INSTALL_DIR"/{bin,config}
mkdir -p "$DATA_DIR"
mkdir -p "$LOG_DIR"
chown -R "$SULTAN_USER:$SULTAN_USER" "$INSTALL_DIR" "$DATA_DIR" "$LOG_DIR"
echo -e "${GREEN}✅ Directories created${NC}"
echo ""

# Step 3: Install dependencies
echo -e "${BLUE}📦 Step 3/10: Installing system dependencies...${NC}"
apt-get update -qq
apt-get install -y -qq \
    curl \
    wget \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    nginx \
    certbot \
    python3-certbot-nginx \
    prometheus \
    prometheus-node-exporter \
    grafana \
    jq \
    bc \
    htop \
    iotop \
    nethogs \
    > /dev/null 2>&1
echo -e "${GREEN}✅ Dependencies installed${NC}"
echo ""

# Step 4: Build Sultan node
echo -e "${BLUE}🔧 Step 4/10: Building Sultan node from source...${NC}"
if [ ! -f "$INSTALL_DIR/bin/sultan-node" ]; then
    # Install Rust if not present
    if ! command -v cargo &> /dev/null; then
        echo "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi

    # Build the node
    cd /tmp
    if [ -d "sultan-build" ]; then
        rm -rf sultan-build
    fi
    
    # Copy source or clone from git
    if [ -d "/workspaces/0xv7/sultan-core" ]; then
        cp -r /workspaces/0xv7/sultan-core sultan-build
    else
        echo -e "${YELLOW}⚠️  Please provide source code location${NC}"
        exit 1
    fi

    cd sultan-build
    cargo build --release --bin sultan-node
    cp target/release/sultan-node "$INSTALL_DIR/bin/"
    chown "$SULTAN_USER:$SULTAN_USER" "$INSTALL_DIR/bin/sultan-node"
    chmod +x "$INSTALL_DIR/bin/sultan-node"
    
    cd /tmp
    rm -rf sultan-build
    
    echo -e "${GREEN}✅ Sultan node built and installed${NC}"
else
    echo -e "${YELLOW}⚠️  Sultan node binary already exists${NC}"
fi
echo ""

# Step 5: Configure systemd service
echo -e "${BLUE}⚙️  Step 5/10: Configuring systemd service...${NC}"
cat > /etc/systemd/system/sultan-node.service <<'EOF'
[Unit]
Description=Sultan L1 Blockchain Node
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=sultan
Group=sultan
WorkingDirectory=/opt/sultan
Environment="RUST_LOG=info"

ExecStart=/opt/sultan/bin/sultan-node \
  --enable-sharding \
  --shard-count 100 \
  --tx-per-shard 10000 \
  --block-time 5 \
  --data-dir /var/lib/sultan/data \
  --rpc-addr 127.0.0.1:26657

NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/sultan

LimitNOFILE=65535
LimitNPROC=4096

Restart=always
RestartSec=10

StandardOutput=journal
StandardError=journal
SyslogIdentifier=sultan-node

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable sultan-node.service
echo -e "${GREEN}✅ Systemd service configured${NC}"
echo ""

# Step 6: Configure Nginx
echo -e "${BLUE}🌐 Step 6/10: Configuring Nginx reverse proxy...${NC}"

# Remove default site
rm -f /etc/nginx/sites-enabled/default

# Copy Sultan RPC config
if [ -f "/workspaces/0xv7/deploy/nginx/sultan-rpc.conf" ]; then
    cp /workspaces/0xv7/deploy/nginx/sultan-rpc.conf /etc/nginx/sites-available/sultan-rpc
else
    # Create basic config
    cat > /etc/nginx/sites-available/sultan-rpc <<EOF
server {
    listen 80;
    server_name $DOMAIN;

    location / {
        proxy_pass http://127.0.0.1:26657;
        proxy_http_version 1.1;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EOF
fi

ln -sf /etc/nginx/sites-available/sultan-rpc /etc/nginx/sites-enabled/
nginx -t && systemctl reload nginx
echo -e "${GREEN}✅ Nginx configured${NC}"
echo ""

# Step 7: Setup SSL with Let's Encrypt
echo -e "${BLUE}🔒 Step 7/10: Setting up SSL certificate...${NC}"
if [ "$DOMAIN" != "rpc.sultanl1.com" ]; then
    certbot --nginx -d "$DOMAIN" --non-interactive --agree-tos -m "$EMAIL" || true
    echo -e "${GREEN}✅ SSL certificate obtained${NC}"
else
    echo -e "${YELLOW}⚠️  Skipping SSL setup (using default domain)${NC}"
fi
echo ""

# Step 8: Configure Prometheus
echo -e "${BLUE}📊 Step 8/10: Configuring Prometheus monitoring...${NC}"
if [ -f "/workspaces/0xv7/deploy/prometheus/prometheus.yml" ]; then
    cp /workspaces/0xv7/deploy/prometheus/prometheus.yml /etc/prometheus/prometheus.yml
    cp /workspaces/0xv7/deploy/prometheus/alerts.yml /etc/prometheus/rules/sultan-alerts.yml
    chown prometheus:prometheus /etc/prometheus/prometheus.yml
    systemctl restart prometheus
    systemctl enable prometheus
    echo -e "${GREEN}✅ Prometheus configured${NC}"
else
    echo -e "${YELLOW}⚠️  Prometheus config not found, skipping${NC}"
fi
echo ""

# Step 9: Configure Grafana
echo -e "${BLUE}📈 Step 9/10: Configuring Grafana dashboards...${NC}"
systemctl start grafana-server
systemctl enable grafana-server

# Wait for Grafana to start
sleep 5

# Add Prometheus as data source
curl -X POST -H "Content-Type: application/json" \
    -d '{
        "name":"Prometheus",
        "type":"prometheus",
        "url":"http://localhost:9090",
        "access":"proxy",
        "isDefault":true
    }' \
    http://admin:admin@localhost:3000/api/datasources || true

echo -e "${GREEN}✅ Grafana configured${NC}"
echo ""

# Step 10: Start services
echo -e "${BLUE}🚀 Step 10/10: Starting services...${NC}"
systemctl start sultan-node
sleep 3

# Check if node is running
if systemctl is-active --quiet sultan-node; then
    echo -e "${GREEN}✅ Sultan node is running${NC}"
else
    echo -e "${RED}❌ Failed to start Sultan node${NC}"
    journalctl -u sultan-node -n 50 --no-pager
    exit 1
fi

# Test RPC endpoint
sleep 2
if curl -s http://127.0.0.1:26657/status > /dev/null; then
    echo -e "${GREEN}✅ RPC endpoint responding${NC}"
else
    echo -e "${RED}❌ RPC endpoint not responding${NC}"
fi
echo ""

# Print summary
echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                                                               ║${NC}"
echo -e "${GREEN}║             🎉 DEPLOYMENT SUCCESSFUL! 🎉                     ║${NC}"
echo -e "${GREEN}║                                                               ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}📋 DEPLOYMENT SUMMARY${NC}"
echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "  Installation Directory: ${YELLOW}$INSTALL_DIR${NC}"
echo -e "  Data Directory:         ${YELLOW}$DATA_DIR${NC}"
echo -e "  Log Directory:          ${YELLOW}$LOG_DIR${NC}"
echo -e "  System User:            ${YELLOW}$SULTAN_USER${NC}"
echo ""
echo -e "${BLUE}🌐 ACCESS POINTS${NC}"
echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "  RPC Endpoint:    ${GREEN}http://$DOMAIN${NC}"
echo -e "  Grafana:         ${GREEN}http://localhost:3000${NC} (admin/admin)"
echo -e "  Prometheus:      ${GREEN}http://localhost:9090${NC}"
echo ""
echo -e "${BLUE}🔧 USEFUL COMMANDS${NC}"
echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "  Status:          ${YELLOW}systemctl status sultan-node${NC}"
echo -e "  Logs:            ${YELLOW}journalctl -fu sultan-node${NC}"
echo -e "  Restart:         ${YELLOW}systemctl restart sultan-node${NC}"
echo -e "  Stop:            ${YELLOW}systemctl stop sultan-node${NC}"
echo -e "  Start:           ${YELLOW}systemctl start sultan-node${NC}"
echo ""
echo -e "${BLUE}📊 MONITORING${NC}"
echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "  Metrics:         ${GREEN}http://localhost:9090/targets${NC}"
echo -e "  Alerts:          ${GREEN}http://localhost:9090/alerts${NC}"
echo -e "  Dashboards:      ${GREEN}http://localhost:3000/dashboards${NC}"
echo ""
echo -e "${GREEN}🎊 Sultan L1 is now running in production mode!${NC}"
echo ""
