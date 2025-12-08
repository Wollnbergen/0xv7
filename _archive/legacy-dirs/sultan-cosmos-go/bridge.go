package sultancosmos

/*
#cgo LDFLAGS: -L/tmp/cargo-target/release -lsultan_cosmos_bridge -ldl -lm
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>

// FFI function declarations matching actual Sultan Rust bridge exports
typedef struct BridgeError {
    int code;
    char message[256];
} BridgeError;

extern void sultan_bridge_init();
extern void sultan_bridge_shutdown();
extern size_t sultan_blockchain_new(BridgeError* error);
extern int sultan_blockchain_destroy(size_t handle);
extern uint64_t sultan_blockchain_height(size_t handle);
extern void sultan_blockchain_latest_hash(size_t handle, char* out_hash, uint32_t max_len);
extern uint64_t sultan_blockchain_get_balance(size_t handle, const char* address, BridgeError* error);
extern bool sultan_blockchain_init_account(size_t handle, const char* address, uint64_t balance, BridgeError* error);
extern size_t sultan_consensus_new();
extern bool sultan_consensus_add_validator(size_t consensus, const char* address, uint64_t stake, BridgeError* error);
*/
import "C"
import (
	"errors"
	"fmt"
	"unsafe"
)

// SultanBridge provides a production-grade Go interface to Sultan Core (Rust)
// This is Layer 2: the bridge between Sultan L1 (Rust) and Cosmos SDK (Go)
type SultanBridge struct {
	blockchainHandle C.size_t
	consensusHandle  C.size_t
}

// NewSultanBridge creates a new bridge instance to Sultan Core
// This initializes the Rust blockchain state via FFI
func NewSultanBridge() (*SultanBridge, error) {
	// Initialize bridge (logging, etc.)
	C.sultan_bridge_init()
	
	// Create blockchain instance (returns handle ID, not pointer)
	blockchain := C.sultan_blockchain_new(nil)
	if blockchain == 0 {
		return nil, errors.New("failed to initialize Sultan blockchain")
	}
	
	// Create consensus instance
	consensus := C.sultan_consensus_new()
	if consensus == 0 {
		C.sultan_blockchain_destroy(blockchain)
		return nil, errors.New("failed to initialize Sultan consensus")
	}
	
	return &SultanBridge{
		blockchainHandle: blockchain,
		consensusHandle:  consensus,
	}, nil
}

// Close frees the Sultan Core instance
// Must be called to prevent memory leaks
func (sb *SultanBridge) Close() {
	if sb.blockchainHandle != 0 {
		C.sultan_blockchain_destroy(sb.blockchainHandle)
		sb.blockchainHandle = 0
	}
	C.sultan_bridge_shutdown()
}

// GetHeight returns the current blockchain height
func (sb *SultanBridge) GetHeight() (uint64, error) {
	if sb.blockchainHandle == 0 {
		return 0, errors.New("bridge not initialized")
	}
	
	height := C.sultan_blockchain_height(sb.blockchainHandle)
	return uint64(height), nil
}

// GetLatestHash returns the hash of the latest block
func (sb *SultanBridge) GetLatestHash() (string, error) {
	if sb.blockchainHandle == 0 {
		return "", errors.New("bridge not initialized")
	}
	
	// Hash is 32 bytes hex-encoded = 64 chars + null terminator
	const hashSize = 65
	outBuf := make([]byte, hashSize)
	
	C.sultan_blockchain_latest_hash(
		sb.blockchainHandle,
		(*C.char)(unsafe.Pointer(&outBuf[0])),
		C.uint32_t(hashSize),
	)
	
	// Find null terminator
	for i, b := range outBuf {
		if b == 0 {
			return string(outBuf[:i]), nil
		}
	}
	
	return string(outBuf), nil
}

// GetBalance queries an account balance in usltn (micro-SLTN)
// Returns balance or error if account doesn't exist
func (sb *SultanBridge) GetBalance(address string) (uint64, error) {
	if sb.blockchainHandle == 0 {
		return 0, errors.New("bridge not initialized")
	}
	
	cAddr := C.CString(address)
	defer C.free(unsafe.Pointer(cAddr))
	
	var bridgeError C.BridgeError
	balance := C.sultan_blockchain_get_balance(
		sb.blockchainHandle,
		cAddr,
		&bridgeError,
	)
	
	if bridgeError.code != 0 {
		errorMsg := C.GoString(&bridgeError.message[0])
		return 0, fmt.Errorf("failed to get balance for %s: %s", address, errorMsg)
	}
	
	return uint64(balance), nil
}

// InitAccount creates a new account with initial balance
// Used for genesis accounts or faucet distributions
func (sb *SultanBridge) InitAccount(address string, balance uint64) error {
	if sb.blockchainHandle == 0 {
		return errors.New("bridge not initialized")
	}
	
	cAddr := C.CString(address)
	defer C.free(unsafe.Pointer(cAddr))
	
	var bridgeError C.BridgeError
	result := C.sultan_blockchain_init_account(
		sb.blockchainHandle,
		cAddr,
		C.uint64_t(balance),
		&bridgeError,
	)
	
	if !result {
		errorMsg := C.GoString(&bridgeError.message[0])
		return fmt.Errorf("failed to initialize account %s: %s", address, errorMsg)
	}
	
	return nil
}

// AddValidator adds a new validator to the Sultan validator set
// Returns error if validator already exists or stake is insufficient
func (sb *SultanBridge) AddValidator(address string, stake uint64) error {
	if sb.consensusHandle == 0 {
		return errors.New("consensus not initialized")
	}
	
	cAddr := C.CString(address)
	defer C.free(unsafe.Pointer(cAddr))
	
	var bridgeError C.BridgeError
	result := C.sultan_consensus_add_validator(
		sb.consensusHandle,
		cAddr,
		C.uint64_t(stake),
		&bridgeError,
	)
	
	if !result {
		errorMsg := C.GoString(&bridgeError.message[0])
		return fmt.Errorf("failed to add validator %s: %s", address, errorMsg)
	}
	
	return nil
}
