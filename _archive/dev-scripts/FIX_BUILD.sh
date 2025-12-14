#!/bin/bash

echo "🔧 FIXING BUILD ISSUES"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cd /workspaces/0xv7/sultan-unified

# Fix missing imports in main.rs
if [ -f "src/main.rs" ]; then
    # Check if types module exists, if not create a basic one
    if [ ! -f "src/types.rs" ]; then
        echo "Creating missing types.rs..."
        cat > src/types.rs << 'TYPES'
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}
TYPES
    fi

    # Check if config module exists
    if [ ! -f "src/config.rs" ]; then
        echo "Creating missing config.rs..."
        cat > src/config.rs << 'CONFIG'
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub chain_id: u32,
    pub zero_fees: bool,
    pub block_time: u64,
    pub max_block_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            chain_id: 1,
            zero_fees: true,
            block_time: 3000,
            max_block_size: 1_000_000,
        }
    }
}
CONFIG
    fi

    # Fix database.rs if it doesn't exist
    if [ ! -f "src/database.rs" ]; then
        echo "Creating missing database.rs..."
        cat > src/database.rs << 'DATABASE'
use anyhow::Result;
use rocksdb::{DB, Options};
use crate::types::Block;

pub struct Database {
    db: Option<DB>,
}

impl Database {
    pub fn new() -> Self {
        Database { db: None }
    }
    
    pub fn open(path: &str) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        Ok(Database {
            db: Some(DB::open(&opts, path)?)
        })
    }
}
DATABASE
    fi
fi

echo ""
echo "🔨 Attempting build again..."
cargo build --release 2>&1 | tail -20

if [ -f "target/release/sultan" ]; then
    echo ""
    echo "✅ BUILD FIXED AND SUCCESSFUL!"
else
    echo ""
    echo "⚠️ Still having issues. Running cargo check for details..."
    cargo check 2>&1
fi
