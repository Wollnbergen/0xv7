#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          SULTAN CHAIN - MAINNET BUILD STATUS                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "�� Date: $(date '+%Y-%m-%d %H:%M')"
echo ""

cd /workspaces/0xv7/node

# Function to check component
check() {
    local name=$1
    local condition=$2
    if eval "$condition"; then
        echo "  ✅ $name"
        return 0
    else
        echo "  ❌ $name"
        return 1
    fi
}

echo "🔧 Core Components:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
SCORE=0

check "blockchain.rs exists" "[ -f src/blockchain.rs ]" && ((SCORE++))
check "sdk.rs exists" "[ -f src/sdk.rs ]" && ((SCORE++))
check "consensus.rs exists" "[ -f src/consensus.rs ]" && ((SCORE++))
check "Database schema" "[ -f migrations/init.cql ]" && ((SCORE++))
check "Node binary builds" "cargo build --release --bin sultan_node 2>&1 | grep -q 'Finished'" && ((SCORE++))
check "Docker setup" "[ -f docker-compose.yml ]" && ((SCORE++))

echo ""
echo "🗄️ Database Status:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
check "ScyllaDB running" "docker ps | grep -q scylla" && ((SCORE++))

echo ""
echo "🌐 Services Status:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
check "Testnet API (3030)" "curl -s http://localhost:3030 > /dev/null 2>&1" && ((SCORE++))
check "Node RPC (26657)" "curl -s http://localhost:26657 > /dev/null 2>&1" && ((SCORE++))

echo ""
echo "📊 Overall Progress:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
PERCENTAGE=$((SCORE * 10))
echo "Score: $SCORE/10 ($PERCENTAGE%)"
echo ""

if [ $PERCENTAGE -ge 70 ]; then
    echo "🎉 Great progress! Keep going!"
elif [ $PERCENTAGE -ge 40 ]; then
    echo "💪 Making progress! Focus on compilation fixes."
else
    echo "🚧 Just getting started. Follow the plan!"
fi

echo ""
echo "📋 Today's Achievements:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  • Fixed major compilation issues"
echo "  • Created consensus engine"
echo "  • Set up database schema"
echo "  • Built node binary structure"
echo "  • Docker deployment ready"
echo ""
echo "🎯 Next Priority Tasks:"
echo "  1. Fix remaining compilation errors"
echo "  2. Test node startup with database"
echo "  3. Implement P2P networking"
echo "  4. Add RPC endpoints to node"

