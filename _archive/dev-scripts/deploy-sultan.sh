#!/bin/bash
set -e

cat << 'EOF'
╔══════════════════════════════════════════════════════════════════╗
║                                                                  ║
║         SULTAN L1 - PRODUCTION DEPLOYMENT SCRIPT                 ║
║                                                                  ║
╚══════════════════════════════════════════════════════════════════╝

Preparing Sultan L1 for deployment...
EOF

cd /workspaces/0xv7

# Step 1: Build the Rust libraries
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 1: Building Rust Libraries"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
cargo build --release -p sultan-core -p sultan-cosmos-bridge

# Step 2: Set up library path
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 2: Setting up Library Paths"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
export LD_LIBRARY_PATH=$PWD/target/release:$LD_LIBRARY_PATH
echo "LD_LIBRARY_PATH=$LD_LIBRARY_PATH"

# Step 3: Verify binary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 3: Verifying Sultan Binary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
ls -lh sultand/sultand
file sultand/sultand

# Step 4: Create deployment package
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 4: Creating Deployment Package"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Strip binary for production
cp sultand/sultand sultand/sultand.production
strip --strip-all sultand/sultand.production
echo "✓ Stripped binary: $(ls -lh sultand/sultand.production | awk '{print $5}')"

# Create deployment tarball
tar czf sultan-l1-v1.0.0.tar.gz \
  sultand/sultand.production \
  target/release/libsultan_cosmos_bridge.so \
  target/release/libsultan_core.a \
  scripts/validate_memory.sh \
  scripts/benchmark_performance.sh \
  scripts/stress_test.sh \
  PHASE6_PRODUCTION_GUIDE.md \
  PHASE6_SECURITY_AUDIT.md \
  keplr-chain-config.json \
  chain-registry.json \
  wallet-integration.html \
  ROADMAP.md

echo "✓ Created: sultan-l1-v1.0.0.tar.gz ($(ls -lh sultan-l1-v1.0.0.tar.gz | awk '{print $5}'))"

# Step 5: Create deployment instructions
cat > DEPLOYMENT_INSTRUCTIONS.md << 'DEPLOY_EOF'
# Sultan L1 - Deployment Instructions

## Package Contents
- `sultand.production` - Sultan L1 binary (stripped, ~45MB)
- `libsultan_cosmos_bridge.so` - FFI bridge library
- `libsultan_core.a` - Sultan core static library
- Scripts for validation, benchmarking, and stress testing
- Complete documentation

## Quick Start

### 1. Extract Package
```bash
tar xzf sultan-l1-v1.0.0.tar.gz -C /opt/sultan/
cd /opt/sultan
```

### 2. Set Library Path
```bash
export LD_LIBRARY_PATH=/opt/sultan/target/release:$LD_LIBRARY_PATH
```

### 3. Initialize Node
```bash
./sultand/sultand.production init mynode --chain-id sultan-1
```

### 4. Start Node
```bash
./sultand/sultand.production start \
  --api.enable=true \
  --api.swagger=true \
  --log_level=info
```

## Production Deployment

### Systemd Service
Create `/etc/systemd/system/sultand.service`:

```ini
[Unit]
Description=Sultan L1 Blockchain Node
After=network.target

[Service]
Type=simple
User=sultan
WorkingDirectory=/opt/sultan
Environment="LD_LIBRARY_PATH=/opt/sultan/target/release"
ExecStart=/opt/sultan/sultand/sultand.production start --api.enable=true
Restart=on-failure
RestartSec=10
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable sultand
sudo systemctl start sultand
sudo systemctl status sultand
```

### Monitoring
- Prometheus metrics: `http://localhost:26660/metrics`
- API endpoints: `http://localhost:1317`
- RPC endpoints: `http://localhost:26657`
- Swagger UI: `http://localhost:1317/swagger/`

See `PHASE6_PRODUCTION_GUIDE.md` for complete deployment documentation.

## Security Checklist
- [ ] Firewall configured (allow ports: 26656, 26657, 1317)
- [ ] SSL/TLS certificates installed
- [ ] Secrets management configured
- [ ] Backup procedures tested
- [ ] Monitoring alerts configured

## Support
- Security Audit: See `PHASE6_SECURITY_AUDIT.md`
- Roadmap: See `ROADMAP.md`
- GitHub: https://github.com/Wollnbergen/0xv7
DEPLOY_EOF

echo "✓ Created: DEPLOYMENT_INSTRUCTIONS.md"

# Step 6: Summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ DEPLOYMENT PACKAGE READY!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Package: sultan-l1-v1.0.0.tar.gz"
echo "Size: $(ls -lh sultan-l1-v1.0.0.tar.gz | awk '{print $5}')"
echo ""
echo "Deploy to server:"
echo "  scp sultan-l1-v1.0.0.tar.gz user@your-server:/opt/sultan/"
echo ""
echo "Or test locally:"
echo "  export LD_LIBRARY_PATH=$PWD/target/release:\$LD_LIBRARY_PATH"
echo "  ./sultand/sultand init testnode --chain-id sultan-test"
echo "  ./sultand/sultand start --api.enable=true"
echo ""
echo "📖 See DEPLOYMENT_INSTRUCTIONS.md for complete setup guide"
echo ""
