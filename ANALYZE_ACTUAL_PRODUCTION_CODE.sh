#!/bin/bash

clear
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║    ANALYZING YOUR ACTUAL PRODUCTION RUST FILES                ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# 1. Check quantum.rs
echo "🔬 QUANTUM.RS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -f "/workspaces/0xv7/node/src/quantum.rs" ]; then
    echo "✅ FOUND - Size: $(wc -l /workspaces/0xv7/node/src/quantum.rs | cut -d' ' -f1) lines"
    echo "Content preview:"
    head -30 /workspaces/0xv7/node/src/quantum.rs
else
    echo "❌ Not found"
fi

# 2. Check p2p.rs
echo ""
echo "🌐 P2P.RS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -f "/workspaces/0xv7/node/src/p2p.rs" ]; then
    echo "✅ FOUND - Size: $(wc -l /workspaces/0xv7/node/src/p2p.rs | cut -d' ' -f1) lines"
    echo "Content preview:"
    head -30 /workspaces/0xv7/node/src/p2p.rs
else
    echo "❌ Not found"
fi

# 3. Check consensus.rs
echo ""
echo "🤝 CONSENSUS.RS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -f "/workspaces/0xv7/node/src/consensus.rs" ]; then
    echo "✅ FOUND - Size: $(wc -l /workspaces/0xv7/node/src/consensus.rs | cut -d' ' -f1) lines"
    echo "Content preview:"
    head -30 /workspaces/0xv7/node/src/consensus.rs
else
    echo "❌ Not found"
fi

# 4. Check blockchain.rs - THE FULL FILE
echo ""
echo "⛓️ BLOCKCHAIN.RS (FULL):"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -f "/workspaces/0xv7/node/src/blockchain.rs" ]; then
    echo "✅ FOUND - Size: $(wc -l /workspaces/0xv7/node/src/blockchain.rs | cut -d' ' -f1) lines"
    echo "FULL CONTENT:"
    cat /workspaces/0xv7/node/src/blockchain.rs
else
    echo "❌ Not found"
fi

# 5. Check scylla_db.rs
echo ""
echo "💾 SCYLLA_DB.RS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -f "/workspaces/0xv7/node/src/scylla_db.rs" ]; then
    echo "✅ FOUND - Size: $(wc -l /workspaces/0xv7/node/src/scylla_db.rs | cut -d' ' -f1) lines"
    echo "Key functions:"
    grep -E "pub fn|pub async fn|impl" /workspaces/0xv7/node/src/scylla_db.rs | head -20
else
    echo "❌ Not found"
fi

# 6. Check transaction_validator.rs
echo ""
echo "✅ TRANSACTION_VALIDATOR.RS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -f "/workspaces/0xv7/node/src/transaction_validator.rs" ]; then
    echo "✅ FOUND - Size: $(wc -l /workspaces/0xv7/node/src/transaction_validator.rs | cut -d' ' -f1) lines"
    cat /workspaces/0xv7/node/src/transaction_validator.rs
else
    echo "❌ Not found"
fi

# 7. Check types.rs
echo ""
echo "📦 TYPES.RS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -f "/workspaces/0xv7/node/src/types.rs" ]; then
    echo "✅ FOUND - Size: $(wc -l /workspaces/0xv7/node/src/types.rs | cut -d' ' -f1) lines"
    echo "Structs defined:"
    grep "pub struct" /workspaces/0xv7/node/src/types.rs
else
    echo "❌ Not found"
fi

# 8. Check migrations.rs
echo ""
echo "🔄 MIGRATIONS.RS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -f "/workspaces/0xv7/node/src/migrations.rs" ]; then
    echo "✅ FOUND - Size: $(wc -l /workspaces/0xv7/node/src/migrations.rs | cut -d' ' -f1) lines"
    head -30 /workspaces/0xv7/node/src/migrations.rs
else
    echo "❌ Not found"
fi

