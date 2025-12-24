// Sultan Chain Economics API
const ECONOMICS = {
    inflation_rate: 4.0,  // Fixed 4% forever - guarantees zero fees at 76M+ TPS
    burn_rate: 1.0,  // 1% burn on high-volume transactions
    validator_apy: {
        base: 13.33,      // Maximum 13.33%
        mobile_bonus: 0,  // No mobile bonus
        min: 5.0,
        max: 13.33
    },
    gas_fees: 0.00,  // Users still pay nothing
    
    getCurrentInflation: function(_year) {
        // Fixed 4% forever
        return this.inflation_rate;
    },
    
    calculateValidatorAPY: function(stakingRatio) {
        const currentYear = 1;  // For testnet
        const inflation = this.getCurrentInflation(currentYear);
        const calculatedAPY = (inflation / 100) / stakingRatio;
        const apyPercent = calculatedAPY * 100;
        
        // Cap at maximum
        return Math.min(apyPercent, this.validator_apy.max);
    },
    
    isDeflationary: function(year) {
        const inflation = this.getCurrentInflation(year);
        return this.burn_rate > inflation;
    }
};

module.exports = ECONOMICS;
