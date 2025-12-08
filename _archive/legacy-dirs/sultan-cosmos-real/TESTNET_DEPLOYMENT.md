# Sultan Testnet Deployment Guide

## 🎯 Deployment Options

### Option 1: Single Node Testnet (Quick Testing)
**Use Case**: Internal testing, development, CI/CD
**Time**: 5 minutes
**Resources**: 1 server, 2 CPU, 4GB RAM

### Option 2: Multi-Node Testnet (Realistic)
**Use Case**: Public testnet, validator testing, load testing
**Time**: 30 minutes
**Resources**: 3+ servers, 2 CPU each, 4GB RAM each

### Option 3: Docker-Based Testnet
**Use Case**: Easy deployment, consistent environments
**Time**: 15 minutes
**Resources**: Docker host with 8GB+ RAM

---

## 🚀 Option 1: Single Node Testnet (Recommended for Initial Testing)

This is what we have running locally - let's deploy it to a public server.

### Prerequisites

- VPS/Cloud server (Ubuntu 22.04/24.04)
- Root/sudo access
- Open ports: 26656 (P2P), 26657 (RPC), 9090 (gRPC), 1317 (REST)
- Domain name (optional but recommended)

### Step 1: Prepare the Server

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install dependencies
sudo apt install -y build-essential git curl wget jq

# Install Go 1.21+
wget https://go.dev/dl/go1.21.5.linux-amd64.tar.gz
sudo rm -rf /usr/local/go
sudo tar -C /usr/local -xzf go1.21.5.linux-amd64.tar.gz
echo 'export PATH=$PATH:/usr/local/go/bin:$HOME/go/bin' >> ~/.bashrc
source ~/.bashrc

# Verify Go installation
go version
```

### Step 2: Build Sultan Binary

```bash
# Clone repository
git clone https://github.com/Wollnbergen/0xv7.git
cd 0xv7/sultan-cosmos-real

# Build binary
go build -o sultand ./cmd/sultand

# Install to system path
sudo cp sultand /usr/local/bin/
sudo chmod +x /usr/local/bin/sultand

# Verify installation
sultand version
```

### Step 3: Initialize Node

```bash
# Set chain ID and node moniker
export CHAIN_ID=sultan-testnet-1
export MONIKER="sultan-testnet-validator-1"

# Initialize node
sultand init $MONIKER --chain-id $CHAIN_ID

# This creates ~/.sultan directory with:
# - config/genesis.json
# - config/config.toml
# - config/app.toml
# - config/priv_validator_key.json
```

### Step 4: Create Genesis

```bash
# Create validator key
sultand keys add validator --keyring-backend file

# Save the mnemonic securely!
# Note the address (sultan...)

# Add genesis account (1 trillion stake)
sultand add-genesis-account validator 1000000000000stake --keyring-backend file

# Create genesis transaction (900 billion bonded)
sultand gentx validator 900000000000stake \
  --chain-id $CHAIN_ID \
  --keyring-backend file \
  --moniker $MONIKER \
  --commission-rate 0.10 \
  --commission-max-rate 0.20 \
  --commission-max-change-rate 0.01 \
  --min-self-delegation 1

# If gentx fails (SDK v0.50.5 issue), use manual genesis creation:
# See "Manual Genesis Creation" section below

# Collect genesis transactions
sultand collect-gentxs

# Validate genesis
sultand validate-genesis
```

### Step 5: Configure Node

```bash
# Edit config.toml
nano ~/.sultan/config/config.toml
```

**Key Settings:**
```toml
# P2P Configuration
[p2p]
laddr = "tcp://0.0.0.0:26656"
external_address = "YOUR_PUBLIC_IP:26656"  # Replace with your server's public IP
persistent_peers = ""  # Add other validators later
max_num_inbound_peers = 40
max_num_outbound_peers = 10

# RPC Configuration
[rpc]
laddr = "tcp://0.0.0.0:26657"  # Make RPC public (or use nginx proxy)
cors_allowed_origins = ["*"]
max_open_connections = 900

# Mempool
[mempool]
size = 5000
cache_size = 10000
```

```bash
# Edit app.toml
nano ~/.sultan/config/app.toml
```

**Key Settings:**
```toml
# API Configuration
[api]
enable = true
swagger = true
address = "tcp://0.0.0.0:1317"
enabled-unsafe-cors = true

# gRPC Configuration  
[grpc]
enable = true
address = "0.0.0.0:9090"

# State Sync (for future validators)
[state-sync]
snapshot-interval = 1000
snapshot-keep-recent = 2

# Minimum Gas Prices (zero fees enabled)
minimum-gas-prices = "0stake"
```

### Step 6: Create Systemd Service

```bash
sudo tee /etc/systemd/system/sultand.service > /dev/null <<EOF
[Unit]
Description=Sultan Cosmos Node
After=network-online.target

[Service]
User=$USER
ExecStart=/usr/local/bin/sultand start --home $HOME/.sultan
Restart=on-failure
RestartSec=3
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd
sudo systemctl daemon-reload

# Enable service
sudo systemctl enable sultand

# Start service
sudo systemctl start sultand

# Check status
sudo systemctl status sultand

# View logs
sudo journalctl -u sultand -f
```

### Step 7: Verify Node is Running

```bash
# Check status
curl localhost:26657/status | jq

# Check latest block
curl localhost:26657/block | jq '.result.block.header.height'

# Check validator info
sultand query staking validators --output json | jq

# Check your balance
sultand query bank balances $(sultand keys show validator -a --keyring-backend file)
```

### Step 8: Firewall Configuration

```bash
# Allow SSH
sudo ufw allow 22/tcp

# Allow P2P
sudo ufw allow 26656/tcp

# Allow RPC (optional - use nginx proxy instead)
sudo ufw allow 26657/tcp

# Allow gRPC
sudo ufw allow 9090/tcp

# Allow REST API
sudo ufw allow 1317/tcp

# Enable firewall
sudo ufw enable
```

---

## 🌐 Option 2: Multi-Node Testnet

For a realistic testnet with multiple validators:

### Architecture

```
Validator 1 (Genesis)     Validator 2              Validator 3
     |                         |                        |
     +-------------------------+------------------------+
                          P2P Network
                               |
                          Seed Nodes
                               |
                          Full Nodes
                               |
                          Public RPC/API
```

### Step 1: Initialize First Validator (Genesis)

Follow Option 1 steps 1-6 on the first server.

**Important**: Save these files to share with other validators:
- `~/.sultan/config/genesis.json` (after collect-gentxs)
- Node ID: `sultand comet show-node-id`
- Public IP

### Step 2: Initialize Additional Validators

On each additional validator server:

```bash
# Initialize node
sultand init validator-2 --chain-id sultan-testnet-1

# Copy genesis from validator 1
scp validator1:~/.sultan/config/genesis.json ~/.sultan/config/

# Create validator key
sultand keys add validator --keyring-backend file

# Configure persistent peers (use validator 1's node ID and IP)
nano ~/.sultan/config/config.toml
# Set: persistent_peers = "NODE_ID@IP:26656"

# Start node
sudo systemctl start sultand

# Wait for sync, then create validator
sultand tx staking create-validator \
  --amount=100000000000stake \
  --pubkey=$(sultand comet show-validator) \
  --moniker="validator-2" \
  --chain-id=sultan-testnet-1 \
  --commission-rate=0.10 \
  --commission-max-rate=0.20 \
  --commission-max-change-rate=0.01 \
  --min-self-delegation=1 \
  --from=validator \
  --keyring-backend=file \
  --fees=0stake
```

### Step 3: Configure Seed Nodes

Dedicated seed nodes help new nodes discover the network:

```bash
# On seed node server
sultand init seed-node --chain-id sultan-testnet-1

# Copy genesis
scp validator1:~/.sultan/config/genesis.json ~/.sultan/config/

# Edit config.toml
nano ~/.sultan/config/config.toml
```

Set:
```toml
[p2p]
seed_mode = true
persistent_peers = "validator1_id@ip:26656,validator2_id@ip:26656"
```

### Step 4: Setup Public RPC/API Gateway

Use nginx as reverse proxy:

```bash
sudo apt install nginx certbot python3-certbot-nginx

# Configure nginx
sudo nano /etc/nginx/sites-available/sultan-rpc
```

```nginx
upstream sultan_rpc {
    server 127.0.0.1:26657;
}

upstream sultan_api {
    server 127.0.0.1:1317;
}

server {
    listen 80;
    server_name rpc.sultan-testnet.com;  # Your domain

    location / {
        proxy_pass http://sultan_rpc;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }
}

server {
    listen 80;
    server_name api.sultan-testnet.com;  # Your domain

    location / {
        proxy_pass http://sultan_api;
        proxy_set_header Host $host;
    }
}
```

```bash
# Enable sites
sudo ln -s /etc/nginx/sites-available/sultan-rpc /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx

# Setup SSL
sudo certbot --nginx -d rpc.sultan-testnet.com -d api.sultan-testnet.com
```

---

## 🐳 Option 3: Docker-Based Testnet

### Step 1: Create Dockerfile

```dockerfile
# Dockerfile
FROM golang:1.21-alpine AS builder

RUN apk add --no-cache git build-base linux-headers

WORKDIR /app
COPY . .

RUN go build -o sultand ./cmd/sultand

FROM alpine:latest

RUN apk add --no-cache ca-certificates bash jq

COPY --from=builder /app/sultand /usr/local/bin/

EXPOSE 26656 26657 9090 1317

ENTRYPOINT ["sultand"]
CMD ["start"]
```

### Step 2: Create Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  validator1:
    build: .
    container_name: sultan-validator-1
    ports:
      - "26656:26656"
      - "26657:26657"
      - "9090:9090"
      - "1317:1317"
    volumes:
      - ./validator1:/root/.sultan
    command: start
    networks:
      - sultan-testnet

  validator2:
    build: .
    container_name: sultan-validator-2
    ports:
      - "26666:26656"
      - "26667:26657"
    volumes:
      - ./validator2:/root/.sultan
    command: start
    networks:
      - sultan-testnet
    depends_on:
      - validator1

  validator3:
    build: .
    container_name: sultan-validator-3
    ports:
      - "26676:26656"
      - "26677:26657"
    volumes:
      - ./validator3:/root/.sultan
    command: start
    networks:
      - sultan-testnet
    depends_on:
      - validator1

networks:
  sultan-testnet:
    driver: bridge
```

### Step 3: Initialize Docker Testnet

```bash
# Create initialization script
cat > init-testnet.sh << 'EOF'
#!/bin/bash

CHAIN_ID="sultan-testnet-1"
BINARY="docker run --rm -v $(pwd)/testnet:/root/.sultan sultand"

# Initialize each validator
for i in 1 2 3; do
  mkdir -p validator$i
  docker run --rm -v $(pwd)/validator$i:/root/.sultan \
    sultand init validator-$i --chain-id $CHAIN_ID
done

# Create keys and genesis
docker run --rm -v $(pwd)/validator1:/root/.sultan \
  sultand keys add validator --keyring-backend test

# Add genesis account
docker run --rm -v $(pwd)/validator1:/root/.sultan \
  sultand add-genesis-account validator 1000000000000stake --keyring-backend test

# Generate genesis tx (or use manual method)
# Copy genesis to other validators
for i in 2 3; do
  cp validator1/config/genesis.json validator$i/config/
done

# Configure persistent peers
# (Add peer configuration logic here)

echo "Testnet initialized! Run: docker-compose up -d"
EOF

chmod +x init-testnet.sh
./init-testnet.sh
```

### Step 4: Launch Docker Testnet

```bash
# Build images
docker-compose build

# Start testnet
docker-compose up -d

# View logs
docker-compose logs -f validator1

# Check status
curl localhost:26657/status
```

---

## 📊 Monitoring & Maintenance

### Prometheus Monitoring

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'sultan-node'
    static_configs:
      - targets: ['localhost:26660']  # CometBFT metrics
```

### Grafana Dashboard

Import Cosmos SDK dashboard:
- https://grafana.com/grafana/dashboards/11036

### Log Management

```bash
# Rotate logs
sudo tee /etc/logrotate.d/sultand > /dev/null <<EOF
/var/log/sultand/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 0640 $USER $USER
}
EOF
```

---

## 🧪 Testing Checklist

### Basic Functionality
- [ ] Node starts successfully
- [ ] Blocks are being produced
- [ ] RPC endpoint responds
- [ ] API endpoint responds
- [ ] Can query balances
- [ ] Can send transactions

### Validator Testing
- [ ] Validator is active
- [ ] Validator is signing blocks
- [ ] Can delegate to validator
- [ ] Can unbond from validator
- [ ] Rewards are accumulating

### Network Testing
- [ ] Nodes can discover each other
- [ ] P2P connections are stable
- [ ] State sync works for new nodes
- [ ] Can replay chain from genesis

### Zero-Fee Testing
- [ ] Transactions with 0 fees are accepted
- [ ] Mempool accepts zero-fee txs
- [ ] Blocks include zero-fee txs

---

## 🔧 Troubleshooting

### Node Won't Start
```bash
# Check logs
sudo journalctl -u sultand -n 100

# Reset data (WARNING: deletes blockchain state)
sultand unsafe-reset-all

# Check genesis validity
sultand validate-genesis
```

### Node Not Syncing
```bash
# Check peers
curl localhost:26657/net_info | jq '.result.peers'

# Check persistent peers in config.toml
# Add more peers if needed
```

### Validator Not Signing
```bash
# Check validator status
sultand query staking validator $(sultand keys show validator --bech val -a --keyring-backend file)

# Check if jailed
sultand query slashing signing-info $(sultand comet show-validator)

# Unjail if needed
sultand tx slashing unjail --from validator --keyring-backend file --fees 0stake
```

---

## 🎁 Next Steps

1. **Deploy to Cloud Provider**
   - DigitalOcean, AWS, GCP, or Hetzner
   - Recommended: 2 vCPU, 4GB RAM, 100GB SSD

2. **Setup Monitoring**
   - Prometheus + Grafana
   - Alerting for downtime

3. **Create Faucet**
   - Allow users to request testnet tokens
   - Rate-limited REST API

4. **Block Explorer**
   - Deploy Ping.pub or Mintscan
   - Point to your RPC endpoint

5. **Documentation**
   - Testnet parameters
   - How to become a validator
   - How to use the chain

6. **Community Engagement**
   - Announce testnet launch
   - Invite validators
   - Run bug bounty program

---

## 📝 Manual Genesis Creation (If gentx Fails)

If the `gentx` command fails due to SDK v0.50.5 limitations:

```bash
# 1. Create validator key
sultand keys add validator --keyring-backend file
# Save address (sultan...)

# 2. Get consensus pubkey from priv_validator_key.json
cat ~/.sultan/config/priv_validator_key.json

# 3. Manually edit genesis.json
# Add account to app_state.auth.accounts
# Add balance to app_state.bank.balances
# Add validator to app_state.staking.validators
# Add delegation to app_state.staking.delegations
# Add bonded pool balance
# Add validator to CometBFT validators array

# See SUCCESS.md for complete structure
```

---

**Ready to deploy? Start with Option 1 on a VPS and you'll have a public testnet in minutes!**
