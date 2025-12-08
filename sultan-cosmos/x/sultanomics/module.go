package sultanomics

import (
    sdk "github.com/cosmos/cosmos-sdk/types"
)

// SultanEconomics overrides Cosmos economics with Sultan's model
type SultanEconomics struct {
    MaxAPY         sdk.Dec  // 0.1333 (13.33%)
    BaseInflation  sdk.Dec  // 0.08 (8%)
    TargetBonded   sdk.Dec  // 0.30 (30%)
}

func NewSultanEconomics() SultanEconomics {
    return SultanEconomics{
        MaxAPY:        sdk.MustNewDecFromStr("0.1333"),
        BaseInflation: sdk.MustNewDecFromStr("0.08"),
        TargetBonded:  sdk.MustNewDecFromStr("0.30"),
    }
}

// CalculateInflation returns inflation to achieve 13.33% APY
func (se SultanEconomics) CalculateInflation(bondedRatio sdk.Dec) sdk.Dec {
    // APY = inflation / bonded_ratio
    // To get 13.33% APY: inflation = 0.1333 * bonded_ratio
    targetInflation := se.MaxAPY.Mul(bondedRatio)
    
    // Ensure minimum 4% inflation (Sultan's base rate)
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

// GetValidatorAPY always returns 13.33% for Sultan
func (se SultanEconomics) GetValidatorAPY() string {
    return "13.33%"
}

// GetZeroGasFee returns zero for all transactions
func (se SultanEconomics) GetZeroGasFee() sdk.Coins {
    return sdk.NewCoins() // Always $0.00
}
