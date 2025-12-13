package types

import (
	"fmt"
)

// Errors for the sultan module
var (
	ErrInvalidGenesisAccount = fmt.Errorf("invalid genesis account")
	ErrBlockchainNotFound    = fmt.Errorf("blockchain handle not found")
	ErrFFICallFailed         = fmt.Errorf("FFI call failed")
	ErrInvalidTransaction    = fmt.Errorf("invalid transaction")
)

// Message response types
type MsgSendResponse struct {
	Success bool   `json:"success"`
	TxHash  []byte `json:"tx_hash,omitempty"`
}

type MsgCreateValidatorResponse struct {
	Success bool `json:"success"`
}

// Query request/response types
type QueryBalanceRequest struct {
	Address string `json:"address"`
}

type QueryBalanceResponse struct {
	Address string `json:"address"`
	Balance uint64 `json:"balance"`
}

type QueryBlockchainInfoRequest struct{}

type QueryBlockchainInfoResponse struct {
	ChainId string                 `json:"chain_id"`
	Height  uint64                 `json:"height"`
	Info    map[string]interface{} `json:"info"`
}

// Proto methods for QueryBalanceResponse
func (m *QueryBalanceResponse) Reset()         { *m = QueryBalanceResponse{} }
func (m *QueryBalanceResponse) String() string { return fmt.Sprintf("%+v", *m) }
func (*QueryBalanceResponse) ProtoMessage()    {}

// Proto methods for QueryBlockchainInfoResponse
func (m *QueryBlockchainInfoResponse) Reset()         { *m = QueryBlockchainInfoResponse{} }
func (m *QueryBlockchainInfoResponse) String() string { return fmt.Sprintf("%+v", *m) }
func (*QueryBlockchainInfoResponse) ProtoMessage()    {}

// Proto methods for QueryBalanceRequest
func (m *QueryBalanceRequest) Reset()         { *m = QueryBalanceRequest{} }
func (m *QueryBalanceRequest) String() string { return fmt.Sprintf("%+v", *m) }
func (*QueryBalanceRequest) ProtoMessage()    {}

// Proto methods for QueryBlockchainInfoRequest
func (m *QueryBlockchainInfoRequest) Reset()         { *m = QueryBlockchainInfoRequest{} }
func (m *QueryBlockchainInfoRequest) String() string { return fmt.Sprintf("%+v", *m) }
func (*QueryBlockchainInfoRequest) ProtoMessage()    {}

// MsgServer interface
type MsgServer interface {
	Send(context interface{}, msg *MsgSend) (*MsgSendResponse, error)
	CreateValidator(context interface{}, msg *MsgCreateValidator) (*MsgCreateValidatorResponse, error)
}

// QueryServer interface
type QueryServer interface {
	Balance(context interface{}, req *QueryBalanceRequest) (*QueryBalanceResponse, error)
	BlockchainInfo(context interface{}, req *QueryBlockchainInfoRequest) (*QueryBlockchainInfoResponse, error)
}
