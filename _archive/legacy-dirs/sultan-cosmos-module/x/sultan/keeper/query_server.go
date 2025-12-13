package keeper

import (
	"context"
	"fmt"
	
	sdk "github.com/cosmos/cosmos-sdk/types"
	
	"github.com/wollnbergen/sultan-cosmos-module/x/sultan/types"
)

var _ types.QueryServer = Keeper{}

// Balance queries the balance of a Sultan account
func (k Keeper) Balance(goCtx interface{}, req *types.QueryBalanceRequest) (*types.QueryBalanceResponse, error) {
	if req == nil {
		return nil, fmt.Errorf("empty request")
	}
	
	if req.Address == "" {
		return nil, fmt.Errorf("address cannot be empty")
	}
	
	ctx := sdk.UnwrapSDKContext(goCtx.(context.Context))
	
	// Query balance via FFI
	balance, err := k.GetBalance(ctx, req.Address)
	if err != nil {
		return nil, fmt.Errorf("invalid request: %w", err)
	}
	
	return &types.QueryBalanceResponse{
		Address: req.Address,
		Balance: balance,
	}, nil
}

// BlockchainInfo queries Sultan blockchain information
func (k Keeper) BlockchainInfo(goCtx interface{}, req *types.QueryBlockchainInfoRequest) (*types.QueryBlockchainInfoResponse, error) {
	if req == nil {
		return nil, fmt.Errorf("empty request")
	}
	
	ctx := sdk.UnwrapSDKContext(goCtx.(context.Context))
	
	// Get blockchain info via FFI
	info, err := k.GetBlockchainInfo(ctx)
	if err != nil {
		return nil, fmt.Errorf("invalid request: %w", err)
	}
	
	// Extract fields
	height, _ := info["height"].(float64)
	chainID, _ := info["chain_id"].(string)
	
	return &types.QueryBlockchainInfoResponse{
		ChainId: chainID,
		Height:  uint64(height),
		Info:    info,
	}, nil
}
