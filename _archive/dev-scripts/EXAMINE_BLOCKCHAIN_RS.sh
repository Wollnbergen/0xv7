#!/bin/bash

echo ""
echo "📜 COMPLETE blockchain.rs ANALYSIS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -f "/workspaces/0xv7/node/src/blockchain.rs" ]; then
    echo "File size: $(wc -l /workspaces/0xv7/node/src/blockchain.rs | cut -d' ' -f1) lines"
    echo ""
    echo "Key structures found:"
    echo "────────────────────"
    
    # Check for Block struct
    if grep -q "struct Block" /workspaces/0xv7/node/src/blockchain.rs; then
        echo "✅ Block structure EXISTS"
        grep -A 10 "struct Block" /workspaces/0xv7/node/src/blockchain.rs | head -15
    else
        echo "❌ No Block struct found"
    fi
    
    echo ""
    # Check for merkle implementation
    if grep -q "merkle" /workspaces/0xv7/node/src/blockchain.rs; then
        echo "✅ Merkle implementation EXISTS"
        grep -C 2 "merkle" /workspaces/0xv7/node/src/blockchain.rs | head -10
    else
        echo "❌ No merkle implementation"
    fi
    
    echo ""
    # Check for validation
    if grep -q "validate\|verify" /workspaces/0xv7/node/src/blockchain.rs; then
        echo "✅ Validation logic EXISTS"
        grep -C 2 "validate\|verify" /workspaces/0xv7/node/src/blockchain.rs | head -10
    else
        echo "❌ No validation logic"
    fi
else
    echo "❌ blockchain.rs not found at expected location"
fi

# Check sultan_mainnet for actual implementation
echo ""
echo "📜 Checking sultan_mainnet/src/main.rs:"
echo "────────────────────────────────────────"
if [ -f "/workspaces/0xv7/sultan_mainnet/src/main.rs" ]; then
    echo "✅ Found sultan_mainnet implementation"
    grep -E "Block|Transaction|hash|validate" /workspaces/0xv7/sultan_mainnet/src/main.rs | head -10
fi

