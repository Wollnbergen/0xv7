//! Transaction safety and atomicity mechanisms

use std::sync::atomic::{AtomicBool, Ordering};

pub struct CircuitBreaker {
    enabled: AtomicBool,
    threshold: u64,
}

impl CircuitBreaker {
    pub fn new(threshold: u64) -> Self {
        Self {
            enabled: AtomicBool::new(true),
            threshold,
        }
    }
    
    pub fn check_and_execute<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce() -> Result<R, String>,
    {
        if !self.enabled.load(Ordering::SeqCst) {
            return Err("Circuit breaker triggered - system paused".to_string());
        }
        f()
    }
    
    pub fn emergency_stop(&self) {
        self.enabled.store(false, Ordering::SeqCst);
    }
}
