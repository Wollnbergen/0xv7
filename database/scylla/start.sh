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
