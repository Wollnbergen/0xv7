use anyhow::Result;
use axum::{extract::State, response::Json, routing::{get, post}, Router};
use prometheus::{Encoder, TextEncoder, Counter, Histogram, register_counter, register_histogram};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, warn};
use serde_json::{json, Value};

#[derive(Clone)]
struct TonService {
    providers: Arc<RwLock<Vec<Provider>>>,
    metrics: Arc<Metrics>,
}

#[derive(Clone)]
struct Provider {
    url: String,
    client: reqwest::Client,
    health_score: f64,
    last_seqno: u64,
    network: String,
}

struct Metrics {
    requests_total: Counter,
    request_duration: Histogram,
    provider_health: Histogram,
}

#[derive(Debug, Serialize, Deserialize)]
struct StateProof {
    workchain: i32,
    shard: i64,
    seqno: u64,
    root_hash: String,
    file_hash: String,
    timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct VerifyRequest {
    workchain: i32,
    shard: i64,
    seqno: u64,
    root_hash: String,
}

impl TonService {
    async fn new() -> Result<Self> {
        info!("🚀 Initializing Enterprise TON Service");

        // Multiple TON providers for redundancy
        let provider_configs = vec![
            ("https://toncenter.com/api/v2", "mainnet"),
            ("https://mainnet.tonhubapi.com", "mainnet"),
            ("https://toncenter.com/api/v2", "mainnet"), // Duplicate for redundancy
            ("https://api.ton.cat/v2", "mainnet"),
        ];

        let mut providers = Vec::new();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;
        
        for (url, network) in provider_configs {
            providers.push(Provider {
                url: url.to_string(),
                client: client.clone(),
                health_score: 1.0,
                last_seqno: 0,
                network: network.to_string(),
            });
            info!("✅ Added TON provider: {} ({})", url, network);
        }

        let metrics = Arc::new(Metrics {
            requests_total: register_counter!("ton_requests_total", "Total requests")?,
            request_duration: register_histogram!("ton_request_duration_seconds", "Request duration")?,
            provider_health: register_histogram!("ton_provider_health", "Provider health scores")?,
        });

        Ok(Self {
            providers: Arc::new(RwLock::new(providers)),
            metrics,
        })
    }

    async fn get_best_provider(&self) -> Option<Provider> {
        let providers = self.providers.read().await;
        providers
            .iter()
            .filter(|p| p.health_score > 0.5)
            .max_by(|a, b| a.health_score.partial_cmp(&b.health_score).unwrap())
            .cloned()
    }

    async fn update_provider_health(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            let mut providers = self.providers.write().await;
            
            for provider in providers.iter_mut() {
                // TON uses different API structure
                let url = format!("{}/getMasterchainInfo", provider.url);
                
                match provider.client.get(&url)
                    .timeout(Duration::from_secs(5))
                    .send()
                    .await
                {
                    Ok(resp) => {
                        if let Ok(data) = resp.json::<Value>().await {
                            if data.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                                if let Some(result) = data.get("result") {
                                    if let Some(seqno) = result.get("last").and_then(|l| l.get("seqno")).and_then(|s| s.as_u64()) {
                                        provider.last_seqno = seqno;
                                        provider.health_score = 1.0;
                                        self.metrics.provider_health.observe(1.0);
                                        info!("✅ TON provider {} healthy at seqno {}", provider.url, seqno);
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("⚠️  TON provider {} error: {}", provider.url, e);
                    }
                }
                
                provider.health_score *= 0.9;
                self.metrics.provider_health.observe(provider.health_score);
            }
        }
    }

    async fn get_masterchain_info(&self) -> Result<StateProof> {
        let _timer = self.metrics.request_duration.start_timer();
        self.metrics.requests_total.inc();

        if let Some(provider) = self.get_best_provider().await {
            info!("🔍 Getting masterchain info from {}", provider.url);
            
            let url = format!("{}/getMasterchainInfo", provider.url);
            let resp = provider.client.get(&url).send().await?;
            let data: Value = resp.json().await?;
            
            if data.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                if let Some(result) = data.get("result") {
                    let last = result.get("last").ok_or_else(|| anyhow::anyhow!("No last block"))?;
                    
                    Ok(StateProof {
                        workchain: -1, // Masterchain
                        shard: last.get("shard").and_then(|s| s.as_str()).unwrap_or("8000000000000000").parse::<u64>().unwrap_or(0) as i64,
                        seqno: last.get("seqno").and_then(|s| s.as_u64()).unwrap_or(0),
                        root_hash: last.get("root_hash").and_then(|h| h.as_str()).unwrap_or("").to_string(),
                        file_hash: last.get("file_hash").and_then(|h| h.as_str()).unwrap_or("").to_string(),
                        timestamp: result.get("last_utime").and_then(|t| t.as_u64()).unwrap_or(0),
                    })
                } else {
                    Err(anyhow::anyhow!("Invalid response format"))
                }
            } else {
                Err(anyhow::anyhow!("API returned error"))
            }
        } else {
            Err(anyhow::anyhow!("No healthy providers available"))
        }
    }
}

async fn health_handler(State(service): State<Arc<TonService>>) -> Json<serde_json::Value> {
    let providers = service.providers.read().await;
    let healthy_count = providers.iter().filter(|p| p.health_score > 0.5).count();
    
    Json(json!({
        "status": if healthy_count > 0 { "healthy" } else { "unhealthy" },
        "chain": "ton",
        "providers": {
            "total": providers.len(),
            "healthy": healthy_count,
        }
    }))
}

async fn verify_handler(
    State(service): State<Arc<TonService>>,
    Json(request): Json<VerifyRequest>,
) -> Json<serde_json::Value> {
    match service.get_masterchain_info().await {
        Ok(info) => Json(json!({
            "verified": info.root_hash == request.root_hash && info.seqno == request.seqno,
            "current_state": info,
        })),
        Err(e) => Json(json!({
            "verified": false,
            "error": e.to_string(),
        })),
    }
}

async fn masterchain_handler(State(service): State<Arc<TonService>>) -> Json<serde_json::Value> {
    match service.get_masterchain_info().await {
        Ok(info) => Json(json!({
            "success": true,
            "masterchain": info,
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": e.to_string(),
        })),
    }
}

async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!(r#"
    ╔═══════════════════════════════════════════════════════╗
    ║           SULTAN TON SERVICE - ENTERPRISE             ║
    ║                                                       ║
    ║   ✅ Multi-Provider Redundancy                        ║
    ║   ✅ Health Monitoring & Failover                     ║
    ║   ✅ Masterchain State Verification                   ║
    ║   ✅ Prometheus Metrics                               ║
    ║   ✅ Production Ready                                 ║
    ╚═══════════════════════════════════════════════════════╝
    "#);

    let service = Arc::new(TonService::new().await?);

    // Start health monitoring
    let service_clone = service.clone();
    tokio::spawn(async move {
        service_clone.update_provider_health().await;
    });

    // Build router
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/verify", post(verify_handler))
        .route("/masterchain", get(masterchain_handler))
        .route("/metrics", get(metrics_handler))
        .with_state(service);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8093").await?;
    info!("🌐 TON Service listening on :8093");
    
    axum::serve(listener, app).await?;
    Ok(())
}
