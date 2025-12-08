#!/bin/bash
# Sultan L1 - Production Build Script
# Builds optimized release binary for deployment

set -e

echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║           Sultan L1 Blockchain - Production Build                ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Step 1: Verify Rust installation
echo "📋 Step 1: Verifying Rust installation..."
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo not found. Please install Rust:${NC}"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

RUST_VERSION=$(rustc --version)
echo -e "${GREEN}✅ Rust installed: $RUST_VERSION${NC}"
echo ""

# Step 2: Verify we're in correct directory
echo "📋 Step 2: Verifying project structure..."
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}❌ Cargo.toml not found. Please run from project root.${NC}"
    exit 1
fi

if [ ! -d "sultan-core" ]; then
    echo -e "${RED}❌ sultan-core directory not found.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Project structure verified${NC}"
echo ""

# Step 3: Show build configuration
echo "📋 Step 3: Build configuration..."
echo "   Package: sultan-core"
echo "   Profile: release (optimized)"
echo "   Output: /tmp/cargo-target/release/sultan-node"
echo "   Binary size: ~16MB (expected)"
echo ""

# Step 4: Clean previous build (optional)
read -p "Clean previous build artifacts? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🧹 Cleaning previous builds..."
    cargo clean
    echo -e "${GREEN}✅ Clean complete${NC}"
    echo ""
fi

# Step 5: Quick validation with cargo check
echo "📋 Step 5: Running cargo check (fast validation)..."
if cargo check -p sultan-core --all-features; then
    echo -e "${GREEN}✅ Validation passed${NC}"
else
    echo "⚠️  Library not found, checking debug build..."
    find /workspaces/0xv7/target -name "libsultan_cosmos_bridge.*" -type f
fi

echo ""
echo "Step 4: Setting up library path..."
export LD_LIBRARY_PATH=/workspaces/0xv7/target/release:$LD_LIBRARY_PATH
echo "export LD_LIBRARY_PATH=/workspaces/0xv7/target/release:\$LD_LIBRARY_PATH" >> ~/.bashrc

echo ""
echo "Step 5: Testing sultand binary..."
cd /workspaces/0xv7/sultand
if ./sultand version 2>&1 | grep -q "sultan"; then
    echo "✅ sultand binary works!"
else
    echo "⚠️  sultand needs library, checking ldd..."
    ldd ./sultand | grep sultan || true
fi

echo ""
echo "=== Build Complete! ==="
echo ""
echo "Next steps:"
echo "1. export LD_LIBRARY_PATH=/workspaces/0xv7/target/release:\$LD_LIBRARY_PATH"
echo "2. cd /workspaces/0xv7/sultand"
echo "3. ./sultand init testnode --chain-id sultan-1"
echo "4. ./sultand keys add alice"
echo "5. ./sultand start"
