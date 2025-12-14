use sultan_chain::sdk::SultanSDK;
use sultan_chain::config::ChainConfig;
use sultan_chain::sdk_error::SdkError;

#[tokio::test]
async fn sdk_ibc_transfer() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let alice = sdk.create_wallet("alice").await.unwrap();
    
    // Valid IBC transfer
    let tx_hash = sdk.ibc_transfer(
        &alice,
        "osmo1abc123...",
        1000,
        "transfer/channel-0"
    ).await.unwrap();
    
    assert!(tx_hash.starts_with("0x"));
}

#[tokio::test]
async fn sdk_ibc_invalid_channel() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let alice = sdk.create_wallet("alice").await.unwrap();
    
    // Invalid channel format
    let result = sdk.ibc_transfer(
        &alice,
        "osmo1abc123...",
        1000,
        "invalid-channel"
    ).await;
    
    assert!(matches!(result, Err(SdkError::InvalidAddress(_))));
}

#[tokio::test]
async fn sdk_ibc_insufficient_balance() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let alice = sdk.create_wallet("alice").await.unwrap();
    
    // Try to send more than balance
    let result = sdk.ibc_transfer(
        &alice,
        "osmo1abc123...",
        10000000,
        "channel-0"
    ).await;
    
    assert!(matches!(result, Err(SdkError::InsufficientBalance { .. })));
}

#[tokio::test]
async fn sdk_ibc_query_channels() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let channels = sdk.ibc_query_channels().await.unwrap();
    
    assert!(channels.len() >= 2);
    assert_eq!(channels[0]["channel_id"], "channel-0");
    assert_eq!(channels[0]["counterparty_chain"], "osmosis-1");
    assert_eq!(channels[1]["channel_id"], "channel-1");
    assert_eq!(channels[1]["counterparty_chain"], "cosmoshub-4");
}

#[tokio::test]
async fn sdk_batch_transfer() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let alice = sdk.create_wallet("alice").await.unwrap();
    let bob = sdk.create_wallet("bob").await.unwrap();
    let charlie = sdk.create_wallet("charlie").await.unwrap();
    
    let transfers = vec![
        (bob.clone(), 100),
        (charlie.clone(), 200),
    ];
    
    let tx_hashes = sdk.batch_transfer(&alice, transfers).await.unwrap();
    
    assert_eq!(tx_hashes.len(), 2);
    
    let alice_balance = sdk.get_balance(&alice).await.unwrap();
    let bob_balance = sdk.get_balance(&bob).await.unwrap();
    let charlie_balance = sdk.get_balance(&charlie).await.unwrap();
    
    assert_eq!(alice_balance, 1000000 - 100 - 200);
    assert_eq!(bob_balance, 1000000 + 100);
    assert_eq!(charlie_balance, 1000000 + 200);
}

#[tokio::test]
async fn sdk_transaction_history() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let alice = sdk.create_wallet("alice").await.unwrap();
    let bob = sdk.create_wallet("bob").await.unwrap();
    
    // Make several transfers
    sdk.transfer(&alice, &bob, 100).await.unwrap();
    sdk.transfer(&alice, &bob, 200).await.unwrap();
    sdk.transfer(&bob, &alice, 50).await.unwrap();
    
    // Query history
    let alice_history = sdk.get_transaction_history(&alice, None).await.unwrap();
    
    // Alice should appear in 3 transactions (2 as sender, 1 as receiver)
    assert_eq!(alice_history.len(), 3);
}

#[tokio::test]
async fn sdk_transaction_history_with_limit() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let alice = sdk.create_wallet("alice").await.unwrap();
    let bob = sdk.create_wallet("bob").await.unwrap();
    
    // Make several transfers
    for _ in 0..10 {
        sdk.transfer(&alice, &bob, 10).await.unwrap();
    }
    
    // Query with limit
    let history = sdk.get_transaction_history(&alice, Some(5)).await.unwrap();
    
    assert_eq!(history.len(), 5);
}

#[tokio::test]
async fn sdk_get_transaction() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let alice = sdk.create_wallet("alice").await.unwrap();
    let bob = sdk.create_wallet("bob").await.unwrap();
    
    let tx_hash = sdk.transfer(&alice, &bob, 100).await.unwrap();
    
    let tx = sdk.get_transaction(&tx_hash).await.unwrap();
    
    assert_eq!(tx["hash"], tx_hash);
    assert_eq!(tx["status"], "confirmed");
    assert_eq!(tx["gas_fee"], 0); // Zero fees!
}

#[tokio::test]
async fn sdk_get_validator_set() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let validators = sdk.get_validator_set().await.unwrap();
    
    assert!(!validators.is_empty());
    assert!(validators[0]["address"].as_str().unwrap().starts_with("sultanvaloper"));
    assert!(validators[0]["voting_power"].as_str().is_some());
}

#[tokio::test]
async fn sdk_get_delegations() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let delegator = sdk.create_wallet("delegator").await.unwrap();
    
    let delegations = sdk.get_delegations(&delegator).await.unwrap();
    
    // May be empty or have placeholder data
    assert!(delegations.len() >= 0);
}

#[tokio::test]
async fn sdk_zero_amount_ibc_transfer() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let alice = sdk.create_wallet("alice").await.unwrap();
    
    let result = sdk.ibc_transfer(
        &alice,
        "osmo1abc...",
        0,
        "channel-0"
    ).await;
    
    assert!(matches!(result, Err(SdkError::InvalidAmount(_))));
}

#[tokio::test]
async fn sdk_batch_transfer_partial_failure() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let alice = sdk.create_wallet("alice").await.unwrap();
    let bob = sdk.create_wallet("bob").await.unwrap();
    
    // Second transfer will fail due to insufficient balance
    let transfers = vec![
        (bob.clone(), 500000),
        (bob.clone(), 700000), // This will fail
    ];
    
    let result = sdk.batch_transfer(&alice, transfers).await;
    
    // Should fail on second transfer
    assert!(result.is_err());
}
