# Sultan Validator Guide

Become a validator on the Sultan Network and earn **~13.33% APY** (variable) in SLTN rewards.

## What is a Validator?

Validators are the backbone of the Sultan blockchain. They:
- Verify and process transactions
- Participate in consensus to produce blocks (2-second block time)
- Earn rewards for keeping the network secure
- Can receive delegated stake from other users

**All validators are equal** - every validator runs consensus and earns APY proportional to their stake. There's no distinction between "infrastructure" and "staking" validators.

## Two Ways to Become a Validator

### Option 1: Sultan Wallet (Recommended)

The easiest way to become a validator is through the Sultan Wallet PWA:

1. **Get 10,000+ SLTN** in your wallet
2. **Open Sultan Wallet** at [wallet.sltn.io](https://wallet.sltn.io)
3. **Navigate to Validators** → **Become a Validator**
4. **Enter your validator name** and stake amount
5. **Sign the transaction** with your wallet

Your validator is immediately active in the network!

### Option 2: Run Your Own Node

For full decentralization, run your own node infrastructure.

#### Requirements

### Hardware
| Component | Minimum | Recommended |
|-----------|---------|-------------|
| RAM | 1 GB | 2 GB |
| Storage | 20 GB SSD | 50 GB SSD |
| CPU | 1 vCPU | 2 vCPU |
| Network | 10 Mbps | 100 Mbps |

### Software
- **OS**: Linux (Ubuntu 24.04 LTS recommended)
- **Ports**: 26656 (P2P), 26657 (RPC)

### Stake
- Minimum stake: **10,000 SLTN**
- Higher stake = more block production opportunities = more rewards

## Quick Start (5 minutes)

### Step 1: Get a Server

Any Linux VPS works. Recommended providers:

| Provider | Price | Specs |
|----------|-------|-------|
| [DigitalOcean](https://digitalocean.com) | $6/mo | 1 vCPU, 1GB RAM, 25GB SSD |
| [Vultr](https://vultr.com) | $5/mo | 1 vCPU, 1GB RAM, 25GB SSD |
| [Hetzner](https://hetzner.com) | €4.51/mo | 2 vCPU, 4GB RAM, 40GB SSD |
| [Linode](https://linode.com) | $5/mo | 1 vCPU, 1GB RAM, 25GB SSD |

### Step 2: Download Sultan Node

SSH into your server and run:

```bash
# Download the latest binary
wget https://github.com/Wollnbergen/DOCS/releases/latest/download/sultan-node
chmod +x sultan-node
```

### Step 3: Open Firewall Ports

```bash
# Ubuntu/Debian
sudo ufw allow 26656/tcp  # P2P
sudo ufw allow 26657/tcp  # RPC
sudo ufw enable

# CentOS/RHEL
sudo firewall-cmd --permanent --add-port=26656/tcp
sudo firewall-cmd --permanent --add-port=26657/tcp
sudo firewall-cmd --reload
```

### Step 4: Start Your Validator

```bash
./sultan-node \
  --name "YourValidatorName" \
  --validator \
  --validator-address "YourValidatorName" \
  --validator-stake 10000 \
  --enable-sharding \
  --shard-count 16 \
  --enable-p2p \
  --bootstrap-peers /ip4/206.189.224.142/tcp/26656 \
  --rpc-addr 0.0.0.0:26657 \
  --p2p-addr /ip4/0.0.0.0/tcp/26656 \
  --data-dir ./sultan-data
```

**Important flags:**
- `--name`: Your validator display name
- `--validator`: Enable validator mode
- `--validator-stake`: Your stake amount (minimum 10,000)
- `--bootstrap-peers`: NYC bootstrap node IP
- `--enable-sharding`: Required for mainnet (16 shards)

### Step 5: Verify Connection

Your validator is working when you see:
```
✅ Sultan Chain is running!
📡 P2P: Connected to X peers
⛏️  Validator: YourValidatorName
💰 Stake: 10,000 SLTN
🔗 Participating in consensus
```

Check your RPC:
```bash
curl http://localhost:26657/status
```

## Running as a Service (Production)

For 24/7 operation, create a systemd service:

```bash
sudo tee /etc/systemd/system/sultan.service > /dev/null << 'EOF'
[Unit]
Description=Sultan Validator Node
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root
ExecStart=/root/sultan-node \
  --name "YourValidatorName" \
  --validator \
  --validator-address "YourValidatorName" \
  --validator-stake 10000 \
  --enable-sharding \
  --shard-count 16 \
  --enable-p2p \
  --bootstrap-peers /ip4/206.189.224.142/tcp/26656 \
  --rpc-addr 0.0.0.0:26657 \
  --p2p-addr /ip4/0.0.0.0/tcp/26656 \
  --data-dir /root/sultan-data
Restart=always
RestartSec=3
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable sultan
sudo systemctl start sultan
```

Monitor:
```bash
sudo systemctl status sultan
sudo journalctl -u sultan -f  # Live logs
```

## CLI Reference

| Flag | Default | Description |
|------|---------|-------------|
| `--name` | sultan-node-1 | Node display name |
| `--data-dir` | ./data | Data storage directory |
| `--block-time` | 2 | Block time in seconds |
| `--validator` | false | Enable validator mode |
| `--validator-address` | - | Validator identity name |
| `--validator-stake` | - | Stake amount (min 10,000) |
| `--p2p-addr` | /ip4/0.0.0.0/tcp/26656 | P2P listen address |
| `--rpc-addr` | 0.0.0.0:26657 | RPC listen address |
| `--enable-sharding` | false | Enable sharding |
| `--shard-count` | 8 | Initial shard count (mainnet: 16) |
| `--enable-p2p` | false | Enable P2P networking |
| `--bootstrap-peers` | - | Bootstrap peer multiaddr |

## Validator Earnings

> ⚠️ **Note**: APY is variable based on total network stake, validator uptime, and block production participation. The ~13.33% figure is an estimate based on current network parameters.

**Estimated earnings at current network conditions:**

| Stake Amount | Daily | Monthly | Yearly (~13.33% APY) |
|--------------|-------|---------|----------------------|
| 10,000 SLTN | ~3.65 SLTN | ~111 SLTN | ~1,333 SLTN |
| 50,000 SLTN | ~18.3 SLTN | ~556 SLTN | ~6,665 SLTN |
| 100,000 SLTN | ~36.5 SLTN | ~1,111 SLTN | ~13,330 SLTN |

**Factors affecting rewards:**
- Your stake weight vs total network stake
- Validator uptime (keep it online 24/7!)
- Block production participation rate
- Network inflation schedule

## Delegation

Other users can delegate their SLTN to your validator through the Sultan Wallet:

1. Delegators stake their tokens with your validator
2. Your validator earns commission on their rewards
3. Default commission: 5% (configurable)
4. Both you and delegators benefit from compound rewards

## Monitoring

**Check network status:**
```bash
curl -s https://rpc.sltn.io/status | jq
```

**View validator count:**
```bash
curl -s https://rpc.sltn.io/status | jq '.validator_count'
```

**Local health check:**
```bash
curl http://localhost:26657/status
```

## Troubleshooting

### "Connection refused" on startup
```bash
# Check if port is open
sudo ufw status
# Check if another process uses the port
sudo lsof -i :26656
sudo lsof -i :26657
```

### "No peers found"
- Verify internet connectivity: `ping 206.189.224.142`
- Check bootstrap peer is correct: `/ip4/206.189.224.142/tcp/26656`
- Ensure port 26656 is not blocked by firewall

### Node crashes on startup
```bash
# Check available memory
free -h
# Check disk space
df -h
# View crash logs
journalctl -u sultan --no-pager -n 100
```

### Validator not earning rewards
- Ensure stake meets minimum (10,000 SLTN)
- Verify node is connected to peers
- Check validator is actively participating in consensus
- Ensure uptime is high (aim for 99%+)

## FAQ

**Q: How do I get SLTN to stake?**  
A: Contact the Sultan team for genesis allocation or acquire through the network.

**Q: Can I run multiple validators?**  
A: Yes, each validator needs a unique name and separate server.

**Q: What happens if my validator goes offline?**  
A: You stop earning rewards while offline. Currently no slashing penalties.

**Q: How do I increase my stake?**  
A: Use the Sultan Wallet to delegate additional stake to your validator.

**Q: Where are my rewards sent?**  
A: Rewards accumulate in your validator account automatically.

**Q: Is the 13.33% APY guaranteed?**  
A: No, APY is variable based on network conditions. 13.33% is an estimate.

## Network Information

| Parameter | Value |
|-----------|-------|
| Chain ID | sultan-mainnet-1 |
| Block Time | 2 seconds |
| TPS Capacity | 64,000 (16 shards × 4,000 TPS) |
| Shard Count | 16 |
| RPC Endpoint | https://rpc.sltn.io |
| Wallet | https://wallet.sltn.io |

## Support

- **Website**: [sltn.io](https://sltn.io)
- **RPC Endpoint**: `https://rpc.sltn.io`
- **Explorer**: Coming soon

---

*Sultan Network - High-performance Layer 1 blockchain with 64K TPS and zero fees*
