//! Performance benchmarks for Sultan Core
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sultan_core::*;

fn benchmark_block_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("block_creation");
    
    for tx_count in [10, 50, 100, 500].iter() {
        group.benchmark_with_input(
            BenchmarkId::from_parameter(tx_count),
            tx_count,
            |b, &tx_count| {
                b.iter(|| {
                    let mut blockchain = Blockchain::new();
                    blockchain.init_account("alice".to_string(), 1_000_000);
                    blockchain.init_account("bob".to_string(), 500_000);
                    
                    // Add transactions
                    for i in 0..tx_count {
                        let tx = Transaction::new(
                            "alice".to_string(),
                            "bob".to_string(),
                            100,
                            i + 1,
                        );
                        blockchain.add_transaction(tx).ok();
                    }
                    
                    // Create block
                    black_box(blockchain.create_block("validator1".to_string()).unwrap());
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_transaction_validation(c: &mut Criterion) {
    c.bench_function("transaction_validation", |b| {
        let mut validator = TransactionValidator::new();
        let tx = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            1000,
            1,
        );
        
        b.iter(|| {
            black_box(validator.quick_validate(&tx).unwrap());
        });
    });
}

fn benchmark_state_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_updates");
    
    for account_count in [100, 1000, 10000].iter() {
        group.benchmark_with_input(
            BenchmarkId::from_parameter(account_count),
            account_count,
            |b, &account_count| {
                b.iter(|| {
                    let mut blockchain = Blockchain::new();
                    
                    // Initialize accounts
                    for i in 0..account_count {
                        blockchain.init_account(format!("account{}", i), 1_000_000);
                    }
                    
                    // Perform transfers
                    for i in 0..(account_count / 10) {
                        let tx = Transaction::new(
                            format!("account{}", i),
                            format!("account{}", i + 1),
                            100,
                            1,
                        );
                        blockchain.add_transaction(tx).ok();
                    }
                    
                    black_box(blockchain.create_block("validator1".to_string()).unwrap());
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_consensus_selection(c: &mut Criterion) {
    c.bench_function("consensus_proposer_selection", |b| {
        let mut consensus = ConsensusEngine::new();
        
        // Add 100 validators
        for i in 0..100 {
            consensus.add_validator(
                format!("validator{}", i),
                10000 + (i * 100) as u64,
            ).ok();
        }
        
        b.iter(|| {
            black_box(consensus.select_proposer());
        });
    });
}

fn benchmark_block_validation(c: &mut Criterion) {
    c.bench_function("block_validation", |b| {
        let mut blockchain = Blockchain::new();
        blockchain.init_account("alice".to_string(), 1_000_000);
        
        // Add 100 transactions
        for i in 0..100 {
            let tx = Transaction::new(
                "alice".to_string(),
                "bob".to_string(),
                100,
                i + 1,
            );
            blockchain.add_transaction(tx).ok();
        }
        
        let block = blockchain.create_block("validator1".to_string()).unwrap();
        
        b.iter(|| {
            black_box(blockchain.validate_block(&block).unwrap());
        });
    });
}

fn benchmark_throughput(c: &mut Criterion) {
    c.bench_function("throughput_1000_blocks", |b| {
        b.iter(|| {
            let mut blockchain = Blockchain::new();
            let mut consensus = ConsensusEngine::new();
            
            blockchain.init_account("alice".to_string(), 100_000_000);
            consensus.add_validator("validator1".to_string(), 100_000).ok();
            
            // Produce 1000 blocks
            for i in 0..1000 {
                let tx = Transaction::new(
                    "alice".to_string(),
                    "bob".to_string(),
                    100,
                    i + 1,
                );
                blockchain.add_transaction(tx).ok();
                
                let proposer = consensus.select_proposer().unwrap();
                black_box(blockchain.create_block(proposer).unwrap());
            }
        });
    });
}

criterion_group!(
    benches,
    benchmark_block_creation,
    benchmark_transaction_validation,
    benchmark_state_updates,
    benchmark_consensus_selection,
    benchmark_block_validation,
    benchmark_throughput,
);

criterion_main!(benches);
