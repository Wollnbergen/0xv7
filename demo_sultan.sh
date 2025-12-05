#!/bin/bash
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║              SULTAN BLOCKCHAIN LIVE DEMO                      ║"
echo "╚══════════════════════════════════════════════════════════════╝"

echo -e "\n1️⃣ QUERYING SULTAN ECONOMICS..."
curl -s http://localhost:3030 | jq '.'

echo -e "\n2️⃣ QUERYING UNIFIED INTEGRATION..."
curl -s http://localhost:8080/status | jq '.unified_features'

echo -e "\n3️⃣ COMPARING WITH STANDARD COSMOS:"
echo "┌─────────────────────┬──────────────┬──────────────┐"
echo "│ Feature             │ Sultan       │ Standard     │"
echo "├─────────────────────┼──────────────┼──────────────┤"
echo "│ Staking APY         │ 26.67%       │ 7%           │"
echo "│ Gas Fees            │ $0.00        │ Variable     │"
echo "│ IBC Support         │ ✅           │ ✅           │"
echo "│ Smart Contracts     │ ✅           │ ✅           │"
echo "│ Target TPS          │ 1,230,000    │ 10,000       │"
echo "└─────────────────────┴──────────────┴──────────────┘"

echo -e "\n🎉 Sultan maintains superior economics with Cosmos infrastructure!"
