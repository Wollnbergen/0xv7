#!/bin/bash

clear
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SULTAN CHAIN NODE - COMPLETE STRUCTURE                ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "📂 NODE FOLDER STRUCTURE:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
tree /workspaces/0xv7/node -L 3 -I "target|node_modules" | head -50

echo ""
echo "📊 SRC MODULES (Your Core Implementation):"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
ls -la /workspaces/0xv7/node/src/*.rs | awk '{print $NF " - " $5 " bytes"}' | column -t

echo ""
echo "🚀 BIN FILES (Your Executables):"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
ls -la /workspaces/0xv7/node/src/bin/*.rs | awk '{print $NF " - " $5 " bytes"}' | column -t

echo ""
echo "🔍 CHECKING SULTAN-INTEROP BRIDGES:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -d "/workspaces/0xv7/sultan-interop" ]; then
    tree /workspaces/0xv7/sultan-interop -L 2
fi

echo ""
echo "📝 CHECKING BUILD STATUS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
cd /workspaces/0xv7/node
if [ -f "Cargo.toml" ]; then
    echo "Cargo.toml workspace configuration:"
    grep -E "^\[package\]|^name|^version|^\[\[bin\]\]" Cargo.toml | head -20
fi

echo ""
echo "🛠️ BUILT BINARIES:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -d "target/debug" ]; then
    ls -lh target/debug/ | grep -E "sultan|rpc|wallet|production|rpcd" | grep -v ".d$" | head -10
fi
if [ -d "target/release" ]; then
    echo "Release builds:"
    ls -lh target/release/ | grep -E "sultan|rpc|wallet|production" | grep -v ".d$" | head -10
fi

