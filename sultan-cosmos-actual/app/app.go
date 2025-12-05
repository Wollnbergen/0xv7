package app

import (
    "io"
    "os"
    "path/filepath"

    "cosmossdk.io/depinject"
    "cosmossdk.io/log"
    storetypes "cosmossdk.io/store/types"
    
    "github.com/cosmos/cosmos-sdk/baseapp"
    "github.com/cosmos/cosmos-sdk/codec"
    "github.com/cosmos/cosmos-sdk/codec/types"
    "github.com/cosmos/cosmos-sdk/runtime"
    "github.com/cosmos/cosmos-sdk/server/api"
    "github.com/cosmos/cosmos-sdk/server/config"
    servertypes "github.com/cosmos/cosmos-sdk/server/types"
    sdk "github.com/cosmos/cosmos-sdk/types"
    "github.com/cosmos/cosmos-sdk/types/module"
    "github.com/cosmos/cosmos-sdk/x/auth"
    authkeeper "github.com/cosmos/cosmos-sdk/x/auth/keeper"
    authsims "github.com/cosmos/cosmos-sdk/x/auth/simulation"
    authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
    "github.com/cosmos/cosmos-sdk/x/auth/vesting"
    "github.com/cosmos/cosmos-sdk/x/bank"
    bankkeeper "github.com/cosmos/cosmos-sdk/x/bank/keeper"
    banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
    "github.com/cosmos/cosmos-sdk/x/consensus"
    consensuskeeper "github.com/cosmos/cosmos-sdk/x/consensus/keeper"
    consensustypes "github.com/cosmos/cosmos-sdk/x/consensus/types"
    "github.com/cosmos/cosmos-sdk/x/staking"
    stakingkeeper "github.com/cosmos/cosmos-sdk/x/staking/keeper"
    stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
    
    dbm "github.com/cosmos/cosmos-db"
)

const (
    AppName = "SultanApp"
    AccountAddressPrefix = "sultan"
)

var (
    // Module account permissions
    maccPerms = map[string][]string{
        authtypes.FeeCollectorName:     nil,
        stakingtypes.BondedPoolName:    {authtypes.Burner, authtypes.Staking},
        stakingtypes.NotBondedPoolName: {authtypes.Burner, authtypes.Staking},
    }
)

type SultanApp struct {
    *baseapp.BaseApp
    
    cdc               *codec.LegacyAmino
    appCodec          codec.Codec
    interfaceRegistry types.InterfaceRegistry
    
    // Keepers
    AccountKeeper    authkeeper.AccountKeeper
    BankKeeper       bankkeeper.Keeper
    StakingKeeper    *stakingkeeper.Keeper
    ConsensusKeeper  consensuskeeper.Keeper
    
    // Module manager
    ModuleManager *module.Manager
    
    // Simulation manager
    sm *module.SimulationManager
}

// NewSultanApp creates and initializes a new SultanApp
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
    
    bApp := baseapp.NewBaseApp(AppName, logger, db, encodingConfig.TxConfig.TxDecoder(), baseAppOptions...)
    bApp.SetCommitMultiStoreTracer(traceStore)
    bApp.SetVersion(version.Version)
    bApp.SetInterfaceRegistry(interfaceRegistry)
    
    keys := storetypes.NewKVStoreKeys(
        authtypes.StoreKey, banktypes.StoreKey, stakingtypes.StoreKey,
        consensustypes.StoreKey,
    )
    
    app := &SultanApp{
        BaseApp:           bApp,
        cdc:              legacyAmino,
        appCodec:         appCodec,
        interfaceRegistry: interfaceRegistry,
    }
    
    // Set up keepers
    app.AccountKeeper = authkeeper.NewAccountKeeper(
        appCodec, runtime.NewKVStoreService(keys[authtypes.StoreKey]),
        authtypes.ProtoBaseAccount, maccPerms, AccountAddressPrefix, authtypes.NewModuleAddress("gov").String(),
    )
    
    app.BankKeeper = bankkeeper.NewBaseKeeper(
        appCodec, runtime.NewKVStoreService(keys[banktypes.StoreKey]),
        app.AccountKeeper, nil, authtypes.NewModuleAddress("gov").String(), logger,
    )
    
    app.StakingKeeper = stakingkeeper.NewKeeper(
        appCodec, runtime.NewKVStoreService(keys[stakingtypes.StoreKey]),
        app.AccountKeeper, app.BankKeeper, authtypes.NewModuleAddress("gov").String(),
        authcodec.NewBech32Codec(AccountAddressPrefix+"valoper"),
        authcodec.NewBech32Codec(AccountAddressPrefix+"valcons"),
    )
    
    app.ConsensusKeeper = consensuskeeper.NewKeeper(
        appCodec, runtime.NewKVStoreService(keys[consensustypes.StoreKey]),
        authtypes.NewModuleAddress("gov").String(),
    )
    
    // Create module manager
    app.ModuleManager = module.NewManager(
        auth.NewAppModule(appCodec, app.AccountKeeper, authsims.RandomGenesisAccounts, nil),
        bank.NewAppModule(appCodec, app.BankKeeper, app.AccountKeeper, nil),
        staking.NewAppModule(appCodec, app.StakingKeeper, app.AccountKeeper, app.BankKeeper, nil),
        consensus.NewAppModule(appCodec, app.ConsensusKeeper),
    )
    
    // Set up routes
    app.ModuleManager.RegisterServices(module.NewConfigurator(appCodec, bApp.MsgServiceRouter(), bApp.GRPCQueryRouter()))
    app.ModuleManager.RegisterInvariants(nil)
    
    // Initialize stores
    app.MountKVStores(keys)
    app.SetInitChainer(app.InitChainer)
    app.SetBeginBlocker(app.BeginBlocker)
    app.SetEndBlocker(app.EndBlocker)
    
    if loadLatest {
        if err := app.LoadLatestVersion(); err != nil {
            panic(err)
        }
    }
    
    return app
}

// InitChainer handles chain initialization
func (app *SultanApp) InitChainer(ctx sdk.Context, req *types.RequestInitChain) (*types.ResponseInitChain, error) {
    var genesisState GenesisState
    if err := json.Unmarshal(req.AppStateBytes, &genesisState); err != nil {
        return nil, err
    }
    
    app.UpgradeKeeper.SetModuleVersionMap(ctx, app.ModuleManager.GetVersionMap())
    return app.ModuleManager.InitGenesis(ctx, app.appCodec, genesisState)
}

// BeginBlocker runs at the beginning of every block
func (app *SultanApp) BeginBlocker(ctx sdk.Context) (sdk.BeginBlock, error) {
    return app.ModuleManager.BeginBlock(ctx)
}

// EndBlocker runs at the end of every block
func (app *SultanApp) EndBlocker(ctx sdk.Context) (sdk.EndBlock, error) {
    return app.ModuleManager.EndBlock(ctx)
}
