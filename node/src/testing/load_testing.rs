//! Production Load Testing for Sultan Chain
//! Target: 1.23M TPS with $0.00 gas fees

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub struct LoadTestConfig {
    pub target_tps: u64,
    pub duration_secs: u64,
    pub batch_size: usize,
    pub parallel_workers: usize,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            target_tps: 1_230_000, // 1.23M TPS target
            duration_secs: 60,
            batch_size: 10_000,
            parallel_workers: num_cpus::get(),
        }
    }
}
