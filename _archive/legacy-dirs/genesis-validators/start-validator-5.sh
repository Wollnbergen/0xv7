#!/bin/bash
# Start Sultan L1 Validator #5

BINARY="/tmp/cargo-target/release/sultan-node"

$BINARY \
  --name "genesis-validator-5" \
  --validator \
  --validator-address "sultan1validatorb0c419c918ea8126a205ecfb3430b0a64b" \
  --validator-stake "10000000000000" \
  --enable-sharding \
  --shard-count 100 \
  --tx-per-shard 10000 \
  --block-time 2 \
  --data-dir "/workspaces/0xv7/data/validator-5" \
  --rpc-addr "0.0.0.0:26661" \
  --p2p-addr "0.0.0.0:26660"
