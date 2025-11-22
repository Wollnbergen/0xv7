package app

import (
	"encoding/json"
	"io"
	"net/http"
	"os"
	"path/filepath"
	
	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/core/genesis"
	
	dbm "github.com/cosmos/cosmos-db"
	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/address"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	"github.com/cosmos/cosmos-sdk/server/api"
	"github.com/cosmos/cosmos-sdk/server/config"
	cmtservice "github.com/cosmos/cosmos-sdk/client/grpc/cmtservice"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	"github.com/cosmos/cosmos-sdk/x/auth"
	authkeeper "github.com/cosmos/cosmos-sdk/x/auth/keeper"
	authsims "github.com/cosmos/cosmos-sdk/x/auth/simulation"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	authtx "github.com/cosmos/cosmos-sdk/x/auth/tx"
	"github.com/cosmos/cosmos-sdk/x/bank"
	bankkeeper "github.com/cosmos/cosmos-sdk/x/bank/keeper"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	bankexported "github.com/cosmos/cosmos-sdk/x/bank/exported"
	"github.com/cosmos/cosmos-sdk/x/consensus"
	consensuskeeper "github.com/cosmos/cosmos-sdk/x/consensus/keeper"
	consensustypes "github.com/cosmos/cosmos-sdk/x/consensus/types"
	"github.com/cosmos/cosmos-sdk/x/staking"
	stakingkeeper "github.com/cosmos/cosmos-sdk/x/staking/keeper"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
	"github.com/cosmos/cosmos-sdk/x/genutil"
	genutiltypes "github.com/cosmos/cosmos-sdk/x/genutil/types"
	
	// Upgrade module - required for IBC
	upgrade "cosmossdk.io/x/upgrade"
	upgradekeeper "cosmossdk.io/x/upgrade/keeper"
	upgradetypes "cosmossdk.io/x/upgrade/types"
	
	// IBC imports - v8 uses capability from ibc-go/modules
	capability "github.com/cosmos/ibc-go/modules/capability"
	capabilitykeeper "github.com/cosmos/ibc-go/modules/capability/keeper"
	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	ibc "github.com/cosmos/ibc-go/v8/modules/core"
	ibcporttypes "github.com/cosmos/ibc-go/v8/modules/core/05-port/types"
	ibcexported "github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibckeeper "github.com/cosmos/ibc-go/v8/modules/core/keeper"
	ibctransfer "github.com/cosmos/ibc-go/v8/modules/apps/transfer"
	ibctransferkeeper "github.com/cosmos/ibc-go/v8/modules/apps/transfer/keeper"
	ibctransfertypes "github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	
	abci "github.com/cometbft/cometbft/abci/types"
	cmttypes "github.com/cometbft/cometbft/types"
	
	sultan "github.com/wollnbergen/sultan-cosmos-module/x/sultan"
	sultankeeper "github.com/wollnbergen/sultan-cosmos-module/x/sultan/keeper"
	sultantypes "github.com/wollnbergen/sultan-cosmos-module/x/sultan/types"
)

var (
	// DefaultNodeHome default home directories for the application daemon
	DefaultNodeHome string
	
	// ModuleBasics defines the module BasicManager that is in charge of setting up basic,
	// non-dependant module elements, such as codec registration and genesis verification.
	ModuleBasics = module.NewBasicManager(
		auth.AppModuleBasic{},
		bank.AppModuleBasic{},
		staking.AppModuleBasic{},
		consensus.AppModuleBasic{},
		genutil.AppModuleBasic{genutiltypes.DefaultMessageValidator},
		upgrade.AppModuleBasic{},
		capability.AppModuleBasic{},
		ibc.AppModuleBasic{},
		ibctransfer.AppModuleBasic{},
		sultan.AppModuleBasic{},
	)
)

var (
	_ runtime.AppI = (*SultanApp)(nil)
)

// SultanApp extends a ABCI application with Sultan blockchain integration
type SultanApp struct {
	*baseapp.BaseApp
	
	legacyAmino       *codec.LegacyAmino
	appCodec          codec.Codec
	txConfig          client.TxConfig
	interfaceRegistry codectypes.InterfaceRegistry
	
	// Keys to access the substores
	keys    map[string]*storetypes.KVStoreKey
	tkeys   map[string]*storetypes.TransientStoreKey
	memKeys map[string]*storetypes.MemoryStoreKey
	
	// Keepers
	AccountKeeper    authkeeper.AccountKeeper
	BankKeeper       bankkeeper.Keeper
	StakingKeeper    *stakingkeeper.Keeper
	ConsensusKeeper  consensuskeeper.Keeper
	UpgradeKeeper    *upgradekeeper.Keeper
	CapabilityKeeper *capabilitykeeper.Keeper
	IBCKeeper        *ibckeeper.Keeper
	TransferKeeper   ibctransferkeeper.Keeper
	SultanKeeper     *sultankeeper.Keeper
	
	// IBC scoped keepers
	ScopedIBCKeeper      capabilitykeeper.ScopedKeeper
	ScopedTransferKeeper capabilitykeeper.ScopedKeeper
	
	// Module manager
	mm           *module.Manager
	sm           *module.SimulationManager
	configurator module.Configurator
}

func init() {
	userHomeDir, err := os.UserHomeDir()
	if err != nil {
		panic(err)
	}
	
	DefaultNodeHome = filepath.Join(userHomeDir, ".sultand")
}

// NewSultanApp returns a reference to an initialized Sultan application
func NewSultanApp(
	logger log.Logger,
	db dbm.DB,
	traceStore io.Writer,
	loadLatest bool,
	appOpts servertypes.AppOptions,
	baseAppOptions ...func(*baseapp.BaseApp),
) *SultanApp {
	// Create encoders
	encodingConfig := MakeEncodingConfig()
	appCodec := encodingConfig.Codec
	legacyAmino := encodingConfig.Amino
	interfaceRegistry := encodingConfig.InterfaceRegistry
	txConfig := encodingConfig.TxConfig
	
	// Create the base app
	bApp := baseapp.NewBaseApp("sultand", logger, db, txConfig.TxDecoder(), baseAppOptions...)
	bApp.SetCommitMultiStoreTracer(traceStore)
	bApp.SetVersion("0.1.0")
	bApp.SetInterfaceRegistry(interfaceRegistry)
	bApp.SetTxEncoder(txConfig.TxEncoder())
	
	app := &SultanApp{
		BaseApp:           bApp,
		legacyAmino:       legacyAmino,
		appCodec:          appCodec,
		txConfig:          txConfig,
		interfaceRegistry: interfaceRegistry,
	}
	
	// Initialize store keys
	app.keys = map[string]*storetypes.KVStoreKey{
		authtypes.StoreKey:      storetypes.NewKVStoreKey(authtypes.StoreKey),
		banktypes.StoreKey:      storetypes.NewKVStoreKey(banktypes.StoreKey),
		stakingtypes.StoreKey:   storetypes.NewKVStoreKey(stakingtypes.StoreKey),
		consensustypes.StoreKey: storetypes.NewKVStoreKey(consensustypes.StoreKey),
		upgradetypes.StoreKey:   storetypes.NewKVStoreKey(upgradetypes.StoreKey),
		capabilitytypes.StoreKey: storetypes.NewKVStoreKey(capabilitytypes.StoreKey),
		ibcexported.StoreKey:    storetypes.NewKVStoreKey(ibcexported.StoreKey),
		ibctransfertypes.StoreKey: storetypes.NewKVStoreKey(ibctransfertypes.StoreKey),
		sultantypes.StoreKey:    storetypes.NewKVStoreKey(sultantypes.StoreKey),
	}
	app.tkeys = map[string]*storetypes.TransientStoreKey{}
	app.memKeys = map[string]*storetypes.MemoryStoreKey{
		capabilitytypes.MemStoreKey: storetypes.NewMemoryStoreKey(capabilitytypes.MemStoreKey),
		sultantypes.MemStoreKey: storetypes.NewMemoryStoreKey(sultantypes.MemStoreKey),
	}
	
	// Create address codecs for accounts, validators, and consensus
	sdkConfig := sdk.GetConfig()
	addressCodec := address.NewBech32Codec(sdkConfig.GetBech32AccountAddrPrefix())
	validatorAddressCodec := address.NewBech32Codec(sdkConfig.GetBech32ValidatorAddrPrefix())
	consensusAddressCodec := address.NewBech32Codec(sdkConfig.GetBech32ConsensusAddrPrefix())
	
	// Create keepers
	app.AccountKeeper = authkeeper.NewAccountKeeper(
		appCodec,
		runtime.NewKVStoreService(app.keys[authtypes.StoreKey]),
		authtypes.ProtoBaseAccount,
		map[string][]string{
			banktypes.ModuleName:      {authtypes.Minter, authtypes.Burner},
			stakingtypes.BondedPoolName: {authtypes.Burner, authtypes.Staking},
			stakingtypes.NotBondedPoolName: {authtypes.Burner, authtypes.Staking},
			ibctransfertypes.ModuleName: {authtypes.Minter, authtypes.Burner},
			sultantypes.ModuleName:    {},
		},
		addressCodec,
		sdk.Bech32MainPrefix,
		authtypes.NewModuleAddress("gov").String(),
	)
	
	app.BankKeeper = bankkeeper.NewBaseKeeper(
		appCodec,
		runtime.NewKVStoreService(app.keys[banktypes.StoreKey]),
		app.AccountKeeper,
		map[string]bool{},
		authtypes.NewModuleAddress("gov").String(),
		logger,
	)
	
	// Create StakingKeeper with proper validator and consensus address codecs
	app.StakingKeeper = stakingkeeper.NewKeeper(
		appCodec,
		runtime.NewKVStoreService(app.keys[stakingtypes.StoreKey]),
		app.AccountKeeper,
		app.BankKeeper,
		authtypes.NewModuleAddress("gov").String(),
		validatorAddressCodec,     // 6th param: validator address codec (cosmosvaloper1...)
		consensusAddressCodec,     // 7th param: consensus address codec (cosmosvalcons1...)
	)
	
	app.ConsensusKeeper = consensuskeeper.NewKeeper(
		appCodec,
		runtime.NewKVStoreService(app.keys[consensustypes.StoreKey]),
		authtypes.NewModuleAddress("gov").String(),
		runtime.EventService{},
	)
	
	// Set consensus params keeper for baseapp
	app.SetParamStore(app.ConsensusKeeper.ParamsStore)
	
	// Create Upgrade keeper - required for IBC
	app.UpgradeKeeper = upgradekeeper.NewKeeper(
		map[int64]bool{}, // skipUpgradeHeights
		runtime.NewKVStoreService(app.keys[upgradetypes.StoreKey]),
		appCodec,
		filepath.Join(DefaultNodeHome, "data"),
		app.BaseApp,
		authtypes.NewModuleAddress("gov").String(),
	)
	
	// Initialize capability keeper - must be done before IBC keeper
	app.CapabilityKeeper = capabilitykeeper.NewKeeper(
		appCodec,
		app.keys[capabilitytypes.StoreKey],
		app.memKeys[capabilitytypes.MemStoreKey],
	)
	
	// Create scoped keepers for IBC modules
	app.ScopedIBCKeeper = app.CapabilityKeeper.ScopeToModule(ibcexported.ModuleName)
	app.ScopedTransferKeeper = app.CapabilityKeeper.ScopeToModule(ibctransfertypes.ModuleName)
	
	// Create IBC keeper - core IBC protocol
	app.IBCKeeper = ibckeeper.NewKeeper(
		appCodec,
		app.keys[ibcexported.StoreKey],
		nil, // params subspace deprecated in v0.50
		app.StakingKeeper,
		app.UpgradeKeeper,
		app.ScopedIBCKeeper,
		authtypes.NewModuleAddress("gov").String(),
	)
	
	// Create IBC Transfer keeper
	app.TransferKeeper = ibctransferkeeper.NewKeeper(
		appCodec,
		app.keys[ibctransfertypes.StoreKey],
		nil, // params subspace deprecated in v0.50
		app.IBCKeeper.ChannelKeeper,
		app.IBCKeeper.ChannelKeeper,
		app.IBCKeeper.PortKeeper,
		app.AccountKeeper,
		app.BankKeeper,
		app.ScopedTransferKeeper,
		authtypes.NewModuleAddress("gov").String(),
	)
	
	// Create IBC router and register transfer route
	ibcRouter := ibcporttypes.NewRouter()
	ibcRouter.AddRoute(ibctransfertypes.ModuleName, ibctransfer.NewIBCModule(app.TransferKeeper))
	// Add more IBC routes here as needed (e.g., ICA, interchain accounts)
	
	// Seal IBC router
	app.IBCKeeper.SetRouter(ibcRouter)
	
	// Seal capability keeper after all scoped keepers are created
	app.CapabilityKeeper.Seal()
	
	// Create Sultan keeper
	app.SultanKeeper = sultankeeper.NewKeeper(
		appCodec,
		runtime.NewKVStoreService(app.keys[sultantypes.StoreKey]),
		logger,
	)
	
	// Create module manager
	app.mm = module.NewManager(
		genutil.NewAppModule(app.AccountKeeper, app.StakingKeeper, app.GetTxHandler(), app.txConfig),
		auth.NewAppModule(app.appCodec, app.AccountKeeper, authsims.RandomGenesisAccounts, nil),
		bank.NewAppModule(app.appCodec, app.BankKeeper, app.AccountKeeper, nil),
		staking.NewAppModule(app.appCodec, app.StakingKeeper, app.AccountKeeper, app.BankKeeper, nil),
		consensus.NewAppModule(app.appCodec, app.ConsensusKeeper),
		upgrade.NewAppModule(app.UpgradeKeeper, addressCodec),
		capability.NewAppModule(app.appCodec, *app.CapabilityKeeper, false),
		ibc.NewAppModule(app.IBCKeeper),
		ibctransfer.NewAppModule(app.TransferKeeper),
		sultan.NewAppModule(app.appCodec, app.SultanKeeper),
	)
	
	// Set module order for init genesis
	app.mm.SetOrderInitGenesis(
		capabilitytypes.ModuleName,
		authtypes.ModuleName,
		banktypes.ModuleName,
		stakingtypes.ModuleName,
		consensustypes.ModuleName,
		upgradetypes.ModuleName,
		ibcexported.ModuleName,
		ibctransfertypes.ModuleName,
		genutiltypes.ModuleName, // genutil must be after staking to collect validators
		sultantypes.ModuleName,
	)
	
	// Set module order for begin/end block
	app.mm.SetOrderBeginBlockers(
		capabilitytypes.ModuleName,
		consensustypes.ModuleName,
		upgradetypes.ModuleName,
		stakingtypes.ModuleName,
		ibcexported.ModuleName,
		ibctransfertypes.ModuleName,
		sultantypes.ModuleName,
	)
	
	app.mm.SetOrderEndBlockers(
		sultantypes.ModuleName,
		stakingtypes.ModuleName,
		consensustypes.ModuleName,
		upgradetypes.ModuleName,
		ibcexported.ModuleName,
		ibctransfertypes.ModuleName,
		capabilitytypes.ModuleName,
	)
	
	// Register services
	app.configurator = module.NewConfigurator(app.appCodec, app.MsgServiceRouter(), app.GRPCQueryRouter())
	app.mm.RegisterServices(app.configurator)
	
	// Set ABCI handlers
	app.SetInitChainer(app.InitChainer)
	app.SetBeginBlocker(app.BeginBlocker)
	app.SetEndBlocker(app.EndBlocker)
	
	// Initialize stores
	app.MountKVStores(app.keys)
	app.MountTransientStores(app.tkeys)
	app.MountMemoryStores(app.memKeys)
	
	// Load latest version
	if loadLatest {
		if err := app.LoadLatestVersion(); err != nil {
			panic(err)
		}
	}
	
	return app
}

// Name returns the name of the App
func (app *SultanApp) Name() string { return app.BaseApp.Name() }

// LegacyAmino returns the app's amino codec
func (app *SultanApp) LegacyAmino() *codec.LegacyAmino {
	return app.legacyAmino
}

// AppCodec returns the app's codec
func (app *SultanApp) AppCodec() codec.Codec {
	return app.appCodec
}

// InterfaceRegistry returns the app's InterfaceRegistry
func (app *SultanApp) InterfaceRegistry() codectypes.InterfaceRegistry {
	return app.interfaceRegistry
}

// TxConfig returns the app's TxConfig
func (app *SultanApp) TxConfig() client.TxConfig {
	return app.txConfig
}

// LoadHeight loads a particular height
func (app *SultanApp) LoadHeight(height int64) error {
	return app.LoadVersion(height)
}

// GetKey returns the KVStoreKey for the provided store key
func (app *SultanApp) GetKey(storeKey string) *storetypes.KVStoreKey {
	return app.keys[storeKey]
}

// SimulationManager returns the app's SimulationManager
func (app *SultanApp) SimulationManager() *module.SimulationManager {
	return app.sm
}

// RegisterAPIRoutes registers all application module routes with the provided API server
func (app *SultanApp) RegisterAPIRoutes(apiSvr *api.Server, apiConfig config.APIConfig) {
	clientCtx := apiSvr.ClientCtx
	
	// Register legacy Amino JSON REST routes (for Cosmos SDK compatibility)
	// Note: In SDK v0.50+, gRPC Gateway is the primary API
	// ModuleBasics will register all module routes
	ModuleBasics.RegisterGRPCGatewayRoutes(clientCtx, apiSvr.GRPCGatewayRouter)
	
	// Register transaction service routes
	authtx.RegisterGRPCGatewayRoutes(clientCtx, apiSvr.GRPCGatewayRouter)
	
	// Enable Swagger/OpenAPI documentation if configured
	if apiConfig.Swagger {
		RegisterSwaggerAPI(clientCtx, apiSvr.Router)
	}
	
	// Configure CORS and health endpoints
	ConfigureAPI(apiSvr)
}

// ConfigureAPI configures API server with CORS and middleware
func ConfigureAPI(apiSvr *api.Server) {
	// Add health check endpoint
	apiSvr.Router.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"status":"healthy","chain":"sultan-l1"}`))
	}).Methods("GET")
	
	// Add status endpoint
	apiSvr.Router.HandleFunc("/status", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"chain_id":"sultan-1","api":"v1","ibc":"enabled","modules":["auth","bank","staking","ibc","transfer","sultan"]}`))
	}).Methods("GET")
	
	// Add Keplr chain info endpoint
	apiSvr.Router.HandleFunc("/chain_info", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		chainInfo := `{
  "chainId": "sultan-1",
  "chainName": "Sultan L1",
  "rpc": "http://localhost:26657",
  "rest": "http://localhost:1317",
  "bip44": {"coinType": 118},
  "bech32Config": {
    "bech32PrefixAccAddr": "cosmos",
    "bech32PrefixAccPub": "cosmospub",
    "bech32PrefixValAddr": "cosmosvaloper",
    "bech32PrefixValPub": "cosmosvaloperpub",
    "bech32PrefixConsAddr": "cosmosvalcons",
    "bech32PrefixConsPub": "cosmosvalconspub"
  },
  "currencies": [{"coinDenom": "SULTAN", "coinMinimalDenom": "stake", "coinDecimals": 6, "coinGeckoId": "sultan"}],
  "feeCurrencies": [{"coinDenom": "SULTAN", "coinMinimalDenom": "stake", "coinDecimals": 6, "gasPriceStep": {"low": 0, "average": 0, "high": 0}}],
  "stakeCurrency": {"coinDenom": "SULTAN", "coinMinimalDenom": "stake", "coinDecimals": 6},
  "features": ["ibc-transfer", "ibc-go", "no-legacy-stdTx"]
}`
		w.Write([]byte(chainInfo))
	}).Methods("GET")
}

// RegisterTxService implements the Application.RegisterTxService method
func (app *SultanApp) RegisterTxService(clientCtx client.Context) {
	authtx.RegisterTxService(
		app.BaseApp.GRPCQueryRouter(),
		clientCtx,
		app.BaseApp.Simulate,
		app.interfaceRegistry,
	)
}

// RegisterNodeService registers node service
func (app *SultanApp) RegisterNodeService(clientCtx client.Context, cfg config.Config) {
	// Node service is automatically registered by the server framework
	// This method satisfies the Application interface
}

// RegisterTendermintService implements the Application.RegisterTendermintService method
func (app *SultanApp) RegisterTendermintService(clientCtx client.Context) {
	cmtservice.RegisterTendermintService(
		clientCtx,
		app.BaseApp.GRPCQueryRouter(),
		app.interfaceRegistry,
		app.Query,
	)
}

// InitChainer application update at chain initialization
func (app *SultanApp) InitChainer(ctx sdk.Context, req *abci.RequestInitChain) (*abci.ResponseInitChain, error) {
	var genesisState map[string]json.RawMessage
	if err := json.Unmarshal(req.AppStateBytes, &genesisState); err != nil {
		return nil, err
	}
	
	// Initialize all modules - staking module will return validators
	res, err := app.mm.InitGenesis(ctx, app.appCodec, genesisState)
	if err != nil {
		return nil, err
	}
	
	// res.Validators comes from staking module's InitGenesis
	// Do not override it
	
	return res, nil
}

// BeginBlocker application updates every begin block
func (app *SultanApp) BeginBlocker(ctx sdk.Context) (sdk.BeginBlock, error) {
	return app.mm.BeginBlock(ctx)
}

// EndBlocker application updates every end block
func (app *SultanApp) EndBlocker(ctx sdk.Context) (sdk.EndBlock, error) {
	return app.mm.EndBlock(ctx)
}

// ExportAppStateAndValidators exports the state of the application for a genesis file
func (app *SultanApp) ExportAppStateAndValidators(forZeroHeight bool, jailAllowedAddrs []string, modulesToExport []string) (servertypes.ExportedApp, error) {
	ctx := app.NewContext(true)
	
	// Export genesis state
	genState, err := app.mm.ExportGenesisForModules(ctx, app.appCodec, modulesToExport)
	if err != nil {
		return servertypes.ExportedApp{}, err
	}
	
	appState, err := json.MarshalIndent(genState, "", "  ")
	if err != nil {
		return servertypes.ExportedApp{}, err
	}
	
	return servertypes.ExportedApp{
		AppState:  appState,
		Validators: []cmttypes.GenesisValidator{},
		Height:    app.LastBlockHeight(),
		ConsensusParams: app.BaseApp.GetConsensusParams(ctx),
	}, nil
}

// Close performs any necessary cleanup before shutting down
func (app *SultanApp) Close() error {
	// Cleanup Sultan keeper (destroys FFI handle)
	if app.SultanKeeper != nil {
		app.SultanKeeper.Cleanup()
	}
	return app.BaseApp.Close()
}

// GetTxHandler returns a TxHandler for gentx processing
// GetTxHandler returns the genesis TxHandler for executing genesis transactions
// We delegate to BaseApp which has proper context and codec handling
func (app *SultanApp) GetTxHandler() genesis.TxHandler {
	return app.BaseApp
}

// SultanGenesisBalancesIterator implements the bank module's GenesisBalancesIterator
type SultanGenesisBalancesIterator struct{}

// IterateGenesisBalances iterates over all the genesis balances found in appGenesis
// and invokes a callback on each genesis account. This is required for gentx validation.
func (SultanGenesisBalancesIterator) IterateGenesisBalances(
	cdc codec.JSONCodec,
	appState map[string]json.RawMessage,
	cb func(bankexported.GenesisBalance) (stop bool),
) {
	// Get bank genesis state
	var bankGenesis banktypes.GenesisState
	if appState[banktypes.ModuleName] != nil {
		cdc.MustUnmarshalJSON(appState[banktypes.ModuleName], &bankGenesis)
	}
	
	// Iterate through all balances
	for _, balance := range bankGenesis.Balances {
		if cb(balance) {
			break
		}
	}
}

// GenesisBalancesIterator returns the iterator for genesis balances
func GenesisBalancesIterator() genutiltypes.GenesisBalancesIterator {
	return SultanGenesisBalancesIterator{}
}
