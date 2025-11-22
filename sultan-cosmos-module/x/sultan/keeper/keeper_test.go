package keeper_test

import (
	"testing"
	
	"cosmossdk.io/log"
	"cosmossdk.io/store"
	"cosmossdk.io/store/metrics"
	storetypes "cosmossdk.io/store/types"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"
	
	dbm "github.com/cosmos/cosmos-db"
	
	bridgetypes "github.com/wollnbergen/sultan-cosmos-bridge/types"
	"github.com/wollnbergen/sultan-cosmos-module/x/sultan/keeper"
	"github.com/wollnbergen/sultan-cosmos-module/x/sultan/types"
)

type KeeperTestSuite struct {
	suite.Suite
	
	ctx    sdk.Context
	keeper *keeper.Keeper
	cdc    codec.BinaryCodec
}

func (suite *KeeperTestSuite) SetupTest() {
	// Create store key
	storeKey := storetypes.NewKVStoreKey(types.StoreKey)
	
	// Create in-memory database
	db := dbm.NewMemDB()
	stateStore := store.NewCommitMultiStore(db, log.NewNopLogger(), metrics.NewNoOpMetrics())
	stateStore.MountStoreWithDB(storeKey, storetypes.StoreTypeIAVL, db)
	require.NoError(suite.T(), stateStore.LoadLatestVersion())
	
	// Create codec
	interfaceRegistry := codectypes.NewInterfaceRegistry()
	types.RegisterInterfaces(interfaceRegistry)
	suite.cdc = codec.NewProtoCodec(interfaceRegistry)
	
	// Create KVStoreService from the store key
	storeService := &kvStoreService{storeKey: storeKey}
	
	// Create keeper
	suite.keeper = keeper.NewKeeper(
		suite.cdc,
		storeService,
		log.NewNopLogger(),
	)
	
	// Create context
	suite.ctx = sdk.NewContext(stateStore, false, log.NewNopLogger())
}

// kvStoreService implements store.KVStoreService interface
type kvStoreService struct {
	storeKey storetypes.StoreKey
}

func (s *kvStoreService) OpenKVStore(ctx sdk.Context) sdk.KVStore {
	return ctx.KVStore(s.storeKey)
}

func (suite *KeeperTestSuite) TearDownTest() {
	// Cleanup FFI resources
	if suite.keeper != nil {
		suite.keeper.Cleanup()
	}
}

func TestKeeperTestSuite(t *testing.T) {
	suite.Run(t, new(KeeperTestSuite))
}

func (suite *KeeperTestSuite) TestInitGenesis() {
	// Prepare genesis accounts
	genesisAccounts := []bridgetypes.GenesisAccount{
		{Address: "alice", Balance: 1000000},
		{Address: "bob", Balance: 500000},
		{Address: "validator1", Balance: 100000},
	}
	
	// Initialize genesis
	err := suite.keeper.InitGenesis(suite.ctx, genesisAccounts)
	suite.Require().NoError(err)
	
	// Verify blockchain handle is stored
	handle, err := suite.keeper.GetBlockchainHandle(suite.ctx)
	suite.Require().NoError(err)
	suite.Require().NotZero(handle)
	
	// Verify balances
	aliceBalance, err := suite.keeper.GetBalance(suite.ctx, "alice")
	suite.Require().NoError(err)
	suite.Require().Equal(uint64(1000000), aliceBalance)
	
	bobBalance, err := suite.keeper.GetBalance(suite.ctx, "bob")
	suite.Require().NoError(err)
	suite.Require().Equal(uint64(500000), bobBalance)
}

func (suite *KeeperTestSuite) TestSubmitTransaction() {
	// Initialize with genesis accounts
	genesisAccounts := []bridgetypes.GenesisAccount{
		{Address: "alice", Balance: 1000000},
		{Address: "bob", Balance: 500000},
	}
	
	err := suite.keeper.InitGenesis(suite.ctx, genesisAccounts)
	suite.Require().NoError(err)
	
	// Submit transaction
	tx := bridgetypes.Transaction{
		From:      "alice",
		To:        "bob",
		Amount:    1000,
		GasFee:    0,
		Timestamp: 1234567890,
		Nonce:     1,
	}
	
	err = suite.keeper.SubmitTransaction(suite.ctx, tx)
	suite.Require().NoError(err)
}

func (suite *KeeperTestSuite) TestABCIProcessing() {
	// Initialize
	genesisAccounts := []bridgetypes.GenesisAccount{
		{Address: "validator1", Balance: 100000},
	}
	
	err := suite.keeper.InitGenesis(suite.ctx, genesisAccounts)
	suite.Require().NoError(err)
	
	// Test BeginBlock
	beginBlockReq := bridgetypes.ABCIRequest{
		Type:     "BeginBlock",
		Height:   1,
		Proposer: "validator1",
	}
	
	response, err := suite.keeper.ProcessABCI(suite.ctx, beginBlockReq)
	suite.Require().NoError(err)
	suite.Require().Equal("BeginBlockOk", response.Type)
	
	// Test EndBlock
	endBlockReq := bridgetypes.ABCIRequest{
		Type:   "EndBlock",
		Height: 1,
	}
	
	response, err = suite.keeper.ProcessABCI(suite.ctx, endBlockReq)
	suite.Require().NoError(err)
	suite.Require().Equal("EndBlockOk", response.Type)
	
	// Test Commit
	commitReq := bridgetypes.ABCIRequest{
		Type: "Commit",
	}
	
	response, err = suite.keeper.ProcessABCI(suite.ctx, commitReq)
	suite.Require().NoError(err)
	suite.Require().Equal("Commit", response.Type)
	suite.Require().NotEmpty(response.Data) // State root
}

func (suite *KeeperTestSuite) TestValidatorManagement() {
	// Initialize
	genesisAccounts := []bridgetypes.GenesisAccount{
		{Address: "validator1", Balance: 100000},
	}
	
	err := suite.keeper.InitGenesis(suite.ctx, genesisAccounts)
	suite.Require().NoError(err)
	
	// Add validator
	err = suite.keeper.AddValidator(suite.ctx, "validator1", 100000)
	suite.Require().NoError(err)
	
	// Select proposer
	proposer, err := suite.keeper.SelectProposer(suite.ctx)
	suite.Require().NoError(err)
	suite.Require().NotEmpty(proposer)
}

func (suite *KeeperTestSuite) TestBlockchainInfo() {
	// Initialize
	genesisAccounts := []bridgetypes.GenesisAccount{
		{Address: "alice", Balance: 1000000},
	}
	
	err := suite.keeper.InitGenesis(suite.ctx, genesisAccounts)
	suite.Require().NoError(err)
	
	// Get blockchain info
	info, err := suite.keeper.GetBlockchainInfo(suite.ctx)
	suite.Require().NoError(err)
	suite.Require().NotNil(info)
	suite.Require().Contains(info, "chain_id")
	suite.Require().Contains(info, "height")
}

func (suite *KeeperTestSuite) TestProduceBlock() {
	// Initialize
	genesisAccounts := []bridgetypes.GenesisAccount{
		{Address: "validator1", Balance: 100000},
	}
	
	err := suite.keeper.InitGenesis(suite.ctx, genesisAccounts)
	suite.Require().NoError(err)
	
	// Produce block
	blockBytes, err := suite.keeper.ProduceBlock(suite.ctx, "validator1")
	suite.Require().NoError(err)
	suite.Require().NotEmpty(blockBytes)
}

func (suite *KeeperTestSuite) TestExportGenesis() {
	// Initialize
	genesisAccounts := []bridgetypes.GenesisAccount{
		{Address: "alice", Balance: 1000000},
	}
	
	err := suite.keeper.InitGenesis(suite.ctx, genesisAccounts)
	suite.Require().NoError(err)
	
	// Export genesis
	exported, err := suite.keeper.ExportGenesis(suite.ctx)
	suite.Require().NoError(err)
	suite.Require().NotNil(exported)
	suite.Require().Equal(uint64(0), exported.LastBlockHeight)
}
