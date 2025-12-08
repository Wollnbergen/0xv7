#!/bin/bash
# Verify ALL production documentation for accuracy
# Catches common errors before giving to auditors

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  Sultan L1 - Documentation Accuracy Verification            ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

ERRORS=0

# Check 1: TPS Claims
echo "=== Checking TPS Claims ==="
echo "Correct values:"
echo "  • Initial: 64,000 TPS (8 shards × 8,000 TPS)"
echo "  • Maximum: 64M TPS (8,000 shards × 8,000 TPS)"
echo ""

# Search for incorrect TPS claims in production docs
WRONG_TPS=$(grep -r "1M+ TPS\|1,000,000 TPS\|million TPS" \
  SECURITY_AUDIT_GUIDE.md \
  BUILD_INSTRUCTIONS.md \
  LAUNCH_SUCCESS.md \
  SOURCE_FILE_MANIFEST.md \
  2>/dev/null | grep -v "64 million" | grep -v "BENCHMARK")

if [ -n "$WRONG_TPS" ]; then
    echo "❌ INCORRECT TPS claims found:"
    echo "$WRONG_TPS"
    ERRORS=$((ERRORS + 1))
else
    echo "✅ TPS claims are accurate"
fi
echo ""

# Check 2: File References
echo "=== Checking File References ==="
echo "Production files:"
echo "  • sharding_production.rs (NOT sharding.rs)"
echo "  • sharded_blockchain_production.rs (NOT sharded_blockchain.rs)"
echo "  • bridge_integration.rs + bridge_fees.rs (NOT bridges.rs)"
echo "  • native_dex.rs (CORRECT - not dex.rs)"
echo ""

# Look for old filenames being INCORRECTLY used as current files
# Exclude mentions in "NOT to audit" sections or where we explain old vs new
WRONG_FILES=$(grep -rE "src/sharding\.rs|src/bridges\.rs|src/dex\.rs|src/sharded_blockchain\.rs[^_]" \
  SECURITY_AUDIT_GUIDE.md \
  BUILD_INSTRUCTIONS.md \
  LAUNCH_SUCCESS.md \
  2>/dev/null | grep -v "NOT\|old\|OLD\|development")

if [ -n "$WRONG_FILES" ]; then
    echo "❌ INCORRECT file references found (old filenames used as current):"
    echo "$WRONG_FILES"
    ERRORS=$((ERRORS + 1))
else
    echo "✅ File references are accurate (all using production filenames)"
fi
echo ""

# Check 3: Build Output Location
echo "=== Checking Build Output Location ==="
echo "Correct location: /tmp/cargo-target/release/sultan-node"
echo ""

WRONG_TARGET=$(grep -r "\./target/release" \
  SECURITY_AUDIT_GUIDE.md \
  BUILD_INSTRUCTIONS.md \
  LAUNCH_SUCCESS.md \
  SOURCE_FILE_MANIFEST.md \
  2>/dev/null | grep -v "NOT\|instead\|Check.*not")

if [ -n "$WRONG_TARGET" ]; then
    echo "❌ INCORRECT build paths found (using ./target instead of /tmp/cargo-target):"
    echo "$WRONG_TARGET"
    ERRORS=$((ERRORS + 1))
else
    echo "✅ Build paths are accurate (using /tmp/cargo-target)"
fi
echo ""

# Check 4: Binary Name
echo "=== Checking Binary Name ==="
echo "Correct name: sultan-node (NOT sultan-core)"
echo ""

WRONG_BINARY=$(grep -r "sultan-core binary\|sultan_core binary" \
  SECURITY_AUDIT_GUIDE.md \
  BUILD_INSTRUCTIONS.md \
  LAUNCH_SUCCESS.md \
  SOURCE_FILE_MANIFEST.md \
  2>/dev/null)

if [ -n "$WRONG_BINARY" ]; then
    echo "❌ INCORRECT binary name references found:"
    echo "$WRONG_BINARY"
    ERRORS=$((ERRORS + 1))
else
    echo "✅ Binary name references are accurate"
fi
echo ""

# Check 5: Shard Count
echo "=== Checking Shard Configuration ==="
echo "Correct values:"
echo "  • Initial: 8 shards"
echo "  • Maximum: 8,000 shards"
echo ""

WRONG_SHARDS=$(grep -r "1024 shards" \
  SECURITY_AUDIT_GUIDE.md \
  BUILD_INSTRUCTIONS.md \
  LAUNCH_SUCCESS.md \
  SOURCE_FILE_MANIFEST.md \
  2>/dev/null)

if [ -n "$WRONG_SHARDS" ]; then
    echo "❌ INCORRECT shard count found:"
    echo "$WRONG_SHARDS"
    ERRORS=$((ERRORS + 1))
else
    echo "✅ Shard configuration is accurate"
fi
echo ""

# Check 6: All Production Files Exist
echo "=== Verifying All Referenced Files Exist ==="
./verify-source-files.sh > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✅ All referenced files exist"
else
    echo "❌ Some referenced files missing (run ./verify-source-files.sh)"
    ERRORS=$((ERRORS + 1))
fi
echo ""

# Final Report
echo "╔══════════════════════════════════════════════════════════════╗"
if [ $ERRORS -eq 0 ]; then
    echo "║  ✅ ALL DOCUMENTATION IS ACCURATE!                          ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo ""
    echo "Safe to send to auditors! 🎉"
    echo ""
    exit 0
else
    echo "║  ❌ ERRORS FOUND: $ERRORS                                       ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo ""
    echo "⚠️  FIX THESE BEFORE SENDING TO AUDITORS!"
    echo ""
    exit 1
fi
