#!/bin/bash
# IBC Relayer Setup Script
# Creates relayer keys and establishes IBC connections

set -e

HERMES="docker exec sultan-ibc-relayer hermes"

echo "🔗 Setting up IBC Relayer for Sultan L1"
echo "========================================"

# Create relayer keys
echo ""
echo "1️⃣  Creating relayer keys..."
$HERMES keys add --chain sultan-1 --mnemonic-file /dev/stdin <<EOF
${SULTAN_MNEMONIC:-word1 word2 word3 ... word24}
EOF

$HERMES keys add --chain osmosis-1 --mnemonic-file /dev/stdin <<EOF
${OSMOSIS_MNEMONIC:-word1 word2 word3 ... word24}
EOF

$HERMES keys add --chain cosmoshub-4 --mnemonic-file /dev/stdin <<EOF
${COSMOSHUB_MNEMONIC:-word1 word2 word3 ... word24}
EOF

# Create IBC clients
echo ""
echo "2️⃣  Creating IBC clients..."
$HERMES create client --host-chain sultan-1 --reference-chain osmosis-1
$HERMES create client --host-chain sultan-1 --reference-chain cosmoshub-4
$HERMES create client --host-chain sultan-1 --reference-chain juno-1

# Create IBC connections
echo ""
echo "3️⃣  Establishing IBC connections..."
$HERMES create connection --a-chain sultan-1 --b-chain osmosis-1
$HERMES create connection --a-chain sultan-1 --b-chain cosmoshub-4
$HERMES create connection --a-chain sultan-1 --b-chain juno-1

# Create transfer channels
echo ""
echo "4️⃣  Creating transfer channels..."
$HERMES create channel --a-chain sultan-1 --a-connection connection-0 --a-port transfer --b-port transfer
$HERMES create channel --a-chain sultan-1 --a-connection connection-1 --a-port transfer --b-port transfer
$HERMES create channel --a-chain sultan-1 --a-connection connection-2 --a-port transfer --b-port transfer

echo ""
echo "✅ IBC relayer setup complete!"
echo ""
echo "Connected chains:"
echo "  • Sultan L1 ↔ Osmosis (DEX)"
echo "  • Sultan L1 ↔ Cosmos Hub"
echo "  • Sultan L1 ↔ Juno (Smart Contracts)"
echo ""
echo "Starting relayer..."
$HERMES start
