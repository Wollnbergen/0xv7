# 🌉 Sultan L1 - Bridge Deployment Guide

**Production deployment guide for all cross-chain bridge services**

---

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Quick Start](#quick-start)
4. [Detailed Deployment](#detailed-deployment)
5. [IBC Relayer Setup](#ibc-relayer-setup)
6. [Monitoring & Alerts](#monitoring--alerts)
7. [Troubleshooting](#troubleshooting)
8. [Production Checklist](#production-checklist)

---

## Overview

This guide covers deployment of:
- ✅ **Bitcoin Bridge** (HTLC + SPV)
- ✅ **Ethereum Bridge** (Light Client + ZK Proofs)
- ✅ **Solana Bridge** (gRPC Streaming)
- ✅ **TON Bridge** (Smart Contracts)
- ✅ **IBC Relayer** (Hermes for 100+ Cosmos chains)
- ✅ **Monitoring** (Prometheus + Grafana)

---

## Prerequisites

### System Requirements

**Minimum (Development):**
- CPU: 4 cores
- RAM: 8GB
- Disk: 100GB SSD
- Network: 100 Mbps

**Recommended (Production):**
- CPU: 8 cores
- RAM: 32GB
- Disk: 500GB NVMe SSD
- Network: 1 Gbps

### Software Requirements

```bash
# Docker & Docker Compose
docker --version  # >= 20.10
docker-compose --version  # >= 1.29

# Optional but recommended
grpc_health_probe  # For gRPC health checks
jq  # For JSON parsing
curl  # For API testing
```

### Installation

```bash
# Install Docker (Ubuntu/Debian)
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# Install grpc_health_probe
wget https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/v0.4.19/grpc_health_probe-linux-amd64
sudo mv grpc_health_probe-linux-amd64 /usr/local/bin/grpc_health_probe
sudo chmod +x /usr/local/bin/grpc_health_probe
```

---

## Quick Start

### 1. Clone Repository

```bash
git clone https://github.com/Wollnbergen/0xv7.git
cd 0xv7/deploy/bridges
```

### 2. Configure Environment

```bash
# Copy example environment file
cp .env.example .env

# Edit configuration
nano .env
```

Example `.env`:
```bash
# Blockchain RPC Endpoints
BITCOIN_RPC_URL=https://blockstream.info/api
ETHEREUM_RPC_URL=https://eth.llamarpc.com
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
TON_ENDPOINT=https://toncenter.com/api/v2/jsonRPC

# Monitoring
GRAFANA_PASSWORD=secure_password_here

# IBC Relayer Mnemonics (24-word phrases)
SULTAN_MNEMONIC="word1 word2 ... word24"
OSMOSIS_MNEMONIC="word1 word2 ... word24"
COSMOSHUB_MNEMONIC="word1 word2 ... word24"
```

### 3. Deploy All Services

```bash
# Make deployment script executable
chmod +x deploy-bridges.sh

# Run deployment
./deploy-bridges.sh production
```

### 4. Verify Deployment

```bash
# Check all services are running
docker-compose ps

# Run health checks
chmod +x health-check.sh
./health-check.sh

# Test bridge endpoints
curl http://localhost:26657/bridges | jq
```

---

## Detailed Deployment

### Step 1: Build Docker Images

```bash
cd deploy/bridges

# Build all images
docker-compose build --parallel

# Or build individual services
docker-compose build bitcoin-service
docker-compose build ethereum-service
docker-compose build solana-service
docker-compose build ton-service
```

### Step 2: Start Core Services

```bash
# Start Sultan node first
docker-compose up -d sultan-node

# Wait for node to be ready
sleep 10
curl http://localhost:26657/status

# Start bridge services
docker-compose up -d bitcoin-service
docker-compose up -d ethereum-service
docker-compose up -d solana-service
docker-compose up -d ton-service
```

### Step 3: Start IBC Relayer

```bash
# Start relayer container
docker-compose up -d ibc-relayer

# Configure relayer (after setting mnemonics in .env)
./setup-ibc-relayer.sh
```

### Step 4: Start Monitoring

```bash
# Start Prometheus and Grafana
docker-compose up -d prometheus grafana

# Access Grafana
open http://localhost:3002
# Login: admin / (password from .env)
```

---

## IBC Relayer Setup

### Hermes Configuration

The IBC relayer connects Sultan L1 to the Cosmos ecosystem. Configuration is in `ibc-config/config.toml`.

**Connected Chains:**
- Osmosis (DEX)
- Cosmos Hub
- Juno (Smart Contracts)
- Stargaze (NFTs)
- Akash (Cloud Computing)
- ...and 95+ more

### Creating IBC Channels

```bash
# Access relayer container
docker exec -it sultan-ibc-relayer bash

# Create client
hermes create client --host-chain sultan-1 --reference-chain osmosis-1

# Create connection
hermes create connection --a-chain sultan-1 --b-chain osmosis-1

# Create transfer channel
hermes create channel \
  --a-chain sultan-1 \
  --a-connection connection-0 \
  --a-port transfer \
  --b-port transfer

# Start relaying
hermes start
```

### Verifying IBC Connections

```bash
# List all channels
hermes query channels --chain sultan-1

# Check connection status
hermes query connection --chain sultan-1 --connection connection-0

# View relayer logs
docker logs -f sultan-ibc-relayer
```

---

## Monitoring & Alerts

### Prometheus Metrics

**Bridge Metrics:**
- `bridge_transactions_total` - Total bridge transactions
- `bridge_transaction_failures_total` - Failed transactions
- `bridge_confirmation_time_seconds` - Average confirmation time
- `bitcoin_bridge_htlc_active` - Active Bitcoin HTLCs
- `ethereum_light_client_blocks_behind` - Light client sync status
- `solana_bridge_slot_lag` - Solana slot lag
- `hermes_packets_pending` - Pending IBC packets

**Access Prometheus:**
```bash
open http://localhost:9090

# Example queries
bridge_transactions_total
rate(bridge_transactions_total[5m])
sum by (chain) (bridge_transactions_total)
```

### Grafana Dashboards

**Access Grafana:**
```bash
open http://localhost:3002
Login: admin / (your password)
```

**Pre-configured Dashboards:**
1. **Bridge Overview** - All bridge statistics
2. **Bitcoin Bridge** - HTLC and SPV metrics
3. **Ethereum Bridge** - Light client sync
4. **Solana Bridge** - Streaming metrics
5. **IBC Relayer** - Channel and packet metrics

### Alert Configuration

Alerts are defined in:
- `prometheus/alerts.yml` - General node alerts
- `prometheus/bridge-alerts.yml` - Bridge-specific alerts

**Critical Alerts:**
- Bridge service down
- High transaction failure rate
- IBC relayer down
- Light client out of sync

**Warning Alerts:**
- High confirmation latency
- Packet backlog
- Validation errors

---

## Service Management

### Docker Compose Commands

```bash
# View all services
docker-compose ps

# View logs
docker-compose logs -f [service-name]
docker-compose logs -f bitcoin-service
docker-compose logs -f ibc-relayer

# Restart service
docker-compose restart bitcoin-service

# Stop all services
docker-compose down

# Stop and remove volumes
docker-compose down -v

# Update service
docker-compose pull [service-name]
docker-compose up -d [service-name]
```

### Individual Service Control

```bash
# Bitcoin Bridge
docker stop sultan-bitcoin-bridge
docker start sultan-bitcoin-bridge
docker logs -f sultan-bitcoin-bridge

# Ethereum Bridge
docker stop sultan-ethereum-bridge
docker start sultan-ethereum-bridge

# Solana Bridge
docker stop sultan-solana-bridge
docker start sultan-solana-bridge

# TON Bridge
docker stop sultan-ton-bridge
docker start sultan-ton-bridge

# IBC Relayer
docker stop sultan-ibc-relayer
docker start sultan-ibc-relayer
```

---

## Troubleshooting

### Common Issues

#### 1. Bridge Service Won't Start

```bash
# Check logs
docker logs sultan-bitcoin-bridge

# Common fixes:
- Verify RPC endpoints in .env
- Check port conflicts (9001, 50051, 50052, 9004)
- Ensure sufficient disk space
```

#### 2. IBC Relayer Connection Issues

```bash
# Verify chain connectivity
docker exec sultan-ibc-relayer hermes health-check

# Check config
docker exec sultan-ibc-relayer cat /root/.hermes/config.toml

# Restart relayer
docker-compose restart ibc-relayer
```

#### 3. High Transaction Failure Rate

```bash
# Check bridge logs
docker-compose logs bitcoin-service | grep ERROR
docker-compose logs ethereum-service | grep ERROR

# Verify blockchain node sync
curl http://localhost:26657/status | jq '.sync_info'

# Check source chain confirmations
```

#### 4. Prometheus Not Scraping Metrics

```bash
# Verify Prometheus config
docker exec sultan-prometheus cat /etc/prometheus/prometheus.yml

# Check targets
curl http://localhost:9090/api/v1/targets

# Restart Prometheus
docker-compose restart prometheus
```

### Health Checks

```bash
# Run automated health check
./health-check.sh

# Manual checks
curl http://localhost:9001/health  # Bitcoin
grpc_health_probe -addr=localhost:50051  # Ethereum
grpc_health_probe -addr=localhost:50052  # Solana
curl http://localhost:9004/health  # TON
curl http://localhost:3000/status  # IBC Relayer
curl http://localhost:26657/status  # Sultan Node
```

---

## Production Checklist

### Security

- [ ] Firewall configured (allow only necessary ports)
- [ ] TLS/SSL certificates installed
- [ ] Relayer mnemonics securely stored
- [ ] Bridge validator keys in hardware wallet/HSM
- [ ] Rate limiting enabled on public endpoints
- [ ] DDoS protection configured
- [ ] Regular security audits scheduled

### Monitoring

- [ ] Prometheus configured and running
- [ ] Grafana dashboards imported
- [ ] Alert rules configured
- [ ] PagerDuty/Slack integration setup
- [ ] Log aggregation configured (ELK/Loki)
- [ ] Uptime monitoring enabled

### Performance

- [ ] SSD/NVMe storage used
- [ ] Sufficient RAM allocated
- [ ] Network bandwidth adequate
- [ ] Load balancing configured (if needed)
- [ ] CDN setup for RPC endpoints
- [ ] Database optimization complete

### Backup & Recovery

- [ ] Automated backup scripts configured
- [ ] Disaster recovery plan documented
- [ ] Regular backup testing
- [ ] Off-site backup storage
- [ ] Recovery time objectives defined

### Operations

- [ ] Runbooks created for common issues
- [ ] On-call rotation established
- [ ] Incident response procedures defined
- [ ] Change management process in place
- [ ] Regular update schedule established

---

## Service Endpoints

### Production URLs

```
Bitcoin Bridge:    https://bitcoin-bridge.sultanl1.com
Ethereum Bridge:   https://ethereum-bridge.sultanl1.com
Solana Bridge:     https://solana-bridge.sultanl1.com
TON Bridge:        https://ton-bridge.sultanl1.com
IBC Relayer API:   https://ibc.sultanl1.com
Sultan Node RPC:   https://rpc.sultanl1.com
Prometheus:        https://metrics.sultanl1.com
Grafana:           https://grafana.sultanl1.com
```

### Local Development

```
Bitcoin Bridge:    http://localhost:9001
Ethereum Bridge:   grpc://localhost:50051
Solana Bridge:     grpc://localhost:50052
TON Bridge:        http://localhost:9004
IBC Relayer API:   http://localhost:3000
Sultan Node RPC:   http://localhost:26657
Prometheus:        http://localhost:9090
Grafana:           http://localhost:3002
```

---

## Scaling Considerations

### Horizontal Scaling

For high-traffic production:

```yaml
# docker-compose.yml
bitcoin-service:
  deploy:
    replicas: 3
  
ethereum-service:
  deploy:
    replicas: 3

# Add load balancer
nginx:
  image: nginx:latest
  volumes:
    - ./nginx.conf:/etc/nginx/nginx.conf
  ports:
    - "80:80"
    - "443:443"
```

### Vertical Scaling

```yaml
# Increase resources per service
bitcoin-service:
  deploy:
    resources:
      limits:
        cpus: '2.0'
        memory: 4G
      reservations:
        cpus: '1.0'
        memory: 2G
```

---

## Support & Resources

- **Documentation:** `INTEROPERABILITY_STATUS.md`
- **GitHub:** https://github.com/Wollnbergen/0xv7
- **Discord:** (link to community)
- **Email:** support@sultanl1.com

---

## Changelog

- **2025-11-23:** Initial production deployment guide
- Bridge services: v1.0.0
- IBC Relayer: Hermes v1.7.4
- Monitoring: Prometheus + Grafana

---

**Last Updated:** November 23, 2025  
**Status:** Production Ready  
**Maintainers:** Sultan L1 Core Team
