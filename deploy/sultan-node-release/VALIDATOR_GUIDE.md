# Sultan L1 Validator Guide

Complete guide to running a Sultan L1 validator node.

## Prerequisites

- Ubuntu 22.04 LTS or later
- 2+ CPU cores
- 4+ GB RAM
- 50+ GB SSD storage
- Static public IP address
- Ports 26656 (P2P) and 26657 (RPC) open

## Installation

### Option 1: Quick Install Script

```bash
curl -L https://wallet.sltn.io/install.sh | bash
```

### Option 2: Manual Download

```bash
wget https://github.com/SultanL1/sultan-node/releases/download/v0.1.4/sultan-node
chmod +x sultan-node
sudo mv sultan-node /usr/local/bin/
```

## Configuration

### Create Data Directory

```bash
sudo mkdir -p /var/lib/sultan
```

### Create Systemd Service

```bash
sudo tee /etc/systemd/system/sultan.service > /dev/null << 'SVCEOF'
[Unit]
Description=Sultan L1 Validator Node
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root
ExecStart=/usr/local/bin/sultan-node --name YOUR_VALIDATOR_NAME --data-dir /var/lib/sultan --validator --validator-address YOUR_VALIDATOR_NAME --validator-stake 10000000000000 --enable-p2p --bootstrap-peers '/ip4/206.189.224.142/tcp/26656/p2p/12D3KooWM9Pza4nMLHapDya6ghiMNL24RFU9VRg9krRbi5kLf5L7' --enable-sharding --shard-count 16 --rpc-addr 0.0.0.0:26657 --allowed-origins '*' --genesis 'sultan19mzzrah6h27draqc5tkh49yj623qwuz5f5t64c:500000000000000000'
Restart=always
RestartSec=5
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
SVCEOF
```

Replace `YOUR_VALIDATOR_NAME` with a unique name for your validator (e.g., `my-validator-tokyo`).

### Enable and Start

```bash
sudo systemctl daemon-reload
sudo systemctl enable sultan
sudo systemctl start sultan
```

### Check Status

```bash
sudo systemctl status sultan
sudo journalctl -u sultan -f
```

## Command Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `--name` | Node name (displayed in logs) | `sultan-node` |
| `--data-dir` | Data directory path | `./data` |
| `--validator` | Enable validator mode | disabled |
| `--validator-address` | Validator address/name | required |
| `--validator-stake` | Stake amount in base units | required |
| `--enable-p2p` | Enable P2P networking | disabled |
| `--bootstrap-peers` | Bootstrap peer multiaddr | none |
| `--p2p-addr` | P2P listen address | `/ip4/0.0.0.0/tcp/26656` |
| `--rpc-addr` | RPC listen address | `0.0.0.0:26657` |
| `--allowed-origins` | CORS allowed origins | none |
| `--enable-sharding` | Enable sharding | disabled |
| `--shard-count` | Number of shards | `16` |
| `--genesis` | Genesis account(s) | required |
| `--block-time` | Block time in seconds | `2` |

## Network Information

### Bootstrap Peer

All validators must connect to the bootstrap peer:

```
/ip4/206.189.224.142/tcp/26656/p2p/12D3KooWM9Pza4nMLHapDya6ghiMNL24RFU9VRg9krRbi5kLf5L7
```

### Ports

| Port | Protocol | Purpose |
|------|----------|---------|
| 26656 | TCP | P2P networking (libp2p) |
| 26657 | TCP | RPC API (HTTP/JSON) |

### Firewall Configuration

```bash
sudo ufw allow 26656/tcp comment 'Sultan P2P'
sudo ufw allow 26657/tcp comment 'Sultan RPC'
```

## Staking Economics

- **APY**: ~13.33%
- **Inflation**: 4% annually
- **Validator Share**: 30% of inflation
- **Minimum Stake**: 10,000,000,000,000 base units (10,000 SLTN)
- **Block Time**: 2 seconds
- **Rewards**: Distributed per block

## P2P Networking

Sultan uses libp2p for peer-to-peer networking:

- **Gossipsub**: Block and transaction propagation
- **Kademlia DHT**: Peer discovery
- **Noise Protocol**: Encrypted connections
- **Yamux**: Stream multiplexing

### Persistent Node Identity

Your node generates a persistent identity key on first startup:
- Saved to: `<data-dir>/node_key.bin`
- Your PeerId remains stable across restarts
- Other nodes can reliably connect to you

### Validator Discovery

Validators automatically discover each other:
1. Connect to bootstrap peer
2. Request validator set from peers
3. Announce your validator to the network
4. Re-announce every 60 seconds

## Monitoring

### Check Node Status

```bash
curl http://localhost:26657/status
```

Response:
```json
{
  "height": 12345,
  "latest_hash": "...",
  "validator_count": 6,
  "pending_txs": 0,
  "sharding_enabled": true,
  "shard_count": 16,
  "inflation_rate": 0.04,
  "validator_apy": 0.1333
}
```

### View Validators

```bash
curl http://localhost:26657/staking/validators
```

### Check Logs

```bash
sudo journalctl -u sultan -f
sudo journalctl -u sultan -n 100
sudo journalctl -u sultan | grep -i peer
```

## Troubleshooting

### Node Won't Start

```bash
sudo lsof -i :26656
sudo lsof -i :26657
sudo pkill -9 sultan-node
sudo journalctl -u sultan -n 50
```

### No Peers Connecting

```bash
nc -zv 206.189.224.142 26656
sudo ufw status
```

### Node Crashes on Startup

```bash
sudo systemctl stop sultan
sudo rm -rf /var/lib/sultan/*
sudo systemctl start sultan
```

## Upgrading

```bash
sudo systemctl stop sultan
wget https://github.com/SultanL1/sultan-node/releases/latest/download/sultan-node
chmod +x sultan-node
sudo mv sultan-node /usr/local/bin/
sudo systemctl start sultan
```

Your node key is preserved, so your PeerId remains the same after upgrade.

## Support

- **RPC Endpoint**: https://rpc.sltn.io
- **Block Explorer**: https://x.sltn.io
- **Wallet**: https://wallet.sltn.io
- **Telegram**: https://t.me/Sultan_L1

## License

Apache-2.0
