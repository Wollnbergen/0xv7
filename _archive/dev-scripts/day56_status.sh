#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║              DAY 5-6 IMPLEMENTATION STATUS                    ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "📁 Module Files:"
[[ -f "node/src/token_transfer.rs" ]] && echo "  ✅ token_transfer.rs exists" || echo "  ❌ token_transfer.rs missing"
[[ -f "node/src/rewards.rs" ]] && echo "  ✅ rewards.rs exists" || echo "  ❌ rewards.rs missing"

echo ""
echo "📦 Module Registration in lib.rs:"
grep -q "pub mod token_transfer;" node/src/lib.rs && echo "  ✅ token_transfer registered" || echo "  ❌ token_transfer not registered"
grep -q "pub mod rewards;" node/src/lib.rs && echo "  ✅ rewards registered" || echo "  ❌ rewards not registered"

echo ""
echo "🔧 RPC Methods in rpc_server.rs:"
grep -q "fn token_transfer" node/src/rpc_server.rs && echo "  ✅ token_transfer function exists" || echo "  ❌ token_transfer function missing"
grep -q "fn calculate_rewards" node/src/rpc_server.rs && echo "  ✅ calculate_rewards function exists" || echo "  ❌ calculate_rewards function missing"
grep -q "fn claim_rewards" node/src/rpc_server.rs && echo "  ✅ claim_rewards function exists" || echo "  ❌ claim_rewards function missing"

echo ""
echo "📝 RPC Method Registration:"
grep -q 'with_method("token_transfer"' node/src/rpc_server.rs && echo "  ✅ token_transfer registered" || echo "  ❌ token_transfer not registered"
grep -q 'with_method("calculate_rewards"' node/src/rpc_server.rs && echo "  ✅ calculate_rewards registered" || echo "  ❌ calculate_rewards not registered"
grep -q 'with_method("claim_rewards"' node/src/rpc_server.rs && echo "  ✅ claim_rewards registered" || echo "  ❌ claim_rewards not registered"

echo ""
echo "🔨 Compilation Status:"
if cargo build -p sultan-coordinator 2>&1 | grep -q "Finished"; then
    echo "  ✅ Compiles successfully"
else
    echo "  ❌ Compilation errors present"
fi

echo ""
echo "🚀 Server Status:"
if pgrep -f "target.*rpc_server" > /dev/null; then
    echo "  ✅ Server is running (PID: $(pgrep -f 'target.*rpc_server' | head -1))"
else
    echo "  ❌ Server is not running"
fi

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "To complete Day 5-6 integration:"
echo "1. Add missing RPC methods to node/src/rpc_server.rs"
echo "2. Register them in main() function"
echo "3. Fix any compilation errors"
echo "4. Restart server: ./server_control.sh restart"
echo "5. Test: ./test_day56_complete.sh"
