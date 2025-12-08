#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          FIX RPC SERVER AND GET IT RUNNING                    ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7

# 1. Fix all client_id issues in RPC server
echo "🔧 Fixing RPC server client_id issues..."

# Create a Python script to fix all client_id issues properly
cat > /tmp/fix_rpc_final.py << 'PYEOF'
#!/usr/bin/env python3
import re

with open('node/src/rpc_server.rs', 'r') as f:
    content = f.read()

# Methods that actually use client_id - keep without underscore
methods_using_client_id = [
    'auth_ping', 'wallet_create', 'wallet_balance', 'token_mint',
    'stake', 'query_apy', 'vote_on_proposal', 'proposal_create'
]

# Fix these to not have underscore
for method in methods_using_client_id:
    content = re.sub(
        f'let _client_id = require_auth\(&meta, "{method}"\)\?;',
        f'let client_id = require_auth(&meta, "{method}")?;',
        content
    )

# Also fix the rate limit and idempotency function calls
content = content.replace('_client_id', 'client_id')

with open('node/src/rpc_server.rs', 'w') as f:
    f.write(content)

print("✅ Fixed all client_id issues")
PYEOF

python3 /tmp/fix_rpc_final.py

# 2. Check if the build works now
echo ""
echo "🔨 Building RPC server..."
if cargo build -p sultan-coordinator --bin rpc_server 2>&1 | tail -1 | grep -q "Finished"; then
    echo "✅ RPC Server built successfully!"
    
    echo ""
    echo "🚀 Starting RPC server..."
    echo "   Server will run at: http://localhost:3030"
    echo "   Press Ctrl+C to stop"
    echo ""
    
    cargo run -p sultan-coordinator --bin rpc_server
else
    echo "⚠️ Still has build issues. Checking..."
    cargo build -p sultan-coordinator --bin rpc_server 2>&1 | grep "error" | head -5
fi
