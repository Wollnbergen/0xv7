#!/bin/bash

echo "📝 Adding Day 5-6 RPC methods to node/src/rpc_server.rs..."

# Backup the current rpc_server.rs
cp node/src/rpc_server.rs node/src/rpc_server.rs.backup_day56_final

# Find the line where we register RPC methods (in main function)
# Look for the pattern .with_method("stake", stake)

# First, let's check if methods already exist
if grep -q "token_transfer" node/src/rpc_server.rs; then
    echo "✅ token_transfer already exists"
else
    echo "❌ token_transfer not found - needs to be added manually"
fi

if grep -q "calculate_rewards" node/src/rpc_server.rs; then
    echo "✅ calculate_rewards already exists"
else
    echo "❌ calculate_rewards not found - needs to be added manually"
fi

if grep -q "claim_rewards" node/src/rpc_server.rs; then
    echo "✅ claim_rewards already exists"
else
    echo "❌ claim_rewards not found - needs to be added manually"
fi

echo ""
echo "📋 Instructions to add RPC methods:"
echo "1. Open node/src/rpc_server.rs"
echo "2. Find the main() function"
echo "3. Look for the section with .with_method() calls"
echo "4. Add these three lines:"
echo '    .with_method("token_transfer", token_transfer)'
echo '    .with_method("calculate_rewards", calculate_rewards)'
echo '    .with_method("claim_rewards", claim_rewards)'
echo ""
echo "5. Add the function implementations from day56_rpc_patch.txt"
