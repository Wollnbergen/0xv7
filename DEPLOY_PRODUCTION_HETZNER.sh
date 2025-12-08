#!/bin/bash
set -e

echo "🚀 Sultan Blockchain - Production Deployment to Hetzner"
echo "======================================================="
echo ""

# Configuration
SERVER_IP="5.161.225.96"
DOMAIN_RPC="rpc.sltn.io"
DOMAIN_API="api.sltn.io"
DOMAIN_GRPC="grpc.sltn.io"
EMAIL="admin@sltn.io"

echo "📋 Server: $SERVER_IP"
echo "📋 RPC Domain: $DOMAIN_RPC"
echo "📋 API Domain: $DOMAIN_API"
echo "📋 gRPC Domain: $DOMAIN_GRPC"
echo ""

# Check if we can SSH to the server
echo "🔐 Testing SSH connection..."
if ! ssh -i ~/.ssh/sultan_hetzner -o ConnectTimeout=5 root@$SERVER_IP "echo 'SSH OK'"; then
    echo "❌ Cannot connect to server. Check your SSH key."
    exit 1
fi

echo "✅ SSH connection successful"
echo ""

# Create nginx configuration on server
echo "📝 Creating nginx configuration..."
ssh -i ~/.ssh/sultan_hetzner root@$SERVER_IP << 'REMOTE_SCRIPT'
# Create nginx config for Sultan RPC
cat > /etc/nginx/sites-available/sultan-rpc << 'NGINX_RPC'
upstream sultan_rpc {
    server 127.0.0.1:8080;
    keepalive 32;
}

server {
    listen 80;
    server_name rpc.sltn.io;
    
    location /.well-known/acme-challenge/ {
        root /var/www/html;
    }
    
    location / {
        return 301 https://$server_name$request_uri;
    }
}

server {
    listen 443 ssl http2;
    server_name rpc.sltn.io;
    
    # SSL certificates (will be created by certbot)
    ssl_certificate /etc/letsencrypt/live/rpc.sltn.io/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/rpc.sltn.io/privkey.pem;
    
    # SSL configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;
    
    # CORS headers for blockchain RPC (always flag applies to all responses including OPTIONS)
    add_header 'Access-Control-Allow-Origin' '*' always;
    add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS' always;
    add_header 'Access-Control-Allow-Headers' 'Content-Type, Authorization' always;
    add_header 'Access-Control-Max-Age' 1728000 always;
    
    location / {
        # Handle OPTIONS (preflight) requests - CORS headers from server block apply automatically
        if ($request_method = 'OPTIONS') {
            return 204;
        }
        
        proxy_pass http://sultan_rpc;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Timeouts for blockchain operations
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }
}
NGINX_RPC

# Enable the site
ln -sf /etc/nginx/sites-available/sultan-rpc /etc/nginx/sites-enabled/
rm -f /etc/nginx/sites-enabled/default

# Test nginx config
nginx -t

echo "✅ Nginx configuration created"
REMOTE_SCRIPT

echo ""
echo "🔒 Setting up SSL certificates with Let's Encrypt..."
ssh -i ~/.ssh/sultan_hetzner root@$SERVER_IP << REMOTE_SSL
# Stop nginx temporarily for certbot standalone
systemctl stop nginx

# Get SSL certificate for rpc.sltn.io
certbot certonly --standalone \
    --non-interactive \
    --agree-tos \
    --email $EMAIL \
    --domains $DOMAIN_RPC

# Start nginx
systemctl start nginx
systemctl enable nginx

echo "✅ SSL certificate installed"
REMOTE_SSL

echo ""
echo "🔧 Creating systemd service for Sultan node..."
ssh -i ~/.ssh/sultan_hetzner root@$SERVER_IP << 'REMOTE_SYSTEMD'
# Create systemd service file
cat > /etc/systemd/system/sultan-node.service << 'SERVICE'
[Unit]
Description=Sultan Blockchain Node
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root/sultan
Environment="TELOXIDE_TOKEN=8069901972:AAGpsmRJEsGT3G7iFbv9TvMbzvTJwAfsoeQ"
Environment="SULTAN_DB_ADDR=127.0.0.1:9042"
Environment="SULTAN_RPC_ADDR=0.0.0.0:8080"
Environment="RUST_LOG=info"
ExecStart=/root/sultan/target/release/p2p_node start
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
SERVICE

# Reload systemd
systemctl daemon-reload

# Stop the currently running node (if any)
pkill -f p2p_node || true
sleep 2

# Enable and start the service
systemctl enable sultan-node
systemctl start sultan-node

echo "✅ Systemd service created and started"
REMOTE_SYSTEMD

echo ""
echo "⏳ Waiting for node to start..."
sleep 5

echo ""
echo "🧪 Testing RPC endpoint..."
if curl -f -s https://$DOMAIN_RPC/ > /dev/null 2>&1; then
    echo "✅ HTTPS endpoint is responding"
else
    echo "⚠️  HTTPS not ready yet (SSL may still be propagating)"
    echo "   Testing HTTP..."
    if ssh -i ~/.ssh/sultan_hetzner root@$SERVER_IP "curl -f -s http://localhost:8080/ > /dev/null"; then
        echo "✅ Node is running locally"
    else
        echo "❌ Node may not be running. Checking logs..."
        ssh -i ~/.ssh/sultan_hetzner root@$SERVER_IP "journalctl -u sultan-node -n 50 --no-pager"
    fi
fi

echo ""
echo "📊 Node Status:"
ssh -i ~/.ssh/sultan_hetzner root@$SERVER_IP "systemctl status sultan-node --no-pager -l"

echo ""
echo "📝 Recent Logs:"
ssh -i ~/.ssh/sultan_hetzner root@$SERVER_IP "journalctl -u sultan-node -n 20 --no-pager"

echo ""
echo "✅ Deployment Complete!"
echo ""
echo "🌐 Your Sultan blockchain is now available at:"
echo "   • RPC: https://$DOMAIN_RPC/"
echo "   • Test: curl https://$DOMAIN_RPC/"
echo ""
echo "📊 Monitor the node:"
echo "   ssh -i ~/.ssh/sultan_hetzner root@$SERVER_IP 'journalctl -u sultan-node -f'"
echo ""
echo "🔄 Restart the node:"
echo "   ssh -i ~/.ssh/sultan_hetzner root@$SERVER_IP 'systemctl restart sultan-node'"
echo ""
