#!/bin/bash
# Sultan L1 - Complete SSL & Nginx Setup Script
# Run this on the production server (5.161.225.96)

set -e

echo "🚀 Setting up SSL and Nginx for Sultan L1..."

# Update and install required packages
echo "📦 Installing nginx and certbot..."
apt update
apt install -y nginx certbot python3-certbot-nginx

# Create nginx configuration for RPC endpoint
echo "⚙️ Creating nginx configuration for rpc.sltn.io..."
cat > /etc/nginx/sites-available/sultan-rpc << 'NGINX_EOF'
server {
    listen 80;
    server_name rpc.sltn.io;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        
        # CORS headers (Sultan RPC already has them, but adding here as backup)
        add_header 'Access-Control-Allow-Origin' '*' always;
        add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS' always;
        add_header 'Access-Control-Allow-Headers' 'Content-Type, Authorization' always;
        
        # Handle OPTIONS preflight
        if ($request_method = 'OPTIONS') {
            add_header 'Access-Control-Allow-Origin' '*';
            add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS';
            add_header 'Access-Control-Allow-Headers' 'Content-Type, Authorization';
            add_header 'Content-Length' 0;
            add_header 'Content-Type' 'text/plain';
            return 204;
        }
    }
}
NGINX_EOF

# Create nginx configuration for API endpoint (if different)
echo "⚙️ Creating nginx configuration for api.sltn.io..."
cat > /etc/nginx/sites-available/sultan-api << 'NGINX_EOF'
server {
    listen 80;
    server_name api.sltn.io;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        
        # CORS headers
        add_header 'Access-Control-Allow-Origin' '*' always;
        add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS' always;
        add_header 'Access-Control-Allow-Headers' 'Content-Type, Authorization' always;
        
        # Handle OPTIONS preflight
        if ($request_method = 'OPTIONS') {
            add_header 'Access-Control-Allow-Origin' '*';
            add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS';
            add_header 'Access-Control-Allow-Headers' 'Content-Type, Authorization';
            add_header 'Content-Length' 0;
            add_header 'Content-Type' 'text/plain';
            return 204;
        }
    }
}
NGINX_EOF

# Enable the sites
echo "🔗 Enabling nginx sites..."
ln -sf /etc/nginx/sites-available/sultan-rpc /etc/nginx/sites-enabled/
ln -sf /etc/nginx/sites-available/sultan-api /etc/nginx/sites-enabled/

# Remove default site if it conflicts
rm -f /etc/nginx/sites-enabled/default

# Test nginx configuration
echo "🧪 Testing nginx configuration..."
nginx -t

# Reload nginx
echo "🔄 Reloading nginx..."
systemctl reload nginx

# Get SSL certificates
echo "🔒 Obtaining SSL certificates from Let's Encrypt..."
echo "    This will set up HTTPS for rpc.sltn.io and api.sltn.io"
echo ""

# Get certificate for rpc.sltn.io
certbot --nginx -d rpc.sltn.io \
    --non-interactive \
    --agree-tos \
    --email admin@sltn.io \
    --redirect

# Get certificate for api.sltn.io  
certbot --nginx -d api.sltn.io \
    --non-interactive \
    --agree-tos \
    --email admin@sltn.io \
    --redirect

# Verify SSL is working
echo ""
echo "✅ SSL Setup Complete!"
echo ""
echo "Testing endpoints:"
echo "  - http://rpc.sltn.io/status (should redirect to https://)"
echo "  - https://rpc.sltn.io/status (should work)"
echo ""

# Test the endpoints
echo "🧪 Testing RPC endpoint..."
curl -s https://rpc.sltn.io/status | jq . || echo "⚠️ RPC endpoint test failed"

echo ""
echo "🎉 Setup complete! Your Sultan L1 blockchain is now accessible at:"
echo "   - https://rpc.sltn.io (RPC endpoint)"
echo "   - https://api.sltn.io (API endpoint)"
echo ""
echo "Next steps:"
echo "1. Update website to use https://rpc.sltn.io (already done)"
echo "2. Test website at https://sltn.io"
echo "3. Verify network statistics are displaying correctly"
echo ""
