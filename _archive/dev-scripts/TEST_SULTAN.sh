#!/bin/bash
echo "🧪 Testing Sultan Chain Features..."
cd /workspaces/0xv7/node

echo "1. Testing Zero-Fee Transactions..."
if [ -f target/release/production_test ]; then
    timeout 5 ./target/release/production_test || true
else
    cargo test --release -- --nocapture 2>&1 | head -20
fi

echo ""
echo "2. Testing SDK Demo..."
if [ -f target/release/sdk_demo ]; then
    timeout 5 ./target/release/sdk_demo || true
fi

echo ""
echo "3. Testing RPC Connection..."
curl -s http://localhost:3030/health 2>/dev/null && echo "✅ RPC Server responding!" || echo "⚠️ RPC Server not yet running"
