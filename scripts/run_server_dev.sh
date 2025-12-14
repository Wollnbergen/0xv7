#!/usr/bin/env bash
# ./scripts/run_server_dev.sh
set -euo pipefail
export RUST_LOG="${RUST_LOG:-info}"
export SULTAN_DB_ADDR="${SULTAN_DB_ADDR:-127.0.0.1:9042}"
export SULTAN_METRICS_PORT="${SULTAN_METRICS_PORT:-9105}"
: "${SULTAN_JWT_SECRET:?Set SULTAN_JWT_SECRET or source scripts/dev_env.sh first}"
export SULTAN_JWT_ALLOW_RAW=0

pushd /workspaces/0xv7/node >/dev/null
cargo run -p sultan-coordinator --bin rpc_server
popd >/dev/null
