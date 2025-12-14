//! RPC Server Module

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub method: String,
    pub params: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub result: String,
    pub error: Option<String>,
}

pub struct RpcServer {
    handlers: HashMap<String, fn(&[String]) -> RpcResponse>,
}

impl RpcServer {
    pub fn new() -> Self {
        let mut handlers = HashMap::new();
        
        // Add default handlers
        handlers.insert("status".to_string(), status_handler as fn(&[String]) -> RpcResponse);
        handlers.insert("block_height".to_string(), block_height_handler as fn(&[String]) -> RpcResponse);
        
        Self { handlers }
    }

    pub fn handle_request(&self, request: RpcRequest) -> RpcResponse {
        if let Some(handler) = self.handlers.get(&request.method) {
            handler(&request.params)
        } else {
            RpcResponse {
                result: String::new(),
                error: Some(format!("Method {} not found", request.method)),
            }
        }
    }
}

fn status_handler(_params: &[String]) -> RpcResponse {
    RpcResponse {
        result: "running".to_string(),
        error: None,
    }
}

fn block_height_handler(_params: &[String]) -> RpcResponse {
    RpcResponse {
        result: "1000".to_string(),
        error: None,
    }
}
