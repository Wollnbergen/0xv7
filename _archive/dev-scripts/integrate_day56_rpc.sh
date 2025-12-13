#!/bin/bash

echo "📝 Adding Day 5-6 RPC methods to server..."

# Create a patch file with the methods to add
cat > day56_rpc_patch.txt << 'PATCH'

// Day 5-6: Token Transfer
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

    let mut db = DATABASE.lock().unwrap();
    
    let sender_balance = db.wallets.get(from).map(|w| w.balance).unwrap_or(0);
    
    if sender_balance < amount {
        return Err(RpcError::invalid_request("Insufficient balance"));
    }
    
    if !db.wallets.contains_key(to) {
        db.create_wallet(to).map_err(|_| RpcError::internal_error())?;
    }
    
    if let Some(sender) = db.wallets.get_mut(from) {
        sender.balance -= amount;
    }
    
    if let Some(recipient) = db.wallets.get_mut(to) {
        recipient.balance += amount;
    }
    
    let transfer_id = format!("tx_{}", uuid::Uuid::new_v4());
    let transfer = crate::database::Transfer {
        id: transfer_id.clone(),
        from: from.to_string(),
        to: to.to_string(),
        amount,
        fee: 0,
        timestamp: chrono::Utc::now().timestamp(),
        memo,
    };
    
    db.transfers.insert(transfer_id.clone(), transfer);
    
    info!("Transfer: {} -> {} amount: {}", from, to, amount);
    
    Ok(json!({
        "id": transfer_id,
        "from": from,
        "to": to,
        "amount": amount,
        "status": "completed"
    }))
}

// Day 5-6: Calculate Rewards
fn calculate_rewards(params: Params, meta: RpcMeta) -> Result<Value, RpcError> {
    RPC_CALLS.inc();
    let client_id = require_auth(&meta, "calculate_rewards")?;
    
    if !check_rate_limit("calculate_rewards", &client_id) {
        return Err(RpcError::invalid_request("Rate limit exceeded"));
    }

    let (address,) = params.parse::<(String,)>()
        .map_err(|_| RpcError::invalid_params("Expected [address]"))?;

    let db = DATABASE.lock().unwrap();
    
    let stake = db.stakes.get(&address)
        .ok_or_else(|| RpcError::invalid_request("No stake found"))?;
    
    let is_validator = db.wallets.get(&address)
        .map(|w| w.is_validator)
        .unwrap_or(false);
    
    let now = chrono::Utc::now().timestamp();
    let staking_duration_days = (now - stake.timestamp) / 86400;
    let apy_rate = if is_validator { 0.12 } else { 0.08 };
    
    let daily_rate = apy_rate / 365.0;
    let reward_amount = (stake.amount as f64 * daily_rate * staking_duration_days as f64) as u64;
    
    Ok(json!({
        "address": address,
        "staked_amount": stake.amount,
        "reward_amount": reward_amount,
        "apy_rate": format!("{:.2}%", apy_rate * 100.0),
        "staking_days": staking_duration_days
    }))
}

// Day 5-6: Claim Rewards
fn claim_rewards(params: Params, meta: RpcMeta) -> Result<Value, RpcError> {
    RPC_CALLS.inc();
    let client_id = require_auth(&meta, "claim_rewards")?;
    
    if !check_rate_limit("claim_rewards", &client_id) {
        return Err(RpcError::invalid_request("Rate limit exceeded"));
    }

    let (address,) = params.parse::<(String,)>()
        .map_err(|_| RpcError::invalid_params("Expected [address]"))?;

    let mut db = DATABASE.lock().unwrap();
    
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
    
    if let Some(wallet) = db.wallets.get_mut(&address) {
        wallet.balance += reward_amount;
    }
    
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
PATCH

echo ""
echo "✅ RPC methods prepared in day56_rpc_patch.txt"
echo ""
echo "⚠️  MANUAL STEP REQUIRED:"
echo "Add these to node/src/rpc_server.rs:"
echo "1. Add the three functions above (token_transfer, calculate_rewards, claim_rewards)"
echo "2. Register them in main():"
echo '   .with_method("token_transfer", token_transfer)'
echo '   .with_method("calculate_rewards", calculate_rewards)'
echo '   .with_method("claim_rewards", claim_rewards)'
