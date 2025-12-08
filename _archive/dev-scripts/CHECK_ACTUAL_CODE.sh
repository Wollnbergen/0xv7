#!/bin/bash

echo ""
echo "�� EXAMINING ACTUAL IMPLEMENTATION FILES:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check blockchain.rs
if [ -f "/workspaces/0xv7/node/src/blockchain.rs" ]; then
    echo ""
    echo "1️⃣ blockchain.rs content:"
    echo "─────────────────────────"
    grep -E "struct Block|merkle|hash|validate" /workspaces/0xv7/node/src/blockchain.rs | head -10
fi

# Check consensus implementation
if [ -f "/workspaces/0xv7/node/src/consensus.rs" ]; then
    echo ""
    echo "2️⃣ consensus.rs content:"
    echo "─────────────────────────"
    grep -E "ConsensusState|vote|round|validator" /workspaces/0xv7/node/src/consensus.rs | head -10
fi

# Check for crypto operations
echo ""
echo "3️⃣ Cryptography in codebase:"
echo "─────────────────────────────"
find /workspaces/0xv7 -name "*.rs" -exec grep -l "sign\|verify\|hash\|sha256\|ed25519" {} \; 2>/dev/null | head -10

# Check database integration
if [ -f "/workspaces/0xv7/node/src/scylla_db.rs" ]; then
    echo ""
    echo "4️⃣ scylla_db.rs content:"
    echo "─────────────────────────"
    grep -E "save|persist|query|create_tables" /workspaces/0xv7/node/src/scylla_db.rs | head -10
fi

# Check P2P implementation
if [ -f "/workspaces/0xv7/node/src/p2p.rs" ]; then
    echo ""
    echo "5️⃣ p2p.rs content:"
    echo "─────────────────"
    grep -E "peer|broadcast|gossip|network" /workspaces/0xv7/node/src/p2p.rs | head -10
fi

# Check transaction processing
echo ""
echo "6️⃣ Transaction processing:"
echo "──────────────────────────"
grep -r "execute_transaction\|process_transaction\|apply_transaction" /workspaces/0xv7 --include="*.rs" 2>/dev/null | head -5

# Check for actual running processes
echo ""
echo "7️⃣ Currently running Sultan processes:"
echo "───────────────────────────────────────"
ps aux | grep -E "sultan|node|cargo" | grep -v grep | head -5

