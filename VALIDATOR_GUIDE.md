# Sultan Validator Guide

Become a validator on the Sultan Network and earn **13.33% APY** in SLTN rewards.

## What is a Validator?

Validators are the backbone of the Sultan blockchain. They:
- Verify and process transactions
- Participate in consensus to produce blocks
- Earn rewards for keeping the network secure

## Requirements

### Hardware
| Component | Minimum | Recommended |
|-----------|---------|-------------|
| RAM | 1 GB | 2 GB |
| Storage | 20 GB SSD | 50 GB SSD |
| CPU | 1 vCPU | 2 vCPU |
| Network | 10 Mbps | 100 Mbps |

### Software
- **OS**: Linux (Ubuntu 22.04+ recommended)
- **Port**: 26656 open for P2P connections

### Stake
- Minimum stake: **10,000 SLTN**
- Higher stake = more block production opportunities

## Quick Start (5 minutes)

### Step 1: Get a Server

Any Linux VPS works. Popular options:
- [DigitalOcean](https://digitalocean.com) - $6/month droplet
- [Vultr](https://vultr.com) - $5/month instance
- [Hetzner](https://hetzner.com) - €4/month cloud server
- [Linode](https://linode.com) - $5/month nanode

### Step 2: Download Sultan Node

SSH into your server and run:

```bash
wget https://github.com/Wollnbergen/DOCS/releases/latest/download/sultan-node
chmod +x sultan-node
```

### Step 3: Open Firewall Port

```bash
# Ubuntu/Debian
sudo ufw allow 26656/tcp
sudo ufw enable

# CentOS/RHEL
sudo firewall-cmd --permanent --add-port=26656/tcp
sudo firewall-cmd --reload
```

### Step 4: Run Your Validator

```bash
./sultan-node \
  --validator \
  --validator-address "YourValidatorName" \
  --validator-stake 10000 \
  --enable-p2p \
  --bootstrap-peers /dns4/rpc.sltn.io/tcp/26656
```

Replace `YourValidatorName` with your chosen name (no spaces, alphanumeric).

### Step 5: Verify Connection

Your validator is working when you see:
```
[INFO] Connected to X peers
[INFO] Validator active: YourValidatorName
[INFO] Participating in consensus
```

## Running as a Service (Recommended)

Keep your validator running 24/7 with systemd:

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
  --validator \
  --validator-address "YourValidatorName" \
  --validator-stake 10000 \
  --enable-p2p \
  --bootstrap-peers /dns4/rpc.sltn.io/tcp/26656
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable sultan
sudo systemctl start sultan
```

Check status:
```bash
sudo systemctl status sultan
sudo journalctl -u sultan -f  # View logs
```

## Validator Earnings

| Stake Amount | Daily Earnings | Monthly | Yearly (13.33% APY) |
|--------------|----------------|---------|---------------------|
| 10,000 SLTN | ~3.65 SLTN | ~111 SLTN | 1,333 SLTN |
| 50,000 SLTN | ~18.3 SLTN | ~556 SLTN | 6,665 SLTN |
| 100,000 SLTN | ~36.5 SLTN | ~1,111 SLTN | 13,330 SLTN |

Rewards are distributed based on:
- Your stake weight vs total network stake
- Uptime (keep your validator online!)
- Block production participation

## Monitoring Your Validator

Check if your validator is in the network:
```bash
curl -s https://rpc.sltn.io/status | jq '.validator_count'
```

View network stats:
```bash
curl -s https://rpc.sltn.io/status | jq
```

## Troubleshooting

### "Connection refused" on startup
- Ensure port 26656 is open: `sudo ufw status`
- Check if another process uses the port: `sudo lsof -i :26656`

### "No peers found"
- Verify internet connectivity
- Check bootstrap peer is correct: `/dns4/rpc.sltn.io/tcp/26656`

### Validator not earning rewards
- Ensure your stake meets minimum (10,000 SLTN)
- Check validator is connected to peers
- Verify uptime - rewards require consistent online presence

## FAQ

**Q: How do I get SLTN to stake?**
A: SLTN can be acquired through the network. Contact the Sultan team for genesis allocation or validator onboarding.

**Q: Can I run multiple validators?**
A: Yes, each validator needs a unique name and separate server.

**Q: What happens if my validator goes offline?**
A: You stop earning rewards while offline. There are no slashing penalties currently.

**Q: How do I increase my stake?**
A: Restart with a higher `--validator-stake` value.

**Q: Where are my rewards sent?**
A: Rewards accumulate in your validator account. Withdrawal features coming soon.

## Support

- Website: [sltn.io](https://sltn.io)
- RPC Endpoint: `https://rpc.sltn.io`

---

*Sultan Network - High-performance Layer 1 blockchain with 64K TPS*
