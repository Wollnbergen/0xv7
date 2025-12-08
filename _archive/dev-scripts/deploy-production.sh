#!/bin/bash
# Sultan L1 - Production Deployment Script
# Deploy Sultan L1 blockchain to production servers

set -e

echo "════════════════════════════════════════════════════════════"
echo "        🚀 Sultan L1 Production Deployment 🚀               "
echo "════════════════════════════════════════════════════════════"
echo ""

# Configuration
DOMAIN="${SULTAN_DOMAIN:-sultanchain.io}"
RPC_PORT="${SULTAN_RPC_PORT:-26657}"
P2P_PORT="${SULTAN_P2P_PORT:-26656}"
API_PORT="${SULTAN_API_PORT:-1317}"
VALIDATOR_NAME="${SULTAN_VALIDATOR_NAME:-genesis-validator-1}"

echo "📋 Deployment Configuration:"
echo "   Domain: $DOMAIN"
echo "   RPC Port: $RPC_PORT"
echo "   P2P Port: $P2P_PORT"
echo "   API Port: $API_PORT"
echo "   Validator: $VALIDATOR_NAME"
echo ""

# Step 1: Install dependencies
echo "📦 Step 1: Installing system dependencies..."
if command -v apt-get &> /dev/null; then
    sudo apt-get update -qq
    sudo apt-get install -y -qq build-essential pkg-config libssl-dev nginx certbot python3-certbot-nginx jq
elif command -v yum &> /dev/null; then
    sudo yum install -y gcc openssl-devel nginx certbot python3-certbot-nginx jq
fi
echo "✅ Dependencies installed"
echo ""

# Step 2: Build production binary
echo "🔨 Step 2: Building production binary..."
cd /workspaces/0xv7
cargo build --release -p sultan-core --bin sultan-node
BINARY_PATH="/tmp/cargo-target/release/sultan-node"

if [ ! -f "$BINARY_PATH" ]; then
    echo "❌ Build failed - binary not found at $BINARY_PATH"
    exit 1
fi

# Install to system
sudo mkdir -p /usr/local/bin
sudo cp "$BINARY_PATH" /usr/local/bin/sultan-node
sudo chmod +x /usr/local/bin/sultan-node
echo "✅ Binary installed to /usr/local/bin/sultan-node"
echo ""

# Step 3: Create system user
echo "👤 Step 3: Creating system user..."
if ! id -u sultan &>/dev/null; then
    sudo useradd -r -s /bin/bash -d /var/lib/sultan -m sultan
    echo "✅ User 'sultan' created"
else
    echo "ℹ️  User 'sultan' already exists"
fi
echo ""

# Step 4: Setup data directories
echo "📁 Step 4: Setting up data directories..."
sudo mkdir -p /var/lib/sultan/data
sudo mkdir -p /var/log/sultan
sudo mkdir -p /etc/sultan
sudo chown -R sultan:sultan /var/lib/sultan /var/log/sultan
echo "✅ Directories created"
echo ""

# Step 5: Generate validator key
echo "🔑 Step 5: Generating validator key..."
VALIDATOR_ADDRESS="sultan1${VALIDATOR_NAME}$(openssl rand -hex 20)"
VALIDATOR_STAKE="10000000000000"  # 10,000 SLTN

cat > /etc/sultan/validator.json <<EOF
{
  "name": "$VALIDATOR_NAME",
  "address": "$VALIDATOR_ADDRESS",
  "stake": "$VALIDATOR_STAKE",
  "commission": 0.05,
  "created_at": $(date +%s)
}
EOF

sudo chown sultan:sultan /etc/sultan/validator.json
echo "✅ Validator key: $VALIDATOR_ADDRESS"
echo ""

# Step 6: Create systemd service
echo "⚙️  Step 6: Creating systemd service..."
sudo tee /etc/systemd/system/sultan-node.service > /dev/null <<EOF
[Unit]
Description=Sultan L1 Blockchain Node
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=sultan
Group=sultan
WorkingDirectory=/var/lib/sultan
ExecStart=/usr/local/bin/sultan-node \\
  --validator \\
  --validator-address $VALIDATOR_ADDRESS \\
  --validator-stake $VALIDATOR_STAKE \\
  --enable-sharding \\
  --shard-count 100 \\
  --tx-per-shard 10000 \\
  --block-time 2 \\
  --data-dir /var/lib/sultan/data \\
  --rpc-addr 0.0.0.0:$RPC_PORT \\
  --p2p-addr 0.0.0.0:$P2P_PORT

Restart=always
RestartSec=10
LimitNOFILE=65535

StandardOutput=append:/var/log/sultan/node.log
StandardError=append:/var/log/sultan/error.log

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
echo "✅ Systemd service created"
echo ""

# Step 7: Configure Nginx reverse proxy
echo "🌐 Step 7: Configuring Nginx..."
sudo tee /etc/nginx/sites-available/sultan > /dev/null <<EOF
# RPC endpoint
server {
    listen 80;
    server_name rpc.$DOMAIN;

    location / {
        proxy_pass http://127.0.0.1:$RPC_PORT;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_cache_bypass \$http_upgrade;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        
        # CORS headers
        add_header Access-Control-Allow-Origin * always;
        add_header Access-Control-Allow-Methods 'GET, POST, OPTIONS' always;
        add_header Access-Control-Allow-Headers 'DNT,User-Agent,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Range' always;
        
        if (\$request_method = 'OPTIONS') {
            return 204;
        }
    }
}

# Website
server {
    listen 80;
    server_name $DOMAIN www.$DOMAIN;

    root /var/www/sultan;
    index index.html;

    location / {
        try_files \$uri \$uri/ =404;
    }
}
EOF

sudo ln -sf /etc/nginx/sites-available/sultan /etc/nginx/sites-enabled/
sudo nginx -t
echo "✅ Nginx configured"
echo ""

# Step 8: Setup website
echo "🌐 Step 8: Deploying website..."
sudo mkdir -p /var/www/sultan
sudo cp /workspaces/0xv7/index.html /var/www/sultan/
sudo cp /workspaces/0xv7/add-to-keplr.html /var/www/sultan/
sudo cp /workspaces/0xv7/keplr-chain-config.json /var/www/sultan/
sudo chown -R www-data:www-data /var/www/sultan
echo "✅ Website deployed"
echo ""

# Step 9: SSL certificates (if in production)
echo "🔒 Step 9: SSL Certificates..."
read -p "Setup SSL certificates now? (requires DNS configured) [y/N]: " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    sudo certbot --nginx -d $DOMAIN -d www.$DOMAIN -d rpc.$DOMAIN --non-interactive --agree-tos --email admin@$DOMAIN
    echo "✅ SSL certificates installed"
else
    echo "⚠️  Skipping SSL - configure manually with: sudo certbot --nginx -d $DOMAIN"
fi
echo ""

# Step 10: Start services
echo "🚀 Step 10: Starting services..."
sudo systemctl enable sultan-node
sudo systemctl start sultan-node
sudo systemctl reload nginx

sleep 3

if sudo systemctl is-active --quiet sultan-node; then
    echo "✅ Sultan node is running"
else
    echo "❌ Sultan node failed to start"
    echo "   Check logs: sudo journalctl -u sultan-node -f"
    exit 1
fi
echo ""

# Verification
echo "════════════════════════════════════════════════════════════"
echo "         ✅ Production Deployment Complete! ✅              "
echo "════════════════════════════════════════════════════════════"
echo ""
echo "🌐 URLs:"
echo "   Website:  http://$DOMAIN"
echo "   RPC:      http://rpc.$DOMAIN"
echo "   Keplr:    http://$DOMAIN/add-to-keplr.html"
echo ""
echo "🔍 Validator:"
echo "   Name:     $VALIDATOR_NAME"
echo "   Address:  $VALIDATOR_ADDRESS"
echo "   Stake:    10,000 SLTN"
echo ""
echo "📝 Management Commands:"
echo "   Status:   sudo systemctl status sultan-node"
echo "   Logs:     sudo journalctl -u sultan-node -f"
echo "   Restart:  sudo systemctl restart sultan-node"
echo "   Stop:     sudo systemctl stop sultan-node"
echo ""
echo "🔒 Next Steps:"
echo "   1. Configure DNS: rpc.$DOMAIN → this server's IP"
echo "   2. Setup SSL: sudo certbot --nginx -d $DOMAIN -d rpc.$DOMAIN"
echo "   3. Monitor logs: tail -f /var/log/sultan/node.log"
echo "   4. Add to Keplr: http://$DOMAIN/add-to-keplr.html"
echo ""
