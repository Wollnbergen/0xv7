// Add these methods to rpc_server.rs

// Import the new modules at the top
use crate::token_transfer::{Transfer, TransferManager, TransferStatus};
use crate::rewards::{RewardManager, RewardCalculation};

// Add these RPC method implementations

fn token_transfer(params: Params, meta: RpcMeta) -> Result<Value, RpcError> {
    RPC_CALLS.inc();
    let client_id = require_auth(&meta, "token_transfer")?;
    
    if !check_rate_limit("token_transfer", &client_id) {
        return Err(RpcError::invalid_request("Rate limit exceeded"));
    }

    let params_vec: Vec<serde_json::Value> = params.parse()
        .map_err(|_| RpcError::invalid_params("Invalid parameters"))?;
    
    if params_vec.len() < 3 {
        return Err(RpcError::invalid_params("Expected [from, to, amount, memo?]"));
    }
    
    let from = params_vec[0].as_str()
        .ok_or_else(|| RpcError::invalid_params("'from' must be string"))?;
    let to = params_vec[1].as_str()
        .ok_or_else(|| RpcError::invalid_params("'to' must be string"))?;
    let amount = params_vec[2].as_u64()
        .ok_or_else(|| RpcError::invalid_params("'amount' must be number"))?;
    let memo = params_vec.get(3).and_then(|v| v.as_str()).map(|s| s.to_string());

    let db = DATABASE.clone();
    let transfer_manager = TransferManager::new(db);
    
    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            transfer_manager.transfer(from, to, amount, memo).await
        })
    });

    match result {
        Ok(transfer) => {
            info!("Transfer: {} -> {} amount: {}", from, to, amount);
            Ok(json!({
                "id": transfer.id,
                "from": transfer.from,
                "to": transfer.to,
                "amount": transfer.amount,
                "status": "completed"
            }))
        }
        Err(e) => Err(RpcError::invalid_request(&e.to_string()))
    }
}

fn calculate_rewards(params: Params, meta: RpcMeta) -> Result<Value, RpcError> {
    RPC_CALLS.inc();
    let client_id = require_auth(&meta, "calculate_rewards")?;
    
    if !check_rate_limit("calculate_rewards", &client_id) {
        return Err(RpcError::invalid_request("Rate limit exceeded"));
    }

    let (address,) = params.parse::<(String,)>()
        .map_err(|_| RpcError::invalid_params("Expected [address]"))?;

    let db = DATABASE.clone();
    let reward_manager = RewardManager::new(db);
    
    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            reward_manager.calculate_rewards(&address).await
        })
    });

    match result {
        Ok(calculation) => {
            info!("Rewards calculated for {}: {} tokens", address, calculation.reward_amount);
            Ok(json!({
                "address": calculation.address,
                "staked_amount": calculation.staked_amount,
                "reward_amount": calculation.reward_amount,
                "apy_rate": format!("{:.2}%", calculation.apy_rate * 100.0),
                "staking_days": calculation.staking_duration_days
            }))
        }
        Err(e) => Err(RpcError::invalid_request(&e.to_string()))
    }
}

fn claim_rewards(params: Params, meta: RpcMeta) -> Result<Value, RpcError> {
    RPC_CALLS.inc();
    let client_id = require_auth(&meta, "claim_rewards")?;
    
    if !check_rate_limit("claim_rewards", &client_id) {
        return Err(RpcError::invalid_request("Rate limit exceeded"));
    }

    let (address,) = params.parse::<(String,)>()
        .map_err(|_| RpcError::invalid_params("Expected [address]"))?;

    let db = DATABASE.clone();
    let reward_manager = RewardManager::new(db);
    
    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            reward_manager.claim_rewards(&address).await
        })
    });

    match result {
        Ok(amount) => {
            info!("Rewards claimed for {}: {} tokens", address, amount);
            Ok(json!({
                "address": address,
                "claimed_amount": amount,
                "status": "success"
            }))
        }
        Err(e) => Err(RpcError::invalid_request(&e.to_string()))
    }
}

fn get_transfer_history(params: Params, meta: RpcMeta) -> Result<Value, RpcError> {
    RPC_CALLS.inc();
    let client_id = require_auth(&meta, "get_transfer_history")?;
    
    if !check_rate_limit("get_transfer_history", &client_id) {
        return Err(RpcError::invalid_request("Rate limit exceeded"));
    }

    let (address,) = params.parse::<(String,)>()
        .map_err(|_| RpcError::invalid_params("Expected [address]"))?;

    let db = DATABASE.clone();
    let transfer_manager = TransferManager::new(db);
    
    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            transfer_manager.get_history(&address, 10).await
        })
    });

    match result {
        Ok(transfers) => {
            info!("Transfer history for {}: {} transfers", address, transfers.len());
            Ok(json!({
                "address": address,
                "count": transfers.len(),
                "transfers": transfers
            }))
        }
        Err(e) => Err(RpcError::invalid_request(&e.to_string()))
    }
}

// Add to the RPC server builder in main():
// .with_method("token_transfer", token_transfer)
// .with_method("calculate_rewards", calculate_rewards)
// .with_method("claim_rewards", claim_rewards)
// .with_method("get_transfer_history", get_transfer_history)
