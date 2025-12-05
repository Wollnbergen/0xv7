#!/bin/bash

# Sultan Chain - Direct RPC Examples (No API Key!)

# Get latest block
curl -X POST https://rpc.sltn.io \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "get_latest_block",
    "params": [],
    "id": 1
  }'

# Send transaction (Zero fees!)
curl -X POST https://rpc.sltn.io \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "send_transaction",
    "params": {
      "from": "sultan1abc...",
      "to": "sultan1xyz...",
      "amount": "1000"
    },
    "id": 2
  }'

# Check balance
curl -X POST https://rpc.sltn.io \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "get_balance",
    "params": ["sultan1..."],
    "id": 3
  }'
