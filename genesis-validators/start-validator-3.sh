#!/bin/bash
# Start Sultan L1 Validator #3

BINARY="/tmp/cargo-target/release/sultan-node"

$BINARY \
  --name "genesis-validator-3" \
  --validator \
  --validator-address "sultan1validator37b1ba9ca57cb2855402c3c9594f9df653" \
  --validator-stake "10000000000000" \
  --enable-sharding \
  --shard-count 100 \
  --tx-per-shard 10000 \
  --block-time 2 \
  --data-dir "/workspaces/0xv7/data/validator-3" \
  --rpc-addr "0.0.0.0:26659" \
  --p2p-addr "0.0.0.0:26658"
