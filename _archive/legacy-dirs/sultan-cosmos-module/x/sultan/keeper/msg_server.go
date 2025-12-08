package keeper

import (
	"context"
	"fmt"
	"time"
	
	sdk "github.com/cosmos/cosmos-sdk/types"
	
	bridgetypes "github.com/wollnbergen/sultan-cosmos-bridge/types"
	"github.com/wollnbergen/sultan-cosmos-module/x/sultan/types"
)

type msgServer struct {
	Keeper
}

// NewMsgServerImpl returns an implementation of the sultan MsgServer interface
func NewMsgServerImpl(keeper Keeper) types.MsgServer {
	return &msgServer{Keeper: keeper}
}

var _ types.MsgServer = msgServer{}

// Send handles MsgSend - transfers tokens between accounts
func (ms msgServer) Send(goCtx interface{}, msg *types.MsgSend) (*types.MsgSendResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx.(context.Context))
	
	// Validate message
	if err := msg.ValidateBasic(); err != nil {
		return nil, fmt.Errorf("invalid request: %w", err)
	}
	
	// Create Sultan transaction
	tx := bridgetypes.Transaction{
		From:      msg.From,
		To:        msg.To,
		Amount:    msg.Amount,
		GasFee:    0, // Sultan has zero gas fees
		Timestamp: uint64(time.Now().Unix()),
		Nonce:     msg.Nonce,
	}
	
	// Submit to Sultan blockchain via FFI
	if err := ms.SubmitTransaction(ctx, tx); err != nil {
		return nil, fmt.Errorf("invalid request: %w", err)
	}
	
	// Emit event
	ctx.EventManager().EmitEvent(
		sdk.NewEvent(
			"sultan_send",
			sdk.NewAttribute("from", msg.From),
			sdk.NewAttribute("to", msg.To),
			sdk.NewAttribute("amount", fmt.Sprintf("%d", msg.Amount)),
		),
	)
	
	ms.Logger(ctx).Info("Transaction processed",
		"from", msg.From,
		"to", msg.To,
		"amount", msg.Amount,
	)
	
	return &types.MsgSendResponse{
		Success: true,
		TxHash:  ctx.TxBytes(), // Use Cosmos TX hash
	}, nil
}

// CreateValidator handles MsgCreateValidator - registers a new validator
func (ms msgServer) CreateValidator(goCtx interface{}, msg *types.MsgCreateValidator) (*types.MsgCreateValidatorResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx.(context.Context))
	
	// Validate message
	if err := msg.ValidateBasic(); err != nil {
		return nil, fmt.Errorf("invalid request: %w", err)
	}
	
	// Add validator to Sultan consensus via FFI
	if err := ms.AddValidator(ctx, msg.ValidatorAddress, msg.Stake); err != nil {
		return nil, fmt.Errorf("invalid request: %w", err)
	}
	
	// Emit event
	ctx.EventManager().EmitEvent(
		sdk.NewEvent(
			"sultan_create_validator",
			sdk.NewAttribute("validator", msg.ValidatorAddress),
			sdk.NewAttribute("stake", fmt.Sprintf("%d", msg.Stake)),
		),
	)
	
	ms.Logger(ctx).Info("Validator created",
		"address", msg.ValidatorAddress,
		"stake", msg.Stake,
	)
	
	return &types.MsgCreateValidatorResponse{
		Success: true,
	}, nil
}
