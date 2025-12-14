package types

import (
	"encoding/json"
	"fmt"
	
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var (
	_ sdk.Msg = &MsgSend{}
	_ sdk.Msg = &MsgCreateValidator{}
)

// MsgSend defines a transaction message to send tokens
type MsgSend struct {
	From   string `json:"from"`
	To     string `json:"to"`
	Amount uint64 `json:"amount"`
	Nonce  uint64 `json:"nonce"`
}

// Reset implements proto.Message
func (msg *MsgSend) Reset() {}

// String implements proto.Message
func (msg *MsgSend) String() string {
	return "MsgSend"
}

// ProtoMessage implements proto.Message
func (msg *MsgSend) ProtoMessage() {}

// XXX_MessageName provides the full name for this message
func (msg *MsgSend) XXX_MessageName() string {
	return "sultan.v1.MsgSend"
}

// Route implements sdk.Msg
func (msg MsgSend) Route() string {
	return RouterKey
}

// Type implements sdk.Msg
func (msg MsgSend) Type() string {
	return "send"
}

// GetSigners implements sdk.Msg
func (msg MsgSend) GetSigners() []sdk.AccAddress {
	from, err := sdk.AccAddressFromBech32(msg.From)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{from}
}

// GetSignBytes implements sdk.Msg
func (msg MsgSend) GetSignBytes() []byte {
	bz, err := json.Marshal(msg)
	if err != nil {
		panic(err)
	}
	return sdk.MustSortJSON(bz)
}

// ValidateBasic implements sdk.Msg
func (msg MsgSend) ValidateBasic() error {
	if msg.From == "" {
		return fmt.Errorf("from address cannot be empty")
	}
	if msg.To == "" {
		return fmt.Errorf("to address cannot be empty")
	}
	if msg.Amount == 0 {
		return fmt.Errorf("amount must be positive")
	}
	
	// Validate addresses
	if _, err := sdk.AccAddressFromBech32(msg.From); err != nil {
		return fmt.Errorf("invalid from address: %w", err)
	}
	if _, err := sdk.AccAddressFromBech32(msg.To); err != nil {
		return fmt.Errorf("invalid to address: %w", err)
	}
	
	return nil
}

// MsgCreateValidator defines a transaction message to create a validator
type MsgCreateValidator struct {
	ValidatorAddress string `json:"validator_address"`
	Stake            uint64 `json:"stake"`
}

// Reset implements proto.Message
func (msg *MsgCreateValidator) Reset() {}

// String implements proto.Message
func (msg *MsgCreateValidator) String() string {
	return "MsgCreateValidator"
}

// ProtoMessage implements proto.Message
func (msg *MsgCreateValidator) ProtoMessage() {}

// XXX_MessageName provides the full name for this message
func (msg *MsgCreateValidator) XXX_MessageName() string {
	return "sultan.v1.MsgCreateValidator"
}

// Route implements sdk.Msg
func (msg MsgCreateValidator) Route() string {
	return RouterKey
}

// Type implements sdk.Msg
func (msg MsgCreateValidator) Type() string {
	return "create_validator"
}

// GetSigners implements sdk.Msg
func (msg MsgCreateValidator) GetSigners() []sdk.AccAddress {
	addr, err := sdk.AccAddressFromBech32(msg.ValidatorAddress)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{addr}
}

// GetSignBytes implements sdk.Msg
func (msg MsgCreateValidator) GetSignBytes() []byte {
	bz, err := json.Marshal(msg)
	if err != nil {
		panic(err)
	}
	return sdk.MustSortJSON(bz)
}

// ValidateBasic implements sdk.Msg
func (msg MsgCreateValidator) ValidateBasic() error {
	if msg.ValidatorAddress == "" {
		return fmt.Errorf("validator address cannot be empty")
	}
	if msg.Stake == 0 {
		return fmt.Errorf("stake must be positive")
	}
	
	// Validate address
	if _, err := sdk.AccAddressFromBech32(msg.ValidatorAddress); err != nil {
		return fmt.Errorf("invalid validator address: %w", err)
	}
	
	return nil
}
