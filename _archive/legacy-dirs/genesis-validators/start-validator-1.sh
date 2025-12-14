#!/bin/bash
# Start Sultan L1 Validator #1

BINARY="/tmp/cargo-target/release/sultan-node"

$BINARY \
  --name "genesis-validator-1" \
  --validator \
  --validator-address "sultan1validator1933ee28148dc2bc9bc862d7488d5a21e4" \
  --validator-stake "10000000000000" \
  --enable-sharding \
  --shard-count 100 \
  --tx-per-shard 10000 \
  --block-time 2 \
  --data-dir "/workspaces/0xv7/data/validator-1" \
  --rpc-addr "0.0.0.0:26657" \
  --p2p-addr "0.0.0.0:26656"
