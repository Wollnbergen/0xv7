# Sultan Blockchain - Hetzner Production Deployment

## 🎯 Goal
Deploy Sultan node with RPC endpoints to your Hetzner VPS and configure `rpc.sultanchain.io`

---

## ⚡ Quick Deploy (Automated)

```bash
./deploy-to-hetzner.sh
```

Follow the prompts. The script will:
1. Build release binary
2. Upload to your Hetzner server
3. Install and configure everything
4. Set up SSL with Let's Encrypt
5. Start the node

---

## 📖 Manual Deployment Steps

### 1️⃣ Build Release Binary

```bash
cd /workspaces/0xv7
cargo build -p sultan-core --bin sultan-node --release
```

### 2️⃣ Prepare Server (on Hetzner VPS)

```bash
# SSH into your server
ssh root@your-hetzner-ip

# Update system
apt update && apt upgrade -y

# Install dependencies
apt install -y build-essential nginx certbot python3-certbot-nginx

# Create sultan user
useradd -r -s /bin/false -d /opt/sultan sultan

# Create directories
mkdir -p /opt/sultan
mkdir -p /var/lib/sultan
chown -R sultan:sultan /opt/sultan /var/lib/sultan
```

### 3️⃣ Upload Binary

```bash
# From your local machine/codespace:
scp target/release/sultan-node root@your-hetzner-ip:/tmp/

# On server:
mv /tmp/sultan-node /opt/sultan/
chmod +x /opt/sultan/sultan-node
chown sultan:sultan /opt/sultan/sultan-node
```

### 4️⃣ Create Systemd Service

```bash
# On server, create /etc/systemd/system/sultan-node.service
cat > /etc/systemd/system/sultan-node.service <<EOF
[Unit]
Description=Sultan Blockchain Node
After=network.target

[Service]
Type=simple
User=sultan
WorkingDirectory=/opt/sultan
ExecStart=/opt/sultan/sultan-node \\
    --validator \\
    --validator-address genesis \\
    --validator-stake 10000000 \\
    --rpc-addr 0.0.0.0:26657 \\
    --data-dir /var/lib/sultan
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=sultan-node

NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/sultan

[Install]
WantedBy=multi-user.target
EOF

# Enable and start
systemctl daemon-reload
systemctl enable sultan-node
systemctl start sultan-node

# Check status
systemctl status sultan-node
journalctl -fu sultan-node
```

### 5️⃣ Configure Nginx Reverse Proxy

```bash
# Create nginx config: /etc/nginx/sites-available/sultan
cat > /etc/nginx/sites-available/sultan <<EOF
upstream sultan_rpc {
    server 127.0.0.1:26657;
    keepalive 32;
}

server {
    listen 80;
    server_name rpc.sultanchain.io;
    
    location / {
        proxy_pass http://sultan_rpc;
        proxy_http_version 1.1;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        
        # CORS for RPC
        add_header 'Access-Control-Allow-Origin' '*' always;
        add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS' always;
        add_header 'Access-Control-Allow-Headers' 'Content-Type' always;
    }
}
EOF

# Enable site
ln -s /etc/nginx/sites-available/sultan /etc/nginx/sites-enabled/
rm /etc/nginx/sites-enabled/default
nginx -t
systemctl reload nginx
```

### 6️⃣ Set Up SSL with Let's Encrypt

```bash
# Make sure DNS is pointed to your server first!
# rpc.sultanchain.io -> your-hetzner-ip

certbot --nginx -d rpc.sultanchain.io
```

### 7️⃣ Configure Firewall

```bash
ufw allow 22/tcp     # SSH
ufw allow 80/tcp     # HTTP
ufw allow 443/tcp    # HTTPS
ufw allow 26656/tcp  # P2P (optional, for future validators)
ufw enable
```

---

## 🧪 Testing

### Test locally (on server):
```bash
curl http://localhost:26657/status
curl http://localhost:26657/tokens/list
curl http://localhost:26657/dex/pools
```

### Test publicly:
```bash
curl https://rpc.sultanchain.io/status
curl https://rpc.sultanchain.io/tokens/list
curl https://rpc.sultanchain.io/dex/pools
```

---

## 📊 Monitoring

```bash
# Watch logs
journalctl -fu sultan-node

# Check service status
systemctl status sultan-node

# Node stats
curl http://localhost:26657/status | jq

# Nginx logs
tail -f /var/log/nginx/access.log
tail -f /var/log/nginx/error.log
```

---

## 🔄 Updating the Node

```bash
# Build new binary locally
cargo build -p sultan-core --bin sultan-node --release

# Upload to server
scp target/release/sultan-node root@your-hetzner-ip:/tmp/

# On server - update binary
systemctl stop sultan-node
mv /tmp/sultan-node /opt/sultan/
chown sultan:sultan /opt/sultan/sultan-node
systemctl start sultan-node
journalctl -fu sultan-node
```

---

## 🌐 DNS Configuration

Point these DNS records to your Hetzner server IP:

```
A     rpc.sultanchain.io  -> your-hetzner-ip
A     api.sultanchain.io  -> your-hetzner-ip (alias for rpc)
```

---

## 📝 Important Files

- **Binary**: `/opt/sultan/sultan-node`
- **Data**: `/var/lib/sultan/`
- **Service**: `/etc/systemd/system/sultan-node.service`
- **Nginx**: `/etc/nginx/sites-available/sultan`
- **Logs**: `journalctl -u sultan-node`

---

## 🔥 Troubleshooting

### Node won't start:
```bash
journalctl -xeu sultan-node
# Check permissions on /var/lib/sultan
ls -la /var/lib/sultan
```

### RPC not accessible:
```bash
# Check if node is listening
netstat -tlnp | grep 26657

# Check nginx
nginx -t
systemctl status nginx

# Check firewall
ufw status
```

### SSL issues:
```bash
# Renew certificate
certbot renew --dry-run

# Check certificate
certbot certificates
```

---

## 🎉 Success Checklist

- [ ] Node binary running as systemd service
- [ ] Node producing blocks (check logs)
- [ ] RPC endpoints responding on port 26657
- [ ] Nginx reverse proxy configured
- [ ] SSL certificate installed
- [ ] DNS pointing to server
- [ ] https://rpc.sultanchain.io/status returns JSON
- [ ] Website can fetch stats from RPC endpoint

---

## 📞 Quick Commands Reference

```bash
# Service management
systemctl start sultan-node
systemctl stop sultan-node
systemctl restart sultan-node
systemctl status sultan-node

# Logs
journalctl -fu sultan-node                    # Follow logs
journalctl -u sultan-node --since "1 hour ago" # Recent logs
journalctl -u sultan-node -n 100               # Last 100 lines

# Health checks
curl http://localhost:26657/status             # Local
curl https://rpc.sultanchain.io/status         # Public

# Nginx
systemctl reload nginx
nginx -t
tail -f /var/log/nginx/error.log
```
