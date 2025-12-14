# Reconnect to Production Server

## Current Status (as of last session)

✅ **Production server running at 5.161.225.96**
✅ **Sultan node binary built and deployed**
✅ **systemd service configured and running**
✅ **Node listening on localhost:8080**
⚠️ **SSL/nginx not configured yet**
⚠️ **RPC not publicly accessible**

## Step 1: Reconnect to Server

```bash
# From your Mac terminal
ssh -i ~/.ssh/sultan_hetzner root@5.161.225.96
```

## Step 2: Check Node Status

```bash
# Check if systemd service is running
systemctl status sultan-node

# View recent logs
journalctl -u sultan-node -n 50 --no-pager

# Or check the log file
tail -f /root/sultan/node.log

# Test local RPC
curl http://localhost:8080
```

## Step 3: Configure SSL and Nginx

Create the SSL setup script on the server:

```bash
cat > /root/setup-ssl-and-nginx.sh << 'SETUP_SCRIPT'
#!/bin/bash
set -e

echo "=== Sultan L1 SSL & Nginx Setup ==="

# 1. Install nginx and certbot
echo "Installing nginx and certbot..."
apt update
apt install -y nginx certbot python3-certbot-nginx

# 2. Stop nginx temporarily
systemctl stop nginx

# 3. Create nginx config for rpc.sltn.io
cat > /etc/nginx/sites-available/rpc.sltn.io << 'NGINX_RPC'
server {
    listen 80;
    server_name rpc.sltn.io;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        
        # CORS headers
        add_header 'Access-Control-Allow-Origin' '*' always;
        add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS' always;
        add_header 'Access-Control-Allow-Headers' 'DNT,User-Agent,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Range,Authorization' always;
        
        # Handle preflight
        if ($request_method = 'OPTIONS') {
            add_header 'Access-Control-Allow-Origin' '*';
            add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS';
            add_header 'Access-Control-Max-Age' 1728000;
            add_header 'Content-Type' 'text/plain; charset=utf-8';
            add_header 'Content-Length' 0;
            return 204;
        }
    }
}
NGINX_RPC

# 4. Enable the site
ln -sf /etc/nginx/sites-available/rpc.sltn.io /etc/nginx/sites-enabled/
rm -f /etc/nginx/sites-enabled/default

# 5. Test nginx config
nginx -t

# 6. Start nginx
systemctl start nginx
systemctl enable nginx

# 7. Get SSL certificate
echo "Getting SSL certificate for rpc.sltn.io..."
certbot --nginx -d rpc.sltn.io \
    --non-interactive \
    --agree-tos \
    --email admin@sltn.io \
    --redirect

# 8. Verify SSL auto-renewal
certbot renew --dry-run

echo ""
echo "=== Setup Complete ==="
echo "RPC endpoint: https://rpc.sltn.io"
echo ""
echo "Testing endpoints:"
curl -s https://rpc.sltn.io 2>&1 | head -5
echo ""
echo "SSL certificate will auto-renew via certbot systemd timer"
SETUP_SCRIPT

chmod +x /root/setup-ssl-and-nginx.sh
```

## Step 4: Run SSL Setup

```bash
./setup-ssl-and-nginx.sh
```

## Step 5: Update Sultan Node Service (Public RPC)

The node is currently listening on `127.0.0.1:8080`. Since nginx will proxy to it, this is fine. But if you want direct public access on a different port:

```bash
# Edit the systemd service
nano /etc/systemd/system/sultan-node.service

# Change:
# Environment="SULTAN_RPC_ADDR=0.0.0.0:3030"

# Reload and restart
systemctl daemon-reload
systemctl restart sultan-node

# Check status
systemctl status sultan-node
```

## Step 6: Test Everything

```bash
# Test HTTPS endpoint
curl https://rpc.sltn.io

# Test from external machine
curl https://rpc.sltn.io

# Check logs
journalctl -u sultan-node -f
```

## Step 7: Update Website (Replit)

In your Replit website, change the JavaScript (around line 1665):

```javascript
// CHANGE FROM:
const API_URL = 'https://rpc.sltn.io/api/stats';

// CHANGE TO:
const API_URL = 'https://rpc.sltn.io/status';
```

## Troubleshooting

### If SSL certificate fails:

```bash
# Check DNS resolution
dig rpc.sltn.io

# Verify port 80 is accessible
netstat -tulpn | grep :80

# Check nginx logs
tail -f /var/log/nginx/error.log
```

### If node crashes:

```bash
# Check logs
journalctl -u sultan-node -n 100

# Restart service
systemctl restart sultan-node

# Check config
cat /root/sultan/config.toml
```

### If connection times out:

```bash
# Check firewall
ufw status

# Ensure ports are open
ufw allow 80
ufw allow 443
ufw allow 8080
```

## DNS Records (Already Configured in Hostinger)

✅ `rpc.sltn.io` → `5.161.225.96` (A record, TTL 300)
✅ `api.sltn.io` → `5.161.225.96` (A record, TTL 300)
✅ CAA records allow Let's Encrypt

## Current Node Configuration

```toml
inflation_rate = 8.0
total_supply = 500000000
min_stake = 10000
shards = 1024
genesis_time = 1733256000
current_block = 0
blocks_per_year = 15768000
last_inflation_block = 0
```

## Production Stats

- **Total Supply**: 540M SLTN (500M initial + 40M minted at 8%)
- **Validators**: 11 validators with 10K SLTN each (110K total staked)
- **APY**: 2666.67% target on minimum stake
- **Shards**: 1024 parallel shards
- **Block Time**: 2 seconds
- **TPS Capacity**: 64,000+ (will scale to 10M+ with full sharding)

## Next Steps After SSL

1. ✅ **SSL configured** - RPC accessible at `https://rpc.sltn.io`
2. 📝 **Update Replit website** - Change `/api/stats` → `/status`
3. 🚀 **Deploy website** - Push changes to production
4. ✅ **Test end-to-end** - Verify website shows live stats
5. 📊 **Monitor** - Set up monitoring and alerts
6. 🔒 **Security hardening** - Follow SECURITY_AUDIT_GUIDE.md

## Commands Quick Reference

```bash
# Service management
systemctl status sultan-node
systemctl restart sultan-node
systemctl stop sultan-node
journalctl -u sultan-node -f

# Nginx management
systemctl status nginx
systemctl restart nginx
nginx -t

# SSL certificate
certbot certificates
certbot renew

# Node testing
curl http://localhost:8080
curl https://rpc.sltn.io
```

## Session was Interrupted At

You were about to run the SSL setup script. The node is running successfully via systemd, but nginx and SSL are not yet configured.

**Resume from Step 3** to continue the deployment.
