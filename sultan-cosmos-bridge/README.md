# Sultan Cosmos Bridge

Production-grade FFI (Foreign Function Interface) bridge between Sultan Core (Rust) and Cosmos SDK (Go).

## Architecture

```
┌─────────────────┐        ┌──────────────┐        ┌─────────────────┐
│   Cosmos SDK    │◄──────►│  FFI Bridge  │◄──────►│   Sultan Core   │
│      (Go)       │  CGo   │   (C/Rust)   │  Rust  │     (Rust)      │
└─────────────────┘        └──────────────┘        └─────────────────┘
```

## Features

✅ **Thread-Safe**: Global state management with RwLock  
✅ **Memory-Safe**: Proper allocation/deallocation across FFI boundary  
✅ **Error-Handling**: Comprehensive error codes and panic catching  
✅ **ABCI Protocol**: Full Application Blockchain Interface implementation  
✅ **Production-Ready**: Zero stubs, comprehensive error handling  

## API Functions

### Initialization
- `sultan_bridge_init()` - Initialize bridge (call once)
- `sultan_bridge_shutdown()` - Cleanup resources

### Blockchain Management
- `sultan_blockchain_new()` - Create blockchain instance
- `sultan_blockchain_destroy()` - Destroy blockchain instance
- `sultan_blockchain_height()` - Get current height
- `sultan_blockchain_latest_hash()` - Get latest block hash
- `sultan_blockchain_create_block()` - Produce new block

### Transaction Management
- `sultan_blockchain_add_transaction()` - Submit transaction
- `sultan_blockchain_get_balance()` - Query account balance
- `sultan_blockchain_init_account()` - Initialize genesis account

### Consensus
- `sultan_consensus_new()` - Create consensus engine
- `sultan_consensus_add_validator()` - Add validator
- `sultan_consensus_select_proposer()` - Select block proposer

### ABCI Protocol
- `sultan_abci_process()` - Process ABCI requests from CometBFT

## Building

```bash
cargo build --release
```

This generates:
- `libsultan_cosmos_bridge.so` (Linux)
- `libsultan_cosmos_bridge.dylib` (macOS)
- `sultan_cosmos_bridge.dll` (Windows)
- `include/sultan_bridge.h` (C header)

## Usage from Go

```go
package main

/*
#cgo LDFLAGS: -L../target/release -lsultan_cosmos_bridge
#include "../include/sultan_bridge.h"
*/
import "C"
import "unsafe"

func main() {
    // Initialize bridge
    C.sultan_bridge_init()
    defer C.sultan_bridge_shutdown()
    
    // Create blockchain
    var err C.BridgeError
    handle := C.sultan_blockchain_new(&err)
    defer C.sultan_blockchain_destroy(handle)
    
    // Get height
    height := C.sultan_blockchain_height(handle, &err)
    println("Height:", height)
}
```

## Error Handling

All FFI functions use proper error handling:

```c
BridgeError error;
uintptr_t handle = sultan_blockchain_new(&error);
if (error.code != 0) {
    // Handle error
    printf("Error: %s\n", error.message);
    sultan_bridge_free_error(error);
}
```

## Memory Management

**Important**: Always free allocated memory:

```c
char* hash = sultan_blockchain_latest_hash(handle, &error);
if (hash != NULL) {
    // Use hash
    sultan_bridge_free_string(hash);
}
```

## ABCI Integration

The bridge implements the complete ABCI protocol:

- **Info**: Query blockchain state
- **InitChain**: Initialize with genesis
- **BeginBlock**: Start new block
- **DeliverTx**: Process transaction
- **EndBlock**: Finalize block
- **Commit**: Commit state changes
- **Query**: Query application state

## Testing

```bash
cargo test
```

## Production Deployment

1. Build release binary: `cargo build --release`
2. Copy library to system path or set `LD_LIBRARY_PATH`
3. Include header in Go: `#include "sultan_bridge.h"`
4. Link library: `#cgo LDFLAGS: -lsultan_cosmos_bridge`

## Safety Guarantees

- ✅ All pointers checked for null
- ✅ All panics caught and converted to errors
- ✅ All UTF-8 strings validated
- ✅ All memory properly freed
- ✅ Thread-safe global state
- ✅ No unsafe memory access

## Performance

- Handle-based design (minimal overhead)
- Zero-copy where possible
- Efficient serialization (bincode/JSON)
- RwLock for concurrent reads

## Next Steps (Phase 3)

- Create Cosmos SDK module wrapper
- Implement CometBFT integration
- Build IBC handlers
- Create Cosmos client libraries
