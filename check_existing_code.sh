#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║            CHECKING EXISTING CODE VIABILITY                   ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7

# Check what actually exists and works
echo "📁 Checking project structure..."
echo ""

# 1. Check node implementation
echo "1. Node Implementation:"
if [ -d "node/src" ]; then
    echo "   ✅ Node source exists"
    echo "   Files: $(find node/src -name "*.rs" | wc -l) Rust files"
    
    # Check for key modules
    for module in "rpc_server" "consensus" "state" "network"; do
        if [ -f "node/src/${module}.rs" ] || [ -d "node/src/${module}" ]; then
            echo "   ✅ ${module} module found"
        else
            echo "   ❌ ${module} module missing"
        fi
    done
fi

# 2. Check SDK
echo ""
echo "2. SDK Implementation:"
if [ -d "sdk/src" ]; then
    echo "   ✅ SDK source exists"
    # Count implemented methods
    if [ -f "sdk/src/lib.rs" ]; then
        methods=$(grep -c "pub async fn\|pub fn" sdk/src/lib.rs || echo 0)
        echo "   Methods implemented: $methods"
    fi
fi

# 3. Check if we can build anything
echo ""
echo "3. Build Status:"
echo "   Testing RPC server build..."
if cargo build -p sultan-coordinator --bin rpc_server 2>&1 | grep -q "Finished"; then
    echo "   ✅ RPC server builds successfully"
else
    echo "   ❌ RPC server build fails"
    # Show specific error
    cargo build -p sultan-coordinator --bin rpc_server 2>&1 | grep "error" | head -2
fi

# 4. Create migration plan
echo ""
echo "📋 Creating migration strategy..."

cat > MIGRATION_STRATEGY.md << 'EOF'
# Migration Strategy: Custom → Cosmos SDK

## What We Can Keep
1. **Business Logic**
   - Token economics
   - Reward calculations
   - Governance rules
   
2. **RPC Interface**
   - Existing RPC methods as custom endpoints
   - SDK wrapper for compatibility

3. **Database Schema**
   - ScyllaDB for indexing/analytics
   - Cosmos for consensus state

## What Must Change
1. **Consensus** → Tendermint BFT
2. **State Management** → Cosmos state machine
3. **P2P Network** → Tendermint P2P
4. **Cryptography** → Cosmos crypto

## Migration Steps
1. Set up Cosmos chain scaffold
2. Create Sultan custom module
3. Port business logic to Cosmos module
4. Wrap RPC methods to Cosmos queries
5. Test with local network
6. Security audit
7. Launch mainnet
EOF

echo "✅ Created MIGRATION_STRATEGY.md"
echo ""
echo "Ready to proceed with migration!"
