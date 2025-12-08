#!/bin/bash

clear
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║     DEEP ANALYSIS OF YOUR ACTUAL PRODUCTION CODE              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# 1. Check quantum.rs
echo "🔬 1. QUANTUM.RS ANALYSIS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -f "/workspaces/0xv7/node/src/quantum.rs" ]; then
    echo "✅ EXISTS - Checking implementation:"
    head -50 /workspaces/0xv7/node/src/quantum.rs
    echo ""
    echo "Functions found:"
    grep -E "pub fn|impl" /workspaces/0xv7/node/src/quantum.rs | head -10
fi

# 2. Check p2p.rs
echo ""
echo "🌐 2. P2P.RS ANALYSIS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -f "/workspaces/0xv7/node/src/p2p.rs" ]; then
    echo "✅ EXISTS - Checking implementation:"
    echo "Line count: $(wc -l /workspaces/0xv7/node/src/p2p.rs | cut -d' ' -f1)"
    echo ""
    echo "Key components:"
    grep -E "pub struct|pub fn|impl NetworkBehaviour" /workspaces/0xv7/node/src/p2p.rs | head -15
    echo ""
    echo "Checking for real P2P functionality:"
    grep -E "Swarm|gossipsub|libp2p" /workspaces/0xv7/node/src/p2p.rs | head -10
fi

# 3. Check blockchain.rs
echo ""
echo "⛓️ 3. BLOCKCHAIN.RS ANALYSIS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -f "/workspaces/0xv7/node/src/blockchain.rs" ]; then
    echo "✅ EXISTS - Full content check:"
    echo "Size: $(wc -l /workspaces/0xv7/node/src/blockchain.rs) lines"
    echo ""
    cat /workspaces/0xv7/node/src/blockchain.rs
fi

# 4. Check consensus.rs
echo ""
echo "🤝 4. CONSENSUS.RS ANALYSIS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -f "/workspaces/0xv7/node/src/consensus.rs" ]; then
    echo "✅ EXISTS - Checking for real consensus:"
    grep -E "vote|round|propose|commit|ConsensusState" /workspaces/0xv7/node/src/consensus.rs | head -20
fi

# 5. Check scylla_db.rs
echo ""
echo "💾 5. SCYLLA_DB.RS ANALYSIS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -f "/workspaces/0xv7/node/src/scylla_db.rs" ]; then
    echo "✅ EXISTS - Database implementation:"
    grep -E "Session|create_tables|save_block|query" /workspaces/0xv7/node/src/scylla_db.rs | head -15
fi

