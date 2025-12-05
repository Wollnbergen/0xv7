// Add these to node/src/rpc_server.rs

// At the top with other imports:
use crate::token_transfer::{Transfer, TransferManager};
use crate::rewards::RewardManager;

// Add these RPC methods (simplified version):

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
    let memo = params_vec.get(3)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Simple transfer logic using database directly
    let mut db = DATABASE.lock().unwrap();
    
    // Check sender balance
    let sender_balance = db.wallets.get(from)
        .map(|w| w.balance)
        .unwrap_or(0);
    
    if sender_balance < amount {
        return Err(RpcError::invalid_request("Insufficient balance"));
    }
    
    // Ensure recipient wallet exists
    if !db.wallets.contains_key(to) {
        db.create_wallet(to).map_err(|e| RpcError::internal_error())?;
    }
    
    // Execute transfer
    if let Some(sender) = db.wallets.get_mut(from) {
        sender.balance -= amount;
    }
    
    if let Some(recipient) = db.wallets.get_mut(to) {
        recipient.balance += amount;
    }
    
    // Record transfer
    let transfer_id = format!("tx_{}", uuid::Uuid::new_v4());
    let transfer = Transfer {
        id: transfer_id.clone(),
        from: from.to_string(),
        to: to.to_string(),
        amount,
        fee: 0, // Simplified: no fee for now
        timestamp: chrono::Utc::now().timestamp(),
        status: crate::token_transfer::TransferStatus::Completed,
        memo,
    };
    
    db.transfers.insert(transfer_id.clone(), transfer);
    
    info!("Transfer completed: {} -> {} amount: {}", from, to, amount);
    
    Ok(json!({
        "id": transfer_id,
        "from": from,
        "to": to,
        "amount": amount,
        "status": "completed"
    }))
}

fn calculate_rewards(params: Params, meta: RpcMeta) -> Result<Value, RpcError> {
    RPC_CALLS.inc();
    let client_id = require_auth(&meta, "calculate_rewards")?;
    
    if !check_rate_limit("calculate_rewards", &client_id) {
        return Err(RpcError::invalid_request("Rate limit exceeded"));
    }

    let (address,) = params.parse::<(String,)>()
        .map_err(|_| RpcError::invalid_params("Expected [address]"))?;

    let db = DATABASE.lock().unwrap();
    
    // Check if stake exists
    let stake = db.stakes.get(&address)
        .ok_or_else(|| RpcError::invalid_request("No stake found"))?;
    
    // Check if wallet exists and get validator status
    let is_validator = db.wallets.get(&address)
        .map(|w| w.is_validator)
        .unwrap_or(false);
    
    // Calculate rewards
    let now = chrono::Utc::now().timestamp();
    let staking_duration_days = (now - stake.timestamp) / 86400;
    let apy_rate = if is_validator { 0.12 } else { 0.08 };
    
    // Simple interest calculation for now
    let daily_rate = apy_rate / 365.0;
    let reward_amount = (stake.amount as f64 * daily_rate * staking_duration_days as f64) as u64;
    
    info!("Calculated rewards for {}: {} tokens", address, reward_amount);
    
    Ok(json!({
        "address": address,
        "staked_amount": stake.amount,
        "reward_amount": reward_amount,
        "apy_rate": format!("{:.2}%", apy_rate * 100.0),
        "staking_days": staking_duration_days
    }))
}

fn claim_rewards(params: Params, meta: RpcMeta) -> Result<Value, RpcError> {
    RPC_CALLS.inc();
    let client_id = require_auth(&meta, "claim_rewards")?;
    
    if !check_rate_limit("claim_rewards", &client_id) {
        return Err(RpcError::invalid_request("Rate limit exceeded"));
    }

    let (address,) = params.parse::<(String,)>()
        .map_err(|_| RpcError::invalid_params("Expected [address]"))?;

    let mut db = DATABASE.lock().unwrap();
    
    // Calculate rewards first
    let stake = db.stakes.get(&address)
        .ok_or_else(|| RpcError::invalid_request("No stake found"))?
        .clone();
    
    let is_validator = db.wallets.get(&address)
        .map(|w| w.is_validator)
        .unwrap_or(false);
    
    let now = chrono::Utc::now().timestamp();
    let staking_duration_days = (now - stake.timestamp) / 86400;
    let apy_rate = if is_validator { 0.12 } else { 0.08 };
    let daily_rate = apy_rate / 365.0;
    let reward_amount = (stake.amount as f64 * daily_rate * staking_duration_days as f64) as u64;
    
    // Add rewards to wallet
    if let Some(wallet) = db.wallets.get_mut(&address) {
        wallet.balance += reward_amount;
    }
    
    // Update stake timestamp to prevent double claiming
    if let Some(stake) = db.stakes.get_mut(&address) {
        stake.timestamp = now;
    }
    
    info!("Rewards claimed for {}: {} tokens", address, reward_amount);
    
    Ok(json!({
        "address": address,
        "claimed_amount": reward_amount,
        "status": "success"
    }))
}

// Register in main():
// .with_method("token_transfer", token_transfer)
// .with_method("calculate_rewards", calculate_rewards)  
// .with_method("claim_rewards", claim_rewards)
