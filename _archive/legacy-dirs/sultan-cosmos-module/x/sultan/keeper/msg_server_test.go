package keeper_test

import (
	"context"
	
	sdk "github.com/cosmos/cosmos-sdk/types"
	
	bridgetypes "github.com/wollnbergen/sultan-cosmos-bridge/types"
	"github.com/wollnbergen/sultan-cosmos-module/x/sultan/keeper"
	"github.com/wollnbergen/sultan-cosmos-module/x/sultan/types"
)

func (suite *KeeperTestSuite) TestMsgSend() {
	// Initialize
	genesisAccounts := []bridgetypes.GenesisAccount{
		{Address: "cosmos1alice", Balance: 1000000},
		{Address: "cosmos1bob", Balance: 500000},
	}
	
	err := suite.keeper.InitGenesis(suite.ctx, genesisAccounts)
	suite.Require().NoError(err)
	
	// Create message server
	msgServer := keeper.NewMsgServerImpl(*suite.keeper)
	
	// Send message
	msg := &types.MsgSend{
		From:   "cosmos1alice",
		To:     "cosmos1bob",
		Amount: 1000,
		Nonce:  1,
	}
	
	// Execute
	ctx := context.Background()
	resp, err := msgServer.Send(sdk.WrapSDKContext(suite.ctx), msg)
	suite.Require().NoError(err)
	suite.Require().NotNil(resp)
	suite.Require().True(resp.Success)
}

func (suite *KeeperTestSuite) TestMsgSendInvalidAddress() {
	// Initialize
	genesisAccounts := []bridgetypes.GenesisAccount{
		{Address: "cosmos1alice", Balance: 1000000},
	}
	
	err := suite.keeper.InitGenesis(suite.ctx, genesisAccounts)
	suite.Require().NoError(err)
	
	// Create message server
	msgServer := keeper.NewMsgServerImpl(*suite.keeper)
	
	// Send message with invalid address
	msg := &types.MsgSend{
		From:   "",
		To:     "cosmos1bob",
		Amount: 1000,
		Nonce:  1,
	}
	
	// Execute - should fail
	ctx := context.Background()
	_, err = msgServer.Send(sdk.WrapSDKContext(suite.ctx), msg)
	suite.Require().Error(err)
}

func (suite *KeeperTestSuite) TestMsgCreateValidator() {
	// Initialize
	genesisAccounts := []bridgetypes.GenesisAccount{
		{Address: "cosmos1validator1", Balance: 100000},
	}
	
	err := suite.keeper.InitGenesis(suite.ctx, genesisAccounts)
	suite.Require().NoError(err)
	
	// Create message server
	msgServer := keeper.NewMsgServerImpl(*suite.keeper)
	
	// Create validator message
	msg := &types.MsgCreateValidator{
		ValidatorAddress: "cosmos1validator1",
		Stake:            100000,
	}
	
	// Execute
	ctx := context.Background()
	resp, err := msgServer.CreateValidator(sdk.WrapSDKContext(suite.ctx), msg)
	suite.Require().NoError(err)
	suite.Require().NotNil(resp)
	suite.Require().True(resp.Success)
}

func (suite *KeeperTestSuite) TestQueryBalance() {
	// Initialize
	genesisAccounts := []bridgetypes.GenesisAccount{
		{Address: "cosmos1alice", Balance: 1000000},
	}
	
	err := suite.keeper.InitGenesis(suite.ctx, genesisAccounts)
	suite.Require().NoError(err)
	
	// Query balance
	req := &types.QueryBalanceRequest{
		Address: "cosmos1alice",
	}
	
	ctx := context.Background()
	resp, err := suite.keeper.Balance(sdk.WrapSDKContext(suite.ctx), req)
	suite.Require().NoError(err)
	suite.Require().NotNil(resp)
	suite.Require().Equal("cosmos1alice", resp.Address)
	suite.Require().Equal(uint64(1000000), resp.Balance)
}

func (suite *KeeperTestSuite) TestQueryBlockchainInfo() {
	// Initialize
	genesisAccounts := []bridgetypes.GenesisAccount{
		{Address: "cosmos1alice", Balance: 1000000},
	}
	
	err := suite.keeper.InitGenesis(suite.ctx, genesisAccounts)
	suite.Require().NoError(err)
	
	// Query blockchain info
	req := &types.QueryBlockchainInfoRequest{}
	
	ctx := context.Background()
	resp, err := suite.keeper.BlockchainInfo(sdk.WrapSDKContext(suite.ctx), req)
	suite.Require().NoError(err)
	suite.Require().NotNil(resp)
	suite.Require().NotEmpty(resp.ChainId)
	suite.Require().NotNil(resp.Info)
}
