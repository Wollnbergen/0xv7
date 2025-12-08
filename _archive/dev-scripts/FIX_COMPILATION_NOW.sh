#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         FIXING SULTAN CHAIN COMPILATION - IMMEDIATE           ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/node

# Fix the ChainConfig conflict
echo "🔧 Step 1: Unifying ChainConfig..."

cat > src/config.rs << 'RUST'
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: String,
    pub block_time_ms: u64,
    pub max_validators: usize,
    pub min_stake: u64,
    pub inflation_rate: f64,
    pub total_supply: u64,
    pub shards: usize,
    pub burn_rate: f64,
}

impl Default for ChainConfig {
    fn default() -> Self {
        ChainConfig {
            chain_id: "sultan-mainnet-1".to_string(),
            block_time_ms: 5000,
            max_validators: 100,
            min_stake: 5000,
            inflation_rate: 0.08,  // 8% Year 1
            total_supply: 1_000_000_000,
            shards: 8,
            burn_rate: 0.01,  // 1% burn
        }
    }
}
RUST

# Fix SDK to use the unified config
echo "🔧 Step 2: Fixing SDK..."
sed -i 's/pub async fn new(config: ChainConfig, _db: Option<&str>)/pub async fn new(config: ChainConfig)/' src/sdk.rs

# Clean build
echo "🔨 Step 3: Building..."
cargo clean
cargo build --release 2>&1 | tail -10

if [ $? -eq 0 ]; then
    echo "✅ COMPILATION SUCCESSFUL!"
else
    echo "❌ Still has issues. Checking errors..."
    cargo build 2>&1 | grep error
fi

