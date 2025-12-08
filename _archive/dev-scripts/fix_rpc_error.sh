#!/bin/bash

cd /workspaces/0xv7

echo "=== 🔧 FIXING RPC SERVER COMPILATION ERROR ==="
echo ""

echo "1. Found error at line 914 in rpc_server.rs"
echo "2. Checking the problematic section..."
echo ""

# Show context
sed -n '910,920p' node/src/rpc_server.rs 2>/dev/null

echo ""
echo "3. Creating backup and fixing..."
cp node/src/rpc_server.rs node/src/rpc_server.rs.backup

# Check if this is inside a match statement
if grep -q '"validator_register" =>' node/src/rpc_server.rs; then
    echo "   Found match arm with syntax error"
    
    # Check if it's an incomplete match arm
    LINE_NUM=$(grep -n '"validator_register" =>' node/src/rpc_server.rs | cut -d: -f1)
    
    # Get the full context of the match statement
    sed -n "$((LINE_NUM-10)),$((LINE_NUM+10))p" node/src/rpc_server.rs
    
    echo ""
    echo "4. Removing the problematic line..."
    sed -i '/"validator_register" => {$/d' node/src/rpc_server.rs
fi

echo ""
echo "5. Testing compilation..."
cargo build -p sultan-coordinator 2>&1 | tail -10

# If still failing, show us what we need to fix
if ! cargo check -p sultan-coordinator 2>&1 | grep -q "Finished"; then
    echo ""
    echo "Still has errors. Showing RPC server structure..."
    grep -n "match.*method" node/src/rpc_server.rs | head -5
    echo ""
    echo "Showing all method handlers..."
    grep '".*" =>' node/src/rpc_server.rs | head -20
fi
