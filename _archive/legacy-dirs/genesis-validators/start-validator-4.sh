#!/bin/bash
# Start Sultan L1 Validator #4

BINARY="/tmp/cargo-target/release/sultan-node"

$BINARY \
  --name "genesis-validator-4" \
  --validator \
  --validator-address "sultan1validator6439f3cc4df6ebaaeb6fffb80cb2dc790e" \
  --validator-stake "10000000000000" \
  --enable-sharding \
  --shard-count 100 \
  --tx-per-shard 10000 \
  --block-time 2 \
  --data-dir "/workspaces/0xv7/data/validator-4" \
  --rpc-addr "0.0.0.0:26660" \
  --p2p-addr "0.0.0.0:26659"
