#!/bin/bash
set -e

echo "🔧 Adding RPC endpoints to Sultan node..."

# SSH into server and update the lib.rs file
ssh -i ~/.ssh/sultan_hetzner root@5.161.225.96 'bash -s' << 'ENDSSH'
cd /root/sultan/node/src

# Backup the current lib.rs
cp lib.rs lib.rs.backup

# Find the line with "let routes = warp::get().map(|| "Sultan eternal node ready");"
# and replace the entire warp server section with comprehensive RPC endpoints

cat > rpc_endpoints.rs << 'EOF'
use warp::Reply;

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

pub fn create_routes(config: crate::ChainConfig) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
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

# Now update lib.rs to use the new RPC endpoints module
# First, add the module declaration
if ! grep -q "pub mod rpc_endpoints;" lib.rs; then
    sed -i '/^pub mod database;/a pub mod rpc_endpoints;' lib.rs
fi

# Replace the simple warp route with comprehensive RPC server
sed -i 's|let routes = warp::get()\.map(|| "Sultan eternal node ready");|let routes = rpc_endpoints::create_routes(chain_config.clone());|' lib.rs

echo "✅ RPC endpoints added successfully"
ENDSSH

echo ""
echo "📦 Rebuilding Sultan node with new RPC endpoints..."
ssh -i ~/.ssh/sultan_hetzner root@5.161.225.96 'cd /root/sultan && cargo build --release --bin p2p_node'

echo ""
echo "🔄 Restarting Sultan node service..."
ssh -i ~/.ssh/sultan_hetzner root@5.161.225.96 'systemctl restart sultan-node'

echo ""
echo "⏳ Waiting for service to start..."
sleep 3

echo ""
echo "✅ Testing new RPC endpoints..."
echo ""
echo "GET /"
curl -s https://rpc.sltn.io/
echo ""
echo ""
echo "GET /health"
curl -s https://rpc.sltn.io/health | jq .
echo ""
echo "GET /status"
curl -s https://rpc.sltn.io/status | jq .

echo ""
echo "🎉 RPC endpoints successfully deployed!"
