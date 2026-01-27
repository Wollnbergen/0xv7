# Sultan L1 Genesis Relaunch v0.2.0

**Date:** January 17, 2026  
**Version:** v0.2.0 (DeFi Hub + Fee Split + Reward Wallet)

## 🎯 Why Relaunch?

1. **Two chains diverged** - 5 nodes at height ~7865, 1 node (NYC) at ~2759
2. **New features need deployment:**
   - `reward_wallet` field for validator APY → genesis wallet
   - Token Factory improvements
   - Native DEX
   - Fee splitting for liquidity providers
3. **Clean state** for proper validator registration

## 📋 Pre-Relaunch Checklist

### Build New Binary
```bash
cd /workspaces/0xv7
cargo build --release -p sultan-core
# Binary: target/release/sultan-node
```

### Genesis Configuration
```
Genesis Wallet: sultan15g5nwnlemn7zt6rtl7ch46ssvx2ym2v2umm07g
Genesis Balance: 500,000,000,000,000 (500M SLTN with 6 decimals)
Block Time: 2 seconds
Shards: 16 (expandable to 64)
Inflation: 4% annually
Validator APY: ~13.33% (at 30% staked)
```

### Validator Nodes (6 Genesis Validators)
| Name | IP | Location | Stake |
|------|-----|----------|-------|
| sultan-nyc | 206.189.224.142 | New York (RPC) | 10M SLTN |
| sultan-sfo | 143.198.205.21 | San Francisco | 10M SLTN |
| sultan-fra | 142.93.238.33 | Frankfurt | 10M SLTN |
| sultan-ams | 46.101.122.13 | Amsterdam | 10M SLTN |
| sultan-sgp | 24.144.94.23 | Singapore | 10M SLTN |
| sultan-lon | 134.122.96.36 | London | 10M SLTN |

**Total Staked:** 60M SLTN (12% of supply)  
**Validator APY at launch:** ~33% (4% / 0.12)

## 🚀 Relaunch Steps

### Phase 1: Stop All Nodes
```bash
# Run on each validator server
for ip in 206.189.224.142 143.198.205.21 142.93.238.33 46.101.122.13 24.144.94.23 134.122.96.36; do
  ssh root@$ip 'systemctl stop sultan-node && rm -rf /var/lib/sultan/*'
done
```

### Phase 2: Deploy New Binary
```bash
# Build locally
cargo build --release -p sultan-core

# Deploy to all validators
for ip in 206.189.224.142 143.198.205.21 142.93.238.33 46.101.122.13 24.144.94.23 134.122.96.36; do
  scp target/release/sultan-node root@$ip:/root/sultan-node
  ssh root@$ip 'chmod +x /root/sultan-node'
done
```

### Phase 3: Update Systemd Service (with CORS fix)
```bash
# Template for each validator (adjust --name and IP)
cat > /etc/systemd/system/sultan-node.service << 'EOF'
[Unit]
Description=Sultan L1 Blockchain Node v0.2.0
After=network.target

[Service]
Type=simple
ExecStart=/root/sultan-node \
    --name sultan-nyc \
    --data-dir /var/lib/sultan \
    --block-time 2 \
    --validator \
    --validator-stake 10000000000000 \
    --p2p-addr /ip4/0.0.0.0/tcp/26656 \
    --rpc-addr 0.0.0.0:8545 \
    --enable-p2p \
    --bootstrap-peers /ip4/143.198.205.21/tcp/26656,/ip4/142.93.238.33/tcp/26656,/ip4/46.101.122.13/tcp/26656,/ip4/24.144.94.23/tcp/26656,/ip4/134.122.96.36/tcp/26656 \
    --genesis sultan15g5nwnlemn7zt6rtl7ch46ssvx2ym2v2umm07g:500000000000000 \
    --enable-sharding \
    --shard-count 16 \
    --allowed-origins "*"
Restart=always
RestartSec=3
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable sultan-node
```

### Phase 4: Start Nodes (in order)
```bash
# Start NYC first (primary RPC)
ssh root@206.189.224.142 'systemctl start sultan-node'
sleep 10

# Start remaining validators
for ip in 143.198.205.21 142.93.238.33 46.101.122.13 24.144.94.23 134.122.96.36; do
  ssh root@$ip 'systemctl start sultan-node'
  sleep 5
done
```

### Phase 5: Verify Consensus
```bash
# All should show same height (within 1-2 blocks)
for ip in 206.189.224.142 143.198.205.21 142.93.238.33 46.101.122.13 24.144.94.23 134.122.96.36; do
  echo -n "$ip: "
  curl -s http://$ip:8545/status | jq '.height'
done
```

### Phase 6: Verify Staking & Rewards
```bash
# Check validators registered
curl -s https://rpc.sltn.io/staking/validators | jq '.[] | {address: .validator_address, stake: .total_stake, reward_wallet}'

# Check genesis wallet balance
curl -s "https://rpc.sltn.io/balance/sultan15g5nwnlemn7zt6rtl7ch46ssvx2ym2v2umm07g"

# After some blocks, check accumulated rewards
curl -s https://rpc.sltn.io/staking/validators | jq '.[] | {address: .validator_address, rewards: .rewards_accumulated}'
```

## 🔧 Post-Launch Tasks

### Update Website (sltn.io)
- Verify network stats display correctly
- Update any version numbers

### Update Wallet Extension
- Ensure using correct RPC endpoint (https://rpc.sltn.io)
- Test transactions

### Update Block Explorer  
- Point to fresh chain
- Verify block display

### Test Token Factory
```bash
# Create a test token
curl -X POST https://rpc.sltn.io/tokens/create \
  -H "Content-Type: application/json" \
  -d '{
    "creator": "sultan15g5nwnlemn7zt6rtl7ch46ssvx2ym2v2umm07g",
    "symbol": "TEST",
    "name": "Test Token",
    "decimals": 6,
    "initial_supply": 1000000000000
  }'
```

### Test Native DEX
```bash
# Create liquidity pool
curl -X POST https://rpc.sltn.io/dex/create_pool \
  -H "Content-Type: application/json" \
  -d '{
    "creator": "sultan15g5nwnlemn7zt6rtl7ch46ssvx2ym2v2umm07g",
    "token_a": "SLTN",
    "token_b": "TEST",
    "amount_a": 1000000000,
    "amount_b": 1000000000
  }'
```

## 📊 Expected Results After Relaunch

| Metric | Value |
|--------|-------|
| Block Height | Starting from 0 |
| Active Validators | 6 |
| Total Staked | 60,000,000 SLTN |
| Genesis Balance | 500,000,000 SLTN |
| Validator APY | ~33% |
| Block Time | 2 seconds |
| TPS Capacity | 64,000 (16 shards × 4K) |

## 🔒 Security Notes

- All validator reward wallets point to genesis address
- CORS configured to allow all origins (production should restrict)
- SSL via Let's Encrypt on rpc.sltn.io

## 📝 Version History

- **v0.1.0** - Initial launch (Christmas 2025)
- **v0.1.5** - P2P fixes, validator discovery
- **v0.1.6** - Enterprise-grade validator registration
- **v0.2.0** - DeFi Hub: Token Factory, Native DEX, Fee Split, Reward Wallet (THIS RELEASE)
