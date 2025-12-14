#!/bin/bash
# Verify all source files mentioned in documentation exist

echo "Verifying Sultan L1 source files..."
echo ""

ERRORS=0

check_file() {
    if [ -f "$1" ]; then
        echo "✅ $1"
    else
        echo "❌ MISSING: $1"
        ERRORS=$((ERRORS + 1))
    fi
}

echo "=== Core Files (Priority 1) ==="
check_file "sultan-core/src/consensus.rs"
check_file "sultan-core/src/blockchain.rs"
check_file "sultan-core/src/staking.rs"
check_file "sultan-core/src/transaction_validator.rs"

echo ""
echo "=== Economic Files (Priority 2) ==="
check_file "sultan-core/src/governance.rs"
check_file "sultan-core/src/bridge_integration.rs"
check_file "sultan-core/src/bridge_fees.rs"
check_file "sultan-core/src/native_dex.rs"
check_file "sultan-core/src/token_factory.rs"
check_file "sultan-core/src/economics.rs"

echo ""
echo "=== Infrastructure Files (Priority 3) ==="
check_file "sultan-core/src/sharding_production.rs"
check_file "sultan-core/src/sharded_blockchain_production.rs"
check_file "sultan-core/src/main.rs"
check_file "sultan-core/src/config.rs"
check_file "sultan-core/src/database.rs"
check_file "sultan-core/src/storage.rs"
check_file "sultan-core/src/p2p.rs"
check_file "sultan-core/src/quantum.rs"

echo ""
echo "=== Supporting Files ==="
check_file "sultan-core/src/types.rs"
check_file "sultan-core/src/lib.rs"

echo ""
echo "=== Documentation ==="
check_file "BUILD_INSTRUCTIONS.md"
check_file "SECURITY_AUDIT_GUIDE.md"
check_file "SOURCE_FILE_MANIFEST.md"
check_file "LAUNCH_SUCCESS.md"

echo ""
echo "=== Build Configuration ==="
check_file ".cargo/config.toml"
check_file "Cargo.toml"
check_file "sultan-core/Cargo.toml"

echo ""
if [ $ERRORS -eq 0 ]; then
    echo "✅ All files verified! Documentation is accurate."
else
    echo "❌ $ERRORS files missing! Documentation needs updates."
    exit 1
fi
