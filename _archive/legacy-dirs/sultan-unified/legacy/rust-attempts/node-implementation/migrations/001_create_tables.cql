-- Create keyspace for mainnet
CREATE KEYSPACE IF NOT EXISTS sultan_mainnet 
WITH REPLICATION = {
    'class': 'SimpleStrategy',
    'replication_factor': 3
};

USE sultan_mainnet;

-- Blocks table
CREATE TABLE IF NOT EXISTS blocks (
    height bigint PRIMARY KEY,
    timestamp timestamp,
    hash text,
    prev_hash text,
    validator text,
    transactions list<text>,
    state_root text
);

-- Transactions table  
CREATE TABLE IF NOT EXISTS transactions (
    tx_hash text PRIMARY KEY,
    from_address text,
    to_address text,
    amount bigint,
    fee bigint,
    timestamp timestamp,
    block_height bigint,
    status text
);

-- Accounts table
CREATE TABLE IF NOT EXISTS accounts (
    address text PRIMARY KEY,
    balance bigint,
    nonce bigint,
    stake bigint,
    is_validator boolean,
    created_at timestamp
);

-- Validators table
CREATE TABLE IF NOT EXISTS validators (
    address text PRIMARY KEY,
    stake bigint,
    commission_rate decimal,
    is_mobile boolean,
    jailed boolean,
    joined_at timestamp
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_tx_from ON transactions(from_address);
CREATE INDEX IF NOT EXISTS idx_tx_to ON transactions(to_address);
CREATE INDEX IF NOT EXISTS idx_tx_block ON transactions(block_height);
