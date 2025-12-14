# Sultan Cosmos Bridge - Testing Guide

## Overview

This guide covers testing the FFI bridge between Sultan Core (Rust) and the Go wrapper layer.

## Test Structure

```
sultan-cosmos-bridge/
├── src/                    # Rust FFI implementation
│   └── tests in ffi.rs, state.rs, abci.rs
├── go/
│   ├── bridge/
│   │   └── bridge_test.go          # Unit tests for Go wrapper
│   └── integration_test.go         # Full integration tests
```

## Prerequisites

### 1. Build the Bridge Library

```bash
cd /workspaces/0xv7
cargo build --release -p sultan-cosmos-bridge
```

This generates:
- `/tmp/cargo-target/release/libsultan_cosmos_bridge.so` (Linux)
- `/workspaces/0xv7/sultan-cosmos-bridge/include/sultan_bridge.h`

### 2. Verify Library

```bash
# Check library exists
ls -lh /tmp/cargo-target/release/libsultan_cosmos_bridge.so

# Check exported symbols
nm -D /tmp/cargo-target/release/libsultan_cosmos_bridge.so | grep sultan

# Expected symbols:
# sultan_bridge_init
# sultan_bridge_shutdown
# sultan_blockchain_new
# sultan_blockchain_destroy
# sultan_blockchain_add_transaction
# ... (25+ functions total)
```

## Running Tests

### Rust Unit Tests

```bash
cd /workspaces/0xv7/sultan-cosmos-bridge
cargo test

# Run specific test module
cargo test --lib state::tests
cargo test --lib ffi::tests
cargo test --lib abci::tests
```

Expected output:
```
running 5 tests
test state::tests::test_add_get_blockchain ... ok
test state::tests::test_add_get_consensus ... ok
test ffi::tests::test_ffi_smoke ... ok
test abci::tests::test_abci_info ... ok
test abci::tests::test_abci_deliver_tx ... ok

test result: ok. 5 passed; 0 failed
```

### Go Unit Tests

```bash
cd /workspaces/0xv7/sultan-cosmos-bridge/go
go test -v ./bridge

# Run specific test
go test -v ./bridge -run TestBlockchainLifecycle
```

Expected tests:
- `TestBridgeInitialization` - Bridge init/shutdown
- `TestBlockchainLifecycle` - Create/destroy blockchain
- `TestGenesisAccounts` - Account initialization
- `TestTransactionSubmission` - TX validation
- `TestBlockProduction` - Block creation
- `TestConsensusEngine` - Validator management
- `TestABCIProtocol` - ABCI message handling
- `TestConcurrentAccess` - Thread safety
- `TestMultipleBlockchains` - State isolation

### Integration Tests

```bash
cd /workspaces/0xv7/sultan-cosmos-bridge/go
go test -v -run TestFull

# Stress test (long-running)
go test -v -run TestStress
```

Expected tests:
- `TestFullBlockchainFlow` - Complete lifecycle
- `TestABCIFullProtocol` - Full ABCI flow
- `TestConsensusWithMultipleValidators` - Weighted selection
- `TestStressTest` - 100 TXs + 10 blocks

### Benchmarks

```bash
cd /workspaces/0xv7/sultan-cosmos-bridge/go
go test -bench=. -benchmem ./bridge

# Specific benchmark
go test -bench=BenchmarkTransactionSubmission -benchtime=10s
```

Expected benchmarks:
- `BenchmarkTransactionSubmission` - TX submission throughput
- `BenchmarkBalanceQuery` - State query performance
- `BenchmarkFullBlockProduction` - Block creation speed

## Test Coverage

### Rust Coverage

```bash
cd /workspaces/0xv7/sultan-cosmos-bridge
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir ./coverage
```

Open `coverage/index.html` to view report.

### Go Coverage

```bash
cd /workspaces/0xv7/sultan-cosmos-bridge/go
go test -coverprofile=coverage.out ./...
go tool cover -html=coverage.out -o coverage.html
```

Open `coverage.html` to view report.

## Memory Leak Testing

### Valgrind (Linux)

```bash
# Build with debug symbols
cargo build -p sultan-cosmos-bridge

# Run tests under valgrind
cd go
go test -c ./bridge
valgrind --leak-check=full --show-leak-kinds=all ./bridge.test
```

Expected: "All heap blocks were freed -- no leaks are possible"

### Go Race Detector

```bash
cd /workspaces/0xv7/sultan-cosmos-bridge/go
go test -race -v ./...
```

Expected: No data race warnings.

## Manual Testing

### Test Script

```bash
cd /workspaces/0xv7/sultan-cosmos-bridge/go
cat > manual_test.go << 'EOF'
package main

import (
	"fmt"
	"github.com/wollnbergen/sultan-cosmos-bridge/bridge"
	"github.com/wollnbergen/sultan-cosmos-bridge/types"
	"time"
)

func main() {
	// Initialize
	bridge.Initialize()
	defer bridge.Shutdown()
	
	// Create blockchain
	bc, _ := bridge.NewBlockchain()
	defer bc.Destroy()
	
	// Setup genesis
	bc.InitAccount("alice", 1000000)
	bc.InitAccount("bob", 500000)
	bc.InitAccount("validator1", 100000)
	
	// Submit transactions
	for i := 1; i <= 10; i++ {
		tx := types.Transaction{
			From:      "alice",
			To:        "bob",
			Amount:    1000,
			Nonce:     uint64(i),
			Timestamp: uint64(time.Now().Unix()),
		}
		bc.AddTransaction(tx)
	}
	
	// Create block
	bc.CreateBlock("validator1")
	
	// Query state
	height, _ := bc.Height()
	aliceBalance, _ := bc.GetBalance("alice")
	
	fmt.Printf("Block height: %d\n", height)
	fmt.Printf("Alice balance: %d\n", aliceBalance)
}
EOF

go run manual_test.go
```

Expected output:
```
Block height: 1
Alice balance: 990000
```

## Troubleshooting

### Library Not Found

Error: `error while loading shared libraries: libsultan_cosmos_bridge.so`

Solution:
```bash
export LD_LIBRARY_PATH=/tmp/cargo-target/release:$LD_LIBRARY_PATH
go test -v ./bridge
```

Or set in test:
```go
func init() {
	os.Setenv("LD_LIBRARY_PATH", "/tmp/cargo-target/release")
}
```

### Symbol Not Found

Error: `undefined: C.sultan_blockchain_new`

Solution: Verify C header is generated:
```bash
ls /workspaces/0xv7/sultan-cosmos-bridge/include/sultan_bridge.h
cat /workspaces/0xv7/sultan-cosmos-bridge/include/sultan_bridge.h | grep sultan_blockchain_new
```

Rebuild if missing:
```bash
cd /workspaces/0xv7
cargo clean -p sultan-cosmos-bridge
cargo build --release -p sultan-cosmos-bridge
```

### CGo Compilation Errors

Error: `cgo: C compiler "gcc" not found`

Solution: Install build tools:
```bash
sudo apt-get update
sudo apt-get install -y build-essential
```

### Panic at FFI Boundary

Error: Rust panic not caught

Check Rust code has panic boundaries:
```rust
#[no_mangle]
pub extern "C" fn sultan_function() -> *mut BridgeError {
    match std::panic::catch_unwind(|| {
        // ... implementation
    }) {
        Ok(result) => result,
        Err(_) => BridgeError::internal_error("Panic occurred"),
    }
}
```

## Success Criteria

✅ All Rust unit tests pass
✅ All Go unit tests pass  
✅ All integration tests pass
✅ No memory leaks (valgrind clean)
✅ No data races (race detector clean)
✅ Benchmarks complete successfully
✅ Manual test script works

## Performance Targets

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Transaction submission | >1000 tx/sec | BenchmarkTransactionSubmission |
| Balance query | >10000 queries/sec | BenchmarkBalanceQuery |
| Block production | >100 blocks/sec | BenchmarkFullBlockProduction |
| FFI call overhead | <10µs | Individual function benchmarks |

## Next Steps

After all tests pass:
1. Update architecture documentation
2. Mark Phase 2 Day 5-6 complete
3. Begin Phase 3: Cosmos SDK module integration
4. Test IBC protocol integration
