#!/bin/bash

cd /workspaces/0xv7

echo "=== 🔧 CLEANING UP SDK CONFUSION ==="
echo ""

# Step 1: Archive old SDK versions
echo "1. Archiving unused SDK versions..."
mkdir -p node/src/archived_sdks 2>/dev/null
mv node/src/sdk_v2.rs node/src/archived_sdks/ 2>/dev/null && echo "   Archived sdk_v2.rs"
mv node/src/sdk_original.rs node/src/archived_sdks/ 2>/dev/null && echo "   Archived sdk_original.rs"  
mv node/src/sdk.backup.rs node/src/archived_sdks/ 2>/dev/null && echo "   Archived sdk.backup.rs"

# Step 2: Clean lib.rs
echo ""
echo "2. Cleaning lib.rs..."
sed -i '/pub mod sdk_v2;/d' node/src/lib.rs 2>/dev/null
sed -i '/pub mod sdk_day34;/d' node/src/lib.rs 2>/dev/null
sed -i '/pub mod sdk_fixed;/d' node/src/lib.rs 2>/dev/null
echo "   ✅ Removed unused SDK references"

# Step 3: Test compilation
echo ""
echo "3. Testing compilation..."
cargo build -p sultan-coordinator 2>&1 | grep -E "Compiling|Finished|error" | tail -5

echo ""
echo "=== ✅ CLEANUP COMPLETE ==="
echo ""
echo "• Main SDK: node/src/sdk.rs" 
echo "• Archived: node/src/archived_sdks/"
echo "• Ready for Day 5-6: Token Economics"
