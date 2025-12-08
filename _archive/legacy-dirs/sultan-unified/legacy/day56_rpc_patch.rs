// ===== DAY 5-6: TOKEN ECONOMICS METHODS =====

fn token_transfer(params: Params, meta: RpcMeta) -> Result<Value, RpcError> {
    RPC_CALLS.inc();
    let client_id = require_auth(&meta, "token_transfer")?;
    
    // Simple transfer implementation for Day 5-6
    let params_vec: Vec<serde_json::Value> = params.parse()
        .map_err(|_| RpcError::invalid_params("Invalid parameters"))?;
    
    if params_vec.len() < 3 {
        return Err(RpcError::invalid_params("Expected [from, to, amount]"));
    }
    
    let from = params_vec[0].as_str().unwrap_or("unknown");
    let to = params_vec[1].as_str().unwrap_or("unknown");
    let amount = params_vec[2].as_u64().unwrap_or(0);
    
    // For now, just return success
    Ok(json!({
        "status": "success",
        "from": from,
        "to": to,
        "amount": amount,
        "id": format!("tx_{}", chrono::Utc::now().timestamp())
    }))
}

fn calculate_rewards(params: Params, meta: RpcMeta) -> Result<Value, RpcError> {
    RPC_CALLS.inc();
    let client_id = require_auth(&meta, "calculate_rewards")?;
    
    let (address,) = params.parse::<(String,)>()
        .map_err(|_| RpcError::invalid_params("Expected [address]"))?;
    
    // Mock reward calculation for Day 5-6
    let mock_staked = 1000;
    let mock_reward = 50;
    let apy_rate = 18.25;
    
    Ok(json!({
        "address": address,
        "staked_amount": mock_staked,
        "reward_amount": mock_reward,
        "apy_rate": format!("{:.2}%", apy_rate),
        "staking_days": 30
    }))
}

fn claim_rewards(params: Params, meta: RpcMeta) -> Result<Value, RpcError> {
    RPC_CALLS.inc();
    let client_id = require_auth(&meta, "claim_rewards")?;
    
    let (address,) = params.parse::<(String,)>()
        .map_err(|_| RpcError::invalid_params("Expected [address]"))?;
    
    // Mock reward claiming for Day 5-6
    let claimed_amount = 50;
    
    Ok(json!({
        "address": address,
        "claimed_amount": claimed_amount,
        "status": "success",
        "timestamp": chrono::Utc::now().timestamp()
    }))
}

