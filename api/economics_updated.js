// Sultan Chain Economics API
const ECONOMICS = {
    inflation_schedule: {
        year_1: 4.0,
        year_2: 3.0,
        year_3: 2.5,
        year_4: 2.0,
        year_5_onwards: 2.0
    },
    burn_rate: 1.0,  // 1% burn on high-volume transactions
    validator_apy: {
        base: 13.33,      // Maximum 13.33%
        mobile_bonus: 0,  // No mobile bonus
        min: 5.0,
        max: 13.33
    },
    gas_fees: 0.00,  // Users still pay nothing
    
    getCurrentInflation: function(year) {
        if (year <= 1) return this.inflation_schedule.year_1;
        if (year === 2) return this.inflation_schedule.year_2;
        if (year === 3) return this.inflation_schedule.year_3;
        if (year === 4) return this.inflation_schedule.year_4;
        return this.inflation_schedule.year_5_onwards;
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
