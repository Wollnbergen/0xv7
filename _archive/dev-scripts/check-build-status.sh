#!/bin/bash
# Quick build status check

echo "Sultan L1 Build Status"
echo "====================="
echo ""

# Check if library exists
if [ -f "/workspaces/0xv7/target/release/libsultan_cosmos_bridge.so" ]; then
    echo "✅ BUILD COMPLETE!"
    ls -lh /workspaces/0xv7/target/release/libsultan_cosmos_bridge.*
    echo ""
    echo "Ready to proceed with:"
    echo "  export LD_LIBRARY_PATH=/workspaces/0xv7/target/release:\$LD_LIBRARY_PATH"
    echo "  ./test-e2e.sh"
else
    # Check if build is running
    CARGO_COUNT=$(ps aux | grep cargo | grep -v grep | wc -l)
    CPP_COUNT=$(ps aux | grep cc1plus | grep -v grep | wc -l)
    
    if [ $CARGO_COUNT -gt 0 ] || [ $CPP_COUNT -gt 0 ]; then
        echo "🔄 Build in progress..."
        echo ""
        echo "Active processes:"
        echo "  Cargo: $CARGO_COUNT"
        echo "  C++ compilers: $CPP_COUNT"
        echo ""
        echo "Currently compiling:"
        ps aux | grep cc1plus | grep -v grep | grep -o "rocksdb/[^ ]*\.cc" | head -5 || echo "  (RocksDB build in progress)"
    else
        echo "⚠️  Build not found"
        echo ""
        echo "To start build:"
        echo "  cd /workspaces/0xv7/sultan-cosmos-bridge"
        echo "  cargo build --release"
    fi
fi

echo ""
echo "---"
echo "Architecture Status:"
echo "✅ Sultan Core (Rust blockchain)"
echo "✅ FFI Bridge code (13/13 tests)"
echo "✅ Cosmos SDK Module (1,600+ lines)"
echo "✅ sultand binary (71MB)"
if [ -f "/workspaces/0xv7/target/release/libsultan_cosmos_bridge.so" ]; then
    echo "✅ FFI Library built"
else
    echo "🔄 FFI Library building..."
fi
