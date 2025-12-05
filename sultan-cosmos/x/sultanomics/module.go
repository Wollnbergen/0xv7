package sultanomics

import (
    sdk "github.com/cosmos/cosmos-sdk/types"
)

// SultanEconomics overrides Cosmos economics with Sultan's model
type SultanEconomics struct {
    MaxAPY         sdk.Dec  // 0.2667 (26.67%)
    BaseInflation  sdk.Dec  // 0.08 (8%)
    TargetBonded   sdk.Dec  // 0.30 (30%)
}

func NewSultanEconomics() SultanEconomics {
    return SultanEconomics{
        MaxAPY:        sdk.MustNewDecFromStr("0.2667"),
        BaseInflation: sdk.MustNewDecFromStr("0.08"),
        TargetBonded:  sdk.MustNewDecFromStr("0.30"),
    }
}

// CalculateInflation returns inflation to achieve 26.67% APY
func (se SultanEconomics) CalculateInflation(bondedRatio sdk.Dec) sdk.Dec {
    // APY = inflation / bonded_ratio
    // To get 26.67% APY: inflation = 0.2667 * bonded_ratio
    targetInflation := se.MaxAPY.Mul(bondedRatio)
    
    // Ensure minimum 8% inflation (Sultan's base rate)
    if targetInflation.LT(se.BaseInflation) {
        return se.BaseInflation
    }
    
    // Cap at reasonable maximum
    maxInflation := sdk.MustNewDecFromStr("0.80")
    if targetInflation.GT(maxInflation) {
        return maxInflation
    }
    
    return targetInflation
}

// GetValidatorAPY always returns 26.67% for Sultan
func (se SultanEconomics) GetValidatorAPY() string {
    return "26.67%"
}

// GetZeroGasFee returns zero for all transactions
func (se SultanEconomics) GetZeroGasFee() sdk.Coins {
    return sdk.NewCoins() // Always $0.00
}
