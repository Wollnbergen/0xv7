package bridge

/*
#cgo LDFLAGS: -L../../target/release -lsultan_cosmos_bridge -ldl -lm
#include <stdlib.h>
#include <stdint.h>

// Error codes from Rust
typedef struct {
    int32_t code;
    char* message;
} BridgeError;

// FFI function declarations
extern BridgeError sultan_bridge_init();
extern BridgeError sultan_bridge_shutdown();
extern uintptr_t sultan_blockchain_new(BridgeError* error);
extern void sultan_blockchain_destroy(uintptr_t handle);
extern char* sultan_blockchain_get_balance(uintptr_t handle, const char* address, BridgeError* error);
extern BridgeError sultan_blockchain_add_transaction(uintptr_t handle, const char* from, const char* to, uint64_t amount);
extern char* sultan_blockchain_create_block(uintptr_t handle, const char* validator, BridgeError* error);
extern uintptr_t sultan_consensus_new(BridgeError* error);
extern void sultan_consensus_destroy(uintptr_t handle);
extern BridgeError sultan_consensus_add_validator(uintptr_t handle, const char* address, uint64_t stake);
extern char* sultan_consensus_select_proposer(uintptr_t handle, BridgeError* error);

// Helper to free Rust-allocated strings
extern void sultan_free_string(char* s);
*/
import "C"
import (
	"fmt"
	"unsafe"
)

// Error represents a bridge error
type Error struct {
	Code    int32
	Message string
}

func (e *Error) Error() string {
	return fmt.Sprintf("bridge error %d: %s", e.Code, e.Message)
}

// convertError converts C error to Go error
func convertError(cerr C.BridgeError) error {
	if cerr.code == 0 {
		return nil
	}
	msg := C.GoString(cerr.message)
	C.sultan_free_string(cerr.message)
	return &Error{Code: int32(cerr.code), Message: msg}
}

// Initialize initializes the FFI bridge
func Initialize() error {
	cerr := C.sultan_bridge_init()
	return convertError(cerr)
}

// Shutdown shuts down the FFI bridge
func Shutdown() error {
	cerr := C.sultan_bridge_shutdown()
	return convertError(cerr)
}

// Blockchain represents a Sultan blockchain instance
type Blockchain struct {
	handle C.uintptr_t
}

// NewBlockchain creates a new blockchain instance
func NewBlockchain() (*Blockchain, error) {
	var cerr C.BridgeError
	handle := C.sultan_blockchain_new(&cerr)
	if err := convertError(cerr); err != nil {
		return nil, err
	}
	if handle == 0 {
		return nil, fmt.Errorf("failed to create blockchain: null handle")
	}
	return &Blockchain{handle: handle}, nil
}

// Destroy destroys the blockchain instance
func (b *Blockchain) Destroy() {
	if b.handle != 0 {
		C.sultan_blockchain_destroy(b.handle)
		b.handle = 0
	}
}

// GetBalance queries account balance
func (b *Blockchain) GetBalance(address string) (string, error) {
	caddr := C.CString(address)
	defer C.free(unsafe.Pointer(caddr))
	
	var cerr C.BridgeError
	cbalance := C.sultan_blockchain_get_balance(b.handle, caddr, &cerr)
	if err := convertError(cerr); err != nil {
		return "", err
	}
	
	balance := C.GoString(cbalance)
	C.sultan_free_string(cbalance)
	return balance, nil
}

// AddTransaction adds a transaction to the blockchain
func (b *Blockchain) AddTransaction(from, to string, amount uint64) error {
	cfrom := C.CString(from)
	cto := C.CString(to)
	defer C.free(unsafe.Pointer(cfrom))
	defer C.free(unsafe.Pointer(cto))
	
	cerr := C.sultan_blockchain_add_transaction(b.handle, cfrom, cto, C.uint64_t(amount))
	return convertError(cerr)
}

// CreateBlock creates a new block
func (b *Blockchain) CreateBlock(validator string) (string, error) {
	cvalidator := C.CString(validator)
	defer C.free(unsafe.Pointer(cvalidator))
	
	var cerr C.BridgeError
	cblock := C.sultan_blockchain_create_block(b.handle, cvalidator, &cerr)
	if err := convertError(cerr); err != nil {
		return "", err
	}
	
	block := C.GoString(cblock)
	C.sultan_free_string(cblock)
	return block, nil
}

// ConsensusEngine represents the consensus engine
type ConsensusEngine struct {
	handle C.uintptr_t
}

// NewConsensusEngine creates a new consensus engine
func NewConsensusEngine() (*ConsensusEngine, error) {
	var cerr C.BridgeError
	handle := C.sultan_consensus_new(&cerr)
	if err := convertError(cerr); err != nil {
		return nil, err
	}
	if handle == 0 {
		return nil, fmt.Errorf("failed to create consensus engine: null handle")
	}
	return &ConsensusEngine{handle: handle}, nil
}

// Destroy destroys the consensus engine
func (c *ConsensusEngine) Destroy() {
	if c.handle != 0 {
		C.sultan_consensus_destroy(c.handle)
		c.handle = 0
	}
}

// AddValidator adds a validator to the consensus
func (c *ConsensusEngine) AddValidator(address string, stake uint64) error {
	caddr := C.CString(address)
	defer C.free(unsafe.Pointer(caddr))
	
	cerr := C.sultan_consensus_add_validator(c.handle, caddr, C.uint64_t(stake))
	return convertError(cerr)
}

// SelectProposer selects the next block proposer
func (c *ConsensusEngine) SelectProposer() (string, error) {
	var cerr C.BridgeError
	cproposer := C.sultan_consensus_select_proposer(c.handle, &cerr)
	if err := convertError(cerr); err != nil {
		return "", err
	}
	
	proposer := C.GoString(cproposer)
	C.sultan_free_string(cproposer)
	return proposer, nil
}
