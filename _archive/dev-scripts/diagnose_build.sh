#!/bin/bash

echo "🔍 Diagnosing build issues..."
cd /workspaces/0xv7/node

# Check if all required source files exist
echo ""
echo "Checking required source files:"
for file in config sdk rpc_server consensus blockchain scylla_db types transaction_validator; do
    if [ -f "src/${file}.rs" ]; then
        echo "✅ src/${file}.rs exists"
    else
        echo "❌ src/${file}.rs MISSING!"
    fi
done

# Check Cargo.toml for missing dependencies
echo ""
echo "Checking key dependencies:"
grep -E "tonic|prost|jsonwebtoken|scylla|tokio" Cargo.toml

# Try a minimal build
echo ""
echo "Attempting minimal build..."
cargo check 2>&1 | grep -E "error|warning" | head -10
