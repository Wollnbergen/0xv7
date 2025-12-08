use jsonrpc_core::{IoHandler, Value, Params};
use jsonrpc_http_server::{ServerBuilder, Server};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::json;
use rand::random;
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
        
        // Block queries
        let _blockchain = self.blockchain.clone();
        io.add_sync_method("eth_blockNumber", move |_params: Params| {
            // Will be async in full implementation
            Ok(Value::String("0x3e8".to_string())) // Placeholder
        });
        
        let _blockchain = self.blockchain.clone();
        io.add_sync_method("eth_getBlockByNumber", move |_params: Params| {
            Ok(json!({
                "number": "0x1",
                "hash": "0x...",
                "transactions": [],
                "timestamp": "0x0",
                "gasUsed": "0x0" // Zero fees
            }))
        });
        
        let _blockchain = self.blockchain.clone();
        io.add_sync_method("eth_getBlockByHash", move |_params: Params| {
            Ok(json!({
                "number": "0x1",
                "hash": "0x...",
                "transactions": []
            }))
        });
        
        // Transaction queries
        let _blockchain = self.blockchain.clone();
        io.add_sync_method("eth_getTransactionByHash", move |_params: Params| {
            Ok(json!({
                "hash": "0x...",
                "from": "sultan1...",
                "to": "sultan1...",
                "value": "0x0",
                "gasPrice": "0x0", // Zero fees
                "gas": "0x0"
            }))
        });
        
        let _blockchain = self.blockchain.clone();
        io.add_sync_method("eth_getTransactionReceipt", move |_params: Params| {
            Ok(json!({
                "transactionHash": "0x...",
                "status": "0x1",
                "gasUsed": "0x0" // Zero fees
            }))
        });
        
        // Balance and account queries
        let _blockchain = self.blockchain.clone();
        io.add_sync_method("eth_getBalance", move |_params: Params| {
            Ok(Value::String("0x0".to_string()))
        });
        
        let _blockchain = self.blockchain.clone();
        io.add_sync_method("eth_getTransactionCount", move |_params: Params| {
            Ok(Value::String("0x0".to_string()))
        });
        
        // Transaction submission
        let _blockchain = self.blockchain.clone();
        io.add_sync_method("eth_sendTransaction", move |params: Params| {
            // Parse transaction params
            let tx_params = params.parse::<Vec<serde_json::Value>>()
                .map_err(|e| jsonrpc_core::Error::invalid_params(e.to_string()))?;
            
            if tx_params.is_empty() {
                return Err(jsonrpc_core::Error::invalid_params("Missing transaction object"));
            }
            
            // Return transaction hash (zero fees!)
            let tx_hash = format!("0x{:064x}", rand::random::<u64>());
            Ok(Value::String(tx_hash))
        });
        
        let _blockchain = self.blockchain.clone();
        io.add_sync_method("eth_sendRawTransaction", move |params: Params| {
            let raw_tx = params.parse::<Vec<String>>()
                .map_err(|e| jsonrpc_core::Error::invalid_params(e.to_string()))?;
            
            if raw_tx.is_empty() {
                return Err(jsonrpc_core::Error::invalid_params("Missing raw transaction"));
            }
            
            let tx_hash = format!("0x{:064x}", rand::random::<u64>());
            Ok(Value::String(tx_hash))
        });
        
        // Contract calls
        let _blockchain = self.blockchain.clone();
        io.add_sync_method("eth_call", move |_params: Params| {
            Ok(Value::String("0x".to_string()))
        });
        
        // Event logs
        let _blockchain = self.blockchain.clone();
        io.add_sync_method("eth_getLogs", move |_params: Params| {
            Ok(json!([]))
        });
        
        // Account enumeration
        io.add_sync_method("eth_accounts", |_| {
            Ok(json!([]))
        });
        
        // Gas price (always zero)
        io.add_sync_method("eth_gasPrice", |_| {
            Ok(Value::String("0x0".to_string()))
        });
        
        io.add_sync_method("eth_estimateGas", |_| {
            Ok(Value::String("0x0".to_string()))
        });
        
        // Chain info
        io.add_sync_method("eth_chainId", |_| {
            Ok(Value::String("0x534c544e".to_string())) // "SLTN" in hex
        });
        
        io.add_sync_method("net_version", |_| {
            Ok(Value::String("1".to_string()))
        });
        
        // Sultan-specific methods
        let _blockchain = self.blockchain.clone();
        io.add_sync_method("sultan_getValidators", move |_params: Params| {
            Ok(json!([]))
        });
        
        let _blockchain = self.blockchain.clone();
        io.add_sync_method("sultan_getStakingInfo", move |_params: Params| {
            Ok(json!({
                "total_staked": "0",
                "validators": [],
                "apy": "13.33"
            }))
        });
        
        let _blockchain = self.blockchain.clone();
        io.add_sync_method("sultan_getProposals", move |_params: Params| {
            Ok(json!([]))
        });
        
        let _blockchain = self.blockchain.clone();
        io.add_sync_method("sultan_getProposal", move |_params: Params| {
            Ok(json!({
                "id": 0,
                "status": "unknown"
            }))
        });
        
        // IBC methods for Cosmos ecosystem
        let _blockchain = self.blockchain.clone();
        io.add_sync_method("sultan_ibcTransfer", move |params: Params| {
            let params = params.parse::<Vec<serde_json::Value>>()
                .map_err(|e| jsonrpc_core::Error::invalid_params(e.to_string()))?;
            
            if params.len() < 4 {
                return Err(jsonrpc_core::Error::invalid_params("Required: sender, receiver, amount, channel"));
            }
            
            let tx_hash = format!("0x{:064x}", rand::random::<u64>());
            Ok(json!({
                "tx_hash": tx_hash,
                "channel": params.get(3),
                "status": "pending"
            }))
        });
        
        let _blockchain = self.blockchain.clone();
        io.add_sync_method("sultan_ibcChannels", move |_params: Params| {
            Ok(json!([
                {
                    "channel_id": "channel-0",
                    "port_id": "transfer",
                    "counterparty_chain": "osmosis-1",
                    "state": "OPEN"
                },
                {
                    "channel_id": "channel-1",
                    "port_id": "transfer",
                    "counterparty_chain": "cosmoshub-4",
                    "state": "OPEN"
                }
            ]))
        });
        
        let _blockchain = self.blockchain.clone();
        io.add_sync_method("sultan_ibcDenomTrace", move |params: Params| {
            let denom_hash = params.parse::<Vec<String>>()
                .map_err(|e| jsonrpc_core::Error::invalid_params(e.to_string()))?;
            
            if denom_hash.is_empty() {
                return Err(jsonrpc_core::Error::invalid_params("Missing denom hash"));
            }
            
            Ok(json!({
                "path": "transfer/channel-0",
                "base_denom": "usltn"
            }))
        });

        let server = ServerBuilder::new(io)
            .start_http(&"127.0.0.1:8545".parse()?)
            .map_err(|e| format!("Failed to start RPC server: {}", e))?;
        
        Ok(server)
    }
}
