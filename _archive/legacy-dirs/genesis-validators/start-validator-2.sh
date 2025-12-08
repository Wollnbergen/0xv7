#!/bin/bash
# Start Sultan L1 Validator #2

BINARY="/tmp/cargo-target/release/sultan-node"

$BINARY \
  --name "genesis-validator-2" \
  --validator \
  --validator-address "sultan1validator2ed4a1f647349315859e9fcfca321ed7de" \
  --validator-stake "10000000000000" \
  --enable-sharding \
  --shard-count 100 \
  --tx-per-shard 10000 \
  --block-time 2 \
  --data-dir "/workspaces/0xv7/data/validator-2" \
  --rpc-addr "0.0.0.0:26658" \
  --p2p-addr "0.0.0.0:26657"
