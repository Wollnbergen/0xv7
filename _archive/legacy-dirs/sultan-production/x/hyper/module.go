package hyper

import (
    "context"
    "encoding/json"
    
    "cosmossdk.io/core/appmodule"
    "github.com/cosmos/cosmos-sdk/client"
    "github.com/cosmos/cosmos-sdk/codec"
    cdctypes "github.com/cosmos/cosmos-sdk/codec/types"
    sdk "github.com/cosmos/cosmos-sdk/types"
    "github.com/cosmos/cosmos-sdk/types/module"
    "github.com/grpc-ecosystem/grpc-gateway/runtime"
    "github.com/spf13/cobra"
)

const (
    ModuleName = "hyper"
    StoreKey = ModuleName
)

type AppModule struct {
    appmodule.AppModule
}

// NewAppModule creates a new hyper module
func NewAppModule() AppModule {
    return AppModule{}
}

func (AppModule) Name() string { return ModuleName }

// RegisterGRPCGatewayRoutes registers the gRPC Gateway routes for the module
func (AppModule) RegisterGRPCGatewayRoutes(clientCtx client.Context, mux *runtime.ServeMux) {}

// GetTxCmd returns the root tx command for the module
func (AppModule) GetTxCmd() *cobra.Command {
    return &cobra.Command{
        Use:   ModuleName,
        Short: "Hyper module for 10M TPS",
    }
}

// GetQueryCmd returns no root query command for the module
func (AppModule) GetQueryCmd() *cobra.Command {
    return &cobra.Command{
        Use:   ModuleName,
        Short: "Querying commands for hyper module",
    }
}
