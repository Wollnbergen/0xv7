# Sultan L1 Node v0.1.4

Official validator node binary for Sultan L1 blockchain.

## Quick Install

```bash
curl -L https://wallet.sltn.io/install.sh | bash
```

Or download directly:
```bash
wget https://github.com/SultanL1/sultan-node/releases/download/v0.1.4/sultan-node
chmod +x sultan-node
```

## What's New in v0.1.4

- **Persistent Node Keys**: PeerId now survives restarts (saved to `data_dir/node_key.bin`)
- **P2P Validator Sync**: Validators automatically discover each other via gossipsub
- **ValidatorSetRequest/Response**: New nodes can sync the full validator set on startup
- **Periodic Re-announcement**: Validators re-announce every 60s for reliability

## Features

- **Zero Gas Fees**: All transactions are gas-free for end users
- **P2P Networking**: libp2p with gossipsub, Kademlia DHT, and noise encryption
- **Ed25519 Cryptography**: Fast, secure digital signatures
- **High Performance**: Native Rust implementation, 64,000+ TPS with 16 shards
- **Dynamic Sharding**: Scales from 16 to 8,000 shards as needed
- **TokenFactory**: Create custom tokens with Ed25519 signature authorization
- **NativeDex**: Built-in AMM for token swaps
- **4% Annual Inflation**: Sustainable validator rewards

## Running a Validator

### Minimum Requirements
- 2 CPU cores
- 4 GB RAM
- 50 GB SSD
- Ubuntu 22.04+

### Start Command

```bash
./sultan-node \
  --name my-validator \
  --data-dir ./data \
  --validator \
  --validator-address my-validator \
  --validator-stake 10000000000000 \
  --enable-p2p \
  --bootstrap-peers '/ip4/206.189.224.142/tcp/26656/p2p/12D3KooWM9Pza4nMLHapDya6ghiMNL24RFU9VRg9krRbi5kLf5L7' \
  --enable-sharding \
  --shard-count 16 \
  --rpc-addr 0.0.0.0:26657 \
  --allowed-origins '*' \
  --genesis 'sultan19mzzrah6h27draqc5tkh49yj623qwuz5f5t64c:500000000000000000'
```

### Bootstrap Peer (Required)
```
/ip4/206.189.224.142/tcp/26656/p2p/12D3KooWM9Pza4nMLHapDya6ghiMNL24RFU9VRg9krRbi5kLf5L7
```

### Ports
- **26656**: P2P (libp2p) - must be open for peer connections
- **26657**: RPC (HTTP/JSON) - for API queries

## Staking

- **APY**: ~13.33% (from 4% inflation, 30% validator share)
- **Minimum Stake**: 10,000,000,000,000 base units (10,000 SLTN)
- **Token**: SLTN (9 decimals)
- **Address Prefix**: sultan1

## Network Status

Check current status:
```bash
curl https://rpc.sltn.io/status
```

View validators:
```bash
curl https://rpc.sltn.io/staking/validators
```

## RPC Endpoints

- **Mainnet RPC**: https://rpc.sltn.io
- **Block Explorer**: https://x.sltn.io
- **Wallet**: https://wallet.sltn.io

## Documentation

- [VALIDATOR_GUIDE.md](VALIDATOR_GUIDE.md) - Detailed setup instructions
- [API Reference](https://rpc.sltn.io) - RPC endpoints

## License

Apache-2.0
