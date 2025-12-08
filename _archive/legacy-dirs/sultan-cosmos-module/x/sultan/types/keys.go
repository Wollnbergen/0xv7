package types

const (
	// ModuleName defines the module name
	ModuleName = "sultan"

	// StoreKey defines the primary module store key
	StoreKey = ModuleName

	// RouterKey defines the module's message routing key
	RouterKey = ModuleName

	// QuerierRoute defines the module's query routing key
	QuerierRoute = ModuleName

	// MemStoreKey defines the in-memory store key
	MemStoreKey = "mem_" + ModuleName
)

var (
	// ParamsKey is the key for module parameters
	ParamsKey = []byte{0x01}
	
	// BlockchainHandleKey stores the FFI blockchain handle
	BlockchainHandleKey = []byte{0x02}
	
	// LastBlockHeightKey stores the last committed block height
	LastBlockHeightKey = []byte{0x03}
	
	// StateRootKey stores the current state root
	StateRootKey = []byte{0x04}
)
