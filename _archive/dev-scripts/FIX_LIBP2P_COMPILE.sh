#!/bin/bash

echo "🔧 Fixing libp2p-noise compilation error..."

# Update to use a compatible libp2p version that doesn't have the type annotation issue
cd /workspaces/0xv7/node

cat > Cargo.toml << 'TOML'
[package]
name = "sultan-coordinator"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.35", features = ["full"] }
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rand = "0.8"
chrono = "0.4"
uuid = { version = "1.6", features = ["v4", "serde"] }
log = "0.4"
env_logger = "0.11"
futures = "0.3"
async-trait = "0.1"
sha2 = "0.10"
hex = "0.4"
lazy_static = "1.4"
tracing = "0.1"
tracing-subscriber = "0.3"
tonic = "0.12"
tokio-stream = "0.1"
prost = "0.13"
prost-types = "0.13"
rocksdb = "0.21"
jsonwebtoken = "9.2"

# Use older libp2p that compiles correctly
libp2p = "0.38"

# Optional dependencies
scylla = { version = "0.13", optional = true }

[features]
default = []
with-scylla = ["scylla"]

[[bin]]
name = "sultan_node"
path = "src/bin/sultan_node.rs"

[[bin]]
name = "rpc_server"
path = "src/bin/rpc_server.rs"

[build-dependencies]
tonic-build = "0.12"
TOML

echo "🔨 Rebuilding with libp2p 0.38..."
cd /workspaces/0xv7
cargo clean -p libp2p-noise
cargo build --package sultan-coordinator 2>&1 | grep -E "Compiling|Finished|error" | head -10

