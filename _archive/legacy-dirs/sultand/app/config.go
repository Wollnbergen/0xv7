package app

import (
	"cosmossdk.io/depinject"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	consensustypes "github.com/cosmos/cosmos-sdk/x/consensus/types"
	
	sultantypes "github.com/wollnbergen/sultan-cosmos-module/x/sultan/types"
)

// AppConfig returns the default app config
func AppConfig() depinject.Config {
	return depinject.Configs(
		depinject.Supply(
			// Supply the application name
			"sultand",
		),
		depinject.Provide(
			// Provide module dependencies
			authtypes.ProtoBaseAccount,
		),
		// Configure modules
		ModuleConfig(),
	)
}

// ModuleConfig returns the module configuration
func ModuleConfig() depinject.Config {
	return depinject.Configs(
		depinject.Supply(
			// Module accounts
			authtypes.NewModuleAddress(authtypes.FeeCollectorName),
			authtypes.NewModuleAddress(banktypes.ModuleName),
			authtypes.NewModuleAddress(consensustypes.ModuleName),
			authtypes.NewModuleAddress(sultantypes.ModuleName),
		),
	)
}
