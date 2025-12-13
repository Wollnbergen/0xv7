package app

import (
    "encoding/json"
    "io"
    "os"
    "path/filepath"
    
    "cosmossdk.io/log"
    storetypes "cosmossdk.io/store/types"
    dbm "github.com/cosmos/cosmos-db"
    abci "github.com/cosmos/cosmos-sdk/abci/types"
    "github.com/cosmos/cosmos-sdk/baseapp"
    "github.com/cosmos/cosmos-sdk/client"
    "github.com/cosmos/cosmos-sdk/codec"
    "github.com/cosmos/cosmos-sdk/codec/address"
    codectypes "github.com/cosmos/cosmos-sdk/codec/types"
    "github.com/cosmos/cosmos-sdk/runtime"
    servertypes "github.com/cosmos/cosmos-sdk/server/types"
    sdk "github.com/cosmos/cosmos-sdk/types"
    "github.com/cosmos/cosmos-sdk/types/module"
    "github.com/cosmos/cosmos-sdk/x/auth"
    authkeeper "github.com/cosmos/cosmos-sdk/x/auth/keeper"
    authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
    "github.com/cosmos/cosmos-sdk/x/bank"
    bankkeeper "github.com/cosmos/cosmos-sdk/x/bank/keeper"
    banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
)

const (
    AppName = "sultan"
)

var (
    DefaultNodeHome string
)

func init() {
    userHomeDir, err := os.UserHomeDir()
    if err != nil {
        panic(err)
    }
    DefaultNodeHome = filepath.Join(userHomeDir, ".sultan")
}

type App struct {
    *baseapp.BaseApp
    
    legacyAmino       *codec.LegacyAmino
    appCodec          codec.Codec
    txConfig          client.TxConfig
    interfaceRegistry codectypes.InterfaceRegistry
    
    // Keepers
    AccountKeeper authkeeper.AccountKeeper
    BankKeeper    bankkeeper.Keeper
    
    // Module Manager
    ModuleManager *module.Manager
}

// NewApp creates a new Sultan application
func NewApp(
    logger log.Logger,
    db dbm.DB,
    traceWriter io.Writer,
    loadLatest bool,
    appOpts servertypes.AppOptions,
    baseAppOptions ...func(*baseapp.BaseApp),
) *App {
    // Create encoding config
    encodingConfig := MakeEncodingConfig()
    
    appCodec := encodingConfig.Codec
    legacyAmino := encodingConfig.Amino
    interfaceRegistry := encodingConfig.InterfaceRegistry
    txConfig := encodingConfig.TxConfig
    
    // Create BaseApp
    bApp := baseapp.NewBaseApp(
        AppName,
        logger,
        db,
        txConfig.TxDecoder(),
        baseAppOptions...,
    )
    bApp.SetCommitMultiStoreTracer(traceWriter)
    bApp.SetInterfaceRegistry(interfaceRegistry)
    
    // Create store keys
    keys := storetypes.NewKVStoreKeys(
        authtypes.StoreKey,
        banktypes.StoreKey,
    )
    
    // Create the application
    app := &App{
        BaseApp:           bApp,
        legacyAmino:       legacyAmino,
        appCodec:          appCodec,
        txConfig:          txConfig,
        interfaceRegistry: interfaceRegistry,
    }
    
    // Create address codec
    addrCodec := address.NewBech32Codec(sdk.GetConfig().GetBech32AccountAddrPrefix())
    
    // Initialize keepers
    app.AccountKeeper = authkeeper.NewAccountKeeper(
        appCodec,
        runtime.NewKVStoreService(keys[authtypes.StoreKey]),
        authtypes.ProtoBaseAccount,
        map[string][]string{},
        addrCodec,
        sdk.GetConfig().GetBech32AccountAddrPrefix(),
        authtypes.NewModuleAddress("gov").String(),
    )
    
    app.BankKeeper = bankkeeper.NewBaseKeeper(
        appCodec,
        runtime.NewKVStoreService(keys[banktypes.StoreKey]),
        app.AccountKeeper,
        map[string]bool{},
        authtypes.NewModuleAddress("gov").String(),
        logger,
    )
    
    // Create module manager
    app.ModuleManager = module.NewManager(
        auth.NewAppModule(appCodec, app.AccountKeeper, nil, nil),
        bank.NewAppModule(appCodec, app.BankKeeper, app.AccountKeeper, nil),
    )
    
    // Set up routes
    app.ModuleManager.SetOrderInitGenesis(
        authtypes.ModuleName,
        banktypes.ModuleName,
    )
    
    app.ModuleManager.SetOrderBeginBlockers(
        authtypes.ModuleName,
        banktypes.ModuleName,
    )
    
    app.ModuleManager.SetOrderEndBlockers(
        authtypes.ModuleName,
        banktypes.ModuleName,
    )
    
    // Register services
    app.ModuleManager.RegisterServices(module.NewConfigurator(appCodec, bApp.MsgServiceRouter(), bApp.GRPCQueryRouter()))
    
    // Mount stores
    bApp.MountKVStores(keys)
    
    // Initialize BaseApp
    bApp.SetInitChainer(app.InitChainer)
    bApp.SetBeginBlocker(app.BeginBlocker)
    bApp.SetEndBlocker(app.EndBlocker)
    
    // Zero gas ante handler
    anteHandler := NewZeroGasAnteHandler()
    bApp.SetAnteHandler(anteHandler)
    
    if loadLatest {
        if err := bApp.LoadLatestVersion(); err != nil {
            panic(err)
        }
    }
    
    return app
}

// Zero gas ante handler - Sultan Chain has ZERO fees!
func NewZeroGasAnteHandler() sdk.AnteHandler {
    return func(ctx sdk.Context, tx sdk.Tx, simulate bool) (sdk.Context, error) {
        // Skip all gas fees
        return ctx.WithGasMeter(storetypes.NewInfiniteGasMeter()), nil
    }
}

// InitChainer initializes the blockchain
func (app *App) InitChainer(ctx sdk.Context, req *abci.RequestInitChain) (*abci.ResponseInitChain, error) {
    var genesisState GenesisState
    if err := json.Unmarshal(req.AppStateBytes, &genesisState); err != nil {
        panic(err)
    }
    response, err := app.ModuleManager.InitGenesis(ctx, app.appCodec, genesisState)
    if err != nil {
        return nil, err
    }
    return &abci.ResponseInitChain{
        Validators: response.Validators,
    }, nil
}

// BeginBlocker processes begin block
func (app *App) BeginBlocker(ctx sdk.Context) (sdk.BeginBlock, error) {
    return app.ModuleManager.BeginBlock(ctx)
}

// EndBlocker processes end block
func (app *App) EndBlocker(ctx sdk.Context) (sdk.EndBlock, error) {
    return app.ModuleManager.EndBlock(ctx)
}

// ExportAppStateAndValidators exports the application state
func (app *App) ExportAppStateAndValidators(
    forZeroHeight bool,
    jailAllowedAddrs []string,
    modulesToExport []string,
) (servertypes.ExportedApp, error) {
    ctx := app.NewContext(true)
    
    genesis, err := app.ModuleManager.ExportGenesisForModules(ctx, app.appCodec, modulesToExport)
    if err != nil {
        return servertypes.ExportedApp{}, err
    }
    
    appState, err := json.MarshalIndent(genesis, "", "  ")
    if err != nil {
        return servertypes.ExportedApp{}, err
    }
    
    return servertypes.ExportedApp{
        AppState:        appState,
        Height:          app.LastBlockHeight(),
    }, nil
}

type GenesisState map[string]json.RawMessage

// Name returns the app name
func (app *App) Name() string { return AppName }

// LegacyAmino returns the legacy amino codec
func (app *App) LegacyAmino() *codec.LegacyAmino {
    return app.legacyAmino
}

// AppCodec returns the app codec
func (app *App) AppCodec() codec.Codec {
    return app.appCodec
}
