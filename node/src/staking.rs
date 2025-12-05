//! Sultan Chain Staking Module
use std::collections::HashMap;

pub struct StakingPool {
    validators: HashMap<String, u128>,
    total_staked: u128,
    apy: f64,
}

impl StakingPool {
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
            total_staked: 0,
            apy: 26.67, // 26.67% APY as specified
        }
    }
    
    pub fn stake(&mut self, validator: String, amount: u128) -> Result<(), String> {
        if amount < 5000 {
            return Err("Minimum stake is 5000 SLTN".to_string());
        }
        
        *self.validators.entry(validator).or_insert(0) += amount;
        self.total_staked += amount;
        Ok(())
    }
}
