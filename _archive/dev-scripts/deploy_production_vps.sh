#!/bin/bash
set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║    SULTAN CHAIN - PRODUCTION VPS DEPLOYMENT                   ║"
echo "║    RPC Node with Token Factory & Native DEX                   ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Configuration
VPS_IP="${VPS_IP:-YOUR_VPS_IP}"
DOMAIN="${DOMAIN:-rpc.sultanchain.io}"
API_DOMAIN="${API_DOMAIN:-api.sultanchain.io}"
SSH_USER="${SSH_USER:-root}"

echo "📋 Deployment Configuration:"
echo "  VPS IP: $VPS_IP"
echo "  RPC Domain: $DOMAIN"
echo "  API Domain: $API_DOMAIN"
echo "  SSH User: $SSH_USER"
echo ""

read -p "Continue with deployment? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Deployment cancelled"
    exit 1
fi

echo ""
echo "🔧 Step 1: Preparing deployment package..."

# Create deployment directory
mkdir -p /tmp/sultan-deploy
cd /tmp/sultan-deploy

# Build release binary
echo "📦 Building release binary..."
cd /workspaces/0xv7
cargo build --release -p sultan-core --bin sultan-node

# Copy binary
cp target/release/sultan-node /tmp/sultan-deploy/

# Create systemd service file
cat > /tmp/sultan-deploy/sultan-node.service << 'EOF'
[Unit]
Description=Sultan Chain RPC Node
After=network.target

[Service]
Type=simple
User=sultan
WorkingDirectory=/opt/sultan
ExecStart=/opt/sultan/sultan-node \
    --validator \
    --validator-address genesis-validator \
    --validator-stake 100000000000 \
    --rpc-addr 0.0.0.0:26657 \
    --data-dir /opt/sultan/data
Restart=always
RestartSec=10
LimitNOFILE=65536

# Environment
Environment="RUST_LOG=info"
Environment="RUST_BACKTRACE=1"

[Install]
WantedBy=multi-user.target
EOF

# Create nginx configuration
cat > /tmp/sultan-deploy/nginx-rpc.conf << EOF
server {
    listen 80;
    server_name $DOMAIN;

    location / {
        proxy_pass http://127.0.0.1:26657;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        
        # CORS headers
        add_header Access-Control-Allow-Origin * always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type" always;
        
        if (\$request_method = OPTIONS) {
            return 204;
        }
    }
}

server {
    listen 80;
    server_name $API_DOMAIN;

    location / {
        proxy_pass http://127.0.0.1:26657;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        
        # CORS headers
        add_header Access-Control-Allow-Origin * always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type" always;
        
        if (\$request_method = OPTIONS) {
            return 204;
        }
    }
}
EOF

# Create installation script
cat > /tmp/sultan-deploy/install.sh << 'INSTALL_EOF'
#!/bin/bash
set -e

echo "🚀 Installing Sultan Chain RPC Node..."

# Update system
apt-get update
apt-get upgrade -y

# Install dependencies
apt-get install -y \
    nginx \
    certbot \
    python3-certbot-nginx \
    ufw \
    htop \
    curl \
    jq

# Create sultan user
if ! id -u sultan > /dev/null 2>&1; then
    useradd -r -s /bin/bash -m -d /opt/sultan sultan
fi

# Create directories
mkdir -p /opt/sultan/data
chown -R sultan:sultan /opt/sultan

# Install binary
cp sultan-node /opt/sultan/
chmod +x /opt/sultan/sultan-node
chown sultan:sultan /opt/sultan/sultan-node

# Install systemd service
cp sultan-node.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable sultan-node

# Configure nginx
cp nginx-rpc.conf /etc/nginx/sites-available/sultan-rpc
ln -sf /etc/nginx/sites-available/sultan-rpc /etc/nginx/sites-enabled/
rm -f /etc/nginx/sites-enabled/default
nginx -t
systemctl restart nginx

# Configure firewall
ufw --force enable
ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 443/tcp
ufw allow 26656/tcp  # P2P
ufw allow 26657/tcp  # RPC (direct access)

echo "✅ Installation complete!"
echo ""
echo "Next steps:"
echo "1. Start node: systemctl start sultan-node"
echo "2. Check logs: journalctl -u sultan-node -f"
echo "3. Get SSL: certbot --nginx -d DOMAIN -d API_DOMAIN"
INSTALL_EOF

chmod +x /tmp/sultan-deploy/install.sh

echo "✅ Deployment package ready!"
echo ""
echo "🚢 Step 2: Uploading to VPS..."

# Upload files
scp -r /tmp/sultan-deploy/* $SSH_USER@$VPS_IP:/tmp/

echo "✅ Files uploaded!"
echo ""
echo "⚙️  Step 3: Running installation on VPS..."

ssh $SSH_USER@$VPS_IP << 'REMOTE_EOF'
cd /tmp
chmod +x install.sh
./install.sh

# Start the node
systemctl start sultan-node
sleep 5

# Check status
systemctl status sultan-node --no-pager

# Test RPC
echo ""
echo "🧪 Testing RPC endpoints..."
sleep 10
curl -s http://localhost:26657/status | jq '.'
curl -s http://localhost:26657/tokens/list | jq '.'
curl -s http://localhost:26657/dex/pools | jq '.'

echo ""
echo "✅ Node is running!"
REMOTE_EOF

echo ""
echo "🔒 Step 4: Setting up SSL certificates..."

ssh $SSH_USER@$VPS_IP << CERT_EOF
certbot --nginx -d $DOMAIN -d $API_DOMAIN --non-interactive --agree-tos --email admin@sultanchain.io

# Test HTTPS
curl -s https://$DOMAIN/status | jq '.height'
CERT_EOF

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                 🎉 DEPLOYMENT SUCCESSFUL! 🎉                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "📊 Your production endpoints:"
echo "  🌐 HTTP RPC:  http://$DOMAIN"
echo "  🔒 HTTPS RPC: https://$DOMAIN"
echo "  🌐 HTTP API:  http://$API_DOMAIN"
echo "  🔒 HTTPS API: https://$API_DOMAIN"
echo ""
echo "🧪 Test commands:"
echo "  curl https://$DOMAIN/status"
echo "  curl https://$DOMAIN/tokens/list"
echo "  curl https://$DOMAIN/dex/pools"
echo ""
echo "📝 Management commands:"
echo "  ssh $SSH_USER@$VPS_IP"
echo "  systemctl status sultan-node"
echo "  journalctl -u sultan-node -f"
echo "  systemctl restart sultan-node"
echo ""
echo "🎯 Next: Update website to use https://$DOMAIN"
