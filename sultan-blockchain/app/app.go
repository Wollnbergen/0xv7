package app

import (
    "io"
    
    "cosmossdk.io/log"
    storetypes "cosmossdk.io/store/types"
    dbm "github.com/cometbft/cometbft-db"
    "github.com/cosmos/cosmos-sdk/baseapp"
    "github.com/cosmos/cosmos-sdk/codec"
    sdk "github.com/cosmos/cosmos-sdk/types"
)

const AppName = "SultanChain"

type SultanApp struct {
    *baseapp.BaseApp
    legacyAmino *codec.LegacyAmino
    appCodec    codec.Codec
}

func NewSultanApp(
    logger log.Logger,
    db dbm.DB,
    traceStore io.Writer,
    loadLatest bool,
    appOpts any,
    baseAppOptions ...func(*baseapp.BaseApp),
) *SultanApp {
    encodingConfig := MakeEncodingConfig()
    
    bApp := baseapp.NewBaseApp(
        AppName,
        logger,
        db,
        encodingConfig.TxConfig.TxDecoder(),
        baseAppOptions...,
    )
    bApp.SetCommitMultiStoreTracer(traceStore)
    
    app := &SultanApp{
        BaseApp:     bApp,
        legacyAmino: encodingConfig.Amino,
        appCodec:    encodingConfig.Codec,
    }
    
    // Initialize stores
    keys := storetypes.NewKVStoreKeys("main")
    bApp.MountStores(keys)
    
    if loadLatest {
        if err := bApp.LoadLatestVersion(); err != nil {
            panic(err)
        }
    }
    
    return app
}

func (app *SultanApp) Name() string { return AppName }

func (app *SultanApp) LegacyAmino() *codec.LegacyAmino {
    return app.legacyAmino
}

func (app *SultanApp) AppCodec() codec.Codec {
    return app.appCodec
}
