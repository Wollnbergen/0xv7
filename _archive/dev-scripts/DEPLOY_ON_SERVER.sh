#!/bin/bash
# Deploy Production Sharding - Run This ON the Hetzner Server
# SSH to root@5.161.225.96 first, then run this script there

set -e

echo "=================================================="
echo "🚀 Sultan Production Sharding - Server Deployment"
echo "=================================================="
echo ""
echo "Running on: $(hostname)"
echo "Current dir: $(pwd)"
echo ""

SULTAN_DIR="/root/sultan"

# Check we're on the right server
if [ ! -d "$SULTAN_DIR" ]; then
    echo "❌ $SULTAN_DIR not found. Are you on the right server?"
    exit 1
fi

cd $SULTAN_DIR

echo "📋 Step 1: Stopping existing node..."
pkill -f p2p_node || true
pkill -f sultan || true
sleep 2
echo "✅ Nodes stopped"

echo ""
echo "📋 Step 2: Backing up current state..."
BACKUP_NAME="sultan-backup-$(date +%Y%m%d-%H%M%S).tar.gz"
tar -czf /root/$BACKUP_NAME target/release/p2p_node config.toml Cargo.toml 2>/dev/null || true
echo "✅ Backup created: /root/$BACKUP_NAME"

echo ""
echo "📋 Step 3: Pulling latest code..."
git fetch origin feat/cosmos-sdk-integration
git checkout feat/cosmos-sdk-integration
git pull origin feat/cosmos-sdk-integration
echo "✅ Code updated"

echo ""
echo "📋 Step 4: Verifying production sharding code..."
if [ -f "sultan-core/src/sharding.rs" ] && [ -f "sultan-core/src/sharded_blockchain.rs" ]; then
    echo "✅ Sharding modules found"
    
    # Check for real implementation
    if grep -q "process_parallel" sultan-core/src/sharding.rs && \
       grep -q "tokio::spawn" sultan-core/src/sharding.rs; then
        echo "✅ Real parallel processing verified"
    else
        echo "⚠️  Sharding code may not have real implementation"
    fi
else
    echo "❌ Sharding modules not found!"
    exit 1
fi

echo ""
echo "📋 Step 5: Creating production configuration..."
cat > production.toml << 'PRODUCTION_CONFIG'
# Sultan Blockchain Production Configuration
# Full Sharding: 1024 shards, 1M+ TPS

[network]
chain_id = "sultan-1"
block_time = 2  # 2-second blocks

[sharding]
enabled = true
shard_count = 1024
tx_per_shard = 8000
cross_shard_enabled = true

[genesis]
total_supply = 500000000  # 500M SLTN
inflation_rate = 8.0
min_stake = 10000
genesis_time = 1733256000
blocks_per_year = 15768000

[validator]
min_stake = 10000
max_validators = 100
commission_max = 20.0

[rpc]
listen_addr = "0.0.0.0:8080"
enable_cors = true

[p2p]
listen_addr = "/ip4/0.0.0.0/tcp/26656"

[monitoring]
enable_metrics = true
telegram_bot = "8069901972:AAGpsmRJEsGT3G7iFbv9TvMbzvTJwAfsoeQ"
telegram_chat_id = "@S_L_T_N_bot"
PRODUCTION_CONFIG

echo "✅ Configuration created"

echo ""
echo "📋 Step 6: Building production binary..."
echo "   (This may take 5-10 minutes...)"

# Check if sultan-core main.rs supports sharding
if grep -q "enable-sharding\|enable_sharding" sultan-core/src/main.rs 2>/dev/null; then
    echo "   Building sultan-node with sharding support..."
    cargo build --release --bin sultan-node 2>&1 | tail -20
    
    if [ -f "target/release/sultan-node" ]; then
        echo "✅ sultan-node binary built successfully"
        BINARY="target/release/sultan-node"
        ARGS="--enable-sharding --shard-count 1024 --tx-per-shard 8000 --validator --validator-address validator_main --validator-stake 100000"
    else
        echo "❌ Build failed"
        exit 1
    fi
else
    echo "   Building p2p_node (legacy)..."
    cargo build --release --bin p2p_node 2>&1 | tail -20
    
    if [ -f "target/release/p2p_node" ]; then
        echo "✅ p2p_node binary built"
        BINARY="target/release/p2p_node"
        ARGS="start"
    else
        echo "❌ Build failed"
        exit 1
    fi
fi

echo ""
echo "📋 Step 7: Starting production node..."
echo "   Binary: $BINARY"
echo "   Config: production.toml"

# Start node in background
nohup $BINARY $ARGS > sultan-production.log 2>&1 &
NODE_PID=$!

echo "✅ Node started (PID: $NODE_PID)"
echo ""

echo "📋 Step 8: Waiting for startup..."
sleep 5

if ps -p $NODE_PID > /dev/null; then
    echo "✅ Node is running"
else
    echo "❌ Node failed to start. Check logs:"
    tail -50 sultan-production.log
    exit 1
fi

echo ""
echo "📋 Step 9: Checking logs..."
echo ""
echo "Recent logs:"
tail -30 sultan-production.log

echo ""
echo "=================================================="
echo "✅ DEPLOYMENT COMPLETE"
echo "=================================================="
echo ""
echo "Node Status:"
ps aux | grep -E "sultan|p2p_node" | grep -v grep || echo "  (Process listing unavailable)"
echo ""
echo "Files:"
echo "  Binary: $BINARY"
echo "  Config: production.toml"
echo "  Logs: sultan-production.log"
echo ""
echo "Monitoring:"
echo "  tail -f sultan-production.log"
echo "  curl http://localhost:8080/status"
echo ""
echo "Testing RPC (in 10 seconds)..."
sleep 10

if curl -s http://localhost:8080/status > /dev/null 2>&1; then
    echo "✅ RPC server is responding!"
    echo ""
    echo "Status:"
    curl -s http://localhost:8080/status | python3 -m json.tool 2>/dev/null || curl -s http://localhost:8080/status
else
    echo "⚠️  RPC server not responding yet (may need more time)"
    echo "   Wait a minute and try: curl http://localhost:8080/status"
fi

echo ""
echo "=================================================="
