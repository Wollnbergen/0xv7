use std::sync::Mutex;
use dotenvy;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use jsonrpc_core::{
    Error as RpcError, ErrorCode as RpcErrorCode, Metadata, MetaIoHandler, Params, Value,
};
use jsonrpc_http_server::ServerBuilder;
use jsonrpc_http_server::hyper; // use re-exported hyper from jsonrpc_http_server
use lazy_static::lazy_static;
use prometheus::{register_int_counter, Encoder, IntCounter, IntGauge, Registry, TextEncoder};
use redis;
use redis::Commands; // bring Redis command trait into scope
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio::runtime::Runtime;
use tracing::{error, info, warn};

// SDK (use the lib crate root)
use crate::ChainConfig;
use crate::sdk::SultanSDK;

// Globals

lazy_static! {
    static ref SDK: Mutex<Option<SultanSDK>> = Mutex::new(None);
    static ref REDIS: Option<redis::Client> = {
        let url = std::env::var("SULTAN_REDIS_URL").ok();
        match url {
            Some(u) if !u.is_empty() => match redis::Client::open(u) {
                Ok(c) => Some(c),
                Err(e) => {
                    warn!("Redis disabled (connect failed): {:?}", e);
                    None
                }
            },
            _ => None,
        }
    };
    static ref RATE_LIMIT_RPS: usize = std::env::var("SULTAN_RATE_LIMIT_RPS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);
    static ref RATE_LIMIT_WINDOW_SECS: usize = std::env::var("SULTAN_RATE_LIMIT_WINDOW_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    static ref IDEMP_TTL_SECS: usize = std::env::var("SULTAN_IDEMP_TTL_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(600);
}

// Prometheus metrics
lazy_static! {
    static ref REGISTRY: Registry = Registry::new();
    static ref RPC_CALLS: IntCounter =
        IntCounter::new("rpc_calls_total", "Total RPC calls").unwrap();
    static ref ACTIVE_WALLETS: IntGauge =
        IntGauge::new("active_wallets", "Active wallets").unwrap();
    static ref RATE_LIMIT_BLOCKS: IntCounter = IntCounter::new(
        "rate_limit_blocks_total",
        "Total requests blocked by rate limiting"
    )
    .unwrap();
    static ref RPC_ERRORS: IntCounter =
        IntCounter::new("rpc_errors_total", "Total RPC errors (any JSON-RPC error response)")
            .unwrap();
    static ref PROPOSAL_CREATES: IntCounter = register_int_counter!(
        "rpc_proposals_created",
        "Total number of proposals created"
    ).unwrap();
    static ref PROPOSAL_GETS: IntCounter = register_int_counter!(
        "rpc_proposals_fetched", 
        "Total number of proposals fetched"
    ).unwrap();
    static ref VOTES_CAST: IntCounter = register_int_counter!(
        "rpc_votes_cast",
        "Total number of votes cast"
    ).unwrap();
    static ref VOTE_TALLIES: IntCounter = register_int_counter!(
        "rpc_vote_tallies",
        "Total number of vote tallies requested"
    ).unwrap();
}
fn register_metrics() {
    let _ = REGISTRY.register(Box::new(RPC_CALLS.clone()));
    let _ = REGISTRY.register(Box::new(ACTIVE_WALLETS.clone()));
    let _ = REGISTRY.register(Box::new(RATE_LIMIT_BLOCKS.clone()));
    let _ = REGISTRY.register(Box::new(RPC_ERRORS.clone()));
}
fn gather_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

use std::thread;
use tiny_http::{Header, Method, Response, Server as TinyServer};
fn start_metrics_server() {
    thread::spawn(|| {
        // Allow disabling via env
        let disabled = std::env::var("SULTAN_METRICS_DISABLE")
            .ok()
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
            .unwrap_or(false);
        if disabled {
            info!("Prometheus metrics disabled via SULTAN_METRICS_DISABLE");
            return;
        }

        // Primary bind with sane default
        let primary =
            std::env::var("SULTAN_METRICS_ADDR").unwrap_or_else(|_| "0.0.0.0:9100".to_string());

        // Parse host:port (simple IPv4/host:port handling)
        let (host, port_str) = primary
            .rsplit_once(':')
            .map_or(("0.0.0.0", "9100"), |(h, p)| {
                (if h.is_empty() { "0.0.0.0" } else { h }, p)
            });
        let base_port = port_str.parse::<u16>().unwrap_or(9100);

        // Try base + up to 10 fallback ports
        let mut server = None;
        for off in 0..=10 {
            let addr = format!("{}:{}", host, base_port.saturating_add(off));
            match TinyServer::http(&addr) {
                Ok(s) => {
                    if off == 0 {
                        info!(
                            "Prometheus metrics endpoint running on http://{}/metrics",
                            addr
                        );
                    } else {
                        warn!(
                            "Metrics port {} busy; using fallback http://{}/metrics",
                            base_port.saturating_add(off - 1),
                            addr
                        );
                        info!(
                            "Prometheus metrics endpoint active on http://{}/metrics",
                            addr
                        );
                    }
                    server = Some(s);
                    break;
                }
                Err(e) => {
                    if off == 0 {
                        warn!("Port {} busy ({}); trying fallbacks", primary, e);
                    }
                    continue;
                }
            }
        }

        let Some(server) = server else {
            warn!(
                "Metrics disabled: could not bind any port in [{}..={}]",
                base_port,
                base_port.saturating_add(10)
            );
            return;
        };

        for request in server.incoming_requests() {
            let url = request.url().to_string();
            let method = request.method().clone();

            // Simple health for browsers/LBs
            if method == Method::Get && (url == "/" || url == "/health") {
                let resp = Response::from_string("OK")
                    .with_header(Header::from_bytes(b"Content-Type", b"text/plain").unwrap());
                let _ = request.respond(resp);
                continue;
            }

            if url == "/metrics" {
                let metrics = gather_metrics();
                let response = Response::from_string(metrics)
                    .with_header(Header::from_bytes(b"Content-Type", b"text/plain").unwrap());
                let _ = request.respond(response);
            } else {
                let _ = request.respond(
                    Response::from_string("Not Found")
                        .with_status_code(404)
                        .with_header(Header::from_bytes(b"Content-Type", b"text/plain").unwrap()),
                );
            }
        }
    });
}

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    #[allow(dead_code)]
    exp: usize,
}

#[derive(Clone, Default)]
struct RpcMeta {
    sub: Option<String>,
}
impl Metadata for RpcMeta {}

// Load JWT secret from environment (.env supported via dotenvy)
fn load_jwt_secret() -> String {
    let _ = dotenvy::dotenv();
    std::env::var("SULTAN_JWT_SECRET")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| {
            warn!("SULTAN_JWT_SECRET missing; using insecure dev default (all tokens will validate with dev secret)");
            "devsecret_change_me_32_bytes_min".to_string()
        })
}

fn secret_fingerprint(secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    let d = hasher.finalize();
    let hex = format!("{:x}", d);
    hex.chars().take(12).collect()
}

fn jsonrpc_err(code: RpcErrorCode, msg: &str) -> Result<Value, RpcError> {
    RPC_ERRORS.inc();
    Err(RpcError {
        code,
        message: msg.to_string(),
        data: None,
    })
}

fn require_auth(meta: &RpcMeta, method: &str) -> Result<String, RpcError> {
    meta.sub.clone().ok_or_else(|| {
        warn!("Unauthorized {} attempt", method);
        RpcError {
            code: RpcErrorCode::ServerError(401),
            message: "unauthorized".into(),
            data: None,
        }
    })
}

fn check_rate_limit(method: &str, client_id: &str) -> bool {
    let Some(client) = REDIS.as_ref() else {
        return true;
    };
    let key = format!("rate:{}:{}", method, client_id);
    match client.get_connection() {
        Ok(mut con) => {
            let count: i32 = con.incr(&key, 1).unwrap_or(0);
            if count == 1 {
                let window = *RATE_LIMIT_WINDOW_SECS as i64;
                let _expire_res: redis::RedisResult<bool> = con.expire(&key, window);
            }
            if count > *RATE_LIMIT_RPS as i32 {
                RATE_LIMIT_BLOCKS.inc();
                warn!("Rate limit exceeded for {} by {}", method, client_id);
                false
            } else {
                true
            }
        }
        Err(e) => {
            error!("Redis connection error (failing open): {:?}", e);
            true
        }
    }
}

fn check_idempotency(method: &str, client_id: &str, key: &str, ttl_secs: usize) -> bool {
    let Some(client) = REDIS.as_ref() else {
        return true;
    };
    let redis_key = format!("idem:{}:{}:{}", method, client_id, key);
    match client.get_connection() {
        Ok(mut con) => {
            match con.exists::<_, bool>(&redis_key) {
                Ok(true) => return false,
                Ok(false) | Err(_) => {}
            }
            let _set_res: redis::RedisResult<()> = con.set_ex(&redis_key, "1", ttl_secs as u64);
            true
        }
        Err(e) => {
            error!("Redis connection error (idempotency failing open): {:?}", e);
            true
        }
    }
}

fn is_valid_address(addr: &str) -> bool {
    let s = addr.strip_prefix("0x").unwrap_or(addr);
    s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit())
}

fn run_on_rt<F, T>(fut: F) -> T
where
    F: std::future::Future<Output = T>,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        tokio::task::block_in_place(|| handle.block_on(fut))
    } else {
        let rt = Runtime::new().expect("create tokio runtime");
        rt.block_on(fut)
    }
}

// Params aliases to support both array and object styles
#[derive(Deserialize)]
struct WalletCreateParams {
    telegram_id: String,
    #[serde(default)]
    idempotency_key: Option<String>,
}
#[derive(Deserialize)]
struct WalletBalanceParams {
    address: String,
}
#[derive(Deserialize)]
struct TokenMintParams {
    to: String,
    amount: u64,
    #[serde(default)]
    idempotency_key: Option<String>,
}
#[derive(Deserialize)]
struct StakeParams {
    validator_id: String,
    amount: u64,
    #[serde(default)]
    idempotency_key: Option<String>,
}
#[derive(Deserialize)]
struct QueryApyParams {
    #[serde(default)]
    is_validator: bool,
}
#[derive(Deserialize)]
struct VoteParams {
    proposal_id: String,
    vote: bool,
    validator_id: String,
    #[serde(default)]
    idempotency_key: Option<String>,
}
#[derive(Deserialize)]
struct SwapParams {
    from: String,
    amount: u64,
    #[serde(default)]
    idempotency_key: Option<String>,
}

// Endpoints

fn auth_ping(_params: Params, meta: RpcMeta) -> Result<Value, RpcError> {
    let client_id = require_auth(&meta, "auth_ping")?;
    Ok(json!({ "ok": true, "client_id": client_id }))
}

fn wallet_create(params: Params, meta: RpcMeta) -> Result<Value, RpcError> {
    RPC_CALLS.inc();
    let client_id = require_auth(&meta, "wallet_create")?;

    let (telegram_id, idempotency_key_opt) =
        if let Ok((id, key)) = params.clone().parse::<(String, Option<String>)>() {
            (id, key)
        } else if let Ok(p) = params.parse::<WalletCreateParams>() {
            (p.telegram_id, p.idempotency_key)
        } else {
            return jsonrpc_err(RpcErrorCode::InvalidParams, "telegram_id required");
        };

    if !check_rate_limit("wallet_create", &client_id) {
        return jsonrpc_err(RpcErrorCode::ServerError(429), "rate limit exceeded");
    }
    if let Some(k) = idempotency_key_opt.as_deref() {
        if !check_idempotency("wallet_create", &client_id, k, *IDEMP_TTL_SECS) {
            return jsonrpc_err(RpcErrorCode::ServerError(409), "duplicate request");
        }
    }

    let sdk = SDK.lock().unwrap();
    if let Some(sdk) = sdk.as_ref() {
        let res = run_on_rt(sdk.create_wallet(&telegram_id));
        match res {
            Ok(address) => {
                ACTIVE_WALLETS.inc();
                info!("Wallet created for telegram_id={}", telegram_id);
                Ok(json!({ "address": address, "status": "created" }))
            }
            Err(e) => {
                error!("Wallet creation failed: {:?}", e);
                jsonrpc_err(RpcErrorCode::InternalError, "wallet creation failed")
            }
        }
    } else {
        jsonrpc_err(RpcErrorCode::ServerError(500), "SDK not initialized")
    }
}

fn wallet_balance(params: Params, meta: RpcMeta) -> Result<Value, RpcError> {
    RPC_CALLS.inc();
    let client_id = require_auth(&meta, "wallet_balance")?;
    let address = if let Ok((addr,)) = params.clone().parse::<(String,)>() {
        addr
    } else if let Ok(p) = params.parse::<WalletBalanceParams>() {
        p.address
    } else {
        return jsonrpc_err(RpcErrorCode::InvalidParams, "valid address required");
    };
    if address.is_empty() || !is_valid_address(&address) {
        return jsonrpc_err(RpcErrorCode::InvalidParams, "valid address required");
    }

    if !check_rate_limit("wallet_balance", &client_id) {
        return jsonrpc_err(RpcErrorCode::ServerError(429), "rate limit exceeded");
    }
    let sdk = SDK.lock().unwrap();
    if let Some(sdk) = sdk.as_ref() {
        let res = run_on_rt(sdk.get_balance(&address));
        match res {
            Ok(balance) => {
                info!("Balance queried for address={}", address);
                Ok(json!({ "address": address, "balance": balance }))
            }
            Err(e) => {
                error!("Balance query failed: {:?}", e);
                jsonrpc_err(RpcErrorCode::InternalError, "balance query failed")
            }
        }
    } else {
        jsonrpc_err(RpcErrorCode::ServerError(500), "SDK not initialized")
    }
}

fn token_mint(params: Params, meta: RpcMeta) -> Result<Value, RpcError> {
    RPC_CALLS.inc();
    let client_id = require_auth(&meta, "token_mint")?;

    let (to, amount, idempotency_key_opt) =
        if let Ok((to, amount, key)) = params.clone().parse::<(String, u64, Option<String>)>() {
            (to, amount, key)
        } else if let Ok(p) = params.parse::<TokenMintParams>() {
            (p.to, p.amount, p.idempotency_key)
        } else {
            return jsonrpc_err(
                RpcErrorCode::InvalidParams,
                "valid 'to' and positive 'amount' required",
            );
        };

    if to.is_empty() || !is_valid_address(&to) || amount == 0 {
        return jsonrpc_err(
            RpcErrorCode::InvalidParams,
            "valid 'to' and positive 'amount' required",
        );
    }

    if !check_rate_limit("token_mint", &client_id) {
        return jsonrpc_err(RpcErrorCode::ServerError(429), "rate limit exceeded");
    }
    if let Some(k) = idempotency_key_opt.as_deref() {
        if !check_idempotency("token_mint", &client_id, k, *IDEMP_TTL_SECS) {
            return jsonrpc_err(RpcErrorCode::ServerError(409), "duplicate request");
        }
    }

    let sdk = SDK.lock().unwrap();
    if let Some(sdk) = sdk.as_ref() {
        // Balance before to detect applied writes even if SDK returns an error.
        let before = run_on_rt(sdk.get_balance(&to)).unwrap_or(0);

        let res = run_on_rt(sdk.mint_token(&to, amount));
        match res {
            Ok(msg) => {
                info!("Minted {} SLTN to {}", amount, to);
                let after = run_on_rt(sdk.get_balance(&to)).unwrap_or(before);
                Ok(json!({ "to": to, "amount": amount, "status": msg, "balance": after }))
            }
            Err(e) => {
                let after = run_on_rt(sdk.get_balance(&to)).unwrap_or(before);

                // If balance increased by expected amount, treat as success.
                if after >= before.saturating_add(amount as i64) {
                    warn!("token_mint applied but SDK returned error: {:?}", e);
                    return Ok(json!({ "to": to, "amount": amount, "status": "ok", "balance": after }));
                }

                // Heuristic: idempotent duplicate or not-applied LWT.
                let emsg = e.to_string().to_lowercase();
                let looks_idempotent = emsg.contains("idempot")
                    || emsg.contains("applied: false")
                    || emsg.contains("if not exists")
                    || emsg.contains("already exists")
                    || emsg.contains("unique");

                if looks_idempotent {
                    info!("token_mint idempotent duplicate");
                    return Ok(json!({ "to": to, "amount": 0, "status": "idempotent", "balance": after }));
                }

                error!("Mint token failed: {:?}", e);
                jsonrpc_err(RpcErrorCode::InternalError, "mint failed")
            }
        }
    } else {
        jsonrpc_err(RpcErrorCode::ServerError(500), "SDK not initialized")
    }
}

fn stake(params: Params, meta: RpcMeta) -> Result<Value, RpcError> {
    RPC_CALLS.inc();
    let client_id = require_auth(&meta, "stake")?;

    let (validator_id, amount, idempotency_key_opt) =
        if let Ok((id, amt, key)) = params.clone().parse::<(String, u64, Option<String>)>() {
            (id, amt, key)
        } else if let Ok(p) = params.parse::<StakeParams>() {
            (p.validator_id, p.amount, p.idempotency_key)
        } else {
            return jsonrpc_err(
                RpcErrorCode::InvalidParams,
                "validator_id and amount required",
            );
        };

    if validator_id.is_empty() || amount == 0 {
        return jsonrpc_err(
            RpcErrorCode::InvalidParams,
            "validator_id and amount required",
        );
    }

    if !check_rate_limit("stake", &client_id) {
        return jsonrpc_err(RpcErrorCode::ServerError(429), "rate limit exceeded");
    }
    if let Some(k) = idempotency_key_opt.as_deref() {
        if !check_idempotency("stake", &client_id, k, *IDEMP_TTL_SECS) {
            return jsonrpc_err(RpcErrorCode::ServerError(409), "duplicate request");
        }
    }

    let sdk = SDK.lock().unwrap();
    if let Some(sdk) = sdk.as_ref() {
        let res = run_on_rt(sdk.stake(&validator_id, amount));
        match res {
            Ok(signed) => {
                info!("Staked {} SLTN for {}", amount, validator_id);
                Ok(json!({ "validator_id": validator_id, "amount": amount, "signed": signed }))
            }
            Err(e) => {
                error!("Stake failed: {:?}", e);
                jsonrpc_err(RpcErrorCode::InternalError, "stake failed")
            }
        }
    } else {
        jsonrpc_err(RpcErrorCode::ServerError(500), "SDK not initialized")
    }
}

fn query_apy(params: Params, meta: RpcMeta) -> Result<Value, RpcError> {
    RPC_CALLS.inc();
    let client_id = require_auth(&meta, "query_apy")?;

    let is_validator = if let Ok((b,)) = params.clone().parse::<(bool,)>() {
        b
    } else if let Ok(p) = params.parse::<QueryApyParams>() {
        p.is_validator
    } else {
        false
    };

    if !check_rate_limit("query_apy", &client_id) {
        return jsonrpc_err(RpcErrorCode::ServerError(429), "rate limit exceeded");
    }
    let sdk = SDK.lock().unwrap();
    if let Some(sdk) = sdk.as_ref() {
        let res = run_on_rt(sdk.query_apy(is_validator));
        match res {
            Ok(apy) => {
                info!("APY queried (is_validator={})", is_validator);
                Ok(json!({ "apy": apy }))
            }
            Err(e) => {
                error!("APY query failed: {:?}", e);
                jsonrpc_err(RpcErrorCode::InternalError, "apy query failed")
            }
        }
    } else {
        jsonrpc_err(RpcErrorCode::ServerError(500), "SDK not initialized")
    }
}

fn vote_on_proposal(params: Params, meta: RpcMeta) -> Result<Value, RpcError> {
    RPC_CALLS.inc();
    VOTES_CAST.inc();
    let client_id = require_auth(&meta, "vote_on_proposal")?;

    let (proposal_id, vote, validator_id, idempotency_key_opt) = if let Ok((pid, v, vid, key)) =
        params
            .clone()
            .parse::<(String, bool, String, Option<String>)>()
    {
        (pid, v, vid, key)
    } else if let Ok(p) = params.parse::<VoteParams>() {
        (p.proposal_id, p.vote, p.validator_id, p.idempotency_key)
    } else {
        return jsonrpc_err(
            RpcErrorCode::InvalidParams,
            "proposal_id, vote, validator_id required",
        );
    };

    if proposal_id.is_empty() || validator_id.is_empty() {
        return jsonrpc_err(
            RpcErrorCode::InvalidParams,
            "proposal_id, vote, validator_id required",
        );
    }

    if !check_rate_limit("vote_on_proposal", &client_id) {
        return jsonrpc_err(RpcErrorCode::ServerError(429), "rate limit exceeded");
    }
    if let Some(k) = idempotency_key_opt.as_deref() {
        if !check_idempotency("vote_on_proposal", &client_id, k, *IDEMP_TTL_SECS) {
            return jsonrpc_err(RpcErrorCode::ServerError(409), "duplicate request");
        }
    }

    let sdk = SDK.lock().unwrap();
    if let Some(sdk) = sdk.as_ref() {
        let res = run_on_rt(sdk.vote_on_proposal(&proposal_id, vote, &validator_id));
        match res {
            Ok(signed) => {
                info!(
                    "Voted {} on proposal {} by {}",
                    vote, proposal_id, validator_id
                );
                Ok(json!({
                    "proposal_id": proposal_id,
                    "vote": vote,
                    "validator_id": validator_id,
                    "signed": signed
                }))
            }
            Err(e) => {
                error!("Vote failed: {:?}", e);
                jsonrpc_err(RpcErrorCode::InternalError, "vote failed")
            }
        }
    } else {
        jsonrpc_err(RpcErrorCode::ServerError(500), "SDK not initialized")
    }
}

fn proposal_create(params: Params, meta: RpcMeta) -> Result<Value, RpcError> {
    RPC_CALLS.inc();
    PROPOSAL_CREATES.inc();
    let client_id = require_auth(&meta, "proposal_create")?;
    
    let params_result = params.parse::<(String, String, String, Option<String>)>();
    let (proposal_id, title, description, status) = match params_result {
        Ok(p) => p,
        Err(e) => {
            error!("Invalid params for proposal_create: {:?}", e);
            return jsonrpc_err(RpcErrorCode::InvalidParams, "invalid parameters");
        }
    };
    
    if !check_rate_limit("proposal_create", &client_id) {
        return jsonrpc_err(RpcErrorCode::ServerError(429), "rate limit exceeded");
    }
    
    let sdk = SDK.lock().unwrap();
    if let Some(sdk) = sdk.as_ref() {
        let res = run_on_rt(sdk.proposal_create(
            &proposal_id,
            Some(&title),
            Some(&description),
            status.as_deref(),
        ));
        
        match res {
            Ok(_) => {
                info!("Created proposal: {} by {}", proposal_id, client_id);
                Ok(json!({
                    "proposal_id": proposal_id,
                    "title": title,
                    "status": "created"
                }))
            }
            Err(e) => {
                error!("Failed to create proposal: {:?}", e);
                jsonrpc_err(RpcErrorCode::InternalError, "failed to create proposal")
            }
        }
    } else {
        jsonrpc_err(RpcErrorCode::ServerError(500), "SDK not initialized")
    }
}

fn proposal_get(params: Params, _meta: RpcMeta) -> Result<Value, RpcError> {
    RPC_CALLS.inc();
    PROPOSAL_GETS.inc();
    // Parse params - handle array or string
    let proposal_id = if let Ok(arr) = params.clone().parse::<Vec<String>>() {
        if arr.is_empty() {
            return Err(RpcError::invalid_params("missing proposal_id"));
        }
        arr[0].clone()
    } else if let Ok(id) = params.parse::<String>() {
        id
    } else {
        return Err(RpcError::invalid_params("invalid parameters"));
    };
    
    let sdk = SDK.lock().unwrap();
    if let Some(sdk) = sdk.as_ref() {
        let res = run_on_rt(sdk.proposal_get(&proposal_id));
        match res {
            Ok(Some(proposal)) => {
                Ok(json!({
                    "proposal_id": proposal.proposal_id,
                    "title": proposal.title,
                    "description": proposal.description,
                    "status": proposal.state.to_string(),
                    "created_at": proposal.created_at,
                }))
            }
            Ok(None) => {
                jsonrpc_err(RpcErrorCode::InvalidParams, "proposal not found")
            }
            Err(e) => {
                error!("Failed to get proposal: {:?}", e);
                jsonrpc_err(RpcErrorCode::InternalError, "failed to get proposal")
            }
        }
    } else {
        jsonrpc_err(RpcErrorCode::ServerError(500), "SDK not initialized")
    }
}

fn votes_tally(params: Params, _meta: RpcMeta) -> Result<Value, RpcError> {
    RPC_CALLS.inc();
    VOTE_TALLIES.inc();
    // Parse params - handle array or string
    let proposal_id = if let Ok(arr) = params.clone().parse::<Vec<String>>() {
        if arr.is_empty() {
            return Err(RpcError::invalid_params("missing proposal_id"));
        }
        arr[0].clone()
    } else if let Ok(id) = params.parse::<String>() {
        id
    } else {
        return Err(RpcError::invalid_params("invalid parameters"));
    };
    
    let sdk = SDK.lock().unwrap();
    if let Some(sdk) = sdk.as_ref() {
        let res = run_on_rt(sdk.votes_tally(&proposal_id));
        match res {
            Ok((yes_votes, no_votes)) => {
                Ok(json!({
                    "proposal_id": proposal_id,
                    "yes_votes": yes_votes,
                    "no_votes": no_votes,
                    "total_votes": yes_votes + no_votes,
                    "result": if yes_votes > no_votes { "passing" } else { "failing" }
                }))
            }
            Err(e) => {
                error!("Failed to tally votes: {:?}", e);
                jsonrpc_err(RpcErrorCode::InternalError, "failed to tally votes")
            }
        }
    } else {
        jsonrpc_err(RpcErrorCode::ServerError(500), "SDK not initialized")
    }
}

pub fn main() {
    tracing_subscriber::fmt::init();
    register_metrics();
    start_metrics_server();

    let rt = Runtime::new().unwrap();

    // Optional DB: set SULTAN_DB_ADDR to Cassandra contact point; leave unset for in-memory/dev
    let db_addr = std::env::var("SULTAN_DB_ADDR").ok();
    let sdk = rt
        .block_on(SultanSDK::new(
            ChainConfig {
                inflation_rate: 8.0,
                total_supply: 0,
                min_stake: 5000,
                shards: 8,
            },
            db_addr.as_deref(),
        ))
        .ok();
    *SDK.lock().unwrap() = sdk;

    let jwt_secret = load_jwt_secret();
    if jwt_secret == "devsecret_change_me_32_bytes_min" {
        warn!("Running with dev JWT secret; do NOT use in production");
    }
    let allow_raw = std::env::var("SULTAN_JWT_ALLOW_RAW")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    info!(
        "JWT configured (HS256). Secret fingerprint={} allow_raw={}",
        secret_fingerprint(&jwt_secret),
        allow_raw
    );

    info!(
        "Rate limiting: {} req/{}s window",
        *RATE_LIMIT_RPS, *RATE_LIMIT_WINDOW_SECS
    );

    let mut io = MetaIoHandler::default();
    io.add_method_with_meta("auth_ping", |p, m| async move { auth_ping(p, m) });
    io.add_method_with_meta("wallet_create", |p, m| async move { wallet_create(p, m) });
    io.add_method_with_meta("wallet_balance", |p, m| async move { wallet_balance(p, m) });
    io.add_method_with_meta("token_mint", |p, m| async move { token_mint(p, m) });
    io.add_method_with_meta("stake", |p, m| async move { stake(p, m) });
    io.add_method_with_meta("query_apy", |p, m| async move { query_apy(p, m) });
    io.add_method_with_meta(
        "vote_on_proposal",
        |p, m| async move { vote_on_proposal(p, m) },
    );
    io.add_method_with_meta(
        "cross_chain_swap",
        |_p, _m| async move { 
            jsonrpc_err(RpcErrorCode::InternalError, "not implemented") 
        },
    );
    
    // Register governance methods
    io.add_method_with_meta("proposal_create", |p, m| async move { proposal_create(p, m) });
    io.add_method_with_meta("proposal_get", |p, m| async move { proposal_get(p, m) });
    io.add_method_with_meta("votes_tally", |p, m| async move { votes_tally(p, m) });

    let secret_for_extractor = jwt_secret.clone();

    let server =
        ServerBuilder::with_meta_extractor(io, move |req: &hyper::Request<hyper::Body>| {
            let auth_header = req
                .headers()
                .get(hyper::header::AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string());

            let bearer = auth_header
                .as_deref()
                .and_then(|s| s.strip_prefix("Bearer ").map(|t| t.to_string()));

            // Optional dev passthrough: token equals secret
            let allow_raw = std::env::var("SULTAN_JWT_ALLOW_RAW")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);

            // Avoid tuple pattern that inferred unsized str; use Option<String> directly
            if let Some(token) = bearer {
                if allow_raw && token == secret_for_extractor {
                    return RpcMeta {
                        sub: Some("dev-user".to_string()),
                    };
                }
                let mut validation = Validation::new(Algorithm::HS256);
                validation.validate_exp = true;
                match jsonwebtoken::decode::<Claims>(
                    &token,
                    &DecodingKey::from_secret(secret_for_extractor.as_bytes()),
                    &validation,
                ) {
                    Ok(data) => RpcMeta {
                        sub: Some(data.claims.sub),
                    },
                    Err(e) => {
                        warn!("JWT validation failed: {:?}", e);
                        RpcMeta { sub: None }
                    }
                }
            } else {
                RpcMeta { sub: None }
            }
        })
        .threads(2)
        .start_http(
            &std::env::var("SULTAN_RPC_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:3030".to_string())
                .parse()
                .expect("SULTAN_RPC_ADDR must be host:port"),
        )
        .expect("Unable to start JSON-RPC server");

    info!(
        "Sultan JSON-RPC server running on http://{} (JWT via Authorization header)",
        std::env::var("SULTAN_RPC_ADDR").unwrap_or_else(|_| "0.0.0.0:3030".to_string())
    );
    server.wait();
}
