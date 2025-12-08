package app

import (
    "github.com/cosmos/cosmos-sdk/baseapp"
    "github.com/cosmos/cosmos-sdk/codec"
    sdk "github.com/cosmos/cosmos-sdk/types"
    "github.com/cosmos/cosmos-sdk/x/auth"
    "github.com/cosmos/cosmos-sdk/x/bank"
    "github.com/cosmos/cosmos-sdk/x/staking"
    "github.com/tendermint/tendermint/libs/log"
)

const appName = "SultanChain"

type SultanApp struct {
    *baseapp.BaseApp
    cdc *codec.Codec
    
    // Keys to access the substores
    keyMain    *sdk.KVStoreKey
    keyAccount *sdk.KVStoreKey
    keyStaking *sdk.KVStoreKey
    
    // Keepers
    accountKeeper auth.AccountKeeper
    bankKeeper    bank.Keeper
    stakingKeeper staking.Keeper
    
    // Sultan custom modules
    mobileValidatorKeeper MobileValidatorKeeper
    rewardsKeeper        RewardsKeeper
}

// NewSultanApp creates a new Sultan blockchain app
func NewSultanApp(logger log.Logger, db dbm.DB) *SultanApp {
    cdc := MakeCodec()
    
    bApp := baseapp.NewBaseApp(appName, logger, db, auth.DefaultTxDecoder(cdc))
    
    var app = &SultanApp{
        BaseApp:    bApp,
        cdc:        cdc,
        keyMain:    sdk.NewKVStoreKey("main"),
        keyAccount: sdk.NewKVStoreKey("acc"),
        keyStaking: sdk.NewKVStoreKey("staking"),
    }
    
    // Initialize keepers with PRODUCTION configuration
    app.accountKeeper = auth.NewAccountKeeper(
        app.cdc,
        app.keyAccount,
        ProtoAccount,
    )
    
    app.bankKeeper = bank.NewBaseKeeper(
        app.accountKeeper,
        app.keyMain,
        app.BlacklistedAddrs(),
    )
    
    app.stakingKeeper = staking.NewKeeper(
        app.cdc,
        app.keyStaking,
        app.accountKeeper,
        app.bankKeeper,
        staking.DefaultParams(),
    )
    
    // Sultan custom modules
    app.mobileValidatorKeeper = NewMobileValidatorKeeper(
        app.cdc,
        app.keyMain,
        app.stakingKeeper,
    )
    
    return app
}
