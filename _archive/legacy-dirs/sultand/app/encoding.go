package app

import (
	txsigning "cosmossdk.io/x/tx/signing"
	
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/address"
	"github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/std"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth/tx"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	consensustypes "github.com/cosmos/cosmos-sdk/x/consensus/types"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
	
	"github.com/cosmos/gogoproto/proto"
	
	sultantypes "github.com/wollnbergen/sultan-cosmos-module/x/sultan/types"
)

// EncodingConfig specifies the concrete encoding types to use for a given app
type EncodingConfig struct {
	InterfaceRegistry types.InterfaceRegistry
	Codec             codec.Codec
	TxConfig          client.TxConfig
	Amino             *codec.LegacyAmino
}

// MakeEncodingConfig creates an EncodingConfig for the app
func MakeEncodingConfig() EncodingConfig {
	amino := codec.NewLegacyAmino()
	
	// Create signing options with proper address codecs
	sdkConfig := sdk.GetConfig()
	signingOptions := txsigning.Options{
		AddressCodec:          address.NewBech32Codec(sdkConfig.GetBech32AccountAddrPrefix()),
		ValidatorAddressCodec: address.NewBech32Codec(sdkConfig.GetBech32ValidatorAddrPrefix()),
		FileResolver:          proto.HybridResolver,
	}
	
	// Create InterfaceRegistry with signing options
	interfaceRegistry, err := types.NewInterfaceRegistryWithOptions(types.InterfaceRegistryOptions{
		ProtoFiles:     proto.HybridResolver,
		SigningOptions: signingOptions,
	})
	if err != nil {
		panic(err)
	}
	
	cdc := codec.NewProtoCodec(interfaceRegistry)
	
	// Create TxConfig with same signing options
	txCfg, err := tx.NewTxConfigWithOptions(cdc, tx.ConfigOptions{
		SigningOptions:   &signingOptions,
		EnabledSignModes: tx.DefaultSignModes,
	})
	if err != nil {
		panic(err)
	}
	
	// Register standard types
	std.RegisterLegacyAminoCodec(amino)
	std.RegisterInterfaces(interfaceRegistry)
	
	// Register all module types
	authtypes.RegisterInterfaces(interfaceRegistry)
	banktypes.RegisterInterfaces(interfaceRegistry)
	stakingtypes.RegisterInterfaces(interfaceRegistry)
	consensustypes.RegisterInterfaces(interfaceRegistry)
	
	// Register Sultan types
	sultantypes.RegisterCodec(amino)
	sultantypes.RegisterInterfaces(interfaceRegistry)
	
	return EncodingConfig{
		InterfaceRegistry: interfaceRegistry,
		Codec:             cdc,
		TxConfig:          txCfg,
		Amino:             amino,
	}
}
