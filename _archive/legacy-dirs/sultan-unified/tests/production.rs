use sultan_chain::{blockchain::{Blockchain, Transaction}, quantum::QuantumCrypto, consensus::{ConsensusEngine, Validator}, database::Database, config::ChainConfig, p2p::P2PNetwork};

#[tokio::test]
async fn block_creation_zero_fee() {
    let mut chain = Blockchain::new();
    chain.add_transaction(Transaction::new("sultan1alice".into(), "sultan1bob".into(), 42));
    let b = chain.create_block();
    assert_eq!(b.index, 1);
    assert!(b.transactions.iter().all(|t| t.gas_fee == 0));
    assert_eq!(chain.get_latest_block().index, 1);
}

#[test]
fn quantum_sign_and_verify() {
    let qc = QuantumCrypto::new();
    let msg = b"final-production";
    let signed = qc.sign(msg);
    assert!(qc.verify(&signed));
}

#[test]
fn consensus_round_robin() {
    let mut engine = ConsensusEngine::new();
    engine.add_validator(Validator { address: "v1".into(), stake: 10, voting_power: 10 });
    engine.add_validator(Validator { address: "v2".into(), stake: 20, voting_power: 20 });
    engine.add_validator(Validator { address: "v3".into(), stake: 30, voting_power: 30 });
    let p1 = engine.select_proposer().unwrap();
    let p2 = engine.select_proposer().unwrap();
    let p3 = engine.select_proposer().unwrap();
    let mut set = std::collections::HashSet::new();
    set.insert(p1);
    set.insert(p2);
    set.insert(p3);
    assert_eq!(set.len(), 3, "Three distinct proposers expected over three rounds");
}

#[test]
fn database_wallet_prefix() {
    let mut db = Database::new();
    db.create_wallet("abc").unwrap();
    assert!(db.wallets.keys().any(|k| k.starts_with("sultan1")));
}

#[test]
fn config_defaults() {
    let cfg = ChainConfig::default();
    assert_eq!(cfg.gas_price, 0);
    assert!(cfg.block_time > 0);
}

#[tokio::test]
async fn p2p_start_stop() {
    let mut p2p = P2PNetwork::new().unwrap();
    p2p.start_listening("127.0.0.1:0").await.unwrap();
    p2p.broadcast_block(Vec::new()).await.unwrap();
    p2p.stop().await.unwrap();
}