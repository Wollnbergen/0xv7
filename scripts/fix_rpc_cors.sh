#!/bin/bash
# =============================================================================
# RPC CORS Fix Script
# =============================================================================
# 
# WHEN TO USE THIS SCRIPT:
# If the website (sltn.io) shows "Network stats unavailable" or console errors:
#   "Access-Control-Allow-Origin cannot contain more than one origin"
#   "Fetch API cannot load https://rpc.sltn.io/... due to access control checks"
#
# ROOT CAUSE:
# Both nginx AND sultan-node are adding CORS headers, resulting in duplicate
# headers that browsers reject. Only ONE should handle CORS.
#
# SOLUTION:
# - sultan-node handles CORS via --allowed-origins flag
# - nginx should NOT add any Access-Control-* headers
#
# =============================================================================

set -e

# Configuration
RPC_SERVER="206.189.224.142"
SSH_KEY="${SSH_KEY:-~/.ssh/sultan_deploy}"

echo "🔧 Sultan RPC CORS Fix Script"
echo "=============================="
echo ""

# Step 1: Test current state
echo "📡 Step 1: Testing current CORS state..."
HEADERS=$(curl -s -D - "https://rpc.sltn.io/status" -H "Origin: https://sltn.io" -o /dev/null 2>&1)
CORS_COUNT=$(echo "$HEADERS" | grep -ci "access-control-allow-origin" || true)

if [ "$CORS_COUNT" -eq 0 ]; then
    echo "❌ No CORS headers found - sultan-node may not have --allowed-origins set"
elif [ "$CORS_COUNT" -eq 1 ]; then
    echo "✅ CORS is correctly configured (single header)"
    echo ""
    echo "Current headers:"
    echo "$HEADERS" | grep -i "access-control\|HTTP"
    echo ""
    echo "No fix needed! If website still broken, try hard refresh (Cmd+Shift+R)"
    exit 0
else
    echo "❌ DUPLICATE CORS HEADERS DETECTED ($CORS_COUNT headers)"
    echo "This causes browser CORS errors!"
fi

echo ""
echo "📋 Step 2: Checking SSH access..."
if [ ! -f "$SSH_KEY" ]; then
    echo "❌ SSH key not found at $SSH_KEY"
    echo "   Set SSH_KEY environment variable to your key path"
    exit 1
fi

echo "✅ SSH key found: $SSH_KEY"
echo ""

# Step 3: Show current nginx config
echo "📄 Step 3: Current nginx config:"
ssh -i "$SSH_KEY" root@$RPC_SERVER 'cat /etc/nginx/sites-available/rpc.sltn.io' 2>/dev/null || {
    echo "❌ SSH connection failed"
    exit 1
}

echo ""
echo "🔍 Looking for CORS headers in nginx config..."
if ssh -i "$SSH_KEY" root@$RPC_SERVER 'grep -q "Access-Control-Allow-Origin" /etc/nginx/sites-available/rpc.sltn.io'; then
    echo "⚠️  FOUND: nginx is adding CORS headers"
    echo ""
    
    read -p "Remove CORS headers from nginx? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "📝 Updating nginx config..."
        
        ssh -i "$SSH_KEY" root@$RPC_SERVER 'cat > /etc/nginx/sites-available/rpc.sltn.io << '\''EOF'\''
server {
    listen 80;
    server_name rpc.sltn.io;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl;
    server_name rpc.sltn.io;

    ssl_certificate /etc/letsencrypt/live/rpc.sltn.io/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/rpc.sltn.io/privkey.pem;
    include /etc/letsencrypt/options-ssl-nginx.conf;
    ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem;

    location / {
        # Prevent caching of API responses
        add_header '\''Cache-Control'\'' '\''no-store, no-cache, must-revalidate, proxy-revalidate, max-age=0'\'' always;
        add_header '\''Pragma'\'' '\''no-cache'\'' always;
        add_header '\''Expires'\'' '\''0'\'' always;

        # CORS is handled by sultan-node (--allowed-origins flag)
        # Do NOT add CORS headers here to avoid duplicates

        proxy_pass http://127.0.0.1:8545;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        
        # Pass Origin header to backend for CORS handling
        proxy_set_header Origin $http_origin;
        
        # Disable proxy buffering for real-time responses
        proxy_buffering off;
        
        # Timeouts
        proxy_connect_timeout 5s;
        proxy_send_timeout 10s;
        proxy_read_timeout 10s;
    }
}
EOF'
        
        echo "🔄 Testing nginx config..."
        ssh -i "$SSH_KEY" root@$RPC_SERVER 'nginx -t' || {
            echo "❌ nginx config test failed!"
            exit 1
        }
        
        echo "🔄 Reloading nginx..."
        ssh -i "$SSH_KEY" root@$RPC_SERVER 'systemctl reload nginx'
        
        echo "✅ nginx updated and reloaded"
    fi
else
    echo "✅ nginx is NOT adding CORS headers (correct)"
fi

echo ""
echo "🔍 Step 4: Checking sultan-node CORS config..."
SYSTEMD_CONFIG=$(ssh -i "$SSH_KEY" root@$RPC_SERVER 'cat /etc/systemd/system/sultan-node.service')

if echo "$SYSTEMD_CONFIG" | grep -q "allowed-origins"; then
    echo "✅ sultan-node has --allowed-origins flag set"
    echo "$SYSTEMD_CONFIG" | grep "ExecStart"
else
    echo "⚠️  sultan-node missing --allowed-origins flag!"
    echo ""
    echo "Current ExecStart:"
    echo "$SYSTEMD_CONFIG" | grep "ExecStart"
    echo ""
    echo "Add '--allowed-origins \"*\"' to the ExecStart line in:"
    echo "  /etc/systemd/system/sultan-node.service"
    echo ""
    echo "Then run:"
    echo "  systemctl daemon-reload"
    echo "  systemctl restart sultan-node"
fi

echo ""
echo "📡 Step 5: Final verification..."
sleep 2
FINAL_HEADERS=$(curl -s -D - "https://rpc.sltn.io/status" -H "Origin: https://sltn.io" -o /dev/null 2>&1)
FINAL_COUNT=$(echo "$FINAL_HEADERS" | grep -ci "access-control-allow-origin" || true)

if [ "$FINAL_COUNT" -eq 1 ]; then
    echo "✅ CORS is now correctly configured!"
    echo ""
    echo "Headers:"
    echo "$FINAL_HEADERS" | grep -i "access-control\|HTTP"
    echo ""
    echo "🎉 Done! Hard refresh the website (Cmd+Shift+R) to see changes."
else
    echo "❌ Still seeing $FINAL_COUNT CORS headers. Manual investigation needed."
fi
