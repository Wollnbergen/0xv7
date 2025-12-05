#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║        SULTAN CHAIN - DEEP CODEBASE ANALYSIS                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "Performing comprehensive analysis of all components..."
echo ""

# Function to analyze a directory
analyze_dir() {
    local dir=$1
    local name=$2
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📂 $name ($dir)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    if [ -d "$dir" ]; then
        # Count files by extension
        echo "File types:"
        find "$dir" -type f -name "*.*" 2>/dev/null | sed 's/.*\.//' | sort | uniq -c | sort -rn | head -5
        
        # Check for actual implementation
        echo ""
        echo "Implementation files:"
        find "$dir" -type f \( -name "*.rs" -o -name "*.go" -o -name "*.js" -o -name "*.ts" \) 2>/dev/null | head -5
        
        # Check file sizes (non-empty files)
        echo ""
        echo "Largest files:"
        find "$dir" -type f -exec ls -lh {} \; 2>/dev/null | sort -k5 -hr | head -3 | awk '{print $5, $9}'
        
        # Look for TODO/FIXME comments
        echo ""
        echo "TODOs/FIXMEs found:"
        grep -r "TODO\|FIXME\|XXX\|HACK" "$dir" 2>/dev/null | wc -l
    else
        echo "❌ Directory not found"
    fi
    echo ""
}

# 1. CORE NODE ANALYSIS
analyze_dir "/workspaces/0xv7/node" "Core Node (Rust)"

# Check if node actually compiles
echo "Compilation check:"
cd /workspaces/0xv7/node 2>/dev/null && cargo check --quiet 2>&1 | head -3 || echo "Cannot compile"
cd /workspaces/0xv7
echo ""

# 2. COSMOS SDK ANALYSIS
analyze_dir "/workspaces/0xv7/sultan-sdk" "Cosmos SDK Integration"

# Check for actual Cosmos integration
echo "Cosmos integration check:"
if [ -f "/workspaces/0xv7/sultan-sdk/go.mod" ]; then
    grep -c "cosmos-sdk" /workspaces/0xv7/sultan-sdk/go.mod 2>/dev/null || echo "0"
else
    echo "No go.mod found - Cosmos SDK not integrated"
fi
echo ""

# 3. BRIDGE ANALYSIS
analyze_dir "/workspaces/0xv7/sultan-interop" "Bridge System"

# 4. Check for actual working binaries
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 WORKING BINARIES CHECK"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Searching for compiled binaries:"
find /workspaces/0xv7 -type f -executable -name "sultan*" 2>/dev/null | head -5
find /workspaces/0xv7 -type f -path "*/target/*" -executable 2>/dev/null | head -5
echo ""

# 5. Database implementation check
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "💾 DATABASE IMPLEMENTATION"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "ScyllaDB integration:"
grep -r "scylla" /workspaces/0xv7/node/src 2>/dev/null | wc -l
echo "files mentioning ScyllaDB"
echo ""

# 6. Smart contract capability
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📜 SMART CONTRACT CAPABILITY"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "CosmWasm contracts:"
find /workspaces/0xv7 -name "*.wasm" 2>/dev/null | wc -l
echo "Solidity contracts:"
find /workspaces/0xv7 -name "*.sol" 2>/dev/null | wc -l
echo ""

# 7. Test coverage
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧪 TEST COVERAGE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test files found:"
find /workspaces/0xv7 -name "*test*" -o -name "*spec*" 2>/dev/null | grep -E "\.(rs|go|js|ts)$" | wc -l
echo ""

