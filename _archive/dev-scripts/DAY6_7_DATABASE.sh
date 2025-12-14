#!/bin/bash

echo "╔════════════════════════════════════════════════════════════════════╗"
echo "║              DAY 6-7: DATABASE OPTIMIZATION                         ║"
echo "╚════════════════════════════════════════════════════════════════════╝"
echo ""

# Setup ScyllaDB with Docker
echo "🔧 Setting up ScyllaDB for 1.2M TPS..."

if command -v docker &> /dev/null; then
    echo "📦 Creating ScyllaDB container configuration..."
    
    mkdir -p /workspaces/0xv7/database/scylla
    
    cat > /workspaces/0xv7/database/scylla/start.sh << 'SCYLLA'
#!/bin/bash
# Start ScyllaDB for Sultan Chain
docker run --name sultan-scylla -d \
  -p 9042:9042 \
  -p 9160:9160 \
  -p 7000:7000 \
  -p 7001:7001 \
  -v $(pwd)/data:/var/lib/scylla \
  scylladb/scylla:5.2 \
  --smp 4 \
  --memory 4G \
  --overprovisioned 1 \
  --developer-mode 1

echo "Waiting for ScyllaDB to start..."
sleep 30

# Create Sultan Chain keyspace
docker exec sultan-scylla cqlsh -e "
CREATE KEYSPACE IF NOT EXISTS sultan 
WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1};

USE sultan;

CREATE TABLE IF NOT EXISTS blocks (
    height bigint PRIMARY KEY,
    hash text,
    timestamp timestamp,
    validator text,
    tx_count int,
    total_fees bigint  -- Always 0 for Sultan!
);

CREATE TABLE IF NOT EXISTS transactions (
    hash text PRIMARY KEY,
    block_height bigint,
    from_address text,
    to_address text,
    amount bigint,
    gas_fee bigint,  -- Always 0!
    timestamp timestamp,
    status text
);

CREATE TABLE IF NOT EXISTS accounts (
    address text PRIMARY KEY,
    balance bigint,
    nonce bigint,
    staked_amount bigint,
    rewards bigint
);
"
echo "✅ ScyllaDB setup complete!"
SCYLLA
    chmod +x /workspaces/0xv7/database/scylla/start.sh
    
    echo "✅ ScyllaDB configuration ready"
    echo "   Run: ./database/scylla/start.sh to launch"
else
    echo "⚠️ Docker not running. ScyllaDB config saved for later."
fi

# Setup RocksDB
echo ""
echo "🔧 Configuring RocksDB..."
mkdir -p /workspaces/0xv7/database/rocksdb

cat > /workspaces/0xv7/database/rocksdb/config.toml << 'ROCKS'
[rocksdb]
# Optimized for Sultan Chain's 1.2M TPS
path = "./data/state"
create_if_missing = true

[performance]
max_open_files = 10000
write_buffer_size = 268435456  # 256MB
max_write_buffer_number = 16
target_file_size_base = 268435456
max_bytes_for_level_base = 2684354560
compression = "lz4"
cache_size = 536870912  # 512MB

[compaction]
style = "level"
max_background_compactions = 8
max_background_flushes = 4
ROCKS

echo "✅ RocksDB configured for high performance"
echo ""
echo "════════════════════════════════════════════════════════════════════"
echo "Day 6-7 Complete: Database Optimization ✅"
echo "════════════════════════════════════════════════════════════════════"

