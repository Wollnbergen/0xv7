-- Create Sultan Chain mainnet keyspace
CREATE KEYSPACE IF NOT EXISTS sultan_mainnet 
WITH REPLICATION = {'class': 'SimpleStrategy', 'replication_factor': 1};

USE sultan_mainnet;

-- Core tables
CREATE TABLE IF NOT EXISTS blocks (
    height bigint PRIMARY KEY,
    hash text,
    prev_hash text,
    timestamp timestamp,
    validator text,
    tx_count int
);

CREATE TABLE IF NOT EXISTS transactions (
    tx_hash text PRIMARY KEY,
    from_address text,
    to_address text,
    amount bigint,
    fee bigint,
    block_height bigint,
    timestamp timestamp
);

CREATE TABLE IF NOT EXISTS wallets (
    address text PRIMARY KEY,
    owner text,
    balance bigint,
    nonce bigint,
    created_at timestamp
);

CREATE TABLE IF NOT EXISTS validators (
    address text PRIMARY KEY,
    stake bigint,
    is_mobile boolean,
    commission decimal,
    created_at timestamp
);

CREATE TABLE IF NOT EXISTS proposals (
    id bigint PRIMARY KEY,
    proposer text,
    title text,
    description text,
    status text,
    yes_votes bigint,
    no_votes bigint,
    created_at timestamp
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_tx_from ON transactions(from_address);
CREATE INDEX IF NOT EXISTS idx_tx_to ON transactions(to_address);
CREATE INDEX IF NOT EXISTS idx_tx_block ON transactions(block_height);
