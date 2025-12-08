package app

import (
    "github.com/cosmos/cosmos-sdk/baseapp"
    sdk "github.com/cosmos/cosmos-sdk/types"
    "github.com/cosmos/cosmos-sdk/x/auth"
    "github.com/cosmos/cosmos-sdk/x/bank"
    "github.com/cosmos/cosmos-sdk/x/staking"
    "github.com/cosmos/cosmos-sdk/x/gov"
    ibctransfer "github.com/cosmos/ibc-go/v8/modules/apps/transfer"
    ibc "github.com/cosmos/ibc-go/v8/modules/core"
)

const (
    AppName = "SultanChain"
    TokenDenom = "usltn" // micro-sultan
)

type SultanApp struct {
    *baseapp.BaseApp
    
    // Zero fee configuration
    ZeroFees bool
    
    // Staking APY: 13.33%
    StakingAPY float64
    
    // IBC enabled
    IBCEnabled bool
}

func NewSultanApp() *SultanApp {
    app := &SultanApp{
        BaseApp: baseapp.NewBaseApp(AppName, nil, nil, nil),
        ZeroFees: true,           // ZERO GAS FEES
        StakingAPY: 0.1333,       // 13.33% APY
        IBCEnabled: true,         // IBC Cross-chain
    }
    return app
}

func (app *SultanApp) GetZeroFeeStatus() bool {
    return app.ZeroFees // Always true - $0.00 fees forever!
}
