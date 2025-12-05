#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SULTAN CHAIN - ECONOMICS MODEL REVIEW                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "🔍 Searching for economics implementation..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check for inflation references
echo "📊 Checking inflation model..."
grep -r "inflation" /workspaces/0xv7 --include="*.rs" --include="*.js" 2>/dev/null | head -5

echo ""
echo "🔥 Checking burn mechanism..."
grep -r "burn" /workspaces/0xv7 --include="*.rs" --include="*.js" 2>/dev/null | head -5

echo ""
echo "📈 Checking APY calculations..."
grep -r "apy\|APY\|26.67" /workspaces/0xv7 --include="*.rs" --include="*.js" 2>/dev/null | head -5

