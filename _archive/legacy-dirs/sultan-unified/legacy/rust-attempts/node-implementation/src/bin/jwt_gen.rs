// filepath: /workspaces/0xv7/node/src/bin/jwt_gen.rs
use anyhow::Result;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
    nbf: usize,
}

fn main() -> Result<()> {
    let secret = std::env::var("SULTAN_JWT_SECRET")
        .unwrap_or_else(|_| "change-this-secret".to_string());
    
    let sub = std::env::args().nth(1).unwrap_or_else(|| "dev".to_string());
    let ttl_secs: usize = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(3600);
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs() as usize;
    
    let claims = Claims {
        sub,
        iat: now,
        nbf: now,
        exp: now + ttl_secs,
    };
    
    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    
    println!("{}", token);
    Ok(())
}
