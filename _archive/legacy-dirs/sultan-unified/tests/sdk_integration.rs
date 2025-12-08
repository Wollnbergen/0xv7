use sultan_chain::sdk::SultanSDK;
use sultan_chain::config::ChainConfig;
use sultan_chain::sdk_error::SdkError;

#[tokio::test]
async fn sdk_wallet_lifecycle() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let wallet = sdk.create_wallet("test_owner").await.unwrap();
    assert!(wallet.starts_with("sultan1"));
    
    let balance = sdk.get_balance(&wallet).await.unwrap();
    assert_eq!(balance, 1000000);
}

#[tokio::test]
async fn sdk_transfer_zero_fees() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let from = sdk.create_wallet("alice").await.unwrap();
    let to = sdk.create_wallet("bob").await.unwrap();
    
    let tx_hash = sdk.transfer(&from, &to, 100).await.unwrap();
    assert!(tx_hash.starts_with("0x"));
    
    let from_balance = sdk.get_balance(&from).await.unwrap();
    let to_balance = sdk.get_balance(&to).await.unwrap();
    assert_eq!(from_balance, 1000000 - 100);
    assert_eq!(to_balance, 1000000 + 100);
}

#[tokio::test]
async fn sdk_transfer_insufficient_balance() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let from = sdk.create_wallet("alice").await.unwrap();
    let to = sdk.create_wallet("bob").await.unwrap();
    
    let result = sdk.transfer(&from, &to, 2000000).await;
    assert!(matches!(result, Err(SdkError::InsufficientBalance { .. })));
}

#[tokio::test]
async fn sdk_transfer_zero_amount() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let from = sdk.create_wallet("alice").await.unwrap();
    let to = sdk.create_wallet("bob").await.unwrap();
    
    let result = sdk.transfer(&from, &to, 0).await;
    assert!(matches!(result, Err(SdkError::InvalidAmount(_))));
}

#[tokio::test]
async fn sdk_staking_below_minimum() {
    let mut cfg = ChainConfig::default();
    cfg.min_stake = 1000;
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let result = sdk.stake("validator1", 500).await;
    assert!(matches!(result, Err(SdkError::BelowMinimumStake { .. })));
}

#[tokio::test]
async fn sdk_governance_proposal_lifecycle() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let proposal_id = sdk.proposal_create("proposer1", "Test Proposal", "Description").await.unwrap();
    
    let proposal = sdk.proposal_get(proposal_id).await.unwrap();
    assert_eq!(proposal["id"], proposal_id);
    assert_eq!(proposal["title"], "Test Proposal");
    assert_eq!(proposal["proposer"], "proposer1");
    
    sdk.vote_on_proposal(proposal_id, "voter1", true).await.unwrap();
    sdk.vote_on_proposal(proposal_id, "voter2", false).await.unwrap();
    
    let (yes, no) = sdk.votes_tally(proposal_id).await.unwrap();
    assert_eq!(yes, 1);
    assert_eq!(no, 1);
}

#[tokio::test]
async fn sdk_proposal_not_found() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let result = sdk.proposal_get(999999).await;
    assert!(matches!(result, Err(SdkError::ProposalNotFound(_))));
}

#[tokio::test]
async fn sdk_vote_on_nonexistent_proposal() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let result = sdk.vote_on_proposal(999999, "voter1", true).await;
    assert!(matches!(result, Err(SdkError::ProposalNotFound(_))));
}

#[tokio::test]
async fn sdk_list_all_proposals() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    sdk.proposal_create("p1", "Proposal 1", "Desc 1").await.unwrap();
    sdk.proposal_create("p2", "Proposal 2", "Desc 2").await.unwrap();
    
    let proposals = sdk.get_all_proposals().await.unwrap();
    assert_eq!(proposals.len(), 2);
}

#[tokio::test]
async fn sdk_mint_token() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let wallet = sdk.create_wallet("treasury").await.unwrap();
    let initial = sdk.get_balance(&wallet).await.unwrap();
    
    let tx_hash = sdk.mint_token(&wallet, 5000).await.unwrap();
    assert!(tx_hash.starts_with("0x"));
    
    let new_balance = sdk.get_balance(&wallet).await.unwrap();
    assert_eq!(new_balance, initial + 5000);
}

#[tokio::test]
async fn sdk_mint_zero_tokens() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let wallet = sdk.create_wallet("treasury").await.unwrap();
    let result = sdk.mint_token(&wallet, 0).await;
    assert!(matches!(result, Err(SdkError::InvalidAmount(_))));
}

#[tokio::test]
async fn sdk_query_block_height() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let height = sdk.get_block_height().await.unwrap();
    assert!(height >= 1); // At least genesis
}

#[tokio::test]
async fn sdk_query_transaction_count() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let wallet = sdk.create_wallet("alice").await.unwrap();
    let count_before = sdk.get_transaction_count(&wallet).await.unwrap();
    
    let to = sdk.create_wallet("bob").await.unwrap();
    sdk.transfer(&wallet, &to, 50).await.unwrap();
    
    let count_after = sdk.get_transaction_count(&wallet).await.unwrap();
    assert_eq!(count_after, count_before + 1);
}

#[tokio::test]
async fn sdk_list_wallets() {
    let cfg = ChainConfig::default();
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let wallets = sdk.list_wallets().await.unwrap();
    assert!(wallets.len() >= 3); // alice, bob, validator1 from initialization
}

#[tokio::test]
async fn sdk_apy_calculation() {
    let cfg = ChainConfig {
        chain_id: "test".into(),
        gas_price: 0,
        block_time: 5,
        max_block_size: 1000,
        min_stake: 1000,
        inflation_rate: 0.08,
    };
    let sdk = SultanSDK::new(cfg, None).await.unwrap();
    
    let validator_apy = sdk.query_apy(true).await.unwrap();
    let delegator_apy = sdk.query_apy(false).await.unwrap();
    
    // 0.04 / 0.3 = 0.2666... (26.66%)
    assert!((validator_apy - 0.2666).abs() < 0.01, "Expected ~0.2666, got {}", validator_apy);
    assert!((delegator_apy - 0.2133).abs() < 0.01, "Expected ~0.2133, got {}", delegator_apy);
}
