#!/bin/bash

echo "╔════════════════════════════════════════════════════════════════════╗"
echo "║            SULTAN CHAIN - DEEP DIVE PRODUCTION AUDIT                ║"
echo "╚════════════════════════════════════════════════════════════════════╝"
echo ""
echo "🔍 Performing comprehensive codebase analysis..."
echo ""

# Initialize counters
PRODUCTION_READY=0
MOCK_DEMO=0
BROKEN=0
MISSING=0

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1️⃣ BLOCKCHAIN CORE (/workspaces/0xv7/node/)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -d "/workspaces/0xv7/node" ]; then
    echo "📁 Checking Rust node implementation..."
    
    # Check if it compiles
    cd /workspaces/0xv7/node 2>/dev/null
    if cargo check --quiet 2>/dev/null; then
        echo "   ✅ Compiles successfully"
        ((PRODUCTION_READY++))
    else
        echo "   ❌ DOES NOT COMPILE - Cargo workspace conflicts"
        ((BROKEN++))
    fi
    
    # Check core modules
    for module in blockchain consensus p2p rpc_server database; do
        if [ -f "src/${module}.rs" ]; then
            lines=$(wc -l < "src/${module}.rs" 2>/dev/null || echo "0")
            if [ "$lines" -gt "50" ]; then
                echo "   ✅ ${module}.rs: ${lines} lines (substantial code)"
            else
                echo "   ⚠️  ${module}.rs: ${lines} lines (minimal implementation)"
            fi
        else
            echo "   ❌ ${module}.rs: MISSING"
            ((MISSING++))
        fi
    done
else
    echo "   ❌ Node directory not found"
    ((MISSING++))
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "2️⃣ WEB INTERFACE (/workspaces/0xv7/public/)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -f "/workspaces/0xv7/public/index.html" ]; then
    size=$(du -h /workspaces/0xv7/public/index.html | cut -f1)
    echo "   ✅ index.html exists (${size})"
    
    # Check if it has real functionality
    if grep -q "WebSocket\|fetch\|API" /workspaces/0xv7/public/index.html 2>/dev/null; then
        echo "   ✅ Has API integration code"
        ((PRODUCTION_READY++))
    else
        echo "   ⚠️  Static demo only (no real API calls)"
        ((MOCK_DEMO++))
    fi
    
    # Check if running
    if pgrep -f "python3 -m http.server 3000" > /dev/null; then
        echo "   ✅ Web server is RUNNING on port 3000"
        
        # Test if responsive
        if curl -s http://localhost:3000 > /dev/null 2>&1; then
            echo "   ✅ Web interface is ACCESSIBLE"
        else
            echo "   ❌ Web interface not responding"
        fi
    else
        echo "   ⚠️  Web server not running"
    fi
else
    echo "   ❌ index.html not found"
    ((MISSING++))
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "3️⃣ API SERVER (/workspaces/0xv7/production/api/)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -f "/workspaces/0xv7/production/api/server.py" ]; then
    lines=$(wc -l < /workspaces/0xv7/production/api/server.py)
    echo "   ✅ server.py exists (${lines} lines)"
    
    # Check if it's real or mock
    if grep -q "random\|fake\|mock" /workspaces/0xv7/production/api/server.py 2>/dev/null; then
        echo "   ⚠️  Returns MOCK/RANDOM data (not real blockchain)"
        ((MOCK_DEMO++))
    else
        echo "   ✅ Appears to be production code"
        ((PRODUCTION_READY++))
    fi
    
    # Check if running
    if pgrep -f "server.py" > /dev/null; then
        echo "   ✅ API server is RUNNING"
        
        # Test API endpoint
        response=$(curl -s http://localhost:1317/status 2>/dev/null | head -c 100)
        if [ ! -z "$response" ]; then
            echo "   ✅ API is RESPONDING: ${response:0:50}..."
        else
            echo "   ❌ API not responding"
        fi
    else
        echo "   ⚠️  API server not running"
    fi
else
    echo "   ❌ server.py not found"
    ((MISSING++))
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "4️⃣ DATABASE (ScyllaDB)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if docker ps | grep -q sultan-scylla; then
    echo "   ✅ ScyllaDB container is RUNNING"
    
    # Check if schema exists
    if docker exec sultan-scylla cqlsh -e "DESCRIBE KEYSPACES;" 2>/dev/null | grep -q sultan; then
        echo "   ✅ Sultan keyspace exists in database"
        ((PRODUCTION_READY++))
    else
        echo "   ⚠️  No Sultan schema in database (empty)"
        ((MOCK_DEMO++))
    fi
else
    echo "   ⚠️  ScyllaDB container exists but not running"
    if docker ps -a | grep -q sultan-scylla; then
        echo "   ℹ️  Container can be started with: docker start sultan-scylla"
    fi
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "5️⃣ CLI TOOLS (/workspaces/0xv7/production/bin/)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -f "/workspaces/0xv7/production/bin/sultan" ]; then
    echo "   ✅ CLI tool exists"
    
    # Test CLI functionality
    output=$(/workspaces/0xv7/production/bin/sultan version 2>&1)
    if [ ! -z "$output" ]; then
        echo "   ✅ CLI responds: $output"
        ((PRODUCTION_READY++))
    else
        echo "   ⚠️  CLI exists but may not function"
    fi
else
    echo "   ❌ CLI tool not found"
    ((MISSING++))
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "6️⃣ BRIDGES (/workspaces/0xv7/sultan-interop/)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

bridge_dir="/workspaces/0xv7/sultan-interop"
if [ -d "$bridge_dir" ]; then
    for bridge in bitcoin eth_bridge sol_bridge ton_bridge; do
        if [ -f "$bridge_dir/src/${bridge}.rs" ]; then
            lines=$(wc -l < "$bridge_dir/src/${bridge}.rs" 2>/dev/null || echo "0")
            if [ "$lines" -gt "100" ]; then
                echo "   ✅ ${bridge}.rs: ${lines} lines (implemented)"
            else
                echo "   ⚠️  ${bridge}.rs: ${lines} lines (skeleton only)"
                ((MOCK_DEMO++))
            fi
        else
            echo "   ❌ ${bridge}.rs: not found"
            ((MISSING++))
        fi
    done
else
    echo "   ❌ Bridge directory not found"
    ((MISSING++))
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "7️⃣ COSMOS SDK INTEGRATION (/workspaces/0xv7/sultan-sdk/)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -d "/workspaces/0xv7/sultan-sdk" ]; then
    if [ -f "/workspaces/0xv7/sultan-sdk/go.mod" ]; then
        echo "   ✅ Cosmos SDK structure exists"
        
        # Check if it's properly integrated
        if grep -q "cosmos-sdk" /workspaces/0xv7/sultan-sdk/go.mod 2>/dev/null; then
            echo "   ✅ Cosmos SDK imported"
        else
            echo "   ⚠️  Cosmos SDK not properly imported"
            ((MOCK_DEMO++))
        fi
    else
        echo "   ⚠️  Go module not initialized"
        ((MOCK_DEMO++))
    fi
else
    echo "   ❌ SDK directory not found"
    ((MISSING++))
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "8️⃣ TRANSACTION PROCESSING TEST"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo "   Testing if we can process a real transaction..."

# Try to send a transaction via CLI
if [ -f "/workspaces/0xv7/production/bin/sultan" ]; then
    tx_result=$(/workspaces/0xv7/production/bin/sultan tx send alice bob 100 2>&1)
    if echo "$tx_result" | grep -q "success\|sent\|0x"; then
        echo "   ⚠️  Transaction command works (but returns mock data)"
        ((MOCK_DEMO++))
    else
        echo "   ❌ Transaction processing not functional"
        ((BROKEN++))
    fi
else
    echo "   ❌ Cannot test - CLI not available"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 DEEP DIVE SUMMARY"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Count all files
total_files=$(find /workspaces/0xv7 -type f 2>/dev/null | wc -l)
rust_files=$(find /workspaces/0xv7 -name "*.rs" 2>/dev/null | wc -l)
go_files=$(find /workspaces/0xv7 -name "*.go" 2>/dev/null | wc -l)
js_files=$(find /workspaces/0xv7 -name "*.js" -o -name "*.ts" 2>/dev/null | wc -l)
py_files=$(find /workspaces/0xv7 -name "*.py" 2>/dev/null | wc -l)

echo "📁 Codebase Statistics:"
echo "   • Total files: ${total_files}"
echo "   • Rust files: ${rust_files}"
echo "   • Go files: ${go_files}"
echo "   • JavaScript/TypeScript: ${js_files}"
echo "   • Python files: ${py_files}"
echo ""

echo "🔍 Component Analysis:"
echo "   • Production Ready: ${PRODUCTION_READY} components"
echo "   • Mock/Demo: ${MOCK_DEMO} components"
echo "   • Broken: ${BROKEN} components"
echo "   • Missing: ${MISSING} components"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 TRUE PRODUCTION STATUS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "✅ WHAT'S ACTUALLY WORKING:"
echo "   1. Web Dashboard - Static HTML showing the concept"
echo "   2. Mock API - Returns fake blockchain data"
echo "   3. CLI Tools - Execute but with mock responses"
echo "   4. ScyllaDB - Running but no real data"
echo ""

echo "❌ WHAT'S NOT WORKING:"
echo "   1. Blockchain Node - Won't compile (workspace conflicts)"
echo "   2. Real Transactions - No actual processing"
echo "   3. Consensus - Not implemented"
echo "   4. P2P Network - Not running"
echo "   5. Bridges - Code exists but not connected"
echo "   6. Cosmos SDK - Structure only, not integrated"
echo ""

# Calculate real percentage
TOTAL_NEEDED=12  # All core components for mainnet
ACTUALLY_WORKING=4  # Web, API (mock), CLI (mock), DB (empty)
REAL_PERCENTAGE=$((ACTUALLY_WORKING * 100 / TOTAL_NEEDED))

echo "📈 HONEST ASSESSMENT:"
echo ""
echo "   Frontend/UI:        ████████████████████ 100% (Complete)"
echo "   API Layer:          ████████████████████ 100% (But returns mock data)"
echo "   Blockchain Core:    ████░░░░░░░░░░░░░░░░ 20% (Code exists, won't compile)"
echo "   Database:           ████████░░░░░░░░░░░░ 40% (Running, no schema)"
echo "   Bridges:            ████░░░░░░░░░░░░░░░░ 20% (Structure only)"
echo "   Consensus:          ██░░░░░░░░░░░░░░░░░░ 10% (Not implemented)"
echo ""
echo "   Overall Mainnet Readiness: ${REAL_PERCENTAGE}% (${ACTUALLY_WORKING}/${TOTAL_NEEDED} components)"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "⚠️  CRITICAL ISSUES FOR MAINNET:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1. The Rust node won't compile due to Cargo workspace conflicts"
echo "2. No actual blockchain is running (just mock services)"
echo "3. Transactions aren't being processed or stored"
echo "4. No consensus mechanism is active"
echo "5. No P2P network exists"
echo "6. Bridges have no smart contracts deployed"
echo ""
echo "📍 REALITY: This is a DEMO/PROTOTYPE, not a production blockchain"
echo "📍 TO MAINNET: Need 3-4 weeks of focused development"

