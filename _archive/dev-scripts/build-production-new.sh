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

# Step 4: Build release binary
echo "📋 Step 4: Building release binary..."
echo -e "${YELLOW}⏱️  This will take 10-15 minutes for first build...${NC}"
echo ""

BUILD_START=$(date +%s)

if cargo build --release -p sultan-core; then
    BUILD_END=$(date +%s)
    BUILD_TIME=$((BUILD_END - BUILD_START))
    echo ""
    echo -e "${GREEN}✅ Build completed in ${BUILD_TIME}s${NC}"
else
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi
echo ""

# Step 5: Verify binary exists
echo "📋 Step 5: Verifying binary output..."
BINARY_PATH="/tmp/cargo-target/release/sultan-node"

if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}❌ Binary not found at $BINARY_PATH${NC}"
    echo "   This is unusual. Check build logs above."
    exit 1
fi

BINARY_SIZE=$(ls -lh "$BINARY_PATH" | awk '{print $5}')
echo -e "${GREEN}✅ Binary exists: $BINARY_SIZE${NC}"
echo ""

# Step 6: Test binary execution
echo "📋 Step 6: Testing binary execution..."
if "$BINARY_PATH" --help > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Binary is executable${NC}"
else
    echo -e "${RED}❌ Binary execution failed${NC}"
    exit 1
fi
echo ""

# Final summary
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║                    🎉 BUILD COMPLETE!                             ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""
echo "📦 Binary Location:"
echo "   $BINARY_PATH"
echo ""
echo "🚀 Next Steps:"
echo ""
echo "   1. Test locally:"
echo "      $BINARY_PATH --help"
echo ""
echo "   2. Run node:"
echo "      $BINARY_PATH --validator --enable-sharding --shard-count 8"
echo ""
echo "   3. Deploy to production:"
echo "      ./DEPLOY_TO_HETZNER.sh"
echo ""
echo "📚 Documentation:"
echo "   - Build instructions: BUILD_INSTRUCTIONS.md"
echo "   - Security audit guide: SECURITY_AUDIT_GUIDE.md"
echo "   - Deployment guide: DEPLOYMENT_PLAN.md"
echo ""
echo "✅ Ready for deployment!"
echo ""
