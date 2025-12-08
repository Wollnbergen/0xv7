package zerofee

import (
    sdk "github.com/cosmos/cosmos-sdk/types"
)

// ZeroFeeModule implements zero gas fees for Sultan Chain
type ZeroFeeModule struct{}

// ProcessTransaction with ZERO fees
func (m *ZeroFeeModule) ProcessTransaction(ctx sdk.Context, tx sdk.Tx) error {
    // No fees charged - subsidized by 4% inflation
    return nil
}

// GetTransactionFee always returns 0
func (m *ZeroFeeModule) GetTransactionFee() sdk.Coins {
    return sdk.NewCoins() // Empty = $0.00
}

// ValidatorAPY returns 13.33%
func (m *ZeroFeeModule) GetValidatorAPY() float64 {
    return 0.1333 // 13.33% APY
}
