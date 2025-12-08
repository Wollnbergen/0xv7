// Sultan Chain Economics Module
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Economics {
    pub current_inflation_rate: f64,
    pub current_burn_rate: f64,
    pub validator_apy: f64,
    pub total_burned: u64,
    pub years_since_genesis: u32,
}

impl Economics {
    pub fn new() -> Self {
        Economics {
            current_inflation_rate: 0.04,  // 4% starting
            current_burn_rate: 0.01,       // 1% burn rate
            validator_apy: 0.1333,          // 13.33% max APY
            total_burned: 0,
            years_since_genesis: 0,
        }
    }
    
    pub fn get_inflation_rate(&self, year: u32) -> f64 {
        match year {
            0..=1 => 0.04,   // Year 1: 4%
            2 => 0.035,      // Year 2: 3.5%
            3 => 0.03,       // Year 3: 3%
            4 => 0.025,      // Year 4: 2.5%
            _ => 0.02,       // Year 5+: 2% (floor)
        }
    }
    
    pub fn calculate_validator_apy(&self, staking_ratio: f64) -> f64 {
        // Dynamic APY based on staking ratio
        // If 30% staked: APY = inflation / staking_ratio
        // Max capped at 13.33%
        let calculated_apy = self.current_inflation_rate / staking_ratio;
        calculated_apy.min(0.1333)  // Cap at 13.33%
    }
    
    pub fn apply_burn(&mut self, amount: u64) -> u64 {
        let burn_amount = (amount as f64 * self.current_burn_rate) as u64;
        self.total_burned += burn_amount;
        burn_amount
    }
    
    pub fn is_deflationary(&self) -> bool {
        // Becomes deflationary when burn > inflation
        self.current_burn_rate > self.current_inflation_rate
    }
    
    pub fn update_year(&mut self) {
        self.years_since_genesis += 1;
        self.current_inflation_rate = self.get_inflation_rate(self.years_since_genesis);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_inflation_schedule() {
        let econ = Economics::new();
        assert_eq!(econ.get_inflation_rate(0), 0.04);
        assert_eq!(econ.get_inflation_rate(2), 0.035);
        assert_eq!(econ.get_inflation_rate(5), 0.02);
    }
    
    #[test]
    fn test_validator_apy_cap() {
        let econ = Economics::new();
        // With 30% staking ratio
        assert_eq!(econ.calculate_validator_apy(0.30), 0.1333);
        // With 20% staking ratio (would be 20% but capped)
        assert_eq!(econ.calculate_validator_apy(0.20), 0.1333);
    }
}
