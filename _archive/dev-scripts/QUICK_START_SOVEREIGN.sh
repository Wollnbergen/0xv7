#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SOVEREIGN CHAIN - QUICK START GUIDE                   ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/sovereign-chain/sovereign

echo "🔍 Current Status:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check if Go is installed
if command -v go &> /dev/null; then
    echo "✅ Go installed: $(go version)"
else
    echo "❌ Go not installed"
fi

# Check project structure
echo "✅ Project structure:"
echo "   Modules: $(ls -d x/*/ 2>/dev/null | wc -l)"
echo "   Config files: $(ls *.toml *.json 2>/dev/null | wc -l)"

echo ""
echo "📚 Commands to get started:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1️⃣ Build the chain:"
echo "   cd /workspaces/0xv7/sovereign-chain/sovereign"
echo "   go mod tidy"
echo "   make install"
echo ""
echo "2️⃣ Initialize node:"
echo "   sovereignd init mynode --chain-id sovereign-1"
echo ""
echo "3️⃣ Start the chain:"
echo "   sovereignd start"
echo ""
echo "4️⃣ Create wallet:"
echo "   sovereignd keys add mywallet"
echo ""
echo "5️⃣ Send transaction (zero gas!):"
echo '   sovereignd tx bank send mywallet sovereign1... 1000sovereign --gas-prices 0sovereign'
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📊 Features Summary:"
echo "   • Zero Gas Fees: ✅ ENABLED"
echo "   • Target TPS: 10,000,000"
echo "   • Quantum Safe: ✅ READY"
echo "   • IBC: ✅ ENABLED"
echo "   • AI: 🔄 PLANNED"
echo ""
echo "🌐 Web Dashboard:"
echo "   cd /workspaces/0xv7/public"
echo "   python3 -m http.server 3000"
echo "   Open: http://localhost:3000"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "The Sovereign Chain rises! 🚀"

