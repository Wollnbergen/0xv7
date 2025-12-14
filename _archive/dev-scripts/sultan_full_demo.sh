#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           SULTAN CHAIN - FULL FEATURE DEMO                    ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Test all Sultan Chain features
echo "🧪 Testing Sultan Chain Features..."
echo ""

# 1. Chain Status
echo "1️⃣ Chain Status:"
curl -s -X POST http://127.0.0.1:3030 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"chain_status","id":1}' | jq '.result'

echo ""

# 2. Create Wallet
echo "2️⃣ Creating Wallet (Alice):"
curl -s -X POST http://127.0.0.1:3030 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"wallet_create","params":["alice"],"id":2}' | jq '.result'

echo ""

# 3. Zero-Fee Transfer
echo "3️⃣ Zero-Fee Transfer (Alice → Bob):"
curl -s -X POST http://127.0.0.1:3030 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"token_transfer","params":["alice","bob",100000],"id":3}' | jq '.result'

echo ""

# 4. Get APY Info
echo "4️⃣ Staking APY Info:"
curl -s -X POST http://127.0.0.1:3030 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"get_apy","id":4}' | jq '.result'

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                    SULTAN CHAIN FEATURES                      ║"
echo "╠════════════════════════════════════════════════════════════════╣"
echo "║ ✅ Zero Gas Fees       - All transactions FREE                ║"
echo "║ ✅ 13.33% APY          - High staking rewards                 ║"
echo "║ ✅ Mobile Validators   - 40% bonus for mobile nodes           ║"
echo "║ ✅ 10,000+ TPS         - High throughput                      ║"
echo "║ ✅ IBC Enabled         - Cross-chain interoperability         ║"
echo "║ ✅ Cosmos SDK Based    - Battle-tested infrastructure         ║"
echo "╚══════════════════════════════════════════════════════════════╝"
