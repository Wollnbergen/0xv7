package app

import (
    "github.com/cosmos/cosmos-sdk/client"
    "github.com/cosmos/cosmos-sdk/codec"
    "github.com/cosmos/cosmos-sdk/codec/types"
    "github.com/cosmos/cosmos-sdk/std"
    "github.com/cosmos/cosmos-sdk/x/auth/tx"
)

type EncodingConfig struct {
    InterfaceRegistry types.InterfaceRegistry
    Codec             codec.Codec
    TxConfig          client.TxConfig
    Amino             *codec.LegacyAmino
}

func MakeEncodingConfig() EncodingConfig {
    amino := codec.NewLegacyAmino()
    interfaceRegistry := types.NewInterfaceRegistry()
    cdc := codec.NewProtoCodec(interfaceRegistry)
    txCfg := tx.NewTxConfig(cdc, tx.DefaultSignModes)
    
    std.RegisterLegacyAminoCodec(amino)
    std.RegisterInterfaces(interfaceRegistry)
    ModuleBasics.RegisterLegacyAminoCodec(amino)
    ModuleBasics.RegisterInterfaces(interfaceRegistry)
    
    return EncodingConfig{
        InterfaceRegistry: interfaceRegistry,
        Codec:             cdc,
        TxConfig:          txCfg,
        Amino:             amino,
    }
}
