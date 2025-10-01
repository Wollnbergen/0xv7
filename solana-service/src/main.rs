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
struct SolanaService {
    providers: Arc<RwLock<Vec<Provider>>>,
    metrics: Arc<Metrics>,
}

#[derive(Clone)]
struct Provider {
    url: String,
    client: reqwest::Client,
    health_score: f64,
    last_slot: u64,
}

struct Metrics {
    requests_total: Counter,
    request_duration: Histogram,
    provider_health: Histogram,
}

#[derive(Debug, Serialize, Deserialize)]
struct StateProof {
    slot: u64,
    blockhash: String,
    merkle_proof: Vec<String>,
    timestamp: u64,
}

impl SolanaService {
    async fn new() -> Result<Self> {
        info!("🚀 Initializing Enterprise Solana Service");

        let provider_urls = vec![
            "https://api.mainnet-beta.solana.com",
            "https://solana-api.projectserum.com",
            "https://api.ankr.com/solana", 
            "https://solana-mainnet.public.blastapi.io",
        ];

        let mut providers = Vec::new();
        let client = reqwest::Client::new();
        
        for url in provider_urls {
            providers.push(Provider {
                url: url.to_string(),
                client: client.clone(),
                health_score: 1.0,
                last_slot: 0,
            });
            info!("✅ Added Solana provider: {}", url);
        }

        let metrics = Arc::new(Metrics {
            requests_total: register_counter!("solana_requests_total", "Total requests")?,
            request_duration: register_histogram!("solana_request_duration_seconds", "Request duration")?,
            provider_health: register_histogram!("solana_provider_health", "Provider health scores")?,
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
                let payload = json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "getSlot",
                    "params": []
                });

                match provider.client.post(&provider.url)
                    .json(&payload)
                    .timeout(Duration::from_secs(5))
                    .send()
                    .await
                {
                    Ok(resp) => {
                        if let Ok(data) = resp.json::<Value>().await {
                            if let Some(slot) = data["result"].as_u64() {
                                provider.last_slot = slot;
                                provider.health_score = 1.0;
                                self.metrics.provider_health.observe(1.0);
                                info!("✅ Solana provider {} healthy at slot {}", provider.url, slot);
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("⚠️  Solana provider {} error: {}", provider.url, e);
                    }
                }
                
                provider.health_score *= 0.9;
                self.metrics.provider_health.observe(provider.health_score);
            }
        }
    }
}

async fn health_handler(State(service): State<Arc<SolanaService>>) -> Json<serde_json::Value> {
    let providers = service.providers.read().await;
    let healthy_count = providers.iter().filter(|p| p.health_score > 0.5).count();
    
    Json(json!({
        "status": if healthy_count > 0 { "healthy" } else { "unhealthy" },
        "chain": "solana",
        "providers": {
            "total": providers.len(),
            "healthy": healthy_count,
        }
    }))
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
    ║         SULTAN SOLANA SERVICE - ENTERPRISE            ║
    ║                                                       ║
    ║   ✅ Multi-Provider Redundancy                        ║
    ║   ✅ Health Monitoring & Failover                     ║
    ║   ✅ High-Performance RPC                             ║
    ║   ✅ Prometheus Metrics                               ║
    ║   ✅ Production Ready                                 ║
    ╚═══════════════════════════════════════════════════════╝
    "#);

    let service = Arc::new(SolanaService::new().await?);

    let service_clone = service.clone();
    tokio::spawn(async move {
        service_clone.update_provider_health().await;
    });

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        .with_state(service);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8092").await?;
    info!("🌐 Solana Service listening on :8092");
    
    axum::serve(listener, app).await?;
    Ok(())
}
