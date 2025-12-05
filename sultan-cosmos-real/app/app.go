package app

import (
    "encoding/json"
    "io"
    "os"
    "path/filepath"

    "cosmossdk.io/log"
    storetypes "cosmossdk.io/store/types"
    "cosmossdk.io/x/tx/signing"
    
    "github.com/cosmos/cosmos-sdk/baseapp"
    "github.com/cosmos/cosmos-sdk/client"
    "github.com/cosmos/cosmos-sdk/codec"
    "github.com/cosmos/cosmos-sdk/codec/types"
    "github.com/cosmos/cosmos-sdk/runtime"
    "github.com/cosmos/cosmos-sdk/server/api"
    serverconfig "github.com/cosmos/cosmos-sdk/server/config"
    servertypes "github.com/cosmos/cosmos-sdk/server/types"
    sdk "github.com/cosmos/cosmos-sdk/types"
    "github.com/cosmos/cosmos-sdk/types/module"
    "github.com/cosmos/cosmos-sdk/version"
    "github.com/cosmos/cosmos-sdk/x/auth"
    "github.com/cosmos/cosmos-sdk/x/auth/ante"
    authcodec "github.com/cosmos/cosmos-sdk/x/auth/codec"
    authkeeper "github.com/cosmos/cosmos-sdk/x/auth/keeper"
    "github.com/cosmos/cosmos-sdk/x/auth/posthandler"
    authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
    "github.com/cosmos/cosmos-sdk/x/bank"
    bankkeeper "github.com/cosmos/cosmos-sdk/x/bank/keeper"
    banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
    "github.com/cosmos/cosmos-sdk/x/consensus"
    consensuskeeper "github.com/cosmos/cosmos-sdk/x/consensus/keeper"
    consensustypes "github.com/cosmos/cosmos-sdk/x/consensus/types"
    "github.com/cosmos/cosmos-sdk/x/genutil"
    genutiltypes "github.com/cosmos/cosmos-sdk/x/genutil/types"
    "github.com/cosmos/cosmos-sdk/x/staking"
    stakingkeeper "github.com/cosmos/cosmos-sdk/x/staking/keeper"
    stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
    
    dbm "github.com/cosmos/cosmos-db"
    abci "github.com/cometbft/cometbft/abci/types"
)

const (
    AppName = "SultanChain"
    AccountAddressPrefix = "sultan"
)

var (
    DefaultNodeHome string
    ModuleBasics = module.NewBasicManager(
        auth.AppModuleBasic{},
        bank.AppModuleBasic{},
        staking.AppModuleBasic{},
        genutil.NewAppModuleBasic(genutiltypes.DefaultMessageValidator),
        consensus.AppModuleBasic{},
    )
)

func init() {
    userHomeDir, err := os.UserHomeDir()
    if err != nil {
        panic(err)
    }
    DefaultNodeHome = filepath.Join(userHomeDir, ".sultan")
}

// SultanApp extends an ABCI application with real Cosmos SDK and Tendermint
type SultanApp struct {
    *baseapp.BaseApp
    legacyAmino       *codec.LegacyAmino
    appCodec          codec.Codec
    txConfig          client.TxConfig
    interfaceRegistry types.InterfaceRegistry

    // Keys to access the substores
    keys    map[string]*storetypes.KVStoreKey
    tkeys   map[string]*storetypes.TransientStoreKey
    memKeys map[string]*storetypes.MemoryStoreKey

    // Keepers - these handle the actual blockchain logic
    AccountKeeper    authkeeper.AccountKeeper
    BankKeeper       bankkeeper.BaseKeeper  
    StakingKeeper    *stakingkeeper.Keeper
    ConsensusKeeper  consensuskeeper.Keeper

    // Module Manager
    ModuleManager      *module.Manager
    BasicModuleManager module.BasicManager
}

// NewSultanApp creates and initializes a new Sultan application with real Cosmos SDK
func NewSultanApp(
    logger log.Logger,
    db dbm.DB,
    traceStore io.Writer,
    loadLatest bool,
    appOpts servertypes.AppOptions,
    baseAppOptions ...func(*baseapp.BaseApp),
) *SultanApp {
    encodingConfig := MakeEncodingConfig()
    appCodec := encodingConfig.Codec
    legacyAmino := encodingConfig.Amino
    interfaceRegistry := encodingConfig.InterfaceRegistry
    txConfig := encodingConfig.TxConfig

    bApp := baseapp.NewBaseApp(AppName, logger, db, txConfig.TxDecoder(), baseAppOptions...)
    bApp.SetCommitMultiStoreTracer(traceStore)
    bApp.SetVersion(version.Version)
    bApp.SetInterfaceRegistry(interfaceRegistry)
    bApp.SetTxEncoder(txConfig.TxEncoder())

    keys := storetypes.NewKVStoreKeys(
        authtypes.StoreKey,
        banktypes.StoreKey,
        stakingtypes.StoreKey,
        consensustypes.StoreKey,
    )
    // Note: genutil doesn't need a dedicated store key

    // No transient keys needed for basic setup
    tkeys := storetypes.NewTransientStoreKeys()

    memKeys := storetypes.NewMemoryStoreKeys()

    app := &SultanApp{
        BaseApp:           bApp,
        legacyAmino:       legacyAmino,
        appCodec:          appCodec,
        txConfig:          txConfig,
        interfaceRegistry: interfaceRegistry,
        keys:              keys,
        tkeys:             tkeys,
        memKeys:           memKeys,
    }

    // Setup keepers with proper store services
    storeService := runtime.NewKVStoreService(keys[authtypes.StoreKey])
    
    app.AccountKeeper = authkeeper.NewAccountKeeper(
        appCodec,
        storeService,
        authtypes.ProtoBaseAccount,
        map[string][]string{
            stakingtypes.NotBondedPoolName: {authtypes.Minter, authtypes.Burner},
            stakingtypes.BondedPoolName:    {authtypes.Minter, authtypes.Burner},
        },
        authcodec.NewBech32Codec(AccountAddressPrefix),
        AccountAddressPrefix,
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

    app.StakingKeeper = stakingkeeper.NewKeeper(
        appCodec,
        runtime.NewKVStoreService(keys[stakingtypes.StoreKey]),
        app.AccountKeeper,
        app.BankKeeper,
        authtypes.NewModuleAddress("gov").String(),
        authcodec.NewBech32Codec(AccountAddressPrefix+"valoper"),
        authcodec.NewBech32Codec(AccountAddressPrefix+"valcons"),
    )

    app.ConsensusKeeper = consensuskeeper.NewKeeper(
        appCodec,
        runtime.NewKVStoreService(keys[consensustypes.StoreKey]),
        authtypes.NewModuleAddress("gov").String(),
        runtime.EventService{},
    )

    // BaseApp needs the consensus params store for handshake and updates
    // Pass the ParamsStore field which implements the ParamStore interface (Get/Has/Set)
    bApp.SetParamStore(app.ConsensusKeeper.ParamsStore)

    // Create module manager
    app.ModuleManager = module.NewManager(
        auth.NewAppModule(appCodec, app.AccountKeeper, nil, nil),
        bank.NewAppModule(appCodec, app.BankKeeper, app.AccountKeeper, nil),
        staking.NewAppModule(appCodec, app.StakingKeeper, app.AccountKeeper, app.BankKeeper, nil),
        genutil.NewAppModule(app.AccountKeeper, app.StakingKeeper, app, txConfig),
        consensus.NewAppModule(appCodec, app.ConsensusKeeper),
    )

    // Set up routes and handlers
    app.ModuleManager.SetOrderInitGenesis(
        authtypes.ModuleName,
        banktypes.ModuleName,
        stakingtypes.ModuleName,
        genutiltypes.ModuleName,
        consensustypes.ModuleName,
    )

    app.ModuleManager.SetOrderBeginBlockers(
        authtypes.ModuleName,
        stakingtypes.ModuleName,
        consensustypes.ModuleName,
    )

    app.ModuleManager.SetOrderEndBlockers(
        stakingtypes.ModuleName,
        authtypes.ModuleName,
        consensustypes.ModuleName,
    )

    app.ModuleManager.RegisterServices(module.NewConfigurator(appCodec, bApp.MsgServiceRouter(), bApp.GRPCQueryRouter()))
    // Skip invariants registration for now to avoid nil keeper panics during startup.
    // Invariants can be re-enabled once all keepers are fully wired and crisis module is added.
    // app.ModuleManager.RegisterInvariants(ir)

    // Mount stores
    bApp.MountKVStores(keys)
    bApp.MountTransientStores(tkeys)
    bApp.MountMemoryStores(memKeys)

    // Set InitChainer
    bApp.SetInitChainer(app.InitChainer)
    bApp.SetPreBlocker(app.PreBlocker)
    bApp.SetBeginBlocker(app.BeginBlocker)
    bApp.SetEndBlocker(app.EndBlocker)
    
    // Set AnteHandler for ZERO FEES!
    anteHandler, err := NewAnteHandler(
        HandlerOptions{
            AccountKeeper:   app.AccountKeeper,
            BankKeeper:      app.BankKeeper,
            SignModeHandler: txConfig.SignModeHandler(),
            SigGasConsumer:  ante.DefaultSigVerificationGasConsumer,
        },
    )
    if err != nil {
        panic(err)
    }
    bApp.SetAnteHandler(anteHandler)

    // Set PostHandler
    postHandler, err := posthandler.NewPostHandler(
        posthandler.HandlerOptions{},
    )
    if err != nil {
        panic(err)
    }
    bApp.SetPostHandler(postHandler)

    if loadLatest {
        if err := bApp.LoadLatestVersion(); err != nil {
            panic(err)
        }
    }

    return app
}

// HandlerOptions are the options for constructing an AnteHandler
type HandlerOptions struct {
    AccountKeeper   authkeeper.AccountKeeper
    BankKeeper      bankkeeper.BaseKeeper
    SignModeHandler *signing.HandlerMap
    SigGasConsumer  ante.SignatureVerificationGasConsumer
}

// NewAnteHandler creates a new AnteHandler with ZERO FEES!
func NewAnteHandler(options HandlerOptions) (sdk.AnteHandler, error) {
    anteDecorators := []sdk.AnteDecorator{
        ante.NewSetUpContextDecorator(),
        ante.NewExtensionOptionsDecorator(nil),
        ante.NewValidateBasicDecorator(),
        ante.NewTxTimeoutHeightDecorator(),
        ante.NewValidateMemoDecorator(options.AccountKeeper),
        // Skip fee decorators for ZERO FEES!
        // ante.NewDeductFeeDecorator(options.AccountKeeper, options.BankKeeper, nil, nil),
        ante.NewSetPubKeyDecorator(options.AccountKeeper),
        ante.NewValidateSigCountDecorator(options.AccountKeeper),
        ante.NewSigGasConsumeDecorator(options.AccountKeeper, options.SigGasConsumer),
        ante.NewSigVerificationDecorator(options.AccountKeeper, options.SignModeHandler),
        ante.NewIncrementSequenceDecorator(options.AccountKeeper),
    }
    
    return sdk.ChainAnteDecorators(anteDecorators...), nil
}

// InitChainer initializes the blockchain
func (app *SultanApp) InitChainer(ctx sdk.Context, req *abci.RequestInitChain) (*abci.ResponseInitChain, error) {
    var genesisState GenesisState
    if err := json.Unmarshal(req.AppStateBytes, &genesisState); err != nil {
        panic(err)
    }
    
    // Initialize module versions
    return app.ModuleManager.InitGenesis(ctx, app.appCodec, genesisState)
}

// PreBlocker runs before each block
func (app *SultanApp) PreBlocker(ctx sdk.Context, req *abci.RequestFinalizeBlock) (*sdk.ResponsePreBlock, error) {
    return app.ModuleManager.PreBlock(ctx)
}

// BeginBlocker processes begin block
func (app *SultanApp) BeginBlocker(ctx sdk.Context) (sdk.BeginBlock, error) {
    return app.ModuleManager.BeginBlock(ctx)
}

// EndBlocker processes end block
func (app *SultanApp) EndBlocker(ctx sdk.Context) (sdk.EndBlock, error) {
    return app.ModuleManager.EndBlock(ctx)
}

// Name returns the app name
func (app *SultanApp) Name() string { return app.BaseApp.Name() }

type GenesisState map[string]json.RawMessage

// RegisterAPIRoutes implements servertypes.Application requirement in v0.50
func (app *SultanApp) RegisterAPIRoutes(apiSvr *api.Server, apiCfg serverconfig.APIConfig) {}

// RegisterTxService implements servertypes.Application requirement
func (app *SultanApp) RegisterTxService(clientCtx client.Context) {}

// RegisterTendermintService implements servertypes.Application requirement
func (app *SultanApp) RegisterTendermintService(clientCtx client.Context) {}

// RegisterNodeService implements servertypes.Application requirement
func (app *SultanApp) RegisterNodeService(clientCtx client.Context, cfg serverconfig.Config) {}
