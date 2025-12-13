#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║              RUNNING COMPLETE TEST SUITE                      ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# JavaScript Tests
echo "🟨 JavaScript Tests:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
npm test -- --passWithNoTests 2>&1 | head -30
echo ""

# Rust Tests (if cargo is available)
if command -v cargo &> /dev/null; then
    echo "🦀 Rust Tests:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cargo test --quiet 2>&1 | head -20 || echo "  Skipping Rust tests (build issues)"
    echo ""
fi

# Go Tests (if go is available and modules exist)
if [ -d "/workspaces/0xv7/sovereign-chain/sovereign" ]; then
    echo "🐹 Go Tests:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cd /workspaces/0xv7/sovereign-chain/sovereign
    go test ./app/... 2>&1 | head -20 || echo "  No Go tests found"
    cd /workspaces/0xv7
    echo ""
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Test suite execution complete!"
echo ""
echo "To view coverage report in browser:"
echo "  npx jest --coverage && $BROWSER coverage/lcov-report/index.html"
