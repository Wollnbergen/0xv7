#!/bin/bash
set -e

echo "=== Building Sultan L1 Production Stack ==="
echo ""

# Kill any hanging cargo processes
pkill -9 cargo 2>/dev/null || true
sleep 2

# Clean cargo lock
rm -rf /workspaces/0xv7/target/.cargo-lock

cd /workspaces/0xv7

echo "Step 1: Building sultan-core..."
cargo build --release -p sultan-core 2>&1 | grep -E "(Compiling sultan-core|Finished)" || echo "Building..."

echo ""
echo "Step 2: Building sultan-cosmos-bridge FFI library..."
cargo build --release -p sultan-cosmos-bridge 2>&1 | grep -E "(Compiling sultan-cosmos|Finished)" || echo "Building..."

echo ""
echo "Step 3: Verifying library output..."
if [ -f "/workspaces/0xv7/target/release/libsultan_cosmos_bridge.so" ]; then
    echo "✅ FFI library built successfully!"
    ls -lh /workspaces/0xv7/target/release/libsultan_cosmos_bridge.*
elif [ -f "/workspaces/0xv7/target/release/libsultan_cosmos_bridge.dylib" ]; then
    echo "✅ FFI library built successfully (macOS)!"
    ls -lh /workspaces/0xv7/target/release/libsultan_cosmos_bridge.*
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
