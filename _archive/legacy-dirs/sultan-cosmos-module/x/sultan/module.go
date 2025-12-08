package sultan

import (
	"context"
	"encoding/json"
	"fmt"
	
	"cosmossdk.io/core/appmodule"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	"github.com/grpc-ecosystem/grpc-gateway/runtime"
	
	"github.com/wollnbergen/sultan-cosmos-module/x/sultan/keeper"
	sultantypes "github.com/wollnbergen/sultan-cosmos-module/x/sultan/types"
)

var (
	_ module.AppModuleBasic   = AppModuleBasic{}
	_ module.HasGenesis       = AppModule{}
	_ module.HasServices      = AppModule{}
	_ appmodule.AppModule     = AppModule{}
	_ appmodule.HasBeginBlocker = AppModule{}
	_ appmodule.HasEndBlocker = AppModule{}
)

// AppModuleBasic defines the basic application module used by the sultan module
type AppModuleBasic struct {
	cdc codec.Codec
}

// Name returns the sultan module's name
func (AppModuleBasic) Name() string {
	return sultantypes.ModuleName
}

// RegisterLegacyAminoCodec registers the sultan module's types on the LegacyAmino codec
func (AppModuleBasic) RegisterLegacyAminoCodec(cdc *codec.LegacyAmino) {
	sultantypes.RegisterCodec(cdc)
}

// RegisterInterfaces registers the module's interface types
func (b AppModuleBasic) RegisterInterfaces(registry types.InterfaceRegistry) {
	sultantypes.RegisterInterfaces(registry)
}

// DefaultGenesis returns default genesis state as raw bytes for the sultan module
func (AppModuleBasic) DefaultGenesis(cdc codec.JSONCodec) json.RawMessage {
	return cdc.MustMarshalJSON(sultantypes.DefaultGenesisState())
}

// ValidateGenesis performs genesis state validation for the sultan module
func (AppModuleBasic) ValidateGenesis(cdc codec.JSONCodec, config client.TxEncodingConfig, bz json.RawMessage) error {
	var data sultantypes.GenesisState
	if err := cdc.UnmarshalJSON(bz, &data); err != nil {
		return fmt.Errorf("failed to unmarshal %s genesis state: %w", sultantypes.ModuleName, err)
	}
	return sultantypes.ValidateGenesis(&data)
}

// RegisterGRPCGatewayRoutes registers the gRPC Gateway routes for the sultan module
func (AppModuleBasic) RegisterGRPCGatewayRoutes(clientCtx client.Context, mux *runtime.ServeMux) {
	// Register query routes
	// Note: In production, we'd generate these from protobuf
}

// AppModule implements the AppModule interface for the sultan module
type AppModule struct {
	AppModuleBasic
	
	keeper *keeper.Keeper
}

// NewAppModule creates a new AppModule object
func NewAppModule(cdc codec.Codec, keeper *keeper.Keeper) AppModule {
	return AppModule{
		AppModuleBasic: AppModuleBasic{cdc: cdc},
		keeper:         keeper,
	}
}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface
func (am AppModule) IsOnePerModuleType() {}

// IsAppModule implements the appmodule.AppModule interface
func (am AppModule) IsAppModule() {}

// RegisterServices registers module services
func (am AppModule) RegisterServices(cfg module.Configurator) {
	// Note: Services disabled - using standard Cosmos SDK modules for queries/txs
	// The Sultan module handles genesis initialization and FFI bridge to Rust core
	// Queries can be added later with proper protobuf definitions
}

// InitGenesis performs genesis initialization for the sultan module
func (am AppModule) InitGenesis(ctx sdk.Context, cdc codec.JSONCodec, data json.RawMessage) {
	var genesisState sultantypes.GenesisState
	cdc.MustUnmarshalJSON(data, &genesisState)
	
	// Initialize Sultan blockchain via FFI
	if err := am.keeper.InitGenesis(ctx, genesisState.GenesisAccounts); err != nil {
		panic(fmt.Sprintf("failed to initialize genesis: %v", err))
	}
}

// ExportGenesis returns the exported genesis state as raw bytes for the sultan module
func (am AppModule) ExportGenesis(ctx sdk.Context, cdc codec.JSONCodec) json.RawMessage {
	gs, err := am.keeper.ExportGenesis(ctx)
	if err != nil {
		panic(fmt.Sprintf("failed to export genesis: %v", err))
	}
	return cdc.MustMarshalJSON(gs)
}

// ConsensusVersion implements AppModule/ConsensusVersion
func (AppModule) ConsensusVersion() uint64 { return 1 }

// BeginBlock implements the AppModule interface
// This is called by CometBFT at the beginning of each block
func (am AppModule) BeginBlock(ctx context.Context) error {
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	height := sdkCtx.BlockHeight()
	
	// Get proposer from context (CometBFT provides this)
	proposerAddr := sdkCtx.BlockHeader().ProposerAddress
	proposer := sdk.ConsAddress(proposerAddr).String()
	
	am.keeper.Logger(sdkCtx).Info("BeginBlock",
		"height", height,
		"proposer", proposer,
		"time", sdkCtx.BlockTime(),
	)
	
	return nil
}

// EndBlock implements the AppModule interface
// This is called by CometBFT at the end of each block
func (am AppModule) EndBlock(ctx context.Context) error {
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	height := sdkCtx.BlockHeight()
	
	// Get proposer from context
	proposerAddr := sdkCtx.BlockHeader().ProposerAddress
	proposer := sdk.ConsAddress(proposerAddr).String()
	
	// Produce a block in the Sultan blockchain
	if err := am.keeper.ProduceBlock(sdkCtx, proposer); err != nil {
		return fmt.Errorf("failed to produce Sultan block: %w", err)
	}
	
	am.keeper.Logger(sdkCtx).Info("EndBlock completed",
		"height", height,
	)
	
	return nil
}
