open_browser() {
  # open URL in host browser without blocking the terminal
  nohup open_browser "$1" >/dev/null 2>&1 &
  disown || true
}

#!/bin/bash

cd /workspaces/0xv7

echo "=== 🔧 COMPLETE FIX FOR ALL COMPILATION ERRORS ==="
echo ""

# 1. Clean up rpc_server.rs
echo "1. Removing stray code from rpc_server.rs..."
sed -i '/^\/\/ Add this to the match statement/d' node/src/rpc_server.rs

# 2. Fix field name mismatches
echo "2. Fixing field name mismatches..."
sed -i 's/"status": proposal\.status/"state": proposal.state.clone()/g' node/src/rpc_server.rs
sed -i 's/"created_at_ms": proposal\.created_at_ms/"created_at": proposal.created_at/g' node/src/rpc_server.rs

# 3. Add Display trait to ProposalState
echo "3. Adding Display implementation for ProposalState..."
cat >> node/src/sdk.rs << 'EOFDISP'

use std::fmt;
impl fmt::Display for ProposalState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProposalState::Active => write!(f, "active"),
            ProposalState::Passed => write!(f, "passed"),
            ProposalState::Rejected => write!(f, "rejected"),
            ProposalState::Executed => write!(f, "executed"),
        }
    }
}
EOFDISP

echo ""
echo "4. Building Sultan Chain..."
cargo build -p sultan-coordinator 2>&1 | tail -5

# Start server if successful
if cargo check -p sultan-coordinator 2>&1 | grep -q "Finished"; then
    echo ""
    echo "✅ BUILD SUCCESSFUL!"
    
    pkill -f "cargo.*rpc_server" 2>/dev/null || true
    sleep 2
    
    export SULTAN_JWT_SECRET='production_secret_32_bytes_minimum_required'
    cargo run -p sultan-coordinator --bin rpc_server > /tmp/sultan.log 2>&1 &
    SERVER_PID=$!
    
    echo ""
    echo "=== 🎉 DAY 3-4 COMPLETE ==="
    echo "📡 Server: http://127.0.0.1:3030 (PID: $SERVER_PID)"
    echo "📝 Logs: tail -f /tmp/sultan.log"
    echo "🌐 Browser: \"$BROWSER\" http://127.0.0.1:3030"
    echo ""
    echo "Ready for Day 5-6! 🚀"
fi
