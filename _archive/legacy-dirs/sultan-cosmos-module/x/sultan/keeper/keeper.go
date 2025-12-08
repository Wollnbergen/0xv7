package keeper

import (
	"fmt"
	
	"cosmossdk.io/log"
	"cosmossdk.io/core/store"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	
	bridge "github.com/wollnbergen/sultan-cosmos-bridge/bridge"
	bridgetypes "github.com/wollnbergen/sultan-cosmos-bridge/types"
	"github.com/wollnbergen/sultan-cosmos-module/x/sultan/types"
)

// Keeper maintains the link to storage and exposes getter/setter methods for the various parts of the state machine
type Keeper struct {
	cdc           codec.BinaryCodec
	storeService  store.KVStoreService
	logger        log.Logger
	
	// FFI bridge instances - points to Rust Sultan blockchain
	blockchain      *bridge.Blockchain
	consensusEngine *bridge.ConsensusEngine
}

// NewKeeper creates a new sultan Keeper instance
func NewKeeper(
	cdc codec.BinaryCodec,
	storeService store.KVStoreService,
	logger log.Logger,
) *Keeper {
	return &Keeper{
		cdc:          cdc,
		storeService: storeService,
		logger:       logger,
	}
}

// Logger returns a module-specific logger
func (k Keeper) Logger(ctx sdk.Context) log.Logger {
	return k.logger.With("module", fmt.Sprintf("x/%s", types.ModuleName))
}

// InitGenesis initializes the Sultan blockchain via FFI with genesis accounts
func (k *Keeper) InitGenesis(ctx sdk.Context, genesisAccounts []bridgetypes.GenesisAccount) error {
	// Initialize FFI bridge
	if err := bridge.Initialize(); err != nil {
		return fmt.Errorf("failed to initialize FFI bridge: %w", err)
	}
	
	// Create new blockchain instance
	blockchain, err := bridge.NewBlockchain()
	if err != nil {
		return fmt.Errorf("failed to create blockchain: %w", err)
	}
	k.blockchain = blockchain
	
	// Create new consensus engine
	consensusEngine, err := bridge.NewConsensusEngine()
	if err != nil {
		return fmt.Errorf("failed to create consensus engine: %w", err)
	}
	k.consensusEngine = consensusEngine
	
	k.logger.Info("Sultan blockchain initialized via FFI")
	
	// Get store
	store := k.storeService.OpenKVStore(ctx)
	
	// Initialize genesis accounts
	for _, acc := range genesisAccounts {
		if err := k.blockchain.InitAccount(acc.Address, acc.Balance); err != nil {
			return fmt.Errorf("failed to initialize account %s: %w", acc.Address, err)
		}
	}
	
	if len(genesisAccounts) > 0 {
		k.logger.Info("Genesis initialized", "accounts", len(genesisAccounts))
	}
	
	// Store initial height
	if err := store.Set(types.LastBlockHeightKey, sdk.Uint64ToBigEndian(0)); err != nil {
		return fmt.Errorf("failed to store initial height: %w", err)
	}
	
	return nil
}

// ExportGenesis exports the module's state
func (k Keeper) ExportGenesis(ctx sdk.Context) (*types.GenesisState, error) {
	store := k.storeService.OpenKVStore(ctx)
	
	heightBytes, err := store.Get(types.LastBlockHeightKey)
	if err != nil {
		return nil, fmt.Errorf("failed to get last block height: %w", err)
	}
	
	var height uint64
	if len(heightBytes) > 0 {
		height = sdk.BigEndianToUint64(heightBytes)
	}
	
	return &types.GenesisState{
		LastBlockHeight: height,
	}, nil
}

// GetBalance queries an account balance via FFI
func (k Keeper) GetBalance(ctx sdk.Context, address string) (uint64, error) {
	if k.blockchain == nil {
		return 0, fmt.Errorf("blockchain not initialized")
	}
	
	balance, err := k.blockchain.GetBalance(address)
	if err != nil {
		return 0, fmt.Errorf("failed to get balance: %w", err)
	}
	
	return balance, nil
}

// SubmitTransaction forwards a transaction to the Sultan blockchain via FFI
func (k *Keeper) SubmitTransaction(ctx sdk.Context, tx bridgetypes.Transaction) error {
	if k.blockchain == nil {
		return fmt.Errorf("blockchain not initialized")
	}
	
	// Submit via FFI
	if err := k.blockchain.AddTransaction(tx); err != nil {
		return fmt.Errorf("transaction submission failed: %w", err)
	}
	
	k.logger.Info("Transaction submitted",
		"from", tx.From,
		"to", tx.To,
		"amount", tx.Amount,
	)
	
	return nil
}

// ProduceBlock produces a new block via FFI
func (k *Keeper) ProduceBlock(ctx sdk.Context, proposer string) error {
	// Re-initialize Sultan Core if needed (e.g., after node restart)
	if k.blockchain == nil {
		k.logger.Info("Re-initializing Sultan blockchain after restart")
		
		// Initialize FFI bridge
		if err := bridge.Initialize(); err != nil {
			return fmt.Errorf("failed to re-initialize FFI bridge: %w", err)
		}
		
		// Create new blockchain instance
		blockchain, err := bridge.NewBlockchain()
		if err != nil {
			return fmt.Errorf("failed to re-create blockchain: %w", err)
		}
		k.blockchain = blockchain
		
		// Create consensus engine
		consensusEngine, err := bridge.NewConsensusEngine()
		if err != nil {
			return fmt.Errorf("failed to re-create consensus engine: %w", err)
		}
		k.consensusEngine = consensusEngine
		
		k.logger.Info("Sultan blockchain re-initialized successfully")
	}
	
	if err := k.blockchain.CreateBlock(proposer); err != nil {
		return fmt.Errorf("block production failed: %w", err)
	}
	
	// Update stored height
	store := k.storeService.OpenKVStore(ctx)
	heightBytes, err := store.Get(types.LastBlockHeightKey)
	if err != nil {
		return fmt.Errorf("failed to get current height: %w", err)
	}
	
	currentHeight := sdk.BigEndianToUint64(heightBytes)
	newHeight := currentHeight + 1
	
	if err := store.Set(types.LastBlockHeightKey, sdk.Uint64ToBigEndian(newHeight)); err != nil {
		return fmt.Errorf("failed to update height: %w", err)
	}
	
	k.logger.Info("Block produced", "height", newHeight, "proposer", proposer)
	
	return nil
}

// AddValidator adds a validator to the consensus engine via FFI
func (k *Keeper) AddValidator(ctx sdk.Context, address string, stake uint64) error {
	if k.consensusEngine == nil {
		return fmt.Errorf("consensus engine not initialized")
	}
	
	if err := k.consensusEngine.AddValidator(address, stake); err != nil {
		return fmt.Errorf("failed to add validator: %w", err)
	}
	
	k.logger.Info("Validator added", "address", address, "stake", stake)
	return nil
}

// SelectProposer selects the next block proposer via FFI
func (k *Keeper) SelectProposer(ctx sdk.Context) (string, error) {
	if k.consensusEngine == nil {
		return "", fmt.Errorf("consensus engine not initialized")
	}
	
	proposer, err := k.consensusEngine.SelectProposer()
	if err != nil {
		return "", fmt.Errorf("failed to select proposer: %w", err)
	}
	
	return proposer, nil
}

// GetBlockchainInfo retrieves blockchain information via FFI
func (k Keeper) GetBlockchainInfo(ctx sdk.Context) (map[string]interface{}, error) {
	if k.blockchain == nil {
		return nil, fmt.Errorf("blockchain not initialized")
	}
	
	// Get height
	height, err := k.blockchain.Height()
	if err != nil {
		return nil, fmt.Errorf("failed to get blockchain height: %w", err)
	}
	
	// Get latest hash
	hash, err := k.blockchain.LatestHash()
	if err != nil {
		return nil, fmt.Errorf("failed to get latest hash: %w", err)
	}
	
	info := map[string]interface{}{
		"chain_id": "sultan-chain-1",
		"height":   float64(height),
		"latest_hash": hash,
	}
	
	return info, nil
}

// Cleanup destroys the blockchain handle when the module is shut down
func (k *Keeper) Cleanup() {
	if k.blockchain != nil {
		k.blockchain.Destroy()
		k.logger.Info("Sultan blockchain destroyed")
		k.blockchain = nil
	}
	
	if k.consensusEngine != nil {
		// Consensus engine doesn't have a Destroy method in the bridge
		k.consensusEngine = nil
	}
}
