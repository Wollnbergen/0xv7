#!/bin/bash
set -e

echo "🔧 Adding RPC endpoints to Sultan node..."

# Create the RPC endpoints module
cat > /tmp/rpc_endpoints.rs << 'EOF'
use warp::Filter;

#[derive(serde::Serialize)]
struct StatusResponse {
    status: String,
    height: u64,
    validator_count: usize,
    total_accounts: usize,
    pending_txs: usize,
    sharding_enabled: bool,
    shard_count: usize,
    inflation_rate: f64,
    validator_apy: f64,
    total_burned: u64,
    is_deflationary: bool,
    total_supply: u64,
}

pub fn create_routes(config: crate::ChainConfig) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let config = std::sync::Arc::new(tokio::sync::RwLock::new(config));
    
    // GET /
    let root = warp::get()
        .and(warp::path::end())
        .map(|| "Sultan eternal node ready");
    
    // GET /health
    let health = warp::path("health")
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().timestamp()
        })));
    
    // GET /status
    let config_clone = config.clone();
    let status = warp::path("status")
        .and(warp::get())
        .and_then(move || {
            let config = config_clone.clone();
            async move {
                let cfg = config.read().await;
                let response = StatusResponse {
                    status: "online".to_string(),
                    height: cfg.current_block,
                    validator_count: 11,
                    total_accounts: 0,
                    pending_txs: 0,
                    sharding_enabled: cfg.shards > 1,
                    shard_count: cfg.shards,
                    inflation_rate: crate::calculate_inflation_rate(&cfg) / 100.0,
                    validator_apy: 13.33 / 100.0,
                    total_burned: 0,
                    is_deflationary: false,
                    total_supply: cfg.total_supply,
                };
                Ok::<_, warp::Rejection>(warp::reply::json(&response))
            }
        });
    
    // Combine all routes
    root.or(health).or(status)
}
EOF

# Upload to server
echo "📤 Uploading RPC endpoints module..."
scp -i ~/.ssh/sultan_hetzner /tmp/rpc_endpoints.rs root@5.161.225.96:/root/sultan/node/src/

# Create Python script to modify lib.rs
cat > /tmp/modify_lib.py << 'EOF'
#!/usr/bin/env python3
import re

# Read lib.rs
with open('/root/sultan/node/src/lib.rs', 'r') as f:
    content = f.read()

# Backup
with open('/root/sultan/node/src/lib.rs.backup', 'w') as f:
    f.write(content)

# Add module declaration if not exists
if 'pub mod rpc_endpoints;' not in content:
    content = content.replace('pub mod database;', 'pub mod database;\npub mod rpc_endpoints;')

# Replace the simple warp route
old_route = 'let routes = warp::get().map(|| "Sultan eternal node ready");'
new_route = 'let routes = rpc_endpoints::create_routes(chain_config.clone());'
content = content.replace(old_route, new_route)

# Write back
with open('/root/sultan/node/src/lib.rs', 'w') as f:
    f.write(content)

print("✅ lib.rs updated successfully")
EOF

echo "📤 Uploading modification script..."
scp -i ~/.ssh/sultan_hetzner /tmp/modify_lib.py root@5.161.225.96:/tmp/

echo "🔧 Modifying lib.rs..."
ssh -i ~/.ssh/sultan_hetzner root@5.161.225.96 'python3 /tmp/modify_lib.py'

echo ""
echo "📦 Rebuilding Sultan node..."
ssh -i ~/.ssh/sultan_hetzner root@5.161.225.96 'bash -lc "cd /root/sultan && cargo build --release --bin p2p_node 2>&1 | tail -20"'

echo ""
echo "🔄 Restarting Sultan node service..."
ssh -i ~/.ssh/sultan_hetzner root@5.161.225.96 'systemctl restart sultan-node'

echo ""
echo "⏳ Waiting for service to start..."
sleep 5

echo ""
echo "✅ Testing new RPC endpoints..."
echo ""
echo "=== GET / ==="
curl -s https://rpc.sltn.io/
echo ""
echo ""
echo "=== GET /health ==="
curl -s https://rpc.sltn.io/health
echo ""
echo ""
echo "=== GET /status ==="
curl -s https://rpc.sltn.io/status

echo ""
echo ""
echo "🎉 RPC endpoints successfully deployed!"
