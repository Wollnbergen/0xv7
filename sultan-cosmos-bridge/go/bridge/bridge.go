package bridge

/*
#cgo LDFLAGS: -L/tmp/cargo-target/release -lsultan_cosmos_bridge -ldl -lm
#cgo CFLAGS: -I../../include
#include <sultan_bridge.h>
#include <stdlib.h>
*/
import "C"
import (
	"encoding/json"
	"errors"
	"unsafe"

	"github.com/wollnbergen/sultan-cosmos-bridge/types"
)

// Blockchain wraps a Sultan blockchain instance via FFI
type Blockchain struct {
	handle C.uintptr_t
}

// ConsensusEngine wraps a Sultan consensus engine via FFI
type ConsensusEngine struct {
	handle C.uintptr_t
}

// Initialize the bridge (call once at startup)
func Initialize() error {
	errResult := C.sultan_bridge_init()
	if errResult.code != C.Success {
		msg := C.GoString(errResult.message)
		C.sultan_bridge_free_error(errResult)
		return errors.New(msg)
	}
	return nil
}

// Shutdown the bridge (cleanup)
func Shutdown() error {
	errResult := C.sultan_bridge_shutdown()
	if errResult.code != C.Success {
		msg := C.GoString(errResult.message)
		C.sultan_bridge_free_error(errResult)
		return errors.New(msg)
	}
	return nil
}

// NewBlockchain creates a new blockchain instance
func NewBlockchain() (*Blockchain, error) {
	var err C.BridgeError
	handle := C.sultan_blockchain_new(&err)
	
	if err.code != C.Success {
		msg := C.GoString(err.message)
		C.sultan_bridge_free_error(err)
		return nil, errors.New(msg)
	}
	
	if handle == 0 {
		return nil, errors.New("failed to create blockchain: null handle")
	}
	
	return &Blockchain{handle: handle}, nil
}

// Destroy frees the blockchain instance
func (b *Blockchain) Destroy() error {
	if b.handle == 0 {
		return errors.New("blockchain already destroyed")
	}
	
	errResult := C.sultan_blockchain_destroy(b.handle)
	if errResult.code != C.Success {
		msg := C.GoString(errResult.message)
		C.sultan_bridge_free_error(errResult)
		return errors.New(msg)
	}
	
	b.handle = 0
	return nil
}

// Height returns the current blockchain height
func (b *Blockchain) Height() (uint64, error) {
	if b.handle == 0 {
		return 0, errors.New("blockchain not initialized")
	}
	
	var err C.BridgeError
	height := C.sultan_blockchain_height(b.handle, &err)
	
	if err.code != C.Success {
		msg := C.GoString(err.message)
		C.sultan_bridge_free_error(err)
		return 0, errors.New(msg)
	}
	
	return uint64(height), nil
}

// LatestHash returns the hash of the latest block
func (b *Blockchain) LatestHash() (string, error) {
	if b.handle == 0 {
		return "", errors.New("blockchain not initialized")
	}
	
	var err C.BridgeError
	hashPtr := C.sultan_blockchain_latest_hash(b.handle, &err)
	
	if err.code != C.Success {
		msg := C.GoString(err.message)
		C.sultan_bridge_free_error(err)
		return "", errors.New(msg)
	}
	
	if hashPtr == nil {
		return "", errors.New("null hash pointer")
	}
	
	hash := C.GoString(hashPtr)
	C.sultan_bridge_free_string(hashPtr)
	
	return hash, nil
}

// AddTransaction adds a transaction to the blockchain
func (b *Blockchain) AddTransaction(tx types.Transaction) error {
	if b.handle == 0 {
		return errors.New("blockchain not initialized")
	}
	
	// Convert Go transaction to C transaction
	from := C.CString(tx.From)
	to := C.CString(tx.To)
	defer C.free(unsafe.Pointer(from))
	defer C.free(unsafe.Pointer(to))
	
	var sig *C.char
	if tx.Signature != "" {
		sig = C.CString(tx.Signature)
		defer C.free(unsafe.Pointer(sig))
	}
	
	cTx := C.CTransaction{
		from:      from,
		to:        to,
		amount:    C.uint64_t(tx.Amount),
		gas_fee:   C.uint64_t(tx.GasFee),
		timestamp: C.uint64_t(tx.Timestamp),
		nonce:     C.uint64_t(tx.Nonce),
		signature: sig,
	}
	
	var err C.BridgeError
	success := C.sultan_blockchain_add_transaction(b.handle, cTx, &err)
	
	if err.code != C.Success {
		msg := C.GoString(err.message)
		C.sultan_bridge_free_error(err)
		return errors.New(msg)
	}
	
	if !success {
		return errors.New("transaction rejected")
	}
	
	return nil
}

// GetBalance returns the balance of an account
func (b *Blockchain) GetBalance(address string) (uint64, error) {
	if b.handle == 0 {
		return 0, errors.New("blockchain not initialized")
	}
	
	addr := C.CString(address)
	defer C.free(unsafe.Pointer(addr))
	
	var err C.BridgeError
	balance := C.sultan_blockchain_get_balance(b.handle, addr, &err)
	
	if err.code != C.Success {
		msg := C.GoString(err.message)
		C.sultan_bridge_free_error(err)
		return 0, errors.New(msg)
	}
	
	return uint64(balance), nil
}

// InitAccount initializes a genesis account
func (b *Blockchain) InitAccount(address string, balance uint64) error {
	if b.handle == 0 {
		return errors.New("blockchain not initialized")
	}
	
	addr := C.CString(address)
	defer C.free(unsafe.Pointer(addr))
	
	var err C.BridgeError
	success := C.sultan_blockchain_init_account(b.handle, addr, C.uint64_t(balance), &err)
	
	if err.code != C.Success {
		msg := C.GoString(err.message)
		C.sultan_bridge_free_error(err)
		return errors.New(msg)
	}
	
	if !success {
		return errors.New("failed to initialize account")
	}
	
	return nil
}

// CreateBlock creates a new block
func (b *Blockchain) CreateBlock(validator string) error {
	if b.handle == 0 {
		return errors.New("blockchain not initialized")
	}
	
	val := C.CString(validator)
	defer C.free(unsafe.Pointer(val))
	
	var err C.BridgeError
	success := C.sultan_blockchain_create_block(b.handle, val, &err)
	
	if err.code != C.Success {
		msg := C.GoString(err.message)
		C.sultan_bridge_free_error(err)
		return errors.New(msg)
	}
	
	if !success {
		return errors.New("failed to create block")
	}
	
	return nil
}

// NewConsensusEngine creates a new consensus engine
func NewConsensusEngine() (*ConsensusEngine, error) {
	var err C.BridgeError
	handle := C.sultan_consensus_new(&err)
	
	if err.code != C.Success {
		msg := C.GoString(err.message)
		C.sultan_bridge_free_error(err)
		return nil, errors.New(msg)
	}
	
	if handle == 0 {
		return nil, errors.New("failed to create consensus engine: null handle")
	}
	
	return &ConsensusEngine{handle: handle}, nil
}

// AddValidator adds a validator to the consensus engine
func (c *ConsensusEngine) AddValidator(address string, stake uint64) error {
	if c.handle == 0 {
		return errors.New("consensus engine not initialized")
	}
	
	addr := C.CString(address)
	defer C.free(unsafe.Pointer(addr))
	
	var err C.BridgeError
	success := C.sultan_consensus_add_validator(c.handle, addr, C.uint64_t(stake), &err)
	
	if err.code != C.Success {
		msg := C.GoString(err.message)
		C.sultan_bridge_free_error(err)
		return errors.New(msg)
	}
	
	if !success {
		return errors.New("failed to add validator")
	}
	
	return nil
}

// SelectProposer selects the next block proposer
func (c *ConsensusEngine) SelectProposer() (string, error) {
	if c.handle == 0 {
		return "", errors.New("consensus engine not initialized")
	}
	
	var err C.BridgeError
	proposerPtr := C.sultan_consensus_select_proposer(c.handle, &err)
	
	if err.code != C.Success {
		msg := C.GoString(err.message)
		C.sultan_bridge_free_error(err)
		return "", errors.New(msg)
	}
	
	if proposerPtr == nil {
		return "", errors.New("no proposer selected")
	}
	
	proposer := C.GoString(proposerPtr)
	C.sultan_bridge_free_string(proposerPtr)
	
	return proposer, nil
}

// ProcessABCI processes an ABCI request
func (b *Blockchain) ProcessABCI(request types.ABCIRequest) (*types.ABCIResponse, error) {
	if b.handle == 0 {
		return nil, errors.New("blockchain not initialized")
	}
	
	// Serialize request
	requestBytes, err := json.Marshal(request)
	if err != nil {
		return nil, err
	}
	
	requestData := C.CByteArray{
		data: (*C.uint8_t)(C.CBytes(requestBytes)),
		len:  C.size_t(len(requestBytes)),
	}
	defer C.free(unsafe.Pointer(requestData.data))
	
	var bridgeErr C.BridgeError
	responseData := C.sultan_abci_process(b.handle, requestData, &bridgeErr)
	
	if bridgeErr.code != C.Success {
		msg := C.GoString(bridgeErr.message)
		C.sultan_bridge_free_error(bridgeErr)
		return nil, errors.New(msg)
	}
	
	if responseData.data == nil || responseData.len == 0 {
		return nil, errors.New("null ABCI response")
	}
	
	// Convert C bytes to Go slice
	responseBytes := C.GoBytes(unsafe.Pointer(responseData.data), C.int(responseData.len))
	C.sultan_bridge_free_bytes(responseData)
	
	// Deserialize response
	var response types.ABCIResponse
	if err := json.Unmarshal(responseBytes, &response); err != nil {
		return nil, err
	}
	
	return &response, nil
}
