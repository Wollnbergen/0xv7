
pub struct RewardsCalculator {
    pub inflation_rate: f64,
    pub total_staked: u64,
}

impl RewardsCalculator {
    pub fn new(inflation_rate: f64) -> Self {
        RewardsCalculator {
            inflation_rate,
            total_staked: 300_000_000, // 30% of total supply staked
        }
    }
    
    pub fn calculate_validator_apy(&self) -> f64 {
        // The famous 26.67% APY formula
        self.inflation_rate / 0.3
    }
    
    pub fn calculate_mobile_bonus(&self, base_apy: f64) -> f64 {
        base_apy * 1.4 // 40% bonus for mobile validators
    }
    
    pub fn calculate_rewards(&self, stake: u64, is_mobile: bool) -> u64 {
        let base_apy = self.calculate_validator_apy();
        let apy = if is_mobile {
            self.calculate_mobile_bonus(base_apy)
        } else {
            base_apy
        };
        
        (stake as f64 * apy / 100.0) as u64
    }
}
