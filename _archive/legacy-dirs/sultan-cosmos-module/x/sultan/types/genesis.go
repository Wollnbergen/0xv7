package types

import (
	bridgetypes "github.com/wollnbergen/sultan-cosmos-bridge/types"
)

// GenesisState defines the sultan module's genesis state
type GenesisState struct {
	// Genesis accounts with initial balances
	GenesisAccounts []bridgetypes.GenesisAccount `json:"genesis_accounts"`
	
	// Last committed block height
	LastBlockHeight uint64 `json:"last_block_height"`
}

// Reset implements proto.Message
func (gs *GenesisState) Reset() {}

// String implements proto.Message
func (gs *GenesisState) String() string {
	return "GenesisState"
}

// ProtoMessage implements proto.Message
func (gs *GenesisState) ProtoMessage() {}

// DefaultGenesisState returns a default genesis state
func DefaultGenesisState() *GenesisState {
	return &GenesisState{
		GenesisAccounts: []bridgetypes.GenesisAccount{},
		LastBlockHeight: 0,
	}
}

// ValidateGenesis validates the provided genesis state
func ValidateGenesis(data *GenesisState) error {
	// Validate genesis accounts
	for _, acc := range data.GenesisAccounts {
		if acc.Address == "" {
			return ErrInvalidGenesisAccount
		}
		if acc.Balance == 0 {
			return ErrInvalidGenesisAccount
		}
	}
	
	return nil
}
