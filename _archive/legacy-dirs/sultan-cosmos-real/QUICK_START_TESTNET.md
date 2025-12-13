# 🚀 Next Steps: Deploy to Testnet

## Quick Summary

You now have a **fully functional Sultan Cosmos blockchain** running locally! Here's how to deploy it to a public testnet:

## 🎯 Recommended Approach: Cloud VPS Deployment

### Best Cloud Providers for Cosmos Nodes

1. **Hetzner** (Most cost-effective)
   - CPX31: 4 vCPU, 8GB RAM, 160GB SSD - €13.90/month
   - Location: EU (Nuremberg, Helsinki)
   - Great network, good for validators

2. **DigitalOcean** (Easy to use)
   - Basic Droplet: 2 vCPU, 4GB RAM, 80GB SSD - $24/month
   - Global locations
   - Simple UI, good docs

3. **AWS/GCP** (Enterprise)
   - t3.medium: 2 vCPU, 4GB RAM - ~$30/month
   - Free tier available for testing
   - Better for production long-term

### Minimum Requirements
- **CPU**: 2 cores (4 recommended)
- **RAM**: 4GB (8GB recommended)
- **Disk**: 100GB SSD
- **Network**: 100Mbps+
- **OS**: Ubuntu 22.04 or 24.04 LTS

---

## 📋 Step-by-Step Deployment

### 1. Provision Server

```bash
# Example: Create Hetzner server via CLI
hcloud server create \
  --name sultan-testnet-1 \
  --type cpx31 \
  --image ubuntu-24.04 \
  --ssh-key your-key
```

### 2. Initial Server Setup

```bash
# SSH into server
ssh root@YOUR_SERVER_IP

# Create non-root user
adduser sultan
usermod -aG sudo sultan
su - sultan

# Install dependencies
sudo apt update && sudo apt upgrade -y
sudo apt install -y build-essential git curl wget jq ufw

# Install Go 1.21
wget https://go.dev/dl/go1.21.5.linux-amd64.tar.gz
sudo tar -C /usr/local -xzf go1.21.5.linux-amd64.tar.gz
echo 'export PATH=$PATH:/usr/local/go/bin:$HOME/go/bin' >> ~/.bashrc
source ~/.bashrc
```

### 3. Deploy Sultan Binary

**Option A: Build from source**
```bash
git clone https://github.com/Wollnbergen/0xv7.git
cd 0xv7/sultan-cosmos-real
go build -o sultand ./cmd/sultand
sudo cp sultand /usr/local/bin/
```

**Option B: Use deployment script**
```bash
# Copy built binary from local dev
scp ./sultand sultan@YOUR_SERVER_IP:~/
ssh sultan@YOUR_SERVER_IP
sudo mv sultand /usr/local/bin/
sudo chmod +x /usr/local/bin/sultand
```

### 4. Run Deployment Script

```bash
# Copy deployment script to server
scp scripts/deploy-testnet.sh sultan@YOUR_SERVER_IP:~/

# Run it
ssh sultan@YOUR_SERVER_IP
chmod +x deploy-testnet.sh
./deploy-testnet.sh
```

The script will:
- ✅ Initialize the node
- ✅ Create validator keys
- ✅ Setup genesis
- ✅ Configure systemd service
- ✅ Start the node

### 5. Configure Firewall

```bash
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 26656/tcp # P2P
sudo ufw allow 26657/tcp # RPC (optional, use nginx proxy instead)
sudo ufw allow 9090/tcp  # gRPC
sudo ufw allow 1317/tcp  # API
sudo ufw enable
```

### 6. Setup Domain & SSL (Optional but Recommended)

```bash
# Install nginx and certbot
sudo apt install -y nginx certbot python3-certbot-nginx

# Configure nginx reverse proxy
sudo nano /etc/nginx/sites-available/sultan-rpc
```

```nginx
server {
    listen 80;
    server_name rpc.yourdomain.com;
    
    location / {
        proxy_pass http://127.0.0.1:26657;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }
}
```

```bash
sudo ln -s /etc/nginx/sites-available/sultan-rpc /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx

# Get SSL certificate
sudo certbot --nginx -d rpc.yourdomain.com
```

### 7. Verify Deployment

```bash
# Check node is running
sudo systemctl status sultand

# Check blocks are being produced
curl localhost:26657/status | jq '.result.sync_info.latest_block_height'

# Watch logs
sudo journalctl -u sultand -f
```

---

## 🐳 Alternative: Docker Deployment

### Quick Start

```bash
# On your server
git clone https://github.com/Wollnbergen/0xv7.git
cd 0xv7/sultan-cosmos-real

# Build and run
docker-compose up -d

# Check logs
docker-compose logs -f validator

# Check status
curl localhost:26657/status
```

---

## 🌍 Multi-Validator Testnet

### Architecture

```
Genesis Validator (You)
     ↓
Validator 2, 3, 4... (Community)
     ↓
Seed Nodes (Discovery)
     ↓
Full Nodes (RPC/API)
```

### Inviting Other Validators

1. **Share Genesis File**
```bash
# On genesis validator
cat ~/.sultan/config/genesis.json
# Share this with other validators
```

2. **Share Your Node ID**
```bash
sultand comet show-node-id
# e.g., a25549d40bd76292c4518f09b692f297ebb2cd67
```

3. **Provide Connection Info**
```bash
# Other validators add to config.toml:
persistent_peers = "YOUR_NODE_ID@YOUR_IP:26656"
```

### For Additional Validators

```bash
# Initialize with same chain ID
sultand init validator-2 --chain-id sultan-testnet-1

# Copy genesis from genesis validator
wget https://yourdomain.com/genesis.json -O ~/.sultan/config/genesis.json

# Add persistent peer
nano ~/.sultan/config/config.toml
# Set: persistent_peers = "NODE_ID@IP:26656"

# Start node and wait for sync
sultand start

# Create validator after sync
sultand tx staking create-validator \
  --amount=100000000000stake \
  --pubkey=$(sultand comet show-validator) \
  --moniker="validator-2" \
  --chain-id=sultan-testnet-1 \
  --commission-rate=0.10 \
  --from=validator \
  --keyring-backend=file
```

---

## 📊 Monitoring Setup

### Prometheus + Grafana

```bash
# On monitoring server
cat > monitoring/prometheus.yml << EOF
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'sultan-validator'
    static_configs:
      - targets: ['YOUR_VALIDATOR_IP:26660']
EOF

# Start with Docker
docker-compose -f monitoring/docker-compose.yml up -d
```

### Grafana Dashboard
- Import Cosmos SDK dashboard: https://grafana.com/grafana/dashboards/11036
- Access: http://your-server:3000 (admin/admin)

---

## 🧪 Testing Your Testnet

### Run Test Suite

```bash
# On server
cd sultan-cosmos-real
./scripts/test-testnet.sh
```

### Manual Tests

```bash
# 1. Check node status
curl https://rpc.yourdomain.com/status | jq

# 2. Check validators
curl https://rpc.yourdomain.com/validators | jq

# 3. Query balance
sultand query bank balances sultan1...

# 4. Send transaction (zero fees!)
sultand tx bank send \
  validator sultan1RECIPIENT 1000stake \
  --fees 0stake \
  --chain-id sultan-testnet-1 \
  --keyring-backend file \
  --yes
```

---

## 🎁 Bonus: Setup Faucet

Create a simple faucet for testnet users:

```bash
# faucet.sh
#!/bin/bash
AMOUNT="${1:-1000000stake}"
ADDRESS="$2"

sultand tx bank send \
  faucet "$ADDRESS" "$AMOUNT" \
  --fees 0stake \
  --chain-id sultan-testnet-1 \
  --keyring-backend file \
  --yes
```

Expose via REST API or Telegram bot for easy access.

---

## 📝 Testnet Documentation

Create these documents for your testnet:

1. **Testnet Info**
   - Chain ID: `sultan-testnet-1`
   - Genesis file location
   - Seed nodes
   - RPC endpoints
   - Faucet URL

2. **Validator Guide**
   - How to join
   - Requirements
   - Setup instructions
   - Monitoring guide

3. **Developer Guide**
   - RPC/API endpoints
   - Example transactions
   - Zero-fee transaction info
   - Smart contract deployment (future)

---

## 🚦 Launch Checklist

Before public announcement:

- [ ] Node is stable and producing blocks
- [ ] RPC/API endpoints are accessible
- [ ] Domain configured with SSL
- [ ] Monitoring is running
- [ ] Backup validator key securely
- [ ] Document genesis parameters
- [ ] Create faucet for testnet tokens
- [ ] Write validator onboarding guide
- [ ] Setup Discord/Telegram for support
- [ ] Announce on social media

---

## 💡 Cost Estimate

**Single Validator Testnet**
- Server: $13-30/month (Hetzner/DO/AWS)
- Domain: $12/year
- **Total: ~$20/month**

**Multi-Validator Testnet (4 validators)**
- 1 Genesis validator: $30/month
- 3 Community validators: Free (community-run)
- 1 Seed node: $15/month
- 1 RPC node: $30/month
- Monitoring: Free (same server)
- **Total: ~$75/month**

---

## 🎯 Recommended Timeline

**Week 1: Single Validator**
- Deploy to VPS
- Configure domain/SSL
- Test thoroughly
- Document everything

**Week 2: Invite Validators**
- Share genesis with 3-5 trusted validators
- Help them join
- Test multi-validator consensus
- Monitor network health

**Week 3: Public Launch**
- Announce publicly
- Create faucet
- Setup block explorer
- Open validator applications

**Week 4: Ecosystem**
- Add REST API
- Developer documentation
- Example dApps
- Community building

---

## 🆘 Need Help?

Common issues and solutions are in `TESTNET_DEPLOYMENT.md`

For this specific setup:
- Current node is running at block height 250+
- All modules working (auth, bank, staking, consensus)
- Zero-fee transactions enabled
- Genesis validator active

**You're ready to deploy! 🚀**

Start with Option 1 (cloud VPS) - it's the most straightforward path to a public testnet.
