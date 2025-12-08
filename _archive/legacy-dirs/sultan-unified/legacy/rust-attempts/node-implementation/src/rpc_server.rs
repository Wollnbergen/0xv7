use jsonrpc_core::{IoHandler, Result, Value};
use jsonrpc_http_server::{ServerBuilder, Server};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::blockchain::Blockchain;

pub struct RpcServer {
    blockchain: Arc<Mutex<Blockchain>>,
}

impl RpcServer {
    pub fn new(blockchain: Arc<Mutex<Blockchain>>) -> Self {
        RpcServer { blockchain }
    }

    pub async fn start(self) -> std::result::Result<Server, Box<dyn std::error::Error>> {
        let mut io = IoHandler::new();
        
        let blockchain = self.blockchain.clone();
        io.add_sync_method("getBlockHeight", move |_| {
            Ok(Value::String("1000".to_string()))
        });
        
        io.add_sync_method("getGasPrice", |_| {
            Ok(Value::String("0".to_string()))
        });

        let server = ServerBuilder::new(io)
            .start_http(&"127.0.0.1:8545".parse()?)
            .expect("Unable to start RPC server");
        
        Ok(server)
    }
}
