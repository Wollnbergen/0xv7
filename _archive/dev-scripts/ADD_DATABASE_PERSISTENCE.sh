#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         ADDING DATABASE PERSISTENCE TO SULTAN CHAIN           ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/node

echo "💾 Adding RocksDB for persistence..."

# Add RocksDB to Cargo.toml
cat >> Cargo.toml << 'TOML'

rocksdb = "0.21"
TOML

# Create persistence module
cat > src/persistence.rs << 'RUST'
use rocksdb::{DB, Options};
use serde::{Serialize, Deserialize};
use anyhow::Result;

pub struct Storage {
    db: DB,
}

impl Storage {
    pub fn new(path: &str) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path)?;
        Ok(Storage { db })
    }
    
    pub fn save_block(&self, height: u64, block_data: &[u8]) -> Result<()> {
        self.db.put(format!("block:{}", height), block_data)?;
        Ok(())
    }
    
    pub fn get_block(&self, height: u64) -> Result<Option<Vec<u8>>> {
        Ok(self.db.get(format!("block:{}", height))?)
    }
    
    pub fn save_state(&self, key: &str, value: &[u8]) -> Result<()> {
        self.db.put(format!("state:{}", key), value)?;
        Ok(())
    }
}
RUST

# Update lib.rs
echo "pub mod persistence;" >> src/lib.rs

echo "✅ Database persistence added!"

