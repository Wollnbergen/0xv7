#!/bin/bash

echo "Testing Smart Contract Functions..."

# Query all stored codes
echo "📝 Stored Contract Codes:"
docker exec cosmos-node wasmd query wasm list-code --output json 2>/dev/null | \
    jq '.code_infos[] | {id: .code_id, creator: .creator}' 2>/dev/null || \
    echo "No contracts deployed yet"

# Query instantiated contracts
echo ""
echo "📦 Instantiated Contracts:"
docker exec cosmos-node wasmd query wasm list-contract-by-code 1 --output json 2>/dev/null | \
    jq '.contracts[]' 2>/dev/null || \
    echo "No contracts instantiated yet"

echo ""
echo "✅ Contract system ready for deployment!"
